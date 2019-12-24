use super::doc::{DocValue, Document, Field};
use super::doc::{FLAG_NOT_STORED, FLAG_TOKENIZED};
use super::metadata::*;
use super::misc::DocID;
use super::store::DocValueStore;
use crate::analysis::{StopWords, StopWordsCN, Tokenizer};
use crate::io::{FileWriter, Writer};
use crate::spi::Result;
use crate::utils::fst::*;
use crate::utils::Stack;
use bytes::{Buf, Bytes};
use glob::glob;
use multimap::MultiMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct IndexWriter<A>
where
    A: Tokenizer,
{
    dir: String,
    amounts: u64,
    metadata: MetadataManager,
    store: DocValueStore,
    values: MultiMap<u32, (DocValue, u32, u8)>,
    tokenizer: A,
    sequence: AtomicU32,
}

pub struct IndexReader {
    metadata: MetadataManager,
    segments: HashMap<String, Segment>,
    store: DocValueStore,
}

type SegmentFST = FST<Vec<u32>, OutputsU32s>;

struct Segment {
    id: u32,
    inner: HashMap<u32, SegmentFST>,
}

impl Segment {
    fn open<P>(path: P) -> Result<Segment>
    where
        P: AsRef<Path>,
    {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);
        // TODO: read one by one.
        let mut all = vec![];
        let _ = reader.read_to_end(&mut all)?;
        let mut bf = Bytes::from(all);
        let segment_id = bf.get_u32();
        let mut inner: HashMap<u32, SegmentFST> = HashMap::new();
        while !bf.is_empty() {
            let f_index = bf.get_u32();
            let fst = FST::decoder(OutputsU32s, CodecVecU32OverFOR).decode(&mut bf)?;
            inner.insert(f_index, fst);
        }
        Ok(Segment {
            id: segment_id,
            inner,
        })
    }

    fn find<R>(&self, field: u32, key: R) -> Option<Vec<u64>>
    where
        R: AsRef<[u8]>,
    {
        match self.inner.get(&field) {
            Some(fst) => match fst.get(key) {
                None => None,
                Some(origin) => {
                    let mut result = vec![];
                    for id in origin {
                        result.push(DocID::reformat(self.id, id));
                    }
                    Some(result)
                }
            },
            None => None,
        }
    }
}

