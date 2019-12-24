mod jieba;
mod stopwords;

// https://nitschinger.at/Text-Analysis-in-Rust-Tokenization/
pub trait Tokenizer {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<&'a str>;
}

pub use jieba::JiebaTokenizer;
pub use stopwords::{StopWords, StopWordsCN};
