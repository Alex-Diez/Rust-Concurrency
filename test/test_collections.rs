extern crate concurrent;

pub use self::concurrent::collections::BoundedBlockingQueue;

describe! bounded_blocking_queue_test {

    before_each {
        const CAPACITY : usize = 16;
        let mut queue = BoundedBlockingQueue::new(CAPACITY);
    }

    it "it should create a new empty queue" {
        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);
    }

    it "should capacity be always highest power of two" {
        let queue = BoundedBlockingQueue::new(6);
        assert_eq!(queue.capacity(), 8);

        let queue = BoundedBlockingQueue::new(10);
        assert_eq!(queue.capacity(), 16);

        let queue = BoundedBlockingQueue::new(20);
        assert_eq!(queue.capacity(), 32);

        let queue = BoundedBlockingQueue::new(40);
        assert_eq!(queue.capacity(), 64);
    }

    it "should increase size when insert into queue" {
        let old_size = queue.size();
        queue.enqueue(1);

        assert_eq!(queue.size(), old_size + 1);
    }

    it "should contains value that was equeued" {
        queue.enqueue(1);

        assert!(queue.contains(1));
        assert!(!queue.contains(2));
    }

    it "should decrise size when remove from queue" {
        queue.enqueue(1);
        let old_size = queue.size();

        queue.dequeue();
        assert_eq!(queue.size(), old_size - 1);
    }

    it "should dequeue last enqueued value" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);

        assert_eq!(queue.dequeue(), 30);
        assert_eq!(queue.dequeue(), 20);
        assert_eq!(queue.dequeue(), 10);
    }

    it "should not enqueue more than capacity" {
        for i in 0..CAPACITY + 1 {
            queue.enqueue(i as i32);
        }
        assert_eq!(queue.size(), CAPACITY);
    }
}
