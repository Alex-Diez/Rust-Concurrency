extern crate alloc;

use self::alloc::raw_vec::RawVec;
use std::ptr;
use std::cmp::PartialEq;
use std::option::Option;
use std::sync::{Mutex, Condvar, Arc};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::BlockingQueue;
use super::super::round_up_to_next_highest_power_of_two;

const MIN_CAPACITY: usize = 16;

fn next_node_index(index: usize, mask: usize) -> usize {
    (index + 1) & mask
}

struct ArrayBlockingQueueInner<T> {
    mutex: Mutex<()>,
    head: AtomicUsize,
    size: AtomicUsize,
    data: RawVec<T>,
    empty: Condvar,
    full: Condvar
}

impl <T: PartialEq> ArrayBlockingQueueInner<T> {

    fn with_capacity(capacity: usize) -> ArrayBlockingQueueInner<T> {
        let capacity = if capacity < MIN_CAPACITY {
            MIN_CAPACITY
        }
        else {
            round_up_to_next_highest_power_of_two(capacity)
        };
        ArrayBlockingQueueInner {
            mutex: Mutex::new(()),
            head: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
            data: RawVec::with_capacity(capacity),
            empty: Condvar::new(),
            full: Condvar::new()
        }
    }

    fn capacity(&self) -> usize {
        self.data.cap()
    }

    fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    fn increase_size(&self) -> usize {
        self.size.fetch_add(1, Ordering::Relaxed)
    }

    fn decrease_size(&self) -> usize {
        self.size.fetch_sub(1, Ordering::Relaxed)
    }

    fn is_full(&self) -> bool {
        self.size() == self.capacity()
    }

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn head(&self) -> usize {
        self.head.load(Ordering::Relaxed)
    }

    fn next_head(&self) -> usize {
        let head = self.head();
        let mask = self.capacity() - 1;
        let new_head = (head + 1) & mask;
        self.head.swap(new_head, Ordering::Relaxed)
    }

    fn remaining_capacity(&self) -> usize {
        let guard = self.mutex.lock().unwrap();
        let remaining_capacity = self.capacity() - self.size();
        drop(guard);
        remaining_capacity
    }

    fn len(&self) -> usize {
        let guard = self.mutex.lock().unwrap();
        let len = self.size();
        drop(guard);
        len
    }

    fn enqueue(&self, val: T) {
        let mut guard = self.mutex.lock().unwrap();
        while self.is_full() {
            guard = self.full.wait(guard).unwrap();
        }
        let index = self.next_free_index();
        unsafe {
            let tail = self.data.ptr().offset(index as isize);
            ptr::write(tail, val);
        }
        self.empty.notify_all();
        drop(guard);
    }

    fn next_free_index(&self) -> usize {
        let mask = self.capacity() - 1;
        (self.head() + self.increase_size()) & mask
    }

    fn dequeue(&self) -> T {
        let mut guard = self.mutex.lock().unwrap();
        while self.is_empty() {
            guard = self.empty.wait(guard).unwrap();
        }
        let index = self.next_head();
        let val = unsafe {
            let head = self.data.ptr().offset(index as isize);
            ptr::read(head)
        };
        self.decrease_size();
        self.full.notify_all();
        drop(guard);
        val
    }

    fn contains(&self, val: T) -> bool {
        let guard = self.mutex.lock().unwrap();
        let mut next = self.head();
        let mut find = false;
        let mask = self.capacity() - 1;
        let tail = (self.head() + self.size()) & mask;
        while next != tail && !find {
            let v = unsafe {
                let p = self.data.ptr().offset(next as isize);
                ptr::read(p)
            };
            find = v == val;
            next = next_node_index(next, mask);
        }
        drop(guard);
        find
    }

    fn offer(&self, val: T) -> bool {
        if !self.is_full() {
            self.enqueue(val);
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<T> {
        let guard = self.mutex.lock().unwrap();
        let result = if self.is_empty() {
            None
        } else {
            unsafe {
                let head = self.data.ptr().offset(self.head() as isize);
                Some(ptr::read(head))
            }
        };
        drop(guard);
        result
    }
}

/// Bounded blocking queue is based on raw vector implementation
/// Current implementation is based on one Mutex and two Condvars
#[derive(Clone)]
pub struct ArrayBlockingQueue<T> {
    inner: Arc<ArrayBlockingQueueInner<T>>
}

impl <T: PartialEq> ArrayBlockingQueue<T> {

    /// Create queue with default capacity
    /// which is 16
    pub fn new() -> ArrayBlockingQueue<T> {
        ArrayBlockingQueue {
            inner: Arc::new(ArrayBlockingQueueInner::with_capacity(MIN_CAPACITY))
        }
    }

    /// Create new queue with specified capacity
    pub fn with_capacity(capacity: usize) -> ArrayBlockingQueue<T> {
        ArrayBlockingQueue {
            inner: Arc::new(ArrayBlockingQueueInner::with_capacity(capacity))
        }
    }

    /// Return remaining capacity for current queue
    pub fn remaining_capacity(&self) -> usize {
        self.inner.remaining_capacity()
    }
}

impl <T: PartialEq> BlockingQueue<T> for ArrayBlockingQueue<T> {

    /// Return size of current queue
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if current queue is empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Enqueue value into queue
    /// Could be blocked until dequeue event if queue is full
    fn enqueue(&self, val: T) {
        self.inner.enqueue(val);
    }

    /// Dequeue value from queue
    /// Could be blocked until enqueue event if queue is empty
    fn dequeue(&self) -> T {
        self.inner.dequeue()
    }

    /// Check if current queue contains specified value
    fn contains(&self, val: T) -> bool {
        self.inner.contains(val)
    }

    /// Offer value into queue
    /// If queue is not full return true otherwise false
    /// Notify threads which blocked on dequeue operation
    fn offer(&self, val: T) -> bool {
        self.inner.offer(val)
    }

    /// Peek queue head value without removing it from queue
    fn peek(&self) -> Option<T> {
        self.inner.peek()
    }
}
