#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod analysis;
pub mod core;
pub mod io;
pub mod search;
pub mod spi;
pub mod utils;

pub mod prelude {
    pub use crate::analysis::{StopWords, Tokenizer};
    pub use crate::core::{
        DocValue, Document, Field, IndexReader, IndexWriter, FLAG_NOT_STORED, FLAG_TOKENIZED,
    };
    pub use crate::search::{Condition, IndexSearcher, Query};
}
