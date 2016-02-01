use std::sync::{Condvar, Mutex, Arc};
use std::sync::atomic::{AtomicUsize, Ordering};

use std::ops::Drop;

use std::option::Option;

use std::fmt::{Debug, Formatter, Result};

struct LatchInner {
    mutex: Mutex<()>,
    counts: AtomicUsize,
    condition: Condvar
}

impl LatchInner {

    fn new(counts: usize) -> LatchInner {
        LatchInner {
            mutex: Mutex::new(()),
            counts: AtomicUsize::new(counts),
            condition: Condvar::new()
        }
    }

    fn await(&self) {
        let mut guard = self.mutex.lock().unwrap();
        while self.get_counts() > 0 {
            guard = self.condition.wait(guard).unwrap();
        }
    }

    fn count_down(&self) {
        let guard = self.mutex.lock().unwrap();
        self.counts.fetch_sub(1, Ordering::Relaxed);
        if self.get_counts() == 0 {
            self.condition.notify_all();
        }
        drop(guard);
    }

    fn get_counts(&self) -> usize {
        self.counts.load(Ordering::Relaxed)
    }
}

/// A synchronization aid that allows one or more threads to wait until a set 
/// of operations being performed in other threads completes.
/// A CountDownLatch is initialized with a given count. The await methods block 
/// until the current count reaches zero due to invocations of the count_down() 
/// method, after which all waiting threads are released and any subsequent 
/// invocations of await return immediately. This is a one-shot phenomenon --
/// the count cannot be reset.
#[derive(Clone)]
pub struct CountDownLatch {
    inner: Arc<LatchInner>
}

impl CountDownLatch {
    
    /// Create new CountDownLatch with specified counts
    pub fn new(counts: usize) -> CountDownLatch {
        CountDownLatch {
            inner: Arc::new(LatchInner::new(counts))
        }
    }

    /// Block thread until number of counts is zero
    pub fn await(&self) {
        self.inner.await();
    }

    /// Decrease number of counts on '1'
    pub fn count_down(&self) {
        self.inner.count_down();
    }

    /// Get current number of counts
    pub fn get_counts(&self) -> usize {
        self.inner.get_counts()
    }
}

/// An RAII guard which will release a resource acquired from a semaphore 
/// when dropped.
pub struct SemaphoreGuard<'owner> {
    owner: &'owner SemaphoreInner
}

impl <'owner> SemaphoreGuard<'owner> {
    
    fn new(inner: &'owner SemaphoreInner) -> SemaphoreGuard<'owner> {
        SemaphoreGuard { owner: inner }
    }
}

impl <'owner> Drop for SemaphoreGuard<'owner> {

    fn drop(&mut self) {
        self.owner.release();
    }
}

impl <'owner> Debug for SemaphoreGuard<'owner> {

    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "[Semaphore Guard]")
    }
}

struct SemaphoreInner {
    mutex: Mutex<()>,
    permissions: AtomicUsize,
    max_permissions: usize,
    condition: Condvar
}

impl SemaphoreInner {

    fn new(permissions: usize) -> SemaphoreInner {
        SemaphoreInner {
            mutex: Mutex::new(()),
            permissions: AtomicUsize::new(permissions),
            max_permissions: permissions,
            condition: Condvar::new()
        }
    }

    fn acquire(&self) -> SemaphoreGuard {
        let mut guard = self.mutex.lock().unwrap();
        while self.get_permissions_number() < 1 {
            guard = self.condition.wait(guard).unwrap();
        }
        self.decrease_permission();
        SemaphoreGuard::new(self)
    }

    fn try_acquire(&self) -> Option<SemaphoreGuard> {
        let try_guard = self.mutex.try_lock();
        match try_guard {
            Ok(_) => {
                if self.get_permissions_number() > 0 {
                    self.decrease_permission();
                    Some(SemaphoreGuard::new(self))
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }

    fn release(&self) {
        let guard = self.mutex.lock().unwrap();
        if self.get_permissions_number() < self.max_permissions {
            self.increase_permission();
            self.condition.notify_all();
        }
        drop(guard);
    }

    fn get_permissions_number(&self) -> usize {
        self.permissions.load(Ordering::Relaxed)
    }

    fn decrease_permission(&self) {
        self.permissions.fetch_sub(1, Ordering::Relaxed);
    }

    fn increase_permission(&self) {
        self.permissions.fetch_add(1, Ordering::Relaxed);
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
#[derive(Clone)]
pub struct Semaphore {
    inner: Arc<SemaphoreInner>
}

impl Semaphore {
    
    /// Create new Semaphore with specified number of permissions
    pub fn new(permissions: usize) -> Semaphore {
        Semaphore {
            inner: Arc::new(SemaphoreInner::new(permissions))
        }
    }

    /// Acquire permission from Semaphore
    /// Block current thread if no permission left
    pub fn acquire(&self) -> SemaphoreGuard {
        self.inner.acquire()
    }

    /// Try acquire permission
    /// Does not block thread
    pub fn try_acquire(&self) -> Option<SemaphoreGuard> {
        self.inner.try_acquire()
    }

    /// Release Semaphore permission
    /// Notify all threads that wait for permission
    /// @see Semaphore::acquire
    pub fn release(&self) {
        self.inner.release()
    }
}
