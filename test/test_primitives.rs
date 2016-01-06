extern crate concurrent;

#[cfg(test)]
mod count_down_latch_prim {

    use super::concurrent::primitives::CountDownLatch;

    #[test]
    fn it_should_decrease_counts_when_latch_count_down() {
        let mut latch = CountDownLatch::new(1);
        let counts = latch.get_counts();
        latch.count_down();

        assert_eq!(latch.get_counts(), counts-1);
    }

    
}

#[cfg(test)]
mod semaphore_prim {

    use super::concurrent::primitives::ResourceHolderSemaphore;

    use std::thread;

    #[test]
    fn it_should_not_acquired_more_than_semaphore_contains_with_try_acquire() {
        let mut s = ResourceHolderSemaphore::new(3);

        let res1 = s.try_acquire();
        let res2 = s.try_acquire();
        let res3 = s.try_acquire();
        let res4 = s.try_acquire();

        assert_eq!(res1, Some(3));
        assert_eq!(res2, Some(2));
        assert_eq!(res3, Some(1));
        assert_eq!(res4, None);
    }

    #[test]
    fn thread_should_try_to_acquire_resource() {
        let mut s = ResourceHolderSemaphore::new(1);

        let res = s.acquire();
        thread::spawn(
            move || {
                let try_res = s.try_acquire();
                assert_eq!(try_res, None);
            }
        );
        assert_eq!(res, 1);
    }

    #[test]
    fn it_should_acquire_each_time_new_resource() {
        let mut s = ResourceHolderSemaphore::new(3);

        let res3 = s.acquire();
        let res2 = s.acquire();

        assert_eq!(res3, 3);
        assert_eq!(res2, 2);
    }

    #[test]
    fn acquire_released_resource() {
        let mut s = ResourceHolderSemaphore::new(3);

        let res = s.acquire();
        assert_eq!(res, 3);
        s.release();

        let reacquired_res = s.acquire();
        assert_eq!(reacquired_res, 3);
    }

    #[test]
    fn acquire_update_release_acquire_resource_with_new_value() {
        let mut s = ResourceHolderSemaphore::new(1);
        let res_index = s.acquire();

        update_and_released(&mut s, 10, res_index);

        let res_index = s.acquire();
        let res_val = s.get(res_index);

        assert_eq!(res_val, 10);
    }

    fn update_and_released(semaphore: &mut ResourceHolderSemaphore, update_to: usize, index: usize) {
        semaphore.update(index, update_to);
        semaphore.release();
    }

    #[test]
    fn acquire_update_released_multiple_resources() {
        let mut s = ResourceHolderSemaphore::new(3);
        let res_index3 = s.acquire();
        let res_index2 = s.acquire();
        let res_index1 = s.acquire();

        update_and_released(&mut s, 10, res_index3);
        update_and_released(&mut s, 20, res_index2);
        update_and_released(&mut s, 30, res_index1);

        let res_index3 = s.acquire();
        let res_index2 = s.acquire();
        let res_index1 = s.acquire();

        assert_eq!(s.get(res_index3), 10);
        assert_eq!(s.get(res_index2), 20);
        assert_eq!(s.get(res_index1), 30);
    }

    #[test]
    fn it_should_not_modify_semaphore_resource_when_modify_local_value_only() {
        let mut s = ResourceHolderSemaphore::new(1);
        let index = s.acquire();

        let res = s.get(index) + 10;

        assert_eq!(res, 10);
        assert_eq!(s.get(index), 0);
    }

    //todo
    //1. test when two thread modify different resources
    //2. find a way how to prevent use resource when thread release it
    //3. test for locking thread on acquire method if there is no free resources
    // optinal / specific functionality
    //4. acquire resourse which was released previously by the thread
}