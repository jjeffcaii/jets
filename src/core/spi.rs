use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error;
use std::result::Result;

pub trait Readable {
    fn read_from(&mut self, bf: &Bytes) -> Result<(), Box<dyn Error>>;
}

pub trait Writeable {
    fn write_to(&self, bf: &mut BytesMut) -> Result<(), Box<dyn Error>>;
}
