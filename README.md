# Jets

Jets is a search engine toolkit written in Rust. (未完工, 纯属业余自娱自乐)

### TODO

#### Milestone 1

- [ ] Basic
  - [x] FST
  - [x] Analyser? [jieba-rs](https://github.com/messense/jieba-rs), [cedarwood](https://github.com/MnO2/cedarwood)
  - [ ] Posting list: FOR codec.
  - [x] Doc Storage
    - [x] LSM: [rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)
    - [x] Storage format: column based.
- [ ] Index
  - [x] Write
  - [x] Search
  - [ ] Delete
  - [ ] Update
- [ ] Advance
  - [ ] Segment Merge
  - [ ] Score
  - [ ] Position
- [ ] Data Types
  - [x] Text
  - [ ] Numbers
  - [ ] Geo: Geohash

#### Milestone 2

- [ ] Cluster
  - [ ] Gossip
  - [ ] Sharding && Replica
  - [ ] ...

### Similar Projects

- [rucene](https://github.com/zhihu/rucene)
- [sonic](https://github.com/valeriansaliou/sonic)
