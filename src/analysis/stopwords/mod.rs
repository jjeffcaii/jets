mod cn;

pub trait StopWords {
    fn contains(&self, word: &str) -> bool;
}

pub use cn::StopWordsCN;
