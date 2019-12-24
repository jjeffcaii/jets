use crate::io::Writer;
use crate::spi::Result;
use bytes::{Buf, BufMut, Bytes};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

pub(crate) struct FieldInfo {
    id: u32,
    kind: u8,
    name: String,
}

pub(crate) struct FieldInfoManager {
    fields_map: Arc<RwLock<HashMap<String, u32>>>,
    fields: Vec<FieldInfo>,
}

#[derive(Default)]
pub(crate) struct MetadataManager {
    magic: u32,
    segments: AtomicU32,
    fields_manager: FieldInfoManager,
}

impl FieldInfo {
    pub(crate) fn get_id(&self) -> u32 {
        self.id
    }

    pub(crate) fn get_kind(&self) -> u8 {
        self.kind
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

impl Default for FieldInfoManager {
    fn default() -> FieldInfoManager {
        FieldInfoManager {
            fields_map: Arc::new(RwLock::new(Default::default())),
            fields: Default::default(),
        }
    }
}

impl FieldInfoManager {
    pub(crate) fn put(&mut self, name: &str, kind: u8) -> Result<u32> {
        let mut m = self.fields_map.write().unwrap();
        match m.get(name) {
            Some(n) => {
                let info = &self.fields[*n as usize];
                if info.kind != kind {
                    return Err("conflict field type!".into());
                }
                Ok(*n)
            }
            None => {
                let i = self.fields.len() as u32;
                self.fields.push(FieldInfo {
                    id: i,
                    name: name.to_string(),
                    kind: kind,
                });
                m.insert(name.to_string(), i);
                Ok(i)
            }
        }
    }

    pub(crate) fn list(&self) -> &Vec<FieldInfo> {
        &self.fields
    }

    pub(crate) fn get(&self, n: u32) -> Option<&FieldInfo> {
        self.fields.get(n as usize)
    }

    pub(crate) fn search(&self, name: &str) -> Option<&FieldInfo> {
        let m = self.fields_map.read().unwrap();
        match m.get(name) {
            Some(n) => self.get(*n as u32),
            None => None,
        }
    }
}

impl MetadataManager {
    pub(crate) fn open(path: &str) -> Result<MetadataManager> {
        let mut f = File::open(path)?;
        let mut all = vec![];
        f.read_to_end(&mut all)?;
        let mut reader = Bytes::from(all);
        let magic = reader.get_u32();
        let segment = reader.get_u32();
        let totals = reader.get_u32();
        let mut fm = FieldInfoManager::default();
        for _ in 0..totals {
            let kind = reader.get_u8();
            let name_len = reader.get_u32();
            let name = String::from_utf8(reader.split_to(name_len as usize).to_vec())?;
            fm.put(&name, kind)?;
        }
        Ok(MetadataManager {
            magic,
            segments: AtomicU32::new(segment),
            fields_manager: fm,
        })
    }

    pub(crate) fn write(&mut self, writer: &mut impl Writer) -> Result<()> {
        writer.put_u32(self.magic);
        writer.put_u32(self.segments.load(Ordering::SeqCst));
        let _ = self.fields_manager.fields_map.write().unwrap();
        let fields = self.fields_manager.list();
        writer.put_u32(fields.len() as u32);
        for it in fields.iter() {
            writer.put_u8(it.kind);
            let b = it.name.as_bytes();
            writer.put_u32(b.len() as u32);
            writer.put_slice(b);
        }
        Ok(())
    }

    pub(crate) fn fields_mut(&mut self) -> &mut FieldInfoManager {
        &mut self.fields_manager
    }

    pub(crate) fn fields(&self) -> &FieldInfoManager {
        &self.fields_manager
    }

    pub(crate) fn next_segment(&mut self) -> u32 {
        self.segments.fetch_add(1, Ordering::SeqCst)
    }
}
