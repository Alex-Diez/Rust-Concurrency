use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

use std::mem;

use super::super::round_up_to_next_highest_power_of_two;

struct Bucket {
    key: i32,
    value: i32,
    link: BucketLink
}

impl Bucket {
    
    fn new(key: i32, value: i32) -> Bucket {
        Bucket {
            key: key,
            value: value,
            link: BucketLink::empty()
        }
    }
}

struct BucketLink {

    bucket: Option<Box<Bucket>>
}

impl BucketLink {

    fn empty() -> BucketLink {
        BucketLink {
            bucket: None
        }
    }

    fn new(bucket: Box<Bucket>) -> BucketLink {
        BucketLink {
            bucket: Some(bucket)
        }
    }

    fn update_bucket(&mut self, bucket: Option<Box<Bucket>>) {
        self.bucket = bucket;
    }
}

pub struct ConcurrentHashMap {
    table: Vec<RwLock<BucketLink>>,
    size: usize
}

impl ConcurrentHashMap {
    
    pub fn new() -> ConcurrentHashMap {
        ConcurrentHashMap::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> ConcurrentHashMap {
        let capacity = round_up_to_next_highest_power_of_two(capacity);
        let mut table = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            table.push(RwLock::new(BucketLink::empty()));
        }
        ConcurrentHashMap {
            table: table,
            size: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    pub fn insert(&mut self, key: i32, val: i32) {
        let index = self.capacity() & key as usize;
        let mut guard = self.table[index].write().unwrap();
        if put(&mut guard, key, val) {
            self.size += 1;
        }
    }

    pub fn remove(&mut self, key: i32) -> Option<i32> {
        let index = self.capacity() & key as usize;
        let mut guard = self.table[index].write().unwrap();
        let result = take(&mut guard, key);
        drop(guard);
        if result.is_some() {
            self.size -= 1;
        }
        result
    }
}

fn take(guard: &mut RwLockWriteGuard<BucketLink>, key: i32) -> Option<i32> {
    let mut removed = false;
    let mut value = 0;
    let mut link = **guard;
    loop {
        let bucket = link.bucket;
        match bucket {
            Some(ref bucket) => {
                if key == (**bucket).key {
                    link.update_bucket((**bucket).link.bucket);
                    return Some((**bucket).value);
                }
            },
            None => break,
        }
        match link.bucket {
            Some(bucket) => link = (*bucket).link,
            None => break,
        }
    }
    None
}

fn put(guard: &mut RwLockWriteGuard<BucketLink>, key: i32, val: i32) -> bool {
    let mut update = false;
    (**guard).bucket = (**guard).bucket.take().map(
        |mut link| {
            unsafe {
                let head = mem::transmute(&mut (*link));
                let mut bucket = *link;
                loop {
                    if bucket.key == key {
                        bucket.value = val;
                        update = true;
                        break;
                    }
                    match bucket.link.bucket {
                        Some(mut next) => bucket = *next,
                        None => {
                            bucket.link = BucketLink::new(Box::new(Bucket::new(key, val)));
                            update = true;
                            break;
                        },
                    }
                }
                head
            }
        }
    );
    if (**guard).bucket.is_none() {
        (**guard).bucket = Some(Box::new(Bucket::new(key, val)));
        update = true;
    }
    update
}
