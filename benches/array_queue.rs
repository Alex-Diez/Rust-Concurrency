#![feature(test)]
extern crate test;
extern crate concrust;
#[macro_use]
extern crate expectest;

use concrust::queue::ArrayBlockingQueue;
use concrust::queue::BlockingQueue;

use expectest::prelude::be_ok;

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Barrier};

const NUMBER_OF_ELEMENTS: i64 = 1000;

#[bench]
fn consumer_producer(b: &mut test::Bencher) {
    let queue: ArrayBlockingQueue<i64> = ArrayBlockingQueue::new();
    let barrier = Arc::new(Barrier::new(3));
    let result = sum(NUMBER_OF_ELEMENTS);
    b.iter(
        || {
            let c_handle = spawn_consumer(&queue, &barrier);
            let p_handle = spawn_producer(&queue, &barrier);
            barrier.wait();
            expect!(c_handle.join()).to(be_ok().value(result));
            p_handle.join()
        }
    );
}

fn sum(last: i64) -> i64 {
    (last - 1) * last / 2
}

fn spawn_consumer<'q, 'b>(queue: &'q ArrayBlockingQueue<i64>, barrier: &'b Arc<Barrier>) -> JoinHandle<i64> {
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

fn spawn_producer<'q, 'b>(queue: &'q ArrayBlockingQueue<i64>, barrier: &'b Arc<Barrier>) -> JoinHandle<()> {
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
