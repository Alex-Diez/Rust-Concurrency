extern crate concurrent;

#[cfg(test)]
mod semaphore_prim {

    use super::concurrent::primitives::Semaphore;

    use std::thread;

    #[test]
    fn it_should_create_semaphore() {
        Semaphore::new(1);
    }

    #[test]
    fn it_should_acquired_more_than_semaphore_contains() {
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
}