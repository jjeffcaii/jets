use rocksdb::{Error, IteratorMode, Options, Snapshot, WriteBatch, DB};

#[test]
fn test() {
    let path = "/Users/jeffsky/foobar/rocks";
    let mut db = DB::open_default(path).unwrap();
    for i in 0..10000 {
        let k = format!("key_{}", i);
        let v = format!("val_{}", i);
        db.put(k, v).unwrap();
    }
}
