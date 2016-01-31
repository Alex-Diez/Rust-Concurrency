pub use concrust::queue::BoundedBlockingQueue;
pub use concrust::queue::BlockingQueue;

pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicBool, Ordering};
pub use std::time::Duration;

pub use std::thread;
pub use std::sync::mpsc;

pub use expectest::prelude::{be_equal_to, be_true, be_false, be_some, be_ok};

describe! bounded_blocking_queue_test {

    before_each {
        const CAPACITY : usize = 16;
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(CAPACITY);
    }

    it "should create a new queue with default capacity" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::new();
        expect!(queue.remaning_capacity()).to(be_equal_to(16));
    }

    it "should create a new empty queue" {
        expect!(queue.is_empty()).to(be_true());
        expect!(queue.len()).to(be_equal_to(0));
    }

    it "should have capacity that is always highest power of two" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(10);
        expect!(queue.remaning_capacity()).to(be_equal_to(16));

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(20);
        expect!(queue.remaning_capacity()).to(be_equal_to(32));

        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(40);
        expect!(queue.remaning_capacity()).to(be_equal_to(64));
    }

    it "should not have less then min capacity" {
        let queue: BoundedBlockingQueue<i32> = BoundedBlockingQueue::with_capacity(6);
        expect!(queue.remaning_capacity()).to(be_equal_to(16));
    }

    it "should increase size when insert into queue" {
        queue.enqueue(1);

        expect!(queue.is_empty()).not_to(be_true());
    }

    it "should contain value that was equeued" {
        queue.enqueue(1);

        expect!(queue.contains(1)).to(be_true());
    }

    it "should contain values that were enqueued" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);
        queue.enqueue(40);

        expect!(queue.contains(10)).to(be_true());
        expect!(queue.contains(20)).to(be_true());
        expect!(queue.contains(30)).to(be_true());
        expect!(queue.contains(40)).to(be_true());
    }

    it "should not contain value that was not enqueued" {
        expect!(queue.contains(10)).to(be_false());
    }

    it "should decrise size when remove from queue" {
        queue.enqueue(1);

        queue.dequeue();

        expect!(queue.is_empty()).to(be_true());
    }

    it "should dequeue first enqueued value" {
        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);

        expect!(queue.dequeue()).to(be_equal_to(10));
        expect!(queue.dequeue()).to(be_equal_to(20));
        expect!(queue.dequeue()).to(be_equal_to(30));
    }

    it "should insert offered value if queue not full" {
        expect!(queue.offer(1)).to(be_true());
        expect!(queue.contains(1)).to(be_true());
    }

    it "should peek first element but not delete" {
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);

        expect!(queue.peek()).to(be_some().value(1));
        expect!(queue.peek()).to(be_some().value(1));
    }

    it "should calculate correct size when alot insertions and deletions" {
        enqeue_times(8, &queue);
        dequeue_times(6, &queue);

        enqeue_times(8, &queue);
        dequeue_times(6, &queue);

        enqeue_times(8, &queue);
        dequeue_times(6, &queue);

        enqeue_times(8, &queue);
        dequeue_times(6, &queue);
    }

    it "should wait when queue is empty" {
        const NUMBER_OF_THREADS: usize = 10;
        let arc = Arc::new(queue);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = arc.clone();
            let jh = thread::spawn(
                move || {
                    expect!(data.dequeue()).to(be_equal_to(1));
                    data.enqueue(1);
                }
            );
            results.push(jh);
        }

        arc.enqueue(1);

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }

        expect!(arc.dequeue()).to(be_equal_to(1));
    }

    it "should notify threads when offer value to queue" {
        const NUMBER_OF_THREADS: usize = 20;
        let arc = Arc::new(queue);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = arc.clone();
            let jh = thread::spawn(
                move || {
                    expect!(data.dequeue()).to(be_equal_to(1));
                    expect!(data.offer(1)).to(be_true());
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));
        expect!(arc.offer(1)).to(be_true());

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }

        expect!(arc.dequeue()).to(be_equal_to(1));
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
                    expect!(data.dequeue()).to(be_equal_to(1));
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));
        expect!(arc.dequeue()).to(be_equal_to(1));

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }

        expect!(arc.len()).to(be_equal_to(CAPACITY-2));
    }
}

pub fn enqeue_times(times: i32, queue: &BlockingQueue<i32>) {
    let size = queue.len();
    for i in 1..times {
        queue.enqueue(i);
        expect!(queue.len()).to(be_equal_to(size + (i as usize)));
    }
}

pub fn dequeue_times(times: i32, queue: &BlockingQueue<i32>) {
    let size = queue.len();
    for i in 1..times {
        queue.dequeue();
        expect!(queue.len()).to(be_equal_to(size - (i as usize)));
    }
}
