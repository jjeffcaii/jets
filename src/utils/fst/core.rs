use super::codec::Codec;
use super::outputs::Outputs;
use crate::io::Writer;
use crate::spi::Result;
use crate::utils::{get_v32, put_v32, Stack};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

const FLAG_FINAL: u8 = 0x01 << 0;
const FLAG_HAS_VALUE: u8 = 0x01 << 1;
const FLAG_HAS_FINAL_VALUE: u8 = 0x01 << 2;

pub struct FST<T, O>
where
    O: Outputs<Item = T>,
{
    outputs: O,
    lines: Vec<Line<T>>,
}

pub struct Line<T> {
    label: u8,
    value: Option<T>,
    flag: u8,
    final_value: Option<T>,
    nexts: Vec<Line<T>>,
}

impl<T, O> FST<T, O>
where
    T: Eq,
    O: Outputs<Item = T>,
{
    pub fn builder(outputs: O) -> Builder<T, O> {
        Builder::new(outputs)
    }

    pub fn decoder<D>(outputs: O, decoder: D) -> Decoder<T, O, D>
    where
        D: Codec<Item = T>,
    {
        Decoder::new(outputs, decoder)
    }

    fn new(outputs: O) -> FST<T, O> {
        FST {
            outputs,
            lines: vec![],
        }
    }

    pub fn traverse(&self, f: impl Fn(&Line<T>)) {
        for it in &self.lines {
            it.traverse(&f);
        }
    }

    pub fn get<K>(&self, key: K) -> Option<T>
    where
        K: AsRef<[u8]>,
    {
        if self.lines.is_empty() {
            return None;
        }
        let mut chars = key.as_ref();
        if chars.len() > 0 {
            let first = chars[0];
            chars = &chars[1..];
            if let Some(bingo) = binary_search(&self.lines, first) {
                let it = &self.lines[bingo];
                let mut sum = self.outputs.zero();
                if it.sum(chars, &mut sum, &self.outputs) {
                    return Some(sum);
                }
            }
        }
        None
    }

    pub fn save(&self, writer: &mut impl Writer, encoder: impl Codec<Item = T>) -> Result<usize> {
        let amount: Rc<AtomicU32> = Rc::new(Default::default());
        let wrote = Rc::new(AtomicU32::new(4));
        let buff = Rc::new(Mutex::new(BytesMut::new()));
        self.traverse(|next: &Line<T>| {
            amount.fetch_add(1, Ordering::SeqCst);
            let mut bf = buff.lock().unwrap();
            bf.put_u8(next.get_label());
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
                put_v32(&mut bf, follower_amount).unwrap();
            }
            if flag & FLAG_HAS_VALUE != 0 {
                if let Some(v) = &next.value {
                    &encoder.write(&mut bf, v).unwrap();
                }
            }
            if flag & FLAG_HAS_FINAL_VALUE != 0 {
                if let Some(v) = &next.final_value {
                    encoder.write(&mut bf, v).unwrap();
                }
            }
            wrote.fetch_add(bf.len() as u32, Ordering::SeqCst);
        });
        writer.put_u32(amount.load(Ordering::SeqCst));
        writer.put_slice(buff.lock().unwrap().bytes());
        Ok(4 + wrote.load(Ordering::SeqCst) as usize)
    }

    fn push<K>(&mut self, key: K, value: T)
    where
        K: AsRef<[u8]>,
    {
        let mut chars = key.as_ref();
        if chars.len() < 1 {
            return;
        }
        let first = chars[0];
        chars = &chars[1..];
        for it in &mut self.lines {
            if it.label == first {
                it.push(chars, Some(value), &self.outputs);
                return;
            }
        }
        let mut newborn = Line::new(first);
        newborn.push(chars, Some(value), &self.outputs);
        self.lines.push(newborn);
    }
}

