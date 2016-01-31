pub use concrust::queue::UnboundedBlockingQueue;

pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicBool, Ordering};
pub use std::time::Duration;

pub use std::thread;
pub use std::sync::mpsc;

describe! unbounded_blocking_queue_test {

    before_each {
        let mut queue: UnboundedBlockingQueue<i32> = UnboundedBlockingQueue::new();
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

    it "should dequeue first enqueued value" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);

        assert_eq!(queue.dequeue(), 10);
        assert_eq!(queue.dequeue(), 20);
        assert_eq!(queue.dequeue(), 30);
    }

    it "should insert offered value" {
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

    it "should wait when queue is empty" {
        const NUMBER_OF_THREADS: usize = 10;
        let arc = Arc::new(queue);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = arc.clone();
            let jh = thread::spawn(
                move || {
                    assert_eq!(data.dequeue(), 1);
                    data.enqueue(1);
                }
            );
            results.push(jh);
        }

        arc.enqueue(1);

        for jh in results {
            assert!(jh.join().is_ok());
        }

        assert_eq!(arc.dequeue(), 1);
    }

    it "should notify threads when offer value to queue" {
        const NUMBER_OF_THREADS: usize = 20;
        let arc = Arc::new(queue);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = arc.clone();
            let jh = thread::spawn(
                move || {
                    assert_eq!(data.dequeue(), 1);
                    assert!(data.offer(1));
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));
        assert!(arc.offer(1));

        for jh in results {
            assert!(jh.join().is_ok());
        }

        assert_eq!(arc.dequeue(), 1);
    }
}
