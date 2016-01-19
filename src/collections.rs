extern crate alloc;

use self::alloc::raw_vec::RawVec;

use std::ptr;
use std::ptr::Shared;
use std::boxed::Box;

use std::clone::Clone;
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

struct Node<T> {
    value: T,
    next: BoxLink<T>
}

impl <T> Node<T> {

    fn new(value: T) -> Node<T> {
        Node { value: value, next: None }
    }
    
    fn box_link(value: T) -> Box<Node<T>> {
        Box::new(Node::new(value))
    }

    fn share_link(ptr: *mut Node<T>) -> Shared<Node<T>> {
        unsafe { Shared::new(ptr) }
    }
}

type BoxLink<T> = Option<Box<Node<T>>>;
type ShareLink<T> = Option<Shared<Node<T>>>;

pub struct UnboundedBlockingQueue<T> {
    head: BoxLink<T>,
    tail: ShareLink<T>,
    size: usize
}

impl <T: PartialEq + Clone> UnboundedBlockingQueue<T> {

    pub fn new() -> UnboundedBlockingQueue<T> {
        UnboundedBlockingQueue {
            size: 0,
            head: None,
            tail: None
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn enqueue(&mut self, val: T) {
        self.size += 1;
        let mut new_tail = Node::box_link(val);
        let raw_tail: *mut _ = &mut *new_tail;
        match self.tail {
            Some(ref mut share) => unsafe { share.as_mut().map(|node| { node.next = Some(new_tail) }); },
            None => self.head = Some(new_tail),
        }
        self.tail = Some(Node::share_link(raw_tail))
    }

    pub fn dequeue(&mut self) -> T {
        self.size -= 1;
        self.head.take().map(
            |head| {
                let head = *head;
                self.head = head.next;
                if self.head.is_none() {
                    self.tail = None;
                }
                head.value
            }
        ).unwrap()
    }

    pub fn contains(&self, val: T) -> bool {
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
                }
                find
            },
            None => false
        }
    }

    pub fn offer(&mut self, val: T) -> bool {
        self.enqueue(val);
        true
    }

    pub fn peek(&self) -> Option<T> {
        match self.head {
            Some(ref node) => Some(node.value.clone()),
            None => None,
        }
    }
}
