extern crate concrust;

pub use self::concrust::primitives::CountDownLatch;
pub use self::concrust::primitives::Semaphore;

pub use std::thread;
pub use std::sync::Arc;
pub use std::time::Duration;

pub use expectest::prelude::{be_equal_to, be_greater_than, be_ok, be_none, be_some};

describe! count_down_latch_prim {

    before_each {
        let latch = CountDownLatch::new(1);
    }

    it "should decrease counts when latch count down" {
        let counts = latch.get_counts();
        latch.count_down();

        expect!(latch.get_counts()).to(be_equal_to(counts - 1));
    }

    it "should await until counts not equals zero" {
        const NUMBER_OF_THREADS: usize = 10;
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let count_down_latch = latch.clone();
            let jh = thread::spawn(
                move || {
                    expect!(count_down_latch.get_counts()).to(be_greater_than(0));

                    count_down_latch.await();

                    expect!(count_down_latch.get_counts()).to(be_equal_to(0));
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));

        latch.count_down();

        expect!(latch.get_counts()).to(be_equal_to(0));
        for jh in results {
            expect!(jh.join()).to(be_ok());
        }
    }
}

describe! semaphore_prim {

    before_each {
        let semaphore = Semaphore::new(1);
    }

    it "should release resource automaticaly" {
        {
            let guard = semaphore.acquire();
            let try_acquire = semaphore.try_acquire();
            expect!(try_acquire).to(be_none());
            drop(guard);
        }

        let try_acquire = semaphore.try_acquire();
        expect!(try_acquire).to(be_some());
    }

    it "should not release more than permissions" {
        semaphore.release();
        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_some());
       // expect!(try_acquire).to(be_some());

        let try_acquire = semaphore.try_acquire();
        expect!(try_acquire).to(be_none());
    }

    it "should block thread until resource will be released" {
        const NUMBER_OF_THREADS: usize = 10;
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        semaphore.acquire();

        for _ in 0..NUMBER_OF_THREADS {
            let semaphore = semaphore.clone();
            let jh = thread::spawn(
                move || {
                    let g = semaphore.acquire();
                    thread::sleep(Duration::from_millis(50));
                    drop(g);
                }
            );
            results.push(jh);
        }
        thread::sleep(Duration::from_millis(50));
        semaphore.release();

        for jh in results {
            expect!(jh.join()).to(be_ok());
        }
    }
}
