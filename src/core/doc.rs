use super::spi::{Readable, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::LinkedList;
use std::error::Error;
use std::result::Result;

pub enum FieldValue {
    String(String),
    Text(String),
}

pub struct Field {
    name: String,
    value: FieldValue,
}

pub struct Document {
    id: u64,
    fields: LinkedList<Field>,
}

impl Document {
    pub fn new(id: u64) -> Document {
        Document {
            id,
            fields: LinkedList::new(),
        }
    }

    pub fn push(&mut self, field: Field) {
        self.fields.push_back(field);
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}

impl Readable for Document {
    fn read_from(&mut self, bf: &Bytes) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

impl Writeable for Document {
    fn write_to(&self, bf: &mut BytesMut) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
