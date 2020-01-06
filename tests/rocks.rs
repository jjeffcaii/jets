extern crate jets;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options, WriteBatch, DB};

#[test]
fn test() {
    let path = "/tmp/jets/_rocks_test";
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    let db = DB::open(&opts, path).unwrap();

    let mut batch = WriteBatch::default();
    let cf1 = db.cf_handle("cf1").unwrap();
    for i in 0..1000 {
        let mut bf = BytesMut::with_capacity(4);
        bf.put_u32(i as u32);
        batch.put_cf(cf1, bf.to_vec(), b"hello world").unwrap();
    }
    db.write(batch).unwrap();

    // let _ = DB::destroy(&db_opts, path);
}
