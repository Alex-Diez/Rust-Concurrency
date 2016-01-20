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

/// some docs here
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
}use std::sync::atomic::{AtomicUsize,Ordering};

struct Node<T> {
    value: Option<T>,
    next: Option<BoxLink<T>>
}

impl <T> Node<T> {

    fn some(value: T) -> Node<T> {
        Node { value: Some(value), next: None }
    }

    fn empty() -> Node<T> {
        Node { value: None, next: None }
    }
    
    fn box_link(node: Node<T>) -> Box<Node<T>> {
        Box::new(node)
    }

    fn share_link(ptr: *mut Node<T>) -> Shared<Node<T>> {
        unsafe { Shared::new(ptr) }
    }
}

type BoxLink<T> = Box<Node<T>>;
type ShareLink<T> = Shared<Node<T>>;

pub struct UnboundedBlockingQueue<T> {
    head: Mutex<BoxLink<T>>,
    tail: Mutex<ShareLink<T>>,
    size: AtomicUsize,
    empty: Condvar
}

impl <T: PartialEq + Clone> UnboundedBlockingQueue<T> {

    pub fn new() -> UnboundedBlockingQueue<T> {
        let mut empty = Node::box_link(Node::empty());
        let raw_node: *mut _ = &mut *empty;
        UnboundedBlockingQueue {
            size: AtomicUsize::new(0),
            head: Mutex::new(empty),
            tail: Mutex::new(Node::share_link(raw_node)),
            empty: Condvar::new()
        }
    }

    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn enqueue(&self, val: T) {
        let mut put_lock = self.tail.lock().unwrap();
        let mut new_tail = Node::box_link(Node::some(val));
        match unsafe { (*put_lock).as_mut() } {
            Some(ref mut node) => {
                let raw_tail: *mut _ = &mut *new_tail;
                node.next = Some(new_tail);
                *put_lock = Node::share_link(raw_tail);
            },
            None => *put_lock = Node::share_link(&mut *new_tail)
        }
        let current_size = self.size.fetch_add(1, Ordering::Relaxed);
        if current_size > 0 {
            self.empty.notify_all();
        }
    }

    pub fn dequeue(&self) -> T {
        let mut poll_lock = self.head.lock().unwrap();
        while self.is_empty() {
            poll_lock = self.empty.wait(poll_lock).unwrap();
        }
        let val = (*poll_lock).value.take().unwrap();
        unsafe {
            match (**poll_lock).next {
                Some(node) => *poll_lock = node,
                None => *poll_lock = Node::box_link(Node::empty()),
            }
        }
        let cnt = self.size.fetch_sub(1, Ordering::Relaxed);
        val
    }

    pub fn contains(&self, val: T) -> bool {
        let mut head_lock = self.head.lock().unwrap();
        let tail_lock = self.tail.lock().unwrap();
        let mut node = **head_lock;
        let mut find = false;
        loop {
            match node.value {
                Some(node_val) => {
                    if node_val == val {
                        find = true;
                        break;
                    }
                },
                None => {},
            }
            match node.next {
                Some(ref next) => {
                    node = **next;
                },
                None => break,
            }
        }
        find
    }

    pub fn offer(&self, val: T) -> bool {
        self.enqueue(val);
        true
    }

    pub fn peek(&self) -> Option<T> {
        let head_lock = self.head.lock().unwrap();
        (*head_lock).value
    }
}
