use std::option::Option;

pub struct Semaphore {
    resource: usize,
    current_permissions: usize,
}

impl Semaphore {
    pub fn new(permissions: usize) -> Semaphore {
        Semaphore { resource: 0, current_permissions: permissions }
    }

    pub fn acquire(&mut self) -> Option<usize> {
        self.acuqire_resource()
    }

    pub fn try_acquire(&mut self) -> Option<usize> {
        self.acuqire_resource()
    }

    pub fn release(&mut self) {
        self.current_permissions += 1;
    }

    pub fn update(&mut self, index: usize, val: usize) {
        self.resource = val;
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        Some(self.resource)
    }

    fn acuqire_resource(&mut self) -> Option<usize> {
        if self.current_permissions > 0 {
            let res = self.current_permissions;
            self.current_permissions -= 1;
            Some(res)
        } else {
            None
        }   
    }
}
