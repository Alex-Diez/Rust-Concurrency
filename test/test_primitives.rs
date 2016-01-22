extern crate concrust;

pub use self::concrust::primitives::CountDownLatch;
pub use self::concrust::primitives::Semaphore;

pub use std::thread;
pub use std::sync::Arc;
pub use std::time::Duration;

describe! count_down_latch_prim {

    before_each {
        let latch = CountDownLatch::new(1);
    }

    it "should decrease counts when latch count down" {
        let counts = latch.get_counts();
        latch.count_down();

        assert_eq!(latch.get_counts(), counts - 1);
    }

    it "should await until counts not equals zero" {
        const NUMBER_OF_THREADS: usize = 10;
        let arc = Arc::new(latch);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        for _ in 0..NUMBER_OF_THREADS {
            let count_down_latch = arc.clone();
            let jh = thread::spawn(
                move || {
                    assert!(count_down_latch.get_counts() > 0);

                    count_down_latch.await();

                    assert!(count_down_latch.get_counts() == 0);
                }
            );
            results.push(jh);
        }

        thread::sleep(Duration::from_millis(100));

        arc.count_down();

        assert_eq!(arc.get_counts(), 0);
        for jh in results {
            let res = jh.join();
            assert!(res.is_ok());
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
            assert!(try_acquire.is_none());
            drop(guard);
        }

        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_some());
    }

    it "should not release more than permissions" {
        semaphore.release();
        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_some());

        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_none());
    }

    it "should block thread until resource will be released" {
        const NUMBER_OF_THREADS: usize = 10;
        let arc = Arc::new(semaphore);
        let mut results = Vec::with_capacity(NUMBER_OF_THREADS);

        arc.acquire();

        for _ in 0..NUMBER_OF_THREADS {
            let semaphore = arc.clone();
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
        arc.release();

        for jh in results {
            let res = jh.join();
            assert!(res.is_ok());
        }
    }
}
