#[macro_use]
extern crate log;
extern crate jets;

use jets::utils::fst::{DefaultOutputs, Outputs, FST};
use rand::prelude::*;
use std::collections::HashMap;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn basic_fst() {
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
    let mut bu = FST::builder(DefaultOutputs::u32());
    for k in keys {
        bu = bu.push(k, data.get(k).unwrap().clone());
    }
    let fst = bu.build();
    for (k, expect) in data.iter() {
        let actual = fst.get(k).unwrap();
        assert_eq!(*expect, actual);
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
