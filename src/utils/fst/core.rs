use super::codec::{Codec, CodecV32};
use super::outputs::Outputs;
use crate::utils::Stack;
use bytes::{Buf, BufMut, BytesMut};
use std::collections::LinkedList;
use std::error::Error;
use std::str::Chars;
use std::sync::{Arc, Mutex};

const FLAG_FINAL: u8 = 0x01 << 0;
const FLAG_HAS_VALUE: u8 = 0x01 << 1;
const FLAG_HAS_FINAL_VALUE: u8 = 0x01 << 2;

pub struct FST<T, O>
where
    O: Outputs<Item = T>,
{
    outputs: O,
    lines: LinkedList<Line<T>>,
}

pub struct Line<T> {
    label: char,
    value: Option<T>,
    flag: u8,
    final_value: Option<T>,
    nexts: LinkedList<Line<T>>,
}

impl<T, O> FST<T, O>
where
    T: Eq,
    O: Outputs<Item = T>,
{
    pub fn builder(outputs: O) -> Builder<T, O> {
        Builder::new(outputs)
    }

    pub fn decode(
        outputs: O,
        decoder: &impl Codec<Item = T>,
        bf: &mut BytesMut,
    ) -> Result<FST<T, O>, Box<dyn Error>> {
        let mut parser = Parser::new(outputs);
        parser.read(decoder, bf);
        Ok(parser.build())
    }

    fn new(outputs: O) -> FST<T, O> {
        FST {
            outputs,
            lines: LinkedList::new(),
        }
    }

    pub fn traverse(&self, f: impl Fn(&Line<T>)) {
        for it in &self.lines {
            it.traverse(&f);
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        if self.lines.is_empty() {
            return None;
        }
        let mut chars = key.chars();
        if let Some(ch) = chars.next() {
            for it in &self.lines {
                if it.label == ch {
                    let mut sum = self.outputs.zero();
                    if it.sum(chars, &mut sum, &self.outputs) {
                        return Some(sum);
                    }
                    break;
                }
            }
        }
        None
    }

    pub fn save(&self, dest: &mut BytesMut, encoder: &impl Codec<Item = T>) {
        let bf = Arc::new(Mutex::new(BytesMut::new()));
        let bf_cloned = bf.clone();
        let amount = Arc::new(Mutex::new(0 as u32));
        self.traverse(|next: &Line<T>| {
            let mut c = amount.lock().unwrap();
            *c = *c + 1;
            let mut bf = bf_cloned.lock().unwrap();
            let label = next.get_label() as u32;
            CodecV32.write(&mut bf, &label).unwrap();
            let mut flag = next.flag;
            if let Some(v) = &next.value {
                if self.outputs.zero() != *v {
                    flag |= FLAG_HAS_VALUE;
                }
            }

            if let Some(v) = &next.final_value {
                if self.outputs.zero() != *v {
                    flag |= FLAG_HAS_FINAL_VALUE;
                }
            }

            let follower_amount = next.nexts.len() as u32;
            if follower_amount >= 31 {
                flag |= 0xF8;
            } else {
                flag |= ((follower_amount & 0xFF) as u8) << 3;
            }

            bf.put_u8(flag);
            if follower_amount >= 31 {
                CodecV32.write(&mut bf, &follower_amount).unwrap();
            }
            if flag & FLAG_HAS_VALUE != 0 {
                if let Some(v) = &next.value {
                    encoder.write(&mut bf, v).unwrap();
                }
            }
            if flag & FLAG_HAS_FINAL_VALUE != 0 {
                if let Some(v) = &next.final_value {
                    encoder.write(&mut bf, v).unwrap();
                }
            }
        });
        dest.put_u32(*amount.lock().unwrap());
        dest.put_slice(bf.lock().unwrap().bytes());
    }

    fn push(&mut self, key: &str, value: T) {
        let mut chars = key.chars();
        if let Some(first) = chars.next() {
            for it in &mut self.lines {
                if it.label == first {
                    it.push(chars, Some(value), &self.outputs);
                    return;
                }
            }
            let mut newborn = Line::new(first);
            newborn.push(chars, Some(value), &self.outputs);
            self.lines.push_back(newborn);
        }
    }
}

impl<T> Line<T>
where
    T: Eq,
{
    #[inline]
    fn new(label: char) -> Line<T> {
        Line {
            label,
            value: None,
            final_value: None,
            flag: 0,
            nexts: LinkedList::new(),
        }
    }

    #[inline]
    fn add(&mut self, inc: &T, op: &impl Outputs<Item = T>) {
        match &mut self.value {
            Some(exist) => {
                *exist = op.add(exist, inc);
            }
            None => {
                self.value = Some(op.add(&op.zero(), inc));
            }
        }
    }

    #[inline]
    fn set_final_value(&mut self, value: T, op: &impl Outputs<Item = T>) {
        let result = match self.final_value.take() {
            Some(ref old) => op.merge(old, &value),
            None => value,
        };
        self.final_value = Some(result);
    }

    #[inline]
    fn push_pre(&mut self, insert: Option<T>, outputs: &impl Outputs<Item = T>) -> Option<T> {
        if insert.is_none() {
            return None;
        }
        let value = insert.unwrap();
        match &mut self.value {
            Some(exist) => {
                let prefix = outputs.common(exist, &value);
                let left = outputs.subtract(&value, &prefix);
                let incr = outputs.subtract(exist, &prefix);
                *exist = prefix;
                for next in &mut self.nexts {
                    next.add(&incr, outputs);
                }
                if self.is_final() {
                    let new_final_value = match self.final_value.take() {
                        Some(v) => outputs.add(&v, &incr),
                        None => incr,
                    };
                    self.final_value = Some(new_final_value);
                }
                Some(left)
            }
            None => {
                self.value = Some(value);
                None
            }
        }
    }

    fn push(&mut self, mut chars: Chars, insert: Option<T>, op: &impl Outputs<Item = T>) {
        let actual = self.push_pre(insert, op);
        match chars.next() {
            Some(ch) => {
                for next in &mut self.nexts {
                    if next.label == ch {
                        next.push(chars, actual, op);
                        return;
                    }
                }
                let mut newborn = Line::new(ch);
                newborn.push(chars, actual, op);
                self.nexts.push_back(newborn);
            }
            None => {
                // check final stat
                self.flag |= FLAG_FINAL;
                if let Some(value) = actual {
                    self.set_final_value(value, op);
                }
            }
        }
    }

    #[inline]
    fn traverse(&self, f: &impl Fn(&Line<T>)) {
        f(self);
        for next in &self.nexts {
            next.traverse(f);
        }
    }

    #[inline]
    fn sum(&self, mut chars: Chars, results: &mut T, op: &impl Outputs<Item = T>) -> bool {
        if let Some(value) = &self.value {
            *results = op.add(results, value);
        }
        match chars.next() {
            Some(ch) => {
                for next in &self.nexts {
                    if next.label == ch {
                        return next.sum(chars, results, op);
                    }
                }
                false
            }
            None => {
                if self.flag & FLAG_FINAL == 0 {
                    return false;
                }
                if let Some(v) = &self.final_value {
                    *results = op.add(results, v);
                }
                true
            }
        }
    }

    pub fn get_nexts(&self) -> &LinkedList<Line<T>> {
        &self.nexts
    }

    pub fn get_label(&self) -> char {
        self.label
    }

    pub fn get_value(&self) -> &Option<T> {
        &self.value
    }

    pub fn is_final(&self) -> bool {
        self.flag & FLAG_FINAL != 0
    }

    pub fn get_final_value(&self) -> &Option<T> {
        &self.final_value
    }
}

pub struct Builder<T, O>
where
    O: Outputs<Item = T>,
{
    inner: FST<T, O>,
}

pub struct Parser<T, O>
where
    O: Outputs<Item = T>,
{
    inner: FST<T, O>,
    stack: Stack<(Line<T>, usize)>,
}

impl<T, O> Parser<T, O>
where
    T: Eq,
    O: Outputs<Item = T>,
{
    fn new(outputs: O) -> Parser<T, O> {
        Parser {
            inner: FST::new(outputs),
            stack: Stack::new(),
        }
    }

    fn read(&mut self, decoder: &impl Codec<Item = T>, bf: &mut BytesMut) {
        for _ in 0..bf.get_u32() {
            let label = std::char::from_u32(CodecV32.read(bf).unwrap()).unwrap();
            let flag = bf.get_u8();
            let mut followers = ((flag & 0xF8) >> 3) as u32;
            if followers == 31 {
                followers = CodecV32.read(bf).unwrap();
            }
            let value = if flag & FLAG_HAS_VALUE != 0 {
                Some(decoder.read(bf).unwrap())
            } else {
                None
            };
            let final_value = if flag & FLAG_HAS_FINAL_VALUE != 0 {
                Some(decoder.read(bf).unwrap())
            } else {
                None
            };

            let current = Line {
                label,
                value,
                final_value,
                flag,
                nexts: LinkedList::new(),
            };
            let is_leaf = current.is_final() && followers < 1;
            if !is_leaf {
                self.stack.push((current, followers as usize));
                continue;
            }
            self.shift(current);
        }
    }

    pub fn shift(&mut self, current: Line<T>) {
        match self.stack.pop() {
            None => self.inner.lines.push_back(current),
            Some((mut parent, n)) => {
                parent.nexts.push_back(current);
                if parent.nexts.len() == n {
                    self.shift(parent);
                } else {
                    self.stack.push((parent, n));
                }
            }
        }
    }

    pub fn build(mut self) -> FST<T, O> {
        self.inner
    }
}

impl<T, O> Builder<T, O>
where
    T: Eq,
    O: Outputs<Item = T>,
{
    fn new(outputs: O) -> Builder<T, O> {
        Builder {
            inner: FST::new(outputs),
        }
    }

    pub fn push(mut self, key: &str, value: T) -> Self {
        self.inner.push(key, value);
        self
    }

    pub fn build(self) -> FST<T, O> {
        self.inner
    }
}
