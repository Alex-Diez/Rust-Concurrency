extern crate alloc;

use self::alloc::raw_vec::RawVec;

use std::ptr;

use std::cmp::PartialEq;
use std::option::Option;

use std::sync::{Mutex, Condvar};

struct BoundedBlockingQueueState<T> {
    head: usize,
    tail: usize,
    data: RawVec<T>
}

impl <T: PartialEq> BoundedBlockingQueueState<T> {

    fn new(capacity: usize) -> BoundedBlockingQueueState<T> {
        let capacity = round_up_to_next_highest_power_of_two(capacity);
        BoundedBlockingQueueState { head: 0, tail: 0, data: RawVec::with_capacity(capacity) }
    }

    #[inline]
    fn remaning_capacity(&self) -> usize {
        self.data.cap() - self.size() - 1
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.data.cap()
    }

    #[inline]
    fn size(&self) -> usize {
        (self.data.cap() - self.head + self.tail)  & (self.data.cap() - 1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.size() == self.capacity() - 1
    }

    fn contains(&self, val: T) -> bool {
        let mut next = self.head;
        let mut find = false;
        while next != self.tail && !find {
            unsafe {
                let p = self.data.ptr().offset(next as isize);
                let v = ptr::read(p);
                find = v == val;
                next = next_node_index(next, self.data.cap() - 1);
            }
        }
        find
    }

    fn dequeue(&mut self) -> T {
        unsafe {
            let head = self.data.ptr().offset(self.head as isize);
            self.head = next_node_index(self.head, self.data.cap() - 1);
            ptr::read(head)
        }
    }

    fn enqueue(&mut self, val: T) -> bool {
        if self.is_full() {
            false
        } else {
            unsafe {
                let tail = self.data.ptr().offset(self.tail as isize);
                self.tail = next_node_index(self.tail, self.data.cap() - 1);
                ptr::write(tail, val);
            }
            true
        }
    }

    fn peek(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let head = self.data.ptr().offset(self.head as isize);
                Some(ptr::read(head))
            }
        }
    }
}

pub struct BoundedBlockingQueue<T> {
    mutex: Mutex<BoundedBlockingQueueState<T>>,
    empty: Condvar,
    full: Condvar
}

impl <T: PartialEq> BoundedBlockingQueue<T> {

    pub fn new() -> BoundedBlockingQueue<T> {
        BoundedBlockingQueue::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> BoundedBlockingQueue<T> {
        let capacity = round_up_to_next_highest_power_of_two(capacity);
        BoundedBlockingQueue {
            mutex: Mutex::new(BoundedBlockingQueueState::new(capacity)),
            empty: Condvar::new(),
            full: Condvar::new()
        }
    }

    pub fn remaning_capacity(&self) -> usize {
        let guard = self.mutex.lock().unwrap();
        guard.remaning_capacity()
    }

    pub fn enqueue(&self, val: T) {
        let mut guard = self.mutex.lock().unwrap();
        while guard.is_full() {
            guard = self.full.wait(guard).unwrap();
        }
        guard.enqueue(val);
        self.empty.notify_all();
    }

    pub fn size(&self) -> usize {
        let guard = self.mutex.lock().unwrap();
        guard.size()
    }

    pub fn is_empty(&self) -> bool {
        let guard = self.mutex.lock().unwrap();
        guard.is_empty()
    }

    pub fn contains(&self, val: T) -> bool {
        let guard = self.mutex.lock().unwrap();
        guard.contains(val)
    }

    pub fn dequeue(&self) -> T {
        let mut guard = self.mutex.lock().unwrap();
        while guard.is_empty() {
            guard = self.empty.wait(guard).unwrap();
        }
        let val = guard.dequeue();
        self.full.notify_all();
        val
    }

    pub fn offer(&self, val: T) -> bool {
        let mut guard = self.mutex.lock().unwrap();
        if guard.enqueue(val) {
            self.empty.notify_all();
            true
        } else {
            false
        }
    }

    pub fn peek(&self) -> Option<T> {
        let guard = self.mutex.lock().unwrap();
        guard.peek()
    }
}

fn round_up_to_next_highest_power_of_two(mut v: usize) -> usize {
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}

fn next_node_index(index: usize, mask: usize) -> usize {
    (index + 1) & mask
}

use std::boxed::Box;
use std::mem;

struct Node {
    value: i32,
    next: Option<Box<Node>>
}

impl Node {
    
    fn new(value: i32) -> Node {
        Node {
            value: value,
            next: None
        }
    }
}

pub struct UnboundedBlockingQueue {
    head: Option<Box<Node>>,
    tail: *mut Node,
    size: usize
}

impl <'a> UnboundedBlockingQueue {

    pub fn new() -> UnboundedBlockingQueue {
        UnboundedBlockingQueue {
            size: 0,
            head: None,
            tail: ptr::null_mut()
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn enqueue(&mut self, val: i32) {
        self.size += 1;
        let mut new_tail = Box::new(Node::new(val));
        let raw_tail: *mut _ = &mut *new_tail;
        if !self.tail.is_null() {
            unsafe {
                (* self.tail).next = Some(new_tail);
            }
        }
        else {
            self.head = Some(new_tail);
        }
        self.tail = raw_tail;
    }

    pub fn dequeue(&mut self) -> Option<i32> {
        self.size -= 1;
        self.head.take().map(|head| {
            let head = *head;
            self.head = head.next;
            if self.head.is_none() {
                self.tail = ptr::null_mut();
            }
            head.value
        })
    }

    pub fn contains(&self, val: i32) -> bool {
        match self.head {
            Some(ref head) => {
                let mut node = head;
                let mut find = false;
                loop {
                    if (*node).value == val {
                        find = true;
                    }
                    match (*node).next {
                        Some(ref next) => {
                            node = next;
                        },
                        None => break,
                    }
                    if find {
                        break;
                    }
                }
                find
            },
            None => false
        }
    }
}
