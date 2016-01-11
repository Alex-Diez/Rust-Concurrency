extern crate concurrent;

#[cfg(test)]
mod bounded_blocking_queue_test {

    use super::concurrent::collections::BoundedBlockingQueue;

    #[test]
    fn it_should_create_a_new_empty_queue() {
        let queue = BoundedBlockingQueue::new(10);

        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn capacity_should_be_always_power_of_two() {
        let queue = BoundedBlockingQueue::new(6);
        assert_eq!(queue.capacity(), 8);

        let queue = BoundedBlockingQueue::new(10);
        assert_eq!(queue.capacity(), 16);

        let queue = BoundedBlockingQueue::new(20);
        assert_eq!(queue.capacity(), 32);

        let queue = BoundedBlockingQueue::new(40);
        assert_eq!(queue.capacity(), 64);
    }

    #[test]
    fn it_should_increase_size_when_insert_into_queue() {
        let mut queue = BoundedBlockingQueue::new(10);
        let old_size = queue.size();
        queue.enqueue(1);

        assert_eq!(queue.size(), old_size + 1);
    }

    #[test]
    fn it_should_contains_value_that_was_equeued() {
        let mut queue = BoundedBlockingQueue::new(10);
        queue.enqueue(1);

        assert!(queue.contains(1));
        assert!(!queue.contains(2));
    }

    #[test]
    fn it_should_decrise_size_when_remove_from_queue() {
        let mut queue = BoundedBlockingQueue::new(10);
        queue.enqueue(1);
        let old_size = queue.size();

        queue.dequeue();
        assert_eq!(queue.size(), old_size - 1);
    }

    #[test]
    fn it_should_dequeue_last_enqueued_value() {
        let mut queue = BoundedBlockingQueue::new(10);
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);

        assert_eq!(queue.dequeue(), 30);
        assert_eq!(queue.dequeue(), 20);
        assert_eq!(queue.dequeue(), 10);
    }

    #[test]
    fn it_should_not_enqueue_more_than_capacity() {
        const CAPACITY : usize = 16;
        let mut queue = BoundedBlockingQueue::new(CAPACITY);
        for i in 0..CAPACITY + 1 {
            queue.enqueue(i as i32);
        }
        assert_eq!(queue.size(), CAPACITY);
    }
}
