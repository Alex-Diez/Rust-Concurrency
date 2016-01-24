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

/// Unbounded Blocking Queue base on linked list
/// 
/// Currnet implementation based on two Mutex, for enqueue and dequeue operations,
/// and one Condvar
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

    /// Current queue size
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Return true if current queue is empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Enqueue value into queue
    /// Notify all threads that wait for dequeue value from queue
    pub fn enqueue(&self, val: T) {
        let mut tail = self.tail.lock().unwrap();
        put(Node::non_empty(val), &mut tail);
        let current_size = self.size.fetch_add(1, Ordering::Relaxed);
        if current_size + 1 > 0 {
            self.empty.notify_all();
        }
    }

    /// Dequeue value from queue
    /// Could be blocked on Condvar if queue is empty
    pub fn dequeue(&self) -> T {
        let mut head = self.head.lock().unwrap();
        while self.is_empty() {
            head = self.empty.wait(head).unwrap();
        }
        let val = take(&mut head);
        self.size.fetch_sub(1, Ordering::Relaxed);
        val
    }

    /// Check if current queue contains value
    pub fn contains(&self, val: T) -> bool {
        let mut head_lock = self.head.lock().unwrap();
        let tail_lock = self.tail.lock().unwrap();
        let find = contains(val, &mut head_lock);
        drop(tail_lock);
        drop(head_lock);
        find
    }

    /// Offer value into queue
    /// alwayes return true due to unobound capacity
    pub fn offer(&self, val: T) -> bool {
        self.enqueue(val);
        true
    }

    /// Peek head of queue with out removing it from queue
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