impl<T> Line<T>
where
    T: Eq,
{
    #[inline]
    fn new(label: u8) -> Line<T> {
        Line {
            label,
            value: None,
            final_value: None,
            flag: 0,
            nexts: vec![],
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

    fn push(&mut self, mut chars: &[u8], insert: Option<T>, op: &impl Outputs<Item = T>) {
        let actual = self.push_pre(insert, op);
        if chars.len() < 1 {
            // check final stat
            self.flag |= FLAG_FINAL;
            if let Some(value) = actual {
                self.set_final_value(value, op);
            }
            return;
        }
        let ch = chars[0];
        chars = &chars[1..];
        for next in &mut self.nexts {
            if next.label == ch {
                next.push(chars, actual, op);
                return;
            }
        }
        let mut newborn = Line::new(ch);
        newborn.push(chars, actual, op);
        self.nexts.push(newborn);
    }

    #[inline]
    fn traverse(&self, f: &impl Fn(&Line<T>)) {
        f(self);
        for next in &self.nexts {
            next.traverse(f);
        }
    }

    #[inline]
    fn sum(&self, mut chars: &[u8], results: &mut T, op: &impl Outputs<Item = T>) -> bool {
        if let Some(value) = &self.value {
            *results = op.add(results, value);
        }
        if chars.len() < 1 {
            if self.flag & FLAG_FINAL == 0 {
                return false;
            }
            if let Some(v) = &self.final_value {
                *results = op.add(results, v);
            }
            return true;
        }
        let ch = chars[0];
        chars = &chars[1..];
        if let Some(bingo) = binary_search(&self.nexts, ch) {
            let next = &self.nexts[bingo];
            return next.sum(chars, results, op);
        }
        false
    }

    pub fn get_nexts(&self) -> &Vec<Line<T>> {
        &self.nexts
    }

    pub fn get_label(&self) -> u8 {
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

pub struct Decoder<T, O, D>
where
    O: Outputs<Item = T>,
    D: Codec<Item = T>,
{
    inner: FST<T, O>,
    decoder: D,
    stack: Stack<(Line<T>, usize)>,
}

impl<T, O, D> Decoder<T, O, D>
where
    T: Eq,
    O: Outputs<Item = T>,
    D: Codec<Item = T>,
{
    #[inline]
    fn new(outputs: O, decoder: D) -> Decoder<T, O, D> {
        Decoder {
            inner: FST::new(outputs),
            decoder,
            stack: Stack::new(),
        }
    }

    pub fn decode(mut self, bf: &mut Bytes) -> Result<FST<T, O>> {
        for _ in 0..bf.get_u32() {
            let label = bf.get_u8();
            let flag = bf.get_u8();
            let mut followers = ((flag & 0xF8) >> 3) as u32;
            if followers == 31 {
                followers = get_v32(bf).unwrap();
            }
            let value = if flag & FLAG_HAS_VALUE != 0 {
                Some(self.decoder.read(bf).unwrap())
            } else {
                None
            };
            let final_value = if flag & FLAG_HAS_FINAL_VALUE != 0 {
                Some(self.decoder.read(bf).unwrap())
            } else {
                None
            };
            let current = Line {
                label,
                value,
                final_value,
                flag,
                nexts: vec![],
            };
            let is_leaf = current.is_final() && followers < 1;
            if !is_leaf {
                self.stack.push((current, followers as usize));
                continue;
            }
            self.shift(current);
        }
        Ok(self.inner)
    }

    #[inline]
    fn shift(&mut self, current: Line<T>) {
        match self.stack.pop() {
            None => self.inner.lines.push(current),
            Some((mut parent, n)) => {
                parent.nexts.push(current);
                if parent.nexts.len() == n {
                    self.shift(parent);
                } else {
                    self.stack.push((parent, n));
                }
            }
        }
    }
}

impl<T, O> Builder<T, O>
where
    T: Eq,
    O: Outputs<Item = T>,
{
    #[inline]
    fn new(outputs: O) -> Builder<T, O> {
        Builder {
            inner: FST::new(outputs),
        }
    }

    pub fn push<K>(mut self, key: K, value: T) -> Self
    where
        K: AsRef<[u8]>,
    {
        self.inner.push(key, value);
        self
    }

    pub fn build(self) -> FST<T, O> {
        self.inner
    }
}

#[inline]
fn binary_search<T>(inputs: &Vec<Line<T>>, target: u8) -> Option<usize> {
    match inputs.binary_search_by(|v| v.label.cmp(&target)) {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}
