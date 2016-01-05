extern crate concurrent;

#[cfg(test)]
mod semaphore_prim {

    use super::concurrent::primitives::Semaphore;

    use std::thread;

    #[test]
    fn it_should_not_acquired_more_than_semaphore_contains_with_try_acquire() {
        let mut s = Semaphore::new(3);

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
        let mut s = Semaphore::new(1);

        let res = s.acquire();
        thread::spawn(
            move || {
                let try_res = s.try_acquire();
                assert_eq!(try_res, None);
            }
        );
        assert_eq!(res, Some(1));
    }

    #[test]
    fn it_should_acquire_each_time_new_resource() {
        let mut s = Semaphore::new(3);

        let res3 = s.acquire();
        let res2 = s.acquire();

        assert_eq!(res3, Some(3));
        assert_eq!(res2, Some(2));
    }

    #[test]
    fn acquire_released_resource() {
        let mut s = Semaphore::new(3);

        let res = s.acquire();
        assert_eq!(res, Some(3));
        s.release();

        let reacquired_res = s.acquire();
        assert_eq!(reacquired_res, Some(3));
    }

    #[test]
    fn acquire_update_release_acquire_resource_with_new_value() {
        let mut s = Semaphore::new(1);
        let res_index = s.acquire();

        update_and_released(&mut s, 10, res_index.unwrap());

        let res_index = s.acquire();
        let res_val = s.get(res_index.unwrap());

        assert_eq!(res_val, Some(10));
    }

    fn update_and_released(semaphore: &mut Semaphore, update_to: usize, index: usize) {
        semaphore.update(index, update_to);
        semaphore.release();
    }

    #[test]
    fn acquire_update_released_multiple_resources() {
        let mut s = Semaphore::new(3);
        let res_index3 = s.acquire();
        let res_index2 = s.acquire();
        let res_index1 = s.acquire();

        update_and_released(&mut s, 10, res_index3.unwrap());
        update_and_released(&mut s, 20, res_index2.unwrap());
        update_and_released(&mut s, 30, res_index1.unwrap());

        let res_index3 = s.acquire();
        let res_index2 = s.acquire();
        let res_index1 = s.acquire();

        assert_eq!(s.get(res_index3.unwrap()), Some(10));
        assert_eq!(s.get(res_index2.unwrap()), Some(20));
        assert_eq!(s.get(res_index1.unwrap()), Some(30));
    }

    #[test]
    fn it_should_not_modify_semaphore_resource_when_modify_local_value_only() {
        let mut s = Semaphore::new(1);
        let index = s.acquire();

        let res = s.get(index.unwrap());
        let res = res.map(|v| v + 10);

        assert_eq!(res, Some(10));
        assert_eq!(s.get(index.unwrap()), Some(0));
    }

    //todo
    //1. test when two thread modify different resources
    //2. find a way how to prevent use resource when thread release it
    //3. test for locking thread on acquire method if there is no free resources
    // optinal / specific functionality
    //4. acquire resourse which was released previously by the thread
}