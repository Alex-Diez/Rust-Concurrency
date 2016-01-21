extern crate alloc;

use self::alloc::raw_vec::RawVec;

use std::mem;
use std::ptr;

use std::boxed::Box;

use std::cmp::PartialEq;

use std::option::Option;

use std::ops::Deref;
use std::ops::DerefMut;

use std::clone::Clone;
use std::marker::Copy;

use std::sync::{Mutex, MutexGuard, Condvar};
use std::sync::atomic::{AtomicUsize,Ordering};

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
    value: Option<T>,
    next: Option<Link<T>>
}

impl <T> Node<T> {

    fn empty() -> Node<T> {
        Node {
            value: None,
            next: None
        }
    }

    fn non_empty(value: T) -> Node<T> {
        Node {
            value: Some(value),
            next: None
        }
    }
}

struct Link<T> {
    ptr: *mut Node<T>
}

impl <T> Link<T> {

    fn new(node: Node<T>) -> Link<T> {
        Link {
            ptr: Box::into_raw(Box::new(node))
        }
    }
}

impl <T> Deref for Link<T> {
    type Target = Node<T>;

    fn deref(&self) -> &Node<T> {
        unsafe { mem::transmute(self.ptr) }
    }
}

impl <T> DerefMut for Link<T> {

    fn deref_mut(&mut self) -> &mut Node<T> {
        unsafe { mem::transmute(self.ptr) }
    }
}

impl<T> Clone for Link<T> {

    fn clone(&self) -> Link<T> {
        Link { ptr: self.ptr }
    }
}

impl <T> Copy for Link<T> { }
unsafe impl <T: Send> Send for Link<T> { }

pub struct UnboundedBlockingQueue<T> {
    head: Mutex<Link<T>>,
    tail: Mutex<Link<T>>,
    size: AtomicUsize,
    empty: Condvar
}

impl <T: PartialEq> UnboundedBlockingQueue<T> {

    pub fn new() -> UnboundedBlockingQueue<T> {
        let empty = Link::new(Node::empty());
        UnboundedBlockingQueue {
            size: AtomicUsize::new(0),
            head: Mutex::new(empty),
            tail: Mutex::new(empty),
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
        let mut tail = self.tail.lock().unwrap();
        put(Node::non_empty(val), &mut tail);
        let current_size = self.size.fetch_add(1, Ordering::Relaxed);
        if current_size + 1 > 0 {
            self.empty.notify_all();
        }
    }

    pub fn dequeue(&self) -> T {
        let mut head = self.head.lock().unwrap();
        while self.is_empty() {
            head = self.empty.wait(head).unwrap();
        }
        let val = take(&mut head);
        self.size.fetch_sub(1, Ordering::Relaxed);
        val
    }

    pub fn contains(&self, val: T) -> bool {
        let mut head_lock = self.head.lock().unwrap();
        let tail_lock = self.tail.lock().unwrap();
        let find = contains(val, &mut head_lock);
        drop(tail_lock);
        drop(head_lock);
        find
    }

    pub fn offer(&self, val: T) -> bool {
        self.enqueue(val);
        true
    }

    pub fn peek(&self) -> Option<T> {
        let mut head_lock = self.head.lock().unwrap();
        get(&mut head_lock)
    }
}

fn put<T: PartialEq>(node: Node<T>, last: &mut MutexGuard<Link<T>>) {
    let link = Link::new(node);
    (***last).next = Some(link);
    **last = link;
}

fn take<T: PartialEq>(head: &mut MutexGuard<Link<T>>) -> T {
    let h = **head;
    let mut first = (*h).next.unwrap();
    **head = first;
    (*first).value.take().unwrap()
}

fn contains<T: PartialEq>(val: T, head: &mut MutexGuard<Link<T>>) -> bool {
    let mut find = false;
    let value = Some(val);
    let mut node = **head;
    loop {
        if (*node).value == value {
            find = true;
            break;
        }
        node = match (*node).next {
            Some(ref next) => *next,
            None => break,
        }
    }
    find
}

fn get<T: PartialEq>(head: &mut MutexGuard<Link<T>>) -> Option<T> {
    let h = **head;
    match (*h).next {
        Some(mut val) => {
            match (*val).value {
                Some(ref mut val) => {
                    let raw: *const _ = &mut *val;
                    unsafe {
                        Some(ptr::read(raw))
                    }
                },
                None => None,
            }
        },
        None => None,
    }
}
