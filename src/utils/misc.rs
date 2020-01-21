use crate::spi::Result;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use mac_address::get_mac_address;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;

lazy_static! {
    static ref MAC_ADDR: [u8; 6] = { get_mac_address().unwrap().unwrap().bytes() };
}

pub struct GUIDGenerator {
    seq: AtomicU32,
    last_time: Mutex<u64>,
}

impl Default for GUIDGenerator {
    fn default() -> GUIDGenerator {
        let seed = rand::random::<u32>();
        GUIDGenerator {
            seq: AtomicU32::new(seed),
            last_time: Mutex::new(0),
        }
    }
}

impl GUIDGenerator {
    pub fn next_b64(&self) -> String {
        let x = self.next();
        base64::encode_config(&x[..], base64::URL_SAFE_NO_PAD)
    }

    pub fn next(&self) -> [u8; 15] {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let mut timestamp = Self::now();

        let mut last_time = self.last_time.lock().unwrap();

        if *last_time > timestamp {
            timestamp = *last_time;
        }
        if seq == 0 {
            timestamp += 1;
        }
        *last_time = timestamp;
        let mut b: [u8; 15] = [0; 15];
        Self::put_u64(&mut b, timestamp, 0, 6);
        for i in 0..6 {
            b[6 + i] = MAC_ADDR[i];
        }
        Self::put_u64(&mut b, seq as u64, 12, 3);
        b
    }

    #[inline]
    fn now() -> u64 {
        return SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    #[inline]
    fn put_u64(b: &mut [u8; 15], n: u64, pos: usize, size: usize) {
        for i in 0..size {
            let x = n >> (i * 8);
            b[pos + size - i - 1] = x as u8;
        }
    }
}

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
