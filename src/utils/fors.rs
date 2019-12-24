use super::misc::{get_v32, put_v32};
use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::LinkedList;
use std::fmt;

const FLAG_MORE: u8 = 0x80;
const FLAG_SIZED: u8 = 0x40;
const BLOCK_SIZE: usize = 128;

// https://www.elastic.co/blog/frame-of-reference-and-roaring-bitmaps
#[derive(Debug)]
pub struct FOR {
    chunk: usize,
    blocks: Vec<Block>,
}

#[derive(Debug)]
struct Block {
    num_bits: u8,
    inner: Bits,
}

struct FORIter<'a> {
    i: usize,
    block: Option<BlockIterator<'a>>,
    inner: &'a FOR,
}

struct BlockIterator<'a> {
    index: usize,
    totals: usize,
    inner: &'a Block,
    sum: u32,
}

impl<'a> From<&'a FOR> for FORIter<'a> {
    fn from(input: &'a FOR) -> FORIter<'a> {
        FORIter {
            i: 0,
            block: None,
            inner: input,
        }
    }
}

impl<'a> Iterator for FORIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        match self.block.as_mut() {
            None => {
                if self.i >= self.inner.blocks.len() {
                    return None;
                }
                let next_block = self.inner.blocks[self.i].inner_iter();
                self.block = Some(next_block);
                self.next()
            }
            Some(found) => match found.next() {
                Some(v) => Some(v),
                None => {
                    self.i += 1;
                    self.block = None;
                    self.next()
                }
            },
        }
    }
}

impl<A> From<A> for FOR
where
    A: AsRef<[u32]>,
{
    fn from(inputs: A) -> FOR {
        FOR::new(BLOCK_SIZE, inputs.as_ref())
    }
}

impl FOR {
    pub fn decode(b: &mut Bytes) -> Result<FOR> {
        let mut blocks: Vec<Block> = vec![];
        let mut more = true;
        while more {
            Self::decode_block(b, &mut blocks, &mut more)?;
        }
        Ok(FOR {
            chunk: BLOCK_SIZE,
            blocks,
        })
    }

    pub fn new(mut chunk: usize, inputs: &[u32]) -> FOR {
        if chunk == 0 {
            chunk = BLOCK_SIZE;
        }
        let mut blocks = vec![];
        let mut i = 0;
        while i < inputs.len() {
            let mut or: u32 = 0;
            let mut diffs: LinkedList<u32> = Default::default();
            let cycle = std::cmp::min(inputs.len() - i, chunk);
            for j in 0..cycle {
                let k = i + j;
                let v = if j == 0 {
                    inputs[k]
                } else {
                    inputs[k] - inputs[k - 1]
                };
                or |= v;
                diffs.push_back(v);
            }
            let bits = calculate_bits(or);
            blocks.push(Block::new(bits, diffs.into_iter()));
            i += cycle;
        }
        FOR { chunk, blocks }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        FORIter::from(self)
    }

    pub fn write_to(&self, bf: &mut BytesMut) -> Result<()> {
        let n = self.blocks.len();
        for (i, block) in self.blocks.iter().enumerate() {
            let is_last = i == n - 1;
            block.write(bf, self.chunk, is_last)?;
        }
        Ok(())
    }

    pub fn bytes(&self) -> Result<Bytes> {
        let mut bf = BytesMut::new();
        self.write_to(&mut bf)?;
        Ok(bf.freeze())
    }

    #[inline]
    fn decode_block(b: &mut Bytes, blocks: &mut Vec<Block>, has_more: &mut bool) -> Result<()> {
        let header = b.get_u8();
        let num_bits = header & 0b00111111;
        let amount = if header & FLAG_SIZED != 0 {
            get_v32(b)? as usize
        } else {
            BLOCK_SIZE
        };

        let (n, cursor) = {
            let bits = (num_bits as usize) * amount;
            let c = bits & 7;
            let n = if c == 0 { bits / 8 } else { 1 + bits / 8 };
            (n, c as u8)
        };
        let block = Block {
            num_bits,
            inner: Bits::new(cursor, b.split_to(n).to_vec()),
        };
        blocks.push(block);
        *has_more = header & FLAG_MORE != 0;
        Ok(())
    }
}

impl<'a> BlockIterator<'a> {
    fn new(f: &'a Block) -> BlockIterator<'a> {
        BlockIterator {
            index: 0,
            inner: f,
            totals: f.len(),
            sum: 0,
        }
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.index >= self.totals {
            None
        } else {
            self.sum += self.inner.get(self.index);
            self.index += 1;
            Some(self.sum)
        }
    }
}

impl Block {
    fn new(num_bits: u8, block: impl Iterator<Item = u32>) -> Block {
        let mut bits: Bits = Default::default();
        for n in block {
            bits.push_u32(n, num_bits);
        }
        Block {
            num_bits,
            inner: bits,
        }
    }

