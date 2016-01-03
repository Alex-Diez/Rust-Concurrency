extern crate concurrent;

#[cfg(test)]
mod semaphore_prim {

    use super::concurrent::primitives::Semaphore;

    #[test]
    fn it_should_create_semaphore() {
        Semaphore::new(1);
    }
}