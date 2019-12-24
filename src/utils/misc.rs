use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub fn get_v32(bf: &mut Bytes) -> Result<u32> {
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
    i |= ((b & 0x7F) as u32) << 28;
    if b & 0x80 == 0 {
        return Ok(i);
    }
    Err("invalid v32 detected: too many bits.".into())
}

pub fn put_v32(bf: &mut BytesMut, mut value: u32) -> Result<()> {
    while value & !0x7F != 0 {
        bf.put_u8(((value & 0x7F) as u8) | 0x80);
        value >>= 7;
    }
    bf.put_u8((value & 0xFF) as u8);
    Ok(())
}

pub fn encode_v32(mut value: u32) -> Vec<u8> {
    let mut result = vec![];
    while value & !0x7F != 0 {
        result.push(((value & 0x7F) as u8) | 0x80);
        value >>= 7;
    }
    result.push((value & 0xFF) as u8);
    result
}

pub fn put_v64(bf: &mut BytesMut, mut value: u64) -> Result<()> {
    while value & !0x7F != 0 {
        let cur = ((value & 0x7F) as u8) | 0x80;
        bf.put_u8(cur);
        value >>= 7;
    }
    bf.put_u8((value & 0xFF) as u8);
    Ok(())
}

pub fn get_v64(bf: &mut Bytes) -> Result<u64> {
    let mut b = bf.get_u8();
    if b & 0x80 == 0 {
        return Ok(b as u64);
    }
    let mut i = (b & 0x7F) as u64;
    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 7;
    if b & 0x80 == 0 {
        return Ok(i);
    }

    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 14;
    if b & 0x80 == 0 {
        return Ok(i);
    }

    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 21;
    if b & 0x80 == 0 {
        return Ok(i);
    }

    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 28;
    if b & 0x80 == 0 {
        return Ok(i);
    }
    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 35;
    if b & 0x80 == 0 {
        return Ok(i);
    }

    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 42;
    if b & 0x80 == 0 {
        return Ok(i);
    }

    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 49;
    if b & 0x80 == 0 {
        return Ok(i);
    }

    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 56;
    if b & 0x80 == 0 {
        return Ok(i);
    }
    b = bf.get_u8();
    i |= ((b & 0x7F) as u64) << 63;
    if b & 0x80 == 0 {
        return Ok(i);
    }
    Err("invalid v64 detected: too many bits.".into())
}
