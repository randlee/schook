use sc_hooks_core::manifest::ResponseTimeRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AsyncBucketRange {
    pub(crate) min_ms: u64,
    pub(crate) max_ms: u64,
}

impl AsyncBucketRange {
    pub(crate) const fn new(min_ms: u64, max_ms: u64) -> Self {
        Self { min_ms, max_ms }
    }

    pub(crate) fn as_bucket(self) -> String {
        format!("{}-{}", self.min_ms, self.max_ms)
    }

    pub(crate) fn from_response_time(response_time: Option<&ResponseTimeRange>) -> Self {
        match response_time {
            Some(range) => Self::new(range.min_ms, range.max_ms),
            None => Self::new(0, 30_000),
        }
    }
}
