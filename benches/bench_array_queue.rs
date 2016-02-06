#![feature(test)]

extern crate concrust;
extern crate test;
#[macro_use]
extern crate expectest;

pub use concrust::queue::ArrayBlockingQueue;
pub use concrust::queue::BlockingQueue;

pub use expectest::prelude::{be_equal_to, be_ok};

pub use std::thread;
pub use std::thread::JoinHandle;
pub use std::sync::{Arc, Barrier};
pub use std::result::Result;

const NUMBER_OF_ELEMENTS: i64 = 1000;
const NUMBER_OF_PROCCESS: i64 = 5;

#[bench]
fn multiple_consumers_multiple_producer_default_queue_size(bencher: &mut test::Bencher) {
    let (operation_number, expected_result) = multiple_consumers_producer_setup();
    bencher.iter(
        || {
            let actual_result = multiple_consumers_multiple_producer_iter(
                ArrayBlockingQueue::new(),
                Arc::new(Barrier::new(11)),
                operation_number
            );
            expect!(actual_result).to(be_equal_to(expected_result));
        }
    );
}

#[bench]
fn multiple_consumers_multiple_producer_small_queue_size(bencher: &mut test::Bencher) {
    let (operation_number, expected_result) = multiple_consumers_producer_setup();
    bencher.iter(
        || {
            let actual_result = multiple_consumers_multiple_producer_iter(
                ArrayBlockingQueue::with_capacity((NUMBER_OF_ELEMENTS / 20) as usize),
                Arc::new(Barrier::new(11)),
                operation_number
            );
            expect!(actual_result).to(be_equal_to(expected_result));
        }
    );
}

#[bench]
fn multiple_consumers_multiple_producer_large_queue_size(bencher: &mut test::Bencher) {
    let (operation_number, expected_result) = multiple_consumers_producer_setup();
    bencher.iter(
        || {
            let actual_result = multiple_consumers_multiple_producer_iter(
                ArrayBlockingQueue::with_capacity((NUMBER_OF_ELEMENTS / 2) as usize),
                Arc::new(Barrier::new(11)),
                operation_number
            );
            expect!(actual_result).to(be_equal_to(expected_result));
        }
    );
}

fn multiple_consumers_multiple_producer_iter(queue: ArrayBlockingQueue<i64>, barrier: Arc<Barrier>, oper: i64) -> i64 {
    let mut consumers = Vec::with_capacity(NUMBER_OF_PROCCESS as usize);
    for _ in 0..NUMBER_OF_PROCCESS {
        consumers.push(spawn_consumer(&queue, &barrier, oper));
        spawn_producer(&queue, &barrier, oper);
    }
    barrier.wait();
    consumers.into_iter().map(|c| c.join().unwrap()).fold(0, |acc, v| { acc + v } )
}

fn multiple_consumers_producer_setup() -> (i64, i64) {
    let operation_number = NUMBER_OF_ELEMENTS / NUMBER_OF_PROCCESS;
    (operation_number, sum(operation_number) * NUMBER_OF_PROCCESS)
}

#[bench]
fn single_consumers_multiple_producer_default_queue_size(bencher: &mut test::Bencher) {
    let (operation_number, expected_result) = multiple_consumers_producer_setup();
    bencher.iter(
        || {
            let actual_result = single_consumers_multiple_producer_iter(
                ArrayBlockingQueue::new(),
                Arc::new(Barrier::new(7)),
                operation_number
            );
            expect!(actual_result).to(be_equal_to(expected_result))
        }
    );
}

#[bench]
fn single_consumers_multiple_producer_small_queue_size(bencher: &mut test::Bencher) {
    let (operation_number, expected_result) = multiple_consumers_producer_setup();
    bencher.iter(
        || {
            let actual_result = single_consumers_multiple_producer_iter(
                ArrayBlockingQueue::with_capacity((NUMBER_OF_PROCCESS / 20) as usize),
                Arc::new(Barrier::new(7)),
                operation_number
            );
            expect!(actual_result).to(be_equal_to(expected_result))
        }
    );
}

