extern crate concurrent;

pub use self::concurrent::collections::BoundedBlockingQueue;

pub use std::sync::Arc;
pub use std::thread;
pub use std::time::Duration;

describe! bounded_blocking_queue_test {

    before_each {
        const CAPACITY : usize = 16;
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(CAPACITY);
    }

    it "should create a new queue with default capacity" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::new();
        assert_eq!(queue.capacity(), 16);
    }

    it "should create a new empty queue" {
        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);
    }

    it "should capacity be always highest power of two" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(6);
        assert_eq!(queue.capacity(), 8);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(10);
        assert_eq!(queue.capacity(), 16);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(20);
        assert_eq!(queue.capacity(), 32);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(40);
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

    it "should dequeue first enqueued value" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);

        assert_eq!(queue.dequeue(), 10);
        assert_eq!(queue.dequeue(), 20);
        assert_eq!(queue.dequeue(), 30);
    }

    it "should not enqueue more than capacity" {
        for i in 0..CAPACITY {
            queue.enqueue(i as i32);
        }
        assert_eq!(queue.size(), CAPACITY - 1);
    }

    it "should insert offered value if queue not full" {
        assert!(queue.offer(1));
        assert!(queue.contains(1));
    }

    it "should peek first element but not delete from queue" {
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);

        assert_eq!(queue.peek(), Some(1));
        assert_eq!(queue.peek(), Some(1));
    }

    it "should calculate correct size when alot insertions and deletions" {
        let size = queue.size();
        for i in 1..8 {
            queue.enqueue(i);
            assert_eq!(queue.size(), size + (i as usize));
        }

        let size = queue.size();
        for i in 1..6 {
            queue.dequeue();
            assert_eq!(queue.size(), size - (i as usize));
        }

        let size = queue.size();
        for i in 1..8 {
            queue.enqueue(i);
            assert_eq!(queue.size(), size + (i as usize));
        }

        let size = queue.size();
        for i in 1..6 {
            queue.dequeue();
            assert_eq!(queue.size(), size - (i as usize));
        }

        let size = queue.size();
        for i in 1..8 {
            queue.enqueue(i);
            assert_eq!(queue.size(), size + (i as usize));
        }

        let size = queue.size();
        for i in 1..6 {
            queue.dequeue();
            assert_eq!(queue.size(), size - (i as usize));
        }

        let size = queue.size();
        for i in 1..8 {
            queue.enqueue(i);
            assert_eq!(queue.size(), size + (i as usize));
        }

        let size = queue.size();
        for i in 1..6 {
            queue.dequeue();
            assert_eq!(queue.size(), size - (i as usize));
        }
    }

    it "should dequeue await when queue is empty" {
        let arc = Arc::new(queue);
        for i in 0..16 {
            arc.enqueue(i);
        }
        let data = arc.clone();
        let jh = thread::spawn(
            move || {
                let mut num = 0;
                while !data.is_empty() {
                    let datum = data.dequeue();
                    assert_eq!(datum, num);
                    num += 1;
                }
                assert_eq!(num, 100);
            }
        );

        for i in 16..100 {
            arc.enqueue(i);
        }

        assert!(jh.join().is_ok());
    }
}
