#[macro_use]
extern crate log;
extern crate jets;

use jets::core::{DocValue, DocValueStore, Document, Field};
use std::time::SystemTime;

const amount: usize = 10;

fn init() {
    let _ = env_logger::builder()
        .format_timestamp_millis()
        .is_test(true)
        .try_init();
}

#[test]
fn test_document_store() {
    init();
    let path = format!("/Users/jeffsky/jets/test_store");
    let mut store = DocValueStore::open(&path).unwrap();
    for i in 0..amount {
        match store.get(i as u64, 0, 0) {
            Ok(Some(v)) => {
                info!("query #{}: {:?}", i, v);
            }
            Ok(None) => {
                warn!("query #{}: None", i);
            }
            Err(e) => {
                error!("query #{} failed: {}", i, e);
            }
        }
    }
}
