use super::doc::Document;
use super::store::DocumentStore;
use std::collections::LinkedList;

pub struct IndexWriter {
    docs: LinkedList<Document>,
    doc_store: DocumentStore,
}

pub struct IndexReader {}

impl IndexWriter {
    pub fn new(path: &str) -> IndexWriter {
        IndexWriter {
            doc_store: DocumentStore::open(path).unwrap(),
            docs: LinkedList::new(),
        }
    }

    pub fn add_document(&mut self, doc: Document) {
        self.docs.push_back(doc);
    }

    pub fn close(mut self) {
        unimplemented!();
    }
}

impl IndexReader {
    pub fn get_document(&self, id: u64) -> Option<&Document> {
        unimplemented!()
    }
}
