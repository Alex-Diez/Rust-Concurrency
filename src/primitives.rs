use std::sync::{Condvar, Mutex};

struct LatchStatus {
    counts: usize
}

impl LatchStatus {

    pub fn new(counts: usize) -> LatchStatus {
        LatchStatus { counts: counts }
    }
}

pub struct CountDownLatch {
    sync: Mutex<LatchStatus>,
    condition: Condvar
}

impl CountDownLatch {
    
    pub fn new(counts: usize) -> CountDownLatch {
        CountDownLatch { sync: Mutex::new(LatchStatus::new(counts)), condition: Condvar::new() }
    }

    pub fn await(&self) {
        let mut guard = self.sync.lock().unwrap();
        if guard.counts > 0 {
            guard = self.condition.wait(guard).unwrap();
        }
    }

    pub fn count_down(&self) {
        let mut guard = self.sync.lock().unwrap();
        guard.counts -= 1;
        if guard.counts == 0 {
            self.condition.notify_all();
        }
    }

    pub fn get_counts(&self) -> usize {
        self.sync.lock().unwrap().counts
    }
}
