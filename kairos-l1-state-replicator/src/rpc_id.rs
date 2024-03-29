use std::sync::atomic::{AtomicI64, Ordering};

const DEFAULT_STARTING_ID: i64 = 0;

pub struct JsonRpcIdGenerator {
    next_id: AtomicI64,
}

impl Default for JsonRpcIdGenerator {
    fn default() -> Self {
        // Assuming you want to start IDs at 0 by default
        JsonRpcIdGenerator::new(0)
    }
}

impl JsonRpcIdGenerator {
    pub fn new(starting_id: i64) -> Self {
        JsonRpcIdGenerator {
            next_id: AtomicI64::new(starting_id),
        }
    }

    pub fn next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}
