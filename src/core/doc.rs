use super::spi::{Readable, Writeable};
use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::{HashMap, LinkedList};
use std::fmt;

pub const FIELD_TYPE_TEXT: u8 = 1;

pub const FLAG_NOT_STORED: u8 = 0x01;
pub const FLAG_TOKENIZED: u8 = 0x01 << 1;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum DocValue {
    Text(String),
}

impl<A> From<A> for DocValue
where
    A: Into<String>,
{
    fn from(s: A) -> DocValue {
        DocValue::Text(s.into())
    }
}

impl fmt::Display for DocValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DocValue::Text(s) => write!(f, "{}", s),
        }
    }
}

impl AsRef<[u8]> for DocValue {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Text(s) => s.as_ref(),
        }
    }
}

impl DocValue {
    pub fn decode(field_type: u8, raw: Vec<u8>) -> Result<DocValue> {
        match field_type {
            FIELD_TYPE_TEXT => match String::from_utf8(raw) {
                Ok(s) => Ok(DocValue::Text(s)),
                Err(e) => Err(Box::new(e)),
            },
            _ => Err("invalid field type".into()),
        }
    }

    pub fn get_type(&self) -> u8 {
        match self {
            Self::Text(_) => FIELD_TYPE_TEXT,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            DocValue::Text(s) => Vec::from(s.as_bytes()),
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub(crate) name: String,
    pub(crate) value: DocValue,
    pub(crate) flag: u8,
}

#[derive(Debug)]
pub struct Document {
    pub(crate) id: u64,
    pub(crate) fields: LinkedList<Field>,
}

pub struct DocumentBuilder {
    inner: Document,
}

impl DocumentBuilder {
    fn new(id: u64) -> DocumentBuilder {
        DocumentBuilder {
            inner: Document::new(id),
        }
    }

    pub fn put<A>(mut self, name: A, value: DocValue, flag: u8) -> Self
    where
        A: Into<String>,
    {
        let f = Field::new(name.into(), value, flag);
        self.put_field(f)
    }

    pub fn put_field(mut self, field: Field) -> Self {
        self.inner.push(field);
        self
    }

    pub fn build(self) -> Document {
        self.inner
    }
}

impl Field {
    pub fn new<A>(name: A, value: DocValue, flag: u8) -> Field
    where
        A: Into<String>,
    {
        Field {
            name: name.into(),
            value,
            flag,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_value(&self) -> &DocValue {
        &self.value
    }
}

impl Document {
    pub(crate) fn new(id: u64) -> Document {
        Document {
            id,
            fields: LinkedList::new(),
        }
    }

    pub fn builder(id: u64) -> DocumentBuilder {
        DocumentBuilder::new(id)
    }

    pub(crate) fn push(&mut self, field: Field) {
        self.fields.push_back(field);
    }

    pub fn get_fields(&self) -> &LinkedList<Field> {
        &self.fields
    }

    pub fn get(&self, name: &str) -> Option<&DocValue> {
        for it in self.fields.iter() {
            if it.get_name() == name {
                return Some(it.get_value());
            }
        }
        None
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}
