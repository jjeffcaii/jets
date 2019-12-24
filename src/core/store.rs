use super::doc::DocValue;
use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rocksdb::DB;

const ROW_KEY_METADATA: [u8; 1] = [0];

pub struct DocValueStore {
    db: DB,
}

impl DocValueStore {
    pub fn open(path: &str) -> Result<DocValueStore> {
        match DB::open_default(path) {
            Ok(db) => Ok(DocValueStore { db }),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn get(&self, id: u64, field: u32, field_type: u8) -> Result<Option<DocValue>> {
        let row = Self::to_row_key(id, field);
        match self.db.get(row) {
            Ok(Some(raw)) => Ok(Some(DocValue::decode(field_type, raw)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn write(&self, id: u64, field: u32, value: &DocValue) -> Result<()> {
        let row = Self::to_row_key(id, field);
        match self.db.put(row, value.bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    #[inline]
    fn to_row_key(id: u64, field: u32) -> [u8; 12] {
        let mut b: [u8; 12] = [0; 12];
        for i in 0..8 {
            b[i] = ((id >> (i * 8)) & 0xFF) as u8;
        }
        for i in 0..4 {
            b[i + 8] = ((field >> (i * 8)) & 0xFF) as u8;
        }
        b
    }
}