impl<A> IndexWriter<A>
where
    A: Tokenizer,
{
    pub fn open(path: &str, tokenizer: A) -> Result<IndexWriter<A>> {
        let store = DocValueStore::open(&get_data_path(path))?;
        let metadata_path = get_metadata_path(path);
        let metadata = {
            let p = Path::new(&metadata_path);
            if !p.exists() {
                Ok(MetadataManager::default())
            } else {
                MetadataManager::open(&metadata_path)
            }
        }?;
        Ok(IndexWriter {
            dir: path.to_string(),
            amounts: 0,
            metadata: metadata,
            store,
            values: Default::default(),
            tokenizer,
            sequence: Default::default(),
        })
    }

    pub fn push(&mut self, doc: Document) -> Result<()> {
        for it in doc.fields {
            let i = self
                .metadata
                .fields_mut()
                .put(&it.name, it.value.get_type())?;
            let field_id = self.sequence.fetch_add(1, Ordering::SeqCst);
            self.values.insert(i, (it.value, field_id, it.flag));
        }
        self.amounts += 1;
        Ok(())
    }

    pub fn counter(&self) -> u64 {
        self.amounts
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.values.is_empty() {
            return Ok(());
        }
        let segment = self.metadata.next_segment();
        let origin = mem::replace(&mut self.values, Default::default());
        let path = format!("{}/_segment_{:08}.index", &self.dir, segment);
        let mut writer = FileWriter::open(path)?;
        writer.put_u32(segment);
        for (findex, values) in origin.into_iter() {
            for (dv, id, flag) in values.iter() {
                let real_id = DocID::reformat(segment, *id);
                if flag & FLAG_NOT_STORED == 0 {
                    self.store.write(real_id, findex, dv)?;
                }
            }
            let processed = self.process(values);
            let mut stack: Stack<(String, u32)> = Stack::new();
            let mut builder = FST::builder(OutputsU32s);
            for (cur, id) in processed {
                if let Some((prev, i)) = stack.pop() {
                    if prev == cur {
                        stack.push((prev, i));
                    } else {
                        let mut ids = vec![];
                        ids.push(i);
                        while let Some(top) = (&mut stack).pop() {
                            ids.push(top.1);
                        }
                        builder = builder.push(prev, to_sorted_unique_ids(ids));
                    }
                }
                stack.push((cur, id));
            }

            if let Some((k, v)) = stack.pop() {
                let mut ids = vec![];
                ids.push(v);
                while let Some(top) = stack.pop() {
                    ids.push(top.1);
                }
                builder = builder.push(k, to_sorted_unique_ids(ids));
            }

            writer.put_u32(findex);
            // generate segment
            let fst = builder.build();
            fst.save(&mut writer, CodecVecU32OverFOR)?;
            writer.flush()?;
        }
        let mut metadata_writer = FileWriter::open(&get_metadata_path(&self.dir))?;
        self.metadata.write(&mut metadata_writer)?;
        metadata_writer.flush()?;
        Ok(())
    }

    #[inline]
    fn process(&self, values: Vec<(DocValue, u32, u8)>) -> Vec<(String, u32)> {
        let mut results = vec![];
        for (v, id, flag) in values {
            match v {
                DocValue::Text(text) => {
                    if flag & FLAG_TOKENIZED != 0 {
                        for word in self.tokenizer.tokenize(&text) {
                            if !StopWordsCN.contains(word) {
                                results.push((word.to_string(), id));
                            }
                        }
                    } else {
                        results.push((text.to_string(), id));
                    }
                }
            }
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }
}

impl IndexReader {
    pub fn open(path: &str) -> Result<IndexReader> {
        let db_path = get_data_path(path);
        let ok = {
            let p = Path::new(&db_path);
            p.exists() && p.is_dir()
        };
        if !ok {
            return Err("open index failed!".into());
        }
        let mut segments = HashMap::new();
        let store = DocValueStore::open(&db_path)?;
        let metadata_path = get_metadata_path(path);
        let metadata = MetadataManager::open(&metadata_path)?;
        for it in glob(&format!("{}/_segment_*.index", path))? {
            let target: PathBuf = it?;
            let k = target.file_name().unwrap().to_str().unwrap().to_string();
            let segment = Segment::open(target)?;
            segments.insert(k, segment);
        }
        Ok(IndexReader {
            metadata,
            store,
            segments,
        })
    }

    pub fn find<R>(&self, field: &str, value: R) -> Option<Vec<u64>>
    where
        R: AsRef<[u8]>,
    {
        match self.metadata.fields().search(field) {
            Some(info) => {
                let mut merge = vec![];
                for (_k, segment) in self.segments.iter() {
                    if let Some(mut found) = segment.find(info.get_id(), &value) {
                        merge.append(&mut found);
                    }
                }
                Some(merge)
            }
            None => None,
        }
    }

    pub fn document(&self, id: u64) -> Option<Document> {
        let fields = self.metadata.fields().list();
        if fields.len() < 1 {
            return None;
        }
        let mut bu = Document::builder(id);
        for field in fields {
            if let Ok(Some(dv)) = self.store.get(id, field.get_id(), field.get_kind()) {
                bu = bu.put(field.get_name(), dv, 0);
            }
        }
        let doc = bu.build();
        if doc.is_empty() {
            None
        } else {
            Some(doc)
        }
    }
}

#[inline]
fn get_data_path(dir: &str) -> String {
    format!("{}/data", dir)
}

#[inline]
fn get_metadata_path(dir: &str) -> String {
    format!("{}/METADATA", dir)
}

#[inline]
fn to_sorted_unique_ids<T>(mut ids: Vec<T>) -> Vec<T>
where
    T: Ord + Copy,
{
    if ids.len() < 1 {
        ids
    } else {
        ids.sort();
        Some(ids[0])
            .into_iter()
            .chain(ids.windows(2).filter(|w| w[0] != w[1]).map(|w| w[1]))
            .collect()
    }
}
