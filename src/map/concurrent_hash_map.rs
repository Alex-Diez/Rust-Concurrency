use super::super::round_up_to_next_highest_power_of_two;

pub struct ConcurrentHashMap {
    size: usize,
    capacity: usize
}

impl ConcurrentHashMap {
    
    pub fn new() -> ConcurrentHashMap {
        ConcurrentHashMap::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> ConcurrentHashMap {
        let capacity = round_up_to_next_highest_power_of_two(capacity);
        ConcurrentHashMap {
            size: 0,
            capacity: capacity
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn insert(&mut self, key: i32, val: i32) {
        self.size += 1;
    }
}
