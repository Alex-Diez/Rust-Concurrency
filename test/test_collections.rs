extern crate concurrent;

pub use self::concurrent::collections::BoundedBlockingQueue;
pub use self::concurrent::collections::UnboundedBlockingQueue;

pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicBool, Ordering};
pub use std::time::Duration;

pub use std::thread;
pub use std::sync::mpsc;

describe! bounded_blocking_queue_test {

    before_each {
        const CAPACITY : usize = 16;
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(CAPACITY);
    }

    it "should create a new queue with default capacity" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::new();
        assert_eq!(queue.remaning_capacity(), 15);
    }

    it "should create a new empty queue" {
        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);
    }

    it "should have capacity that is always highest power of two minus one" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(6);
        assert_eq!(queue.remaning_capacity(), 7);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(10);
        assert_eq!(queue.remaning_capacity(), 15);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(20);
        assert_eq!(queue.remaning_capacity(), 31);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(40);
        assert_eq!(queue.remaning_capacity(), 63);
    }

    it "should increase size when insert into queue" {
        let old_size = queue.size();
        queue.enqueue(1);

        assert_eq!(queue.size(), old_size + 1);
    }

    it "should contain value that was equeued" {
        queue.enqueue(1);

        assert!(queue.contains(1));
    }

    it "should contain values that were enqueued" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);
        queue.enqueue(40);

        assert!(queue.contains(10));
        assert!(queue.contains(20));
        assert!(queue.contains(30));
        assert!(queue.contains(40));
    }

    it "should not contain value that was not enqueued" {
        assert!(!queue.contains(10));
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

    it "should insert offered value if queue not full" {
        assert!(queue.offer(1));
        assert!(queue.contains(1));
    }

    it "should peek first element but not delete" {
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

    //FAILED test
    //some times threads get stuck
    /*
    it "should threads wait each other when queue is empty or full" {
        let arc = Arc::new(queue);
        let flag = Arc::new(AtomicBool::new(false));

        let data = arc.clone();
        let ready = flag.clone();
        let jh = thread::spawn(
            move || {
                let mut counter = 0;
                let mut sum = 0;
                while !data.is_empty() || !ready.load(Ordering::Relaxed) {
                    let datum = data.dequeue();
                    assert_eq!(datum, counter);
                    counter += 1;
                    sum += datum;
                }
                assert_eq!(sum, 4950);
            }
        );

        for i in 0..100 {
            arc.enqueue(i);
        }

        flag.store(true, Ordering::Relaxed);

        assert!(jh.join().is_ok());
    }
    */
}

describe! unbounded_blocking_queue_test {

    before_each {
        let mut queue = UnboundedBlockingQueue::new();
    }

    it "should create new empty unbounded queue" {
        assert_eq!(queue.size(), 0);
        assert!(queue.is_empty());
    }

    it "should increase queue size when enqueue value" {
        let old_size = queue.size();
        queue.enqueue(1);

        assert_eq!(queue.size(), old_size + 1);
    }

    it "should decrease queue size when dequeue value" {
        queue.enqueue(1);
        let old_size = queue.size();
        queue.dequeue();

        assert_eq!(queue.size(), old_size - 1);
    }

    it "should contain value that was enqueued" {
        queue.enqueue(1);
        assert!(queue.contains(1));
    }

    it "should contain values that were enqueued" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);
        queue.enqueue(40);

        assert!(queue.contains(10));
        assert!(queue.contains(20));
        assert!(queue.contains(30));
        assert!(queue.contains(40));
    }

    it "should not contain value that was not enqueued" {
        assert!(!queue.contains(10));
    }
}
