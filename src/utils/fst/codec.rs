use crate::utils::FOR;
use crate::utils::{get_v32, put_v32};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error;
use std::result::Result;

pub trait Codec {
    type Item;

    fn write(&self, bf: &mut BytesMut, value: &Self::Item) -> Result<(), Box<dyn Error>>;
    fn read(&self, bf: &mut Bytes) -> Result<Self::Item, Box<dyn Error>>;
}

pub struct CodecVecU32;

impl Codec for CodecVecU32 {
    type Item = Vec<u32>;

    fn write(&self, bf: &mut BytesMut, value: &Vec<u32>) -> Result<(), Box<dyn Error>> {
        bf.put_u32(value.len() as u32);
        for it in value {
            bf.put_u32(*it);
        }
        Ok(())
    }

    fn read(&self, bf: &mut Bytes) -> Result<Vec<u32>, Box<dyn Error>> {
        let mut results = vec![];
        let l = bf.get_u32();
        for _ in 0..l {
            results.push(bf.get_u32());
        }
        Ok(results)
    }
}

pub struct CodecVecU64;

impl Codec for CodecVecU64 {
    type Item = Vec<u64>;

    fn write(&self, bf: &mut BytesMut, value: &Vec<u64>) -> Result<(), Box<dyn Error>> {
        bf.put_u32(value.len() as u32);
        for it in value {
            bf.put_u64(*it);
        }
        Ok(())
    }

    fn read(&self, bf: &mut Bytes) -> Result<Vec<u64>, Box<dyn Error>> {
        let mut results = vec![];
        let l = bf.get_u32();
        for _ in 0..l {
            results.push(bf.get_u64());
        }
        Ok(results)
    }
}

pub struct CodecFOR;

impl Codec for CodecFOR {
    type Item = FOR;

    fn write(&self, bf: &mut BytesMut, value: &FOR) -> Result<(), Box<dyn Error>> {
        value.write_to(bf)
    }

    fn read(&self, bf: &mut Bytes) -> Result<FOR, Box<dyn Error>> {
        FOR::decode(bf)
    }
}

pub struct CodecVecU32OverFOR;

impl Codec for CodecVecU32OverFOR {
    type Item = Vec<u32>;

    fn write(&self, bf: &mut BytesMut, value: &Vec<u32>) -> Result<(), Box<dyn Error>> {
        let convert = FOR::from(value);
        convert.write_to(bf)
    }

    fn read(&self, bf: &mut Bytes) -> Result<Vec<u32>, Box<dyn Error>> {
        let for32 = FOR::decode(bf)?;
        Ok(for32.iter().collect())
    }
}

pub struct CodecV32;

impl Codec for CodecV32 {
    type Item = u32;

    fn write(&self, bf: &mut BytesMut, input: &u32) -> Result<(), Box<dyn Error>> {
        put_v32(bf, *input)
    }

    fn read(&self, bf: &mut Bytes) -> Result<Self::Item, Box<dyn Error>> {
        get_v32(bf)
    }
}
