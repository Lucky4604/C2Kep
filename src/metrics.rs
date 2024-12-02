use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Metrics {
    produced: AtomicUsize,
    consumed: AtomicUsize,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            produced: AtomicUsize::new(0),
            consumed: AtomicUsize::new(0),
        }
    }

    pub fn increment_produced(&self) {
        self.produced.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_consumed(&self) {
        self.consumed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_produced(&self) -> usize {
        self.produced.load(Ordering::SeqCst)
    }

    pub fn get_consumed(&self) -> usize {
        self.consumed.load(Ordering::SeqCst)
    }
}