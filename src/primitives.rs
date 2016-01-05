use std::option::Option;
use std::collections::HashMap;

pub struct Semaphore {
    resources: HashMap<usize, usize>,
    current_permissions: usize,
}

impl Semaphore {
    pub fn new(permissions: usize) -> Semaphore {
        let mut resources = HashMap::with_capacity(permissions);
        for i in (1..permissions+1) {
            resources.insert(i, 0);
        }
        Semaphore { resources: resources, current_permissions: permissions }
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
       let res_val = self.resources.get_mut(&index);
       *(res_val.unwrap()) = val;
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        self.resources.get(&index).map(|v| *v)
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
