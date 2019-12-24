use std::time::SystemTime;

mod fors;
pub mod fst;
mod misc;
mod stack;

pub use fors::FOR;
pub use misc::*;
pub use stack::Stack;

pub fn unique_id() -> u64 {
    // TODO: find a good snowflake implementataion.
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
