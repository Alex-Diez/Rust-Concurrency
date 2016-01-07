use std::option::Option;
use std::collections::HashMap;
use std::sync::{Condvar, Mutex};

pub struct CountDownLatch {
    sync: Mutex<LatchStatus>,
    condition: Condvar
}

struct LatchStatus {
    counts: usize
}

impl LatchStatus {

    pub fn new(counts: usize) -> LatchStatus {
        LatchStatus { counts: counts }
    }
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

pub struct ResourceHolderSemaphore {
    resources: HashMap<usize, usize>,
    current_permissions: usize,
}

impl ResourceHolderSemaphore {
    pub fn new(permissions: usize) -> ResourceHolderSemaphore {
        ResourceHolderSemaphore {
            resources: HashMap::with_capacity(permissions),
            current_permissions: permissions
        }
    }

    pub fn acquire(&mut self) -> usize {
        self.acuqire_resource().unwrap_or(0)
    }

    pub fn try_acquire(&mut self) -> Option<usize> {
        self.acuqire_resource()
    }

    pub fn release(&mut self) {
        self.current_permissions += 1;
    }

    pub fn update(&mut self, index: usize, val: usize) {
        if self.resources.contains_key(&index) {
            let res_val = self.resources.get_mut(&index);
            *(res_val.unwrap()) = val;
        } else {
            self.resources.insert(index, val);
        }
    }

    pub fn get(&self, index: usize) -> usize {
        self.resources.get(&index).map(|v| *v).unwrap_or(0)
    }

    fn acuqire_resource(&mut self) -> Option<usize> {
        if self.current_permissions > 0 {
            let res = self.current_permissions;
            self.current_permissions -= 1;
            Some(res)
        } else {
            None
        }   
    }
}
