use std::option::Option;

pub struct Semaphore {
    permitions: usize
}

impl Semaphore {
    
    pub fn new(permitions: usize) -> Semaphore {
        Semaphore { permitions: permitions }
    }

    pub fn acquire(&mut self) -> Option<usize> {
        Some(self.permitions)
    }

    pub fn try_acquire(&mut self) -> Option<usize> {
        None
    }
}