#[bench]
fn single_consumers_multiple_producer_large_queue_size(bencher: &mut test::Bencher) {
    let (oper, expected_result) = multiple_consumers_producer_setup();
    bencher.iter(
        || {
            let actual_result = single_consumers_multiple_producer_iter(
                ArrayBlockingQueue::with_capacity((NUMBER_OF_PROCCESS / 2) as usize),
                Arc::new(Barrier::new(7)),
                oper
            );
            expect!(actual_result).to(be_equal_to(expected_result))
        }
    );
}

fn single_consumers_multiple_producer_iter(queue: ArrayBlockingQueue<i64>, barrier: Arc<Barrier>, oper: i64) -> i64{
    let c_handle = spawn_consumer(&queue, &barrier, NUMBER_OF_PROCCESS);
    for _ in 0..NUMBER_OF_PROCCESS {
        spawn_producer(&queue, &barrier, oper);
    }
    barrier.wait();
    c_handle.join().unwrap()
}

#[bench]
fn single_consumer_single_producer_default_queue_size(bencher: &mut test::Bencher) {
    let expected_result = single_consumer_single_producer_setup();
    bencher.iter(
        || {
            let actual_result = single_consumer_single_producer_iter(
                ArrayBlockingQueue::new(),
                Arc::new(Barrier::new(3))
            );
            expect!(actual_result).to(be_equal_to(expected_result));
        }
    );
}

#[bench]
fn single_consumer_single_producer_small_queue_size(bencher: &mut test::Bencher) {
    let expected_result = single_consumer_single_producer_setup();
    bencher.iter(
        || {
            let actual_result = single_consumer_single_producer_iter(
                ArrayBlockingQueue::with_capacity((NUMBER_OF_ELEMENTS / 20) as usize),
                Arc::new(Barrier::new(3))
            );
            expect!(actual_result).to(be_equal_to(expected_result));
        }
    );
}

#[bench]
fn single_consumer_single_producer_large_queue_size(bencher: &mut test::Bencher) {
    let expected_result = single_consumer_single_producer_setup();
    bencher.iter(
        || {
            let actual_result = single_consumer_single_producer_iter(
                ArrayBlockingQueue::with_capacity((NUMBER_OF_ELEMENTS / 2) as usize),
                Arc::new(Barrier::new(3))
            );
            expect!(actual_result).to(be_equal_to(expected_result));
        }
    );
}

fn single_consumer_single_producer_iter(queue: ArrayBlockingQueue<i64>, barrier: Arc<Barrier>) -> i64 {
    let handle = spawn_consumer(&queue, &barrier, NUMBER_OF_ELEMENTS);
    spawn_producer(&queue, &barrier, NUMBER_OF_ELEMENTS);
    barrier.wait();
    handle.join().unwrap()
}

fn single_consumer_single_producer_setup() -> i64 {
    sum(NUMBER_OF_ELEMENTS)
}

pub fn sum(last: i64) -> i64 {
    (last - 1) * last / 2
}

pub fn spawn_consumer<'q, 'b>(queue: &'q ArrayBlockingQueue<i64>, barrier: &'b Arc<Barrier>, oper: i64) -> JoinHandle<i64> {
    let consume = queue.clone();
    let barrier_cons = barrier.clone();
    thread::spawn(
        move || {
            barrier_cons.wait();
            let mut sum = 0;
            for _ in 0..oper {
                sum += consume.dequeue();
            }
            sum
        }
    )
}

pub fn spawn_producer<'q, 'b>(queue: &'q ArrayBlockingQueue<i64>, barrier: &'b Arc<Barrier>, oper: i64) {
    let produce = queue.clone();
    let barrier_prod = barrier.clone();
    thread::spawn(
        move || {
            barrier_prod.wait();
            for i in 0..oper {
                produce.enqueue(i);
            }
        }
    );
}
