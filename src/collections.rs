pub struct BoundedBlockingQueue {
    data: Vec<i32>
}

impl BoundedBlockingQueue {

    pub fn new(capacity: usize) -> BoundedBlockingQueue {
        let capacity = BoundedBlockingQueue::round_up_to_next_highest_power_of_two(capacity);
        BoundedBlockingQueue { data: Vec::with_capacity(capacity) }
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

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn enqueue(&mut self, val: i32) {
        if self.size() < self.capacity() {
            self.data.push(val);
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn contains(&self, val: i32) -> bool {
        self.data.contains(&val)
    }

    pub fn dequeue(&mut self) -> i32 {
        self.data.pop().unwrap()
    }
}
