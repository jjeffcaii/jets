use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub trait Reader {
    fn get_u8(&mut self) -> u8;

    fn get_u32(&mut self) -> u32 {
        let mut result = 0;
        result += (self.get_u8() as u32) << 24;
        result += (self.get_u8() as u32) << 16;
        result += (self.get_u8() as u32) << 8;
        result += self.get_u8() as u32;
        result
    }

    fn get_v32(&mut self) -> u32 {
        let mut b = self.get_u8();
        if b & 0x80 == 0 {
            return b as u32;
        }
        let mut i: u32 = (b & 0x7F) as u32;
        b = self.get_u8();
        i |= ((b & 0x7F) as u32) << 7;
        if b & 0x80 == 0 {
            return i;
        }

        b = self.get_u8();
        i |= ((b & 0x7F) as u32) << 14;
        if b & 0x80 == 0 {
            return i;
        }

        b = self.get_u8();
        i |= ((b & 0x7F) as u32) << 21;
        if b & 0x80 == 0 {
            return i;
        }

        b = self.get_u8();
        i |= ((b & 0x7F) as u32) << 18;
        if b & 0x80 == 0 {
            return i;
        }
        panic!("invalid v32 detected: too many bits.")
    }
}

pub trait Writer {
    fn put_slice<B>(&mut self, input: B)
    where
        B: AsRef<[u8]>;
    fn put_u8(&mut self, input: u8);

    fn put_u16(&mut self, input: u16) {
        self.put_slice(input.to_be_bytes());
    }

    fn put_u32(&mut self, input: u32) {
        self.put_slice(input.to_be_bytes());
    }

    fn put_v32(&mut self, mut input: u32) {
        let mut b: Vec<u8> = vec![];
        while input & !0x7F != 0 {
            b.push(((input & 0x7F) as u8) | 0x80);
            input >>= 7;
        }
        b.push((input & 0xFF) as u8);
        self.put_slice(&b[..]);
    }
}

pub struct MemWriter {
    inner: BytesMut,
}

pub struct MemReader {
    inner: Bytes,
}

pub struct FileWriter {
    inner: BufWriter<File>,
}

pub struct FileReader {
    inner: BufReader<File>,
}

impl MemReader {
    pub fn new(b: Bytes) -> MemReader {
        MemReader { inner: b }
    }
}

impl MemWriter {
    pub fn new() -> MemWriter {
        MemWriter {
            inner: BytesMut::new(),
        }
    }

    pub fn to_bytes(self) -> Bytes {
        self.inner.freeze()
    }

    pub fn freeze(self) -> impl Reader {
        MemReader::new(self.inner.freeze())
    }
}

impl Reader for MemReader {
    fn get_u8(&mut self) -> u8 {
        self.inner.get_u8()
    }
}

impl Writer for MemWriter {
    fn put_slice<B>(&mut self, input: B)
    where
        B: AsRef<[u8]>,
    {
        self.inner.put_slice(input.as_ref());
    }

    fn put_u8(&mut self, input: u8) {
        self.inner.put_u8(input);
    }

    fn put_u16(&mut self, input: u16) {
        self.inner.put_u16(input);
    }

    fn put_u32(&mut self, input: u32) {
        self.inner.put_u32(input);
    }
}

impl FileWriter {
    pub fn open<P>(path: P) -> Result<FileWriter>
    where
        P: AsRef<Path>,
    {
        let f = File::create(path)?;
        Ok(FileWriter {
            inner: BufWriter::new(f),
        })
    }

    pub fn flush(&mut self) -> Result<()> {
        match self.inner.flush() {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Writer for FileWriter {
    fn put_slice<B>(&mut self, input: B)
    where
        B: AsRef<[u8]>,
    {
        self.inner.write(input.as_ref()).unwrap();
    }

    fn put_u8(&mut self, input: u8) {
        let b: [u8; 1] = [input];
        self.inner.write(&b[..]).unwrap();
    }
}

impl FileReader {
    pub fn open<P>(path: P) -> Result<FileReader>
    where
        P: AsRef<Path>,
    {
        let f = File::open(path)?;
        Ok(FileReader {
            inner: BufReader::new(f),
        })
    }
}

impl Reader for FileReader {
    fn get_u8(&mut self) -> u8 {
        let mut bf = [0; 1];
        self.inner.read_exact(&mut bf).unwrap();
        bf[0]
    }

    fn get_u32(&mut self) -> u32 {
        let mut bf = [0; 4];
        self.inner.read_exact(&mut bf).unwrap();
        let mut result = 0;
        result += (bf[3] as u32) << 24;
        result += (bf[2] as u32) << 16;
        result += (bf[1] as u32) << 8;
        result += bf[0] as u32;
        result
    }
}
