use std::sync::{Condvar, Mutex};
use std::ops::Drop;
use std::option::Option;

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
        while guard.counts > 0 {
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

pub struct SemaphoreGuard<'owner> {
    lock: &'owner Semaphore
}

impl <'owner> SemaphoreGuard<'owner> {
    
    pub fn new(semaphore: &'owner Semaphore) -> SemaphoreGuard<'owner> {
        SemaphoreGuard { lock: semaphore }
    }
}

impl <'owner> Drop for SemaphoreGuard<'owner> {

    fn drop(&mut self) {
        self.lock.release();
    }
}

struct SemaphoreState {
    permissions: usize,
    max_permissions: usize
}

impl SemaphoreState {

    fn new(permissions: usize) -> SemaphoreState {
        SemaphoreState {
            permissions: permissions,
            max_permissions: permissions
        }
    }
}

pub struct Semaphore {
    sync: Mutex<SemaphoreState>,
    condition: Condvar
}

impl Semaphore {
    
    pub fn new(permissions: usize) -> Semaphore {
        Semaphore {
            sync: Mutex::new(SemaphoreState::new(permissions)),
            condition: Condvar::new()
        }
    }

    pub fn acquire(&self) -> SemaphoreGuard {
        let mut lock = self.sync.lock().unwrap();
        while lock.permissions < 1 {
            lock = self.condition.wait(lock).unwrap();
        }
        lock.permissions -= 1;
        SemaphoreGuard::new(self)
    }

    pub fn try_acquire(&self) -> Option<SemaphoreGuard> {
        let try_lock = self.sync.try_lock();
        match try_lock {
            Ok(mut d) => if d.permissions > 0 {
                            d.permissions -= 1;
                            Some(SemaphoreGuard::new(self))
                        } else {
                            None
                        },
            Err(_) => None,
        }
    }

    pub fn release(&self) {
        let mut lock = self.sync.lock().unwrap();
        if lock.permissions < lock.max_permissions {
            lock.permissions += 1;
            self.condition.notify_all();
        }
    }
}
