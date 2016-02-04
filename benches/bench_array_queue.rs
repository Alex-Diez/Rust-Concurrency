#![feature(test)]

extern crate concrust;
extern crate test;
#[macro_use]
extern crate expectest;

pub use concrust::queue::ArrayBlockingQueue;
pub use concrust::queue::BlockingQueue;

pub use expectest::prelude::be_ok;

pub use std::thread;
pub use std::thread::JoinHandle;
pub use std::sync::{Arc, Barrier};

const NUMBER_OF_ELEMENTS: i64 = 1000;

#[bench]
fn consumer_producer_default_queue_size(bencher: &mut test::Bencher) {
    // queue size will be 16
    let queue: ArrayBlockingQueue<i64> = ArrayBlockingQueue::new();
    let barrier = Arc::new(Barrier::new(3));
    let result = sum(NUMBER_OF_ELEMENTS);
    bencher.iter(
        || {
            let c_handle = spawn_consumer(&queue, &barrier);
            let p_handle = spawn_producer(&queue, &barrier);
            barrier.wait();
            expect!(c_handle.join()).to(be_ok().value(result));
            p_handle.join()
        }
    );
}

#[bench]
fn consumer_producer_small_queue_size(bencher: &mut test::Bencher) {
            let queue_size = (NUMBER_OF_ELEMENTS / 20) as usize;
            // queue size will be 1000 / 20 >> 64
            let queue: ArrayBlockingQueue<i64> = ArrayBlockingQueue::with_capacity(queue_size);
            let barrier = Arc::new(Barrier::new(3));
            let result = sum(NUMBER_OF_ELEMENTS);
            bencher.iter(
                || {
                    let c_handle = spawn_consumer(&queue, &barrier);
                    let p_handle = spawn_producer(&queue, &barrier);
                    barrier.wait();
                    expect!(c_handle.join()).to(be_ok().value(result));
                    p_handle.join()
                }
            );
        }

#[bench]
fn consumer_producer_large_queue_size(bencher: &mut test::Bencher) {
    let queue_size = (NUMBER_OF_ELEMENTS / 2) as usize;
    // queue size will be 1000 / 2 >> 512
    let queue: ArrayBlockingQueue<i64> = ArrayBlockingQueue::with_capacity(queue_size);
    let barrier = Arc::new(Barrier::new(3));
    let result = sum(NUMBER_OF_ELEMENTS);
    bencher.iter(
        || {
            let c_handle = spawn_consumer(&queue, &barrier);
            let p_handle = spawn_producer(&queue, &barrier);
            barrier.wait();
            expect!(c_handle.join()).to(be_ok().value(result));
            p_handle.join()
        }
    );
}

pub fn sum(last: i64) -> i64 {
    (last - 1) * last / 2
}

pub fn spawn_consumer<'q, 'b>(queue: &'q ArrayBlockingQueue<i64>, barrier: &'b Arc<Barrier>) -> JoinHandle<i64> {
    let consume = queue.clone();
    let barrier_cons = barrier.clone();
    thread::spawn(
        move || {
            barrier_cons.wait();
            let mut sum = 0;
            for _ in 0..NUMBER_OF_ELEMENTS {
                sum += consume.dequeue();
            }
            sum
        }
    )
}

pub fn spawn_producer<'q, 'b>(queue: &'q ArrayBlockingQueue<i64>, barrier: &'b Arc<Barrier>) -> JoinHandle<()> {
    let produce = queue.clone();
    let barrier_prod = barrier.clone();
    thread::spawn(
        move || {
            barrier_prod.wait();
            for i in 0..NUMBER_OF_ELEMENTS {
                produce.enqueue(i);
            }
        }
    )
}
