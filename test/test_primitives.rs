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