#[macro_use]
extern crate log;
extern crate jets;

use jets::io::{MemReader, MemWriter, Reader, Writer};
use jets::utils::fst::*;
use rand::prelude::*;
use std::collections::HashMap;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_fst_v32() {
    init();
    let mut rng = rand::thread_rng();
    let mut data: HashMap<String, u32> = HashMap::new();
    for _ in 0..100 {
        let k = rand_str(&mut rng, 32);
        let v: u32 = rng.gen_range(0, 10000);
        data.insert(k, v);
    }
    let mut keys: Vec<&str> = vec![];
    for k in data.keys() {
        keys.push(k)
    }
    keys.sort();
    let mut bu = FST::builder(OutputsU32);
    for k in keys {
        bu = bu.push(k, data.get(k).unwrap().clone());
    }
    let mut fst = bu.build();

    let mut writer = MemWriter::new();
    fst.save(&mut writer, CodecV32).unwrap();
    let mut reader = writer.to_bytes();

    fst = FST::decoder(OutputsU32, CodecV32)
        .decode(&mut reader)
        .unwrap();

    for (k, expect) in data.iter() {
        let actual = fst.get(k).unwrap();
        assert_eq!(*expect, actual);
    }
}

#[test]
fn test_fst_vec_v64() {
    init();
    let mut rng = rand::thread_rng();
    let mut data: HashMap<String, Vec<u64>> = HashMap::new();
    for _ in 0..100 {
        let k = rand_str(&mut rng, 32);
        let v: u32 = rng.gen_range(0, 10000);
        data.insert(k, vec![v as u64]);
    }
    let mut keys: Vec<&str> = vec![];
    for k in data.keys() {
        keys.push(k)
    }
    keys.sort();
    let mut bu = FST::builder(OutputsU64s);
    for k in keys {
        bu = bu.push(k, data.get(k).unwrap().clone());
    }
    let mut fst = bu.build();

    let mut writer = MemWriter::new();
    fst.save(&mut writer, CodecVecU64).unwrap();
    let mut reader = writer.to_bytes();

    fst = FST::decoder(OutputsU64s, CodecVecU64)
        .decode(&mut reader)
        .unwrap();

    for (k, expect) in data.iter() {
        let actual = fst.get(k).unwrap();
        assert_eq!(*expect, actual);
    }
}

#[test]
fn test_chinese() {
    let fst = FST::builder(OutputsU32)
        .push("好好学习", 1)
        .push("好一朵美丽的茉莉花", 2)
        .build();
    assert_eq!(1, fst.get("好好学习").unwrap());
    assert_eq!(2, fst.get("好一朵美丽的茉莉花").unwrap());
}

#[test]
fn test_fst_for() {
    init();
    let fst = FST::builder(OutputsU32s)
        .push("bar", vec![2, 3, 4])
        .push("baz", vec![5])
        .push("foo", vec![1, 2, 3])
        .build();
    let mut writer = MemWriter::new();
    fst.save(&mut writer, CodecVecU32OverFOR).unwrap();
    let mut bs = writer.to_bytes();
    let fst2 = FST::decoder(OutputsU32s, CodecVecU32OverFOR)
        .decode(&mut bs)
        .unwrap();
    for it in vec!["foo", "bar", "baz"] {
        info!("seek {}: {:?}", it, fst2.get(it).unwrap());
    }
}

fn rand_str(rng: &mut ThreadRng, max_len: usize) -> String {
    let mut s = String::new();
    let mut x: u8 = rng.gen_range(0x61, 0x7B);
    s.push(x as char);
    for _ in 0..rng.gen_range(0, max_len) {
        x = rng.gen_range(0x61, 0x7B);
        s.push(x as char);
    }
    s
}
