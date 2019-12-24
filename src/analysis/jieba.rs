use super::Tokenizer;
use jieba_rs::Jieba;

#[derive(Default)]
pub struct JiebaTokenizer {
    inner: Jieba,
}

impl Tokenizer for JiebaTokenizer {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<&'a str> {
        self.inner.cut_for_search(input, false)
    }
}
