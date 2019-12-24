use crate::io::Writer;
use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub trait Readable {
    fn read_from(&mut self, bf: &mut Bytes) -> Result<()>;
}

pub trait Writeable {
    fn write_to(&self, bf: &mut BytesMut) -> Result<()>;
}
