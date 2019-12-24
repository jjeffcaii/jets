use crate::utils::FOR;

pub trait Outputs {
    type Item;
    fn zero(&self) -> Self::Item;
    fn add(&self, prefix: &Self::Item, output: &Self::Item) -> Self::Item;
    fn subtract(&self, output: &Self::Item, inc: &Self::Item) -> Self::Item;
    fn common(&self, a: &Self::Item, b: &Self::Item) -> Self::Item;
    fn merge(&self, prev: &Self::Item, next: &Self::Item) -> Self::Item;
}

pub struct OutputsU32s;
pub struct OutputsU32;
pub struct OutputsU64;
pub struct OutputsU64s;
pub struct OutputsFOR;

impl Outputs for OutputsU32 {
    type Item = u32;

    fn add(&self, prefix: &u32, output: &u32) -> u32 {
        prefix + output
    }

    fn subtract(&self, output: &u32, inc: &u32) -> u32 {
        output - inc
    }

    fn common(&self, a: &u32, b: &u32) -> u32 {
        if a > b {
            b.clone()
        } else {
            a.clone()
        }
    }

    fn zero(&self) -> u32 {
        0
    }

    fn merge(&self, _prev: &u32, next: &u32) -> u32 {
        next.clone()
    }
}

impl Outputs for OutputsU64 {
    type Item = u64;

    fn zero(&self) -> u64 {
        0
    }

    fn add(&self, prefix: &u64, output: &u64) -> u64 {
        prefix + output
    }

    fn subtract(&self, output: &u64, inc: &u64) -> u64 {
        output - inc
    }

    fn common(&self, a: &u64, b: &u64) -> u64 {
        if a > b {
            b.clone()
        } else {
            a.clone()
        }
    }

    fn merge(&self, _prev: &u64, next: &u64) -> u64 {
        next.clone()
    }
}

impl Outputs for OutputsFOR {
    type Item = FOR;

    fn add(&self, prefix: &FOR, output: &FOR) -> FOR {
        unimplemented!()
    }

    fn zero(&self) -> FOR {
        unimplemented!()
    }

    fn subtract(&self, output: &FOR, inc: &FOR) -> FOR {
        unimplemented!()
    }
    fn common(&self, a: &FOR, b: &FOR) -> FOR {
        unimplemented!()
    }
    fn merge(&self, prev: &FOR, next: &FOR) -> FOR {
        unimplemented!()
    }
}

impl Outputs for OutputsU32s {
    type Item = Vec<u32>;

    fn add(&self, prefix: &Vec<u32>, output: &Vec<u32>) -> Vec<u32> {
        let mut result = vec![];
        for it in prefix {
            result.push(it.clone());
        }
        for it in output {
            result.push(it.clone());
        }
        result
    }

    fn merge(&self, prev: &Vec<u32>, next: &Vec<u32>) -> Vec<u32> {
        self.add(prev, next)
    }

    fn subtract(&self, output: &Vec<u32>, inc: &Vec<u32>) -> Vec<u32> {
        if inc.is_empty() {
            return output.clone();
        }
        if output.len() == inc.len() {
            return self.zero();
        }
        if inc.len() > output.len() {
            panic!("inc.length={} vs output.length={}", inc.len(), output.len());
        }
        let mut results = vec![];
        for i in inc.len()..output.len() {
            results.push(output[i]);
        }
        results
    }

    fn common(&self, a: &Vec<u32>, b: &Vec<u32>) -> Vec<u32> {
        let mut results = vec![];
        let size = if a.len() > b.len() { b.len() } else { a.len() };
        for i in 0..size {
            if a[i] != b[i] {
                break;
            }
            results.push(a[i]);
        }
        results
    }

    fn zero(&self) -> Vec<u32> {
        vec![]
    }
}

impl Outputs for OutputsU64s {
    type Item = Vec<u64>;

    fn add(&self, prefix: &Vec<u64>, output: &Vec<u64>) -> Vec<u64> {
        let mut result = vec![];
        for it in prefix {
            result.push(it.clone());
        }
        for it in output {
            result.push(it.clone());
        }
        result
    }

    fn merge(&self, prev: &Vec<u64>, next: &Vec<u64>) -> Vec<u64> {
        self.add(prev, next)
    }

    fn subtract(&self, output: &Vec<u64>, inc: &Vec<u64>) -> Vec<u64> {
        if inc.is_empty() {
            return output.clone();
        }
        if output.len() == inc.len() {
            return self.zero();
        }
        if inc.len() > output.len() {
            panic!("inc.length={} vs output.length={}", inc.len(), output.len());
        }
        let mut results = vec![];
        for i in inc.len()..output.len() {
            results.push(output[i]);
        }
        results
    }

    fn common(&self, a: &Vec<u64>, b: &Vec<u64>) -> Vec<u64> {
        let mut results = vec![];
        let size = if a.len() > b.len() { b.len() } else { a.len() };
        for i in 0..size {
            if a[i] != b[i] {
                break;
            }
            results.push(a[i]);
        }
        results
    }

    fn zero(&self) -> Vec<u64> {
        vec![]
    }
}