    #[inline]
    fn inner_iter<'a>(&'a self) -> BlockIterator<'a> {
        BlockIterator::new(self)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        BlockIterator::new(self)
    }

    fn write(&self, bf: &mut BytesMut, chunk: usize, is_last: bool) -> Result<()> {
        // 0(FLAG_MORE) 0(FLAG_SIZED) 0 0 0 0 0 0 (lower 6 bits is num bits)
        let mut header = 0u8;
        header |= self.num_bits;
        let amount = self.inner.len() / (self.num_bits as usize);
        if amount != chunk {
            header |= FLAG_SIZED;
        }
        if !is_last {
            header |= FLAG_MORE;
        }
        bf.put_u8(header);
        if header & FLAG_SIZED != 0 {
            put_v32(bf, amount as u32)?;
        }
        bf.put_slice(self.inner.get_bytes());
        Ok(())
    }

    fn len(&self) -> usize {
        8 * self.inner.inner.len() / (self.num_bits as usize)
    }

    fn get(&self, i: usize) -> u32 {
        let offset = i * (self.num_bits as usize);
        self.inner.get_u32(offset, self.num_bits as usize)
    }
}

// fn number_of_leading_zeros_old(mut i: u32) -> u8 {
//     let mut mask: u32 = 0x8000_0000;
//     let mut n: u8 = 0;
//     while mask > 0 && mask & i == 0 {
//         n += 1;
//         mask >>= 1;
//     }
//     n
// }

#[inline]
fn calculate_bits(input: u32) -> u8 {
    if input == 0 {
        return 1;
    }
    let n = number_of_leading_zeros(input);
    32 - n
}

#[inline]
fn number_of_leading_zeros(mut i: u32) -> u8 {
    if i == 0 {
        return 32;
    }
    let mut n = 1u32;
    if i >> 16 == 0 {
        n += 16;
        i <<= 16;
    }
    if i >> 24 == 0 {
        n += 8;
        i <<= 8;
    }
    if i >> 28 == 0 {
        n += 4;
        i <<= 4;
    }
    if i >> 30 == 0 {
        n += 2;
        i <<= 2;
    }
    n -= i >> 31;
    n as u8
}

#[derive(Default)]
pub struct Bits {
    inner: Vec<u8>,
    cursor: u8,
}

impl fmt::Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for b in self.as_ref() {
            s.push_str(&format!(" {:08b}", b))
        }
        write!(f, "({}){}", self.cursor, s)
    }
}

impl AsRef<[u8]> for Bits {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl Bits {
    pub fn new(cursor: u8, bs: Vec<u8>) -> Bits {
        Bits { cursor, inner: bs }
    }

    pub fn len(&self) -> usize {
        let mut amount = self.inner.len() * 8;
        if self.cursor > 0 {
            amount -= 8;
            amount += self.cursor as usize;
        }
        amount
    }

    pub fn get(&self, offset: usize) -> Option<bool> {
        if offset >= self.len() {
            return None;
        }
        if offset == 0 {
            // info!("get: offset={},value={}", offset, self.inner[0] & 0x80 != 0);
            return Some(self.inner[0] & 0x80 != 0);
        }

        let cursor = (offset & 7) as u8;
        let slot = offset >> 3;
        let target = self.inner[slot];
        let n = target & (0x80 >> cursor);
        // info!(
        //     "get: offset={}, loc=[{},{}], target={:08b}, value={}",
        //     offset,
        //     slot,
        //     cursor,
        //     target,
        //     if n != 0 { 1 } else { 0 }
        // );
        Some(n != 0)
    }

    pub fn get_u32(&self, offset: usize, n: usize) -> u32 {
        let mut v = 0u32;
        for i in 0..n {
            match self.get(offset + i) {
                None => {
                    return v;
                }
                Some(ok) => {
                    v <<= 1;
                    if ok {
                        v |= 1;
                    }
                }
            }
        }
        v
    }

    pub fn get_cursor(&self) -> u8 {
        self.cursor
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.inner[..]
    }

    pub fn push_u32(&mut self, value: u32, expand: u8) {
        let b = value.to_be_bytes();
        // let sb = format!("{:028b}", value);
        // info!("++++ TOBE: value={}, b={}, expand={}", value, sb, expand);
        if expand <= 8 {
            self.push(b[3], expand);
        } else if expand <= 16 {
            self.push(b[2], expand - 8);
            self.push(b[3], 8);
        } else if expand <= 24 {
            self.push(b[1], expand - 16);
            self.push(b[2], 8);
            self.push(b[3], 8);
        } else {
            self.push(b[0], expand - 24);
            self.push(b[1], 8);
            self.push(b[2], 8);
            self.push(b[3], 8);
        }
    }

    pub fn push(&mut self, value: u8, expand: u8) {
        let c = self.cursor & 7;
        if c == 0 {
            if expand == 8 {
                self.inner.push(value)
            } else {
                // 00000101 expand=5 -> 01010000
                self.cursor = expand;
                self.inner.push(value << (8 - expand));
            }
        } else {
            let lefts = 8 - c;
            let expanded = value << (8 - expand);
            if expand <= lefts {
                *self.inner.last_mut().unwrap() |= expanded >> c;
                self.cursor += expand;
            } else {
                let merge = expanded >> (8 - lefts);
                *self.inner.last_mut().unwrap() |= merge;
                self.inner.push(expanded << lefts);
                self.cursor = expand - lefts;
            }
        }
    }
}
