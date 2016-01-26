use std::collections::HashMap;

use super::super::round_up_to_next_highest_power_of_two;

struct Bucket {
    hash: u64,
    key: i32,
    value: i32,
    next: Option<Box<Bucket>>
}

impl Bucket {

    fn new(key: i32, value: i32) -> Bucket {
        Bucket {
            hash: 0,
            key: key,
            value: value,
            next: None
        }
    }

    fn set_next(&mut self, next: Option<Box<Bucket>>) {
        self.next = next;
    }
}

pub struct ConcurrentHashMap {
    map: HashMap<i32, i32>,
    capacity: usize
}

impl ConcurrentHashMap {
    
    pub fn new() -> ConcurrentHashMap {
        ConcurrentHashMap::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> ConcurrentHashMap {
        let capacity = round_up_to_next_highest_power_of_two(capacity);
        ConcurrentHashMap {
            capacity: capacity,
            map: HashMap::with_capacity(capacity)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn insert(&mut self, key: i32, val: i32) {
        //self.size += 1;
        //let mut bucket = Box::new(Bucket::new(key, val));
        //bucket.next = self.head.take();
        //self.head = Some(bucket);
        self.map.insert(key, val);
    }

    pub fn remove(&mut self, key: i32) -> Option<i32> {
        self.map.remove(&key)
    }
}
