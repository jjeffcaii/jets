use super::doc::Document;
use super::spi::{Readable, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rocksdb::{IteratorMode, Options, Snapshot, WriteBatch, DB};
use std::error::Error;
use std::result::Result;

pub struct DocumentStore {
    db: DB,
}

impl DocumentStore {
    pub(crate) fn open(path: &str) -> Result<DocumentStore, Box<dyn Error>> {
        match DB::open_default(path) {
            Ok(db) => Ok(DocumentStore { db }),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub(crate) fn get(&self, id: u64) -> Result<Option<Document>, Box<dyn Error>> {
        match self.db.get(Self::get_u64_bytes(id)) {
            Ok(Some(raw)) => {
                let mut doc = Document::new(id);
                let bf = Bytes::from(raw);
                match doc.read_from(&bf) {
                    Ok(()) => Ok(Some(doc)),
                    Err(e) => Err(e),
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub(crate) fn write(&self, doc: Document) -> Result<(), Box<dyn Error>> {
        let id = Self::get_u64_bytes(doc.get_id());
        let mut bf = BytesMut::new();
        doc.write_to(&mut bf)?;
        match self.db.put(id, bf.to_vec()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    #[inline]
    fn get_u64_bytes(n: u64) -> [u8; 8] {
        let mut b: [u8; 8] = [0; 8];
        for i in 0..8 {
            b[i] = (n >> (i * 8) & 0xFF) as u8;
        }
        b
    }
}
