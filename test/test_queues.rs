extern crate concrust;

pub use self::concrust::queue::BoundedBlockingQueue;
pub use self::concrust::queue::UnboundedBlockingQueue;
pub use self::concrust::queue::BlockingQueue;

pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicBool, Ordering};
pub use std::time::Duration;

pub use std::thread;
pub use std::sync::mpsc;

describe! bounded_blocking_queue_test {

    before_each {
        const CAPACITY : usize = 16;
        let mut queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(CAPACITY);
    }

    it "should create a new queue with default capacity" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::new();
        assert_eq!(queue.remaning_capacity(), 16);
    }

    it "should create a new empty queue" {
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    it "should have capacity that is always highest power of two" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(10);
        assert_eq!(queue.remaning_capacity(), 16);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(20);
        assert_eq!(queue.remaning_capacity(), 32);

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(40);
        assert_eq!(queue.remaning_capacity(), 64);
    }

    it "should not have less then min capacity" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(6);
        assert_eq!(queue.remaning_capacity(), 16);
    }

    it "should increase size when insert into queue" {
        queue.enqueue(1);

        assert!(!queue.is_empty());
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

        queue.dequeue();

        assert!(queue.is_empty());
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
        enqeue_times(8, &mut queue);
        dequeue_times(6, &mut queue);

        enqeue_times(8, &mut queue);
        dequeue_times(6, &mut queue);

        enqeue_times(8, &mut queue);
        dequeue_times(6, &mut queue);

        enqeue_times(8, &mut queue);
        dequeue_times(6, &mut queue);
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

    it "should wait when queue is full" {
        const NUMBER_OF_THREADS: usize = CAPACITY-2;
        for _ in 0..CAPACITY-1 {
            queue.enqueue(1);
        }
        let arc = Arc::new(queue);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = arc.clone();
            let jh = thread::spawn(
                move || {
                    data.enqueue(10);
                    assert_eq!(data.dequeue(), 1);
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));
        assert_eq!(arc.dequeue(), 1);

        for jh in results {
            assert!(jh.join().is_ok());
        }

        assert_eq!(arc.len(), CAPACITY-2);
    }
}

pub fn enqeue_times(times: i32, queue: &mut BlockingQueue<i32>) {
    let size = queue.len();
    for i in 1..times {
        queue.enqueue(i);
        assert_eq!(queue.len(), size + (i as usize));
    }
}

pub fn dequeue_times(times: i32, queue: &mut BlockingQueue<i32>) {
    let size = queue.len();
    for i in 1..times {
        queue.dequeue();
        assert_eq!(queue.len(), size - (i as usize));
    }
}

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
