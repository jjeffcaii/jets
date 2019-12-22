use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error;
use std::result::Result;

pub trait Codec {
    type Item;

    fn write(&self, bf: &mut BytesMut, value: &Self::Item) -> Result<(), Box<dyn Error>>;
    fn read(&self, bf: &mut BytesMut) -> Result<Self::Item, Box<dyn Error>>;
}

// https://www.elastic.co/blog/frame-of-reference-and-roaring-bitmaps
pub struct CodecFOR;

impl CodecFOR {
    fn bits(n: u32) -> u8 {
        let mut mask: u32 = 0x80000000;
        let mut j = 0;
        while mask & n == 0 {
            mask = mask >> 1;
            j += 1;
        }
        32 - j
    }
}

impl Codec for CodecFOR {
    type Item = Vec<u32>;

    fn write(&self, bf: &mut BytesMut, value: &Vec<u32>) -> Result<(), Box<dyn Error>> {
        let amount = value.len();
        let mut deltas = vec![];
        for i in 0..amount {
            if i == 0 {
                deltas[i] = value[i];
            } else {
                deltas[i] = value[i] - value[i - 1];
            }
        }
        let mut i = 0;
        while i < amount {
            let mut result: Vec<u8> = vec![];
            let first = (deltas[i], Self::bits(deltas[i]));
            let second = (deltas[i + 1], Self::bits(deltas[i + 1]));
            let third = (deltas[i + 2], Self::bits(deltas[i + 2]));
            let mut avg = std::cmp::max(std::cmp::max(first.1, second.1), third.1);
        }
        unimplemented!()
    }

    fn read(&self, bf: &mut BytesMut) -> Result<Vec<u32>, Box<dyn Error>> {
        unimplemented!()
    }
}

pub struct CodecV32;

impl Codec for CodecV32 {
    type Item = u32;

    fn write(&self, bf: &mut BytesMut, input: &u32) -> Result<(), Box<dyn Error>> {
        let mut value: u32 = input.clone();
        while value & !0x7F != 0 {
            bf.put_u8(((value & 0x7F) as u8) | 0x80);
            value >>= 7;
        }
        bf.put_u8((value & 0xFF) as u8);
        Ok(())
    }

    fn read(&self, bf: &mut BytesMut) -> Result<Self::Item, Box<dyn Error>> {
        let mut b = bf.get_u8();
        if b & 0x80 == 0 {
            return Ok(b as u32);
        }
        let mut i: u32 = (b & 0x7F) as u32;
        b = bf.get_u8();
        i |= ((b & 0x7F) as u32) << 7;
        if b & 0x80 == 0 {
            return Ok(i);
        }

        b = bf.get_u8();
        i |= ((b & 0x7F) as u32) << 14;
        if b & 0x80 == 0 {
            return Ok(i);
        }

        b = bf.get_u8();
        i |= ((b & 0x7F) as u32) << 21;
        if b & 0x80 == 0 {
            return Ok(i);
        }

        b = bf.get_u8();
        i |= ((b & 0x7F) as u32) << 18;
        if b & 0x80 == 0 {
            return Ok(i);
        }
        Err("invalid v32 detected: too many bits.".into())
    }
}
