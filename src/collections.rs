extern crate alloc;

use self::alloc::raw_vec::RawVec;

use std::ptr;
use std::cmp::PartialEq;
use std::option::Option;
use std::usize;

pub struct BoundedBlockingQueue<T> {
    head: usize,
    tail: usize,
    data: RawVec<T>
}

impl <T: PartialEq> BoundedBlockingQueue<T> {

    pub fn new() -> BoundedBlockingQueue<T> {
        BoundedBlockingQueue::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> BoundedBlockingQueue<T> {
        let capacity = round_up_to_next_highest_power_of_two(capacity);
        BoundedBlockingQueue { head: 0, tail: 0, data: RawVec::with_capacity(capacity) }
    }

    pub fn capacity(&self) -> usize {
        self.data.cap()
    }

    pub fn enqueue(&mut self, val: T) {
        self.offer(val);
    }

    pub fn size(&self) -> usize {
        if self.head < self.tail {
            (self.head + self.tail) & (self.data.cap() - 1)
        } else {
            usize::MAX - (usize::MAX - self.head + self.tail) & (self.data.cap() - 1)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn contains(&self, val: T) -> bool {
        let mut next = self.head;
        let mut find = false;
        while next != self.tail && !find {
            unsafe {
                let p = self.data.ptr().offset(next as isize);
                let v = ptr::read(p);
                find = v == val;
                next = (next + 1) & (self.data.cap() - 1);
            }
        }
        find
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let first = self.data.ptr().offset(self.head as isize);
                self.head = (self.head + 1) & (self.data.cap() - 1);
                Some(ptr::read(first))
            }
        }
    }

    pub fn offer(&mut self, val: T) -> bool {
        let size = self.size();
        if size == self.capacity() - 1 {
            false
        } else {
            unsafe {
                let last = self.data.ptr().offset(self.tail as isize);
                self.tail = (self.tail + 1) & (self.data.cap() - 1);
                ptr::write(last, val);
            }
            true
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let first = self.data.ptr().offset(self.head as isize);
                Some(&(ptr::read(first)))
            }
        }
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
