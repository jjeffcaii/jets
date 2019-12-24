use super::query::{Condition, GroupOp, Query};
use crate::core::DocValue;
use crate::core::Document;
use crate::core::IndexReader;
use crate::utils::Stack;
use std::collections::{HashMap, HashSet};

pub struct IndexSearcher {
    inner: IndexReader,
}

pub struct TopDocs<'a> {
    reader: &'a IndexReader,
    docs: Option<Vec<u64>>,
}

impl<'a> TopDocs<'a> {
    pub fn documents(&'a self) -> Option<Vec<Document>> {
        match &self.docs {
            Some(ids) => {
                let mut vv = vec![];
                for id in ids {
                    if let Some(d) = self.reader.document(*id) {
                        vv.push(d);
                    }
                }
                Some(vv)
            }
            None => None,
        }
    }
}

impl From<IndexReader> for IndexSearcher {
    fn from(reader: IndexReader) -> IndexSearcher {
        IndexSearcher { inner: reader }
    }
}

enum RuntimeCond {
    Group,
    Bingo(Vec<u64>),
}

impl IndexSearcher {
    pub fn search<'a>(&'a self, query: &Query) -> TopDocs<'a> {
        let mut stack: Stack<RuntimeCond> = Stack::new();
        self.process(query.root(), &mut stack);
        let docs = match stack.pop() {
            None => None,
            Some(r) => match r {
                RuntimeCond::Bingo(result) => {
                    if result.len() < 1 {
                        None
                    } else {
                        Some(result)
                    }
                }
                RuntimeCond::Group => unreachable!(),
            },
        };
        TopDocs {
            reader: &self.inner,
            docs: docs,
        }
    }

    fn process(&self, cond: &Condition, stack: &mut Stack<RuntimeCond>) {
        match cond {
            Condition::Term(k, v) => match self.inner.find(&k, DocValue::Text(v.clone())) {
                Some(found) => stack.push(RuntimeCond::Bingo(found)),
                None => stack.push(RuntimeCond::Bingo(vec![])),
            },
            Condition::Group(op, conds) => {
                stack.push(RuntimeCond::Group);
                for next in conds {
                    self.process(next, stack);
                }
                let mut holder = vec![];
                while let Some(RuntimeCond::Bingo(bingo)) = stack.pop() {
                    holder.push(bingo);
                }
                // TODO: tuning: use FOR skip table or bitsets filter.
                let merge: Vec<u64> = match op {
                    GroupOp::AND => {
                        let should = holder.len();
                        let mut map: HashMap<u64, usize> = HashMap::new();
                        for each in holder {
                            for n in each {
                                if map.contains_key(&n) {
                                    let val = map.get(&n).unwrap() + 1;
                                    map.insert(n, val);
                                } else {
                                    map.insert(n, 1);
                                }
                            }
                        }
                        let mut result = vec![];
                        for (k, v) in map.into_iter() {
                            if v >= should {
                                result.push(k);
                            }
                        }
                        result.sort();
                        result
                    }
                    GroupOp::OR => {
                        let mut sets = HashSet::new();
                        for it in holder {
                            sets.insert(it);
                        }
                        let mut v = vec![];
                        for it in sets.into_iter() {
                            for n in it {
                                v.push(n);
                            }
                        }
                        v.sort();
                        v
                    }
                };
                stack.push(RuntimeCond::Bingo(merge));
            }
        }
    }
}
