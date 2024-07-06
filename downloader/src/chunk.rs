use std::ops::{Bound, RangeBounds};

#[derive(Debug, Copy, Clone)]
pub struct ChunkRange {
    pub start: u64,
    pub end: u64,
}

impl ChunkRange {
    pub fn new(start: u64, end: u64) -> Self {
        debug_assert!(end >= start, "start: {},end:{}", start, end);
        Self { start, end }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        (self.end - self.start) + 1
    }

    pub fn to_range_header(&self) -> headers::Range {
        headers::Range::bytes(self).unwrap()
    }

    pub fn from_len(start: u64, len: u64) -> Self {
        Self {
            start,
            end: start + len - 1,
        }
    }
}

impl std::fmt::Display for ChunkRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl<'a> RangeBounds<u64> for &'a ChunkRange {
    fn start_bound(&self) -> Bound<&u64> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&u64> {
        Bound::Included(&self.end)
    }
}
