extern crate concurrent;

#[cfg(test)]
mod count_down_latch_prim {

    use super::concurrent::primitives::CountDownLatch;

    use std::thread;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn it_should_decrease_counts_when_latch_count_down() {
        let latch = CountDownLatch::new(1);
        let counts = latch.get_counts();
        latch.count_down();

        assert_eq!(latch.get_counts(), counts - 1);
    }

    #[test]
    fn thread_should_await_until_counts_not_equals_zero() {
        const NUMBER_OF_THREADS: usize = 10;
        let arc = Arc::new(CountDownLatch::new(1));
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

#[cfg(test)]
mod semaphore_prim {

    use super::concurrent::primitives::Semaphore;

    use std::thread;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn it_should_create_a_semaphore() {
        Semaphore::new(10);
    }

    #[test]
    fn it_should_release_resource_automaticaly() {
        let semaphore = Semaphore::new(1);

        {
            let guard = semaphore.acquire();
            let try_acquire = semaphore.try_acquire();
            assert!(try_acquire.is_none());
            drop(guard);
        }

        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_some());
    }

    #[test]
    fn it_should_not_release_more_than_permissions() {
        let semaphore = Semaphore::new(1);

        semaphore.release();
        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_some());

        let try_acquire = semaphore.try_acquire();
        assert!(try_acquire.is_none());
    }

    #[test]
    fn it_should_block_thread_until_resource_will_be_released() {
        const NUMBER_OF_THREADS: usize = 10;
        let arc = Arc::new(Semaphore::new(1));
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
