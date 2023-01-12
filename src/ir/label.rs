use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Label(u64);

impl Label {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "label {}", self.0)
    }
}
