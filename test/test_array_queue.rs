pub use concrust::queue::ArrayBlockingQueue;
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
        let queue: ArrayBlockingQueue<i32> = ArrayBlockingQueue::with_capacity(CAPACITY);
    }

    it "should create a new queue with default capacity" {
        let queue: ArrayBlockingQueue<i32> = ArrayBlockingQueue::new();
        expect!(queue.remaning_capacity()).to(be_equal_to(16));
    }

    it "should create a new empty queue" {
        expect!(queue.is_empty()).to(be_true());
        expect!(queue.len()).to(be_equal_to(0));
    }

    it "should have capacity that is always highest power of two" {
        let queue: ArrayBlockingQueue<i32> = ArrayBlockingQueue::with_capacity(10);
        expect!(queue.remaning_capacity()).to(be_equal_to(16));

        let queue: ArrayBlockingQueue<i32> = ArrayBlockingQueue::with_capacity(20);
        expect!(queue.remaning_capacity()).to(be_equal_to(32));

        let queue: ArrayBlockingQueue<i32> = ArrayBlockingQueue::with_capacity(40);
        expect!(queue.remaning_capacity()).to(be_equal_to(64));
    }

    it "should not have less then min capacity" {
        let queue: ArrayBlockingQueue<i32> = ArrayBlockingQueue::with_capacity(6);
        expect!(queue.remaning_capacity()).to(be_equal_to(16));
    }

    it "should increase size when insert into queue" {
        queue.enqueue(1);

        expect!(queue.is_empty()).not_to(be_true());
    }

    it "should have size be equal to capasity when is full" {
        enqeue_times(CAPACITY as i32, &queue);
        expect!(queue.len()).to(be_equal_to(CAPACITY));
    }

    it "should contain value that was equeued" {
        queue.enqueue(1);

        expect!(queue.contains(1)).to(be_true());
    }

    it "should not contain value that was not enqueued" {
        expect!(queue.contains(10)).to(be_false());
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

    it "should decrise size when remove from queue" {
        queue.enqueue(1);

        queue.dequeue();

        expect!(queue.is_empty()).to(be_true());
    }

    it "should dequeue enqueued value" {
        queue.enqueue(10);

        expect!(queue.dequeue()).to(be_equal_to(10));

        queue.enqueue(20);

        expect!(queue.dequeue()).to(be_equal_to(20));

        queue.enqueue(30);

        expect!(queue.dequeue()).to(be_equal_to(30));
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
        for iter in 0..5 {
            let size = queue.len();
            enqeue_times(8, &queue);
            expect!(queue.len()).to(be_equal_to(size + 8));

            let size = queue.len();
            dequeue_times(6, &queue);
            expect!(queue.len()).to(be_equal_to(size - 6));
        }
    }

    it "should enqueue dequeue more than capacity times" {
        for i in 0..2*CAPACITY {
            let elem = i as i32;
            queue.enqueue(elem);
            expect!(queue.dequeue()).to(be_equal_to(elem));
        }
    }

    it "should accept offered value when queue is not full" {
        for i in 0..CAPACITY {
            let elem = i as i32;
            expect!(queue.offer(elem)).to(be_true());
        }
    }

    it "should reject offered value when queue is full" {
        for i in 0..CAPACITY {
            let elem = i as i32;
            (queue.enqueue(elem));
        }

        expect!(queue.offer(1)).to(be_false());
    }

    it "should wait when queue is empty" {
        const NUMBER_OF_THREADS: usize = 10;
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = queue.clone();
            let jh = thread::spawn(
                move || {
                    expect!(data.dequeue()).to(be_equal_to(1));
                    data.enqueue(1);
                }
            );
            results.push(jh);
        }

        queue.enqueue(1);

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }

        expect!(queue.dequeue()).to(be_equal_to(1));
    }

    it "should notify threads when offer value to queue" {
        const NUMBER_OF_THREADS: usize = 20;
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = queue.clone();
            let jh = thread::spawn(
                move || {
                    expect!(data.dequeue()).to(be_equal_to(1));
                    expect!(data.offer(1)).to(be_true());
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));
        expect!(queue.offer(1)).to(be_true());

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }

        expect!(queue.dequeue()).to(be_equal_to(1));
    }

    it "should wait when queue is full" {
        const NUMBER_OF_THREADS: usize = CAPACITY-1;
        for _ in 0..CAPACITY {
            queue.enqueue(1);
        }
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let data = queue.clone();
            let jh = thread::spawn(
                move || {
                    data.enqueue(10);
                    expect!(data.dequeue()).to(be_equal_to(1));
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));
        expect!(queue.dequeue()).to(be_equal_to(1));

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }

        expect!(queue.len()).to(be_equal_to(CAPACITY-1));
    }
}

pub fn enqeue_times(times: i32, queue: &BlockingQueue<i32>) {
    for i in 0..times {
        queue.enqueue(i);
    }
}

pub fn dequeue_times(times: i32, queue: &BlockingQueue<i32>) {
    for i in 0..times {
        queue.dequeue();
    }
}
