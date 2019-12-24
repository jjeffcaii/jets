use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) struct DocID {
    segment: u64,
    sequence: AtomicU32,
}

impl DocID {
    pub(crate) fn new(segment: u32, sequence: u32) -> DocID {
        DocID {
            segment: (segment as u64) << 32,
            sequence: AtomicU32::new(sequence),
        }
    }

    pub(crate) fn next(&mut self) -> u64 {
        let before = self.sequence.fetch_add(1, Ordering::SeqCst) as u64;
        self.segment | (before + 1)
    }

    pub(crate) fn wrap(&self, id: u32) -> u64 {
        self.segment | (id as u64)
    }

    pub(crate) fn reformat(segment: u32, id: u32) -> u64 {
        let segment64 = (segment as u64) << 32;
        segment64 | (id as u64)
    }
}
