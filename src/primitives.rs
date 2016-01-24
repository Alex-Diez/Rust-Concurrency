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

/// A synchronization aid that allows one or more threads to wait until a set 
/// of operations being performed in other threads completes.
/// A CountDownLatch is initialized with a given count. The await methods block 
/// until the current count reaches zero due to invocations of the count_down() 
/// method, after which all waiting threads are released and any subsequent 
/// invocations of await return immediately. This is a one-shot phenomenon --
/// the count cannot be reset.
pub struct CountDownLatch {
    sync: Mutex<LatchStatus>,
    condition: Condvar
}

impl CountDownLatch {
    
    /// Create new CountDownLatch with specified counts
    pub fn new(counts: usize) -> CountDownLatch {
        CountDownLatch {
            sync: Mutex::new(LatchStatus::new(counts)),
            condition: Condvar::new()
        }
    }

    /// Block thread until number of counts is zero
    pub fn await(&self) {
        let mut guard = self.sync.lock().unwrap();
        while guard.counts > 0 {
            guard = self.condition.wait(guard).unwrap();
        }
    }

    /// Decrease number of counts on '1'
    pub fn count_down(&self) {
        let mut guard = self.sync.lock().unwrap();
        guard.counts -= 1;
        if guard.counts == 0 {
            self.condition.notify_all();
        }
    }

    /// Get current number of counts
    pub fn get_counts(&self) -> usize {
        self.sync.lock().unwrap().counts
    }
}

/// An RAII guard which will release a resource acquired from a semaphore 
/// when dropped.
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

/// A counting, blocking, semaphore.
/// Semaphores are a form of atomic counter where access is only granted 
/// if the counter is a positive value. Each acquisition will block the calling 
/// thread until the counter is positive, and each release will increment the 
/// counter and unblock any threads if necessary.
/// Semaphores are often used to restrict the number of threads than can access 
/// some (physical or logical) resource. For example, here is a class that uses 
/// a semaphore to control access to a pool of items:
pub struct Semaphore {
    sync: Mutex<SemaphoreState>,
    condition: Condvar
}

impl Semaphore {
    
    /// Create new Semaphore with specified number of permissions
    pub fn new(permissions: usize) -> Semaphore {
        Semaphore {
            sync: Mutex::new(SemaphoreState::new(permissions)),
            condition: Condvar::new()
        }
    }

    /// Acquire permission from Semaphore
    /// Block current thread if no permission left
    pub fn acquire(&self) -> SemaphoreGuard {
        let mut lock = self.sync.lock().unwrap();
        while lock.permissions < 1 {
            lock = self.condition.wait(lock).unwrap();
        }
        lock.permissions -= 1;
        SemaphoreGuard::new(self)
    }

    /// Try acquire permission
    /// Does not block thread
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

    /// Release Semaphore permission
    /// Notify all threads that wait for permission
    /// @see Semaphore::acquire
    pub fn release(&self) {
        let mut lock = self.sync.lock().unwrap();
        if lock.permissions < lock.max_permissions {
            lock.permissions += 1;
            self.condition.notify_all();
        }
    }
}
