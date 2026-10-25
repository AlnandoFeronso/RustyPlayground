use std::alloc::Layout;

#[derive(Debug)]
pub struct Vejktor<T> {
    data: *mut T,
    size: u64,
    capacity: u64,
}

impl<T> Vejktor<T> {
    pub fn new() -> Self {
        Vejktor {
            data: std::ptr::null_mut(),
            size: 0,
            capacity: 0,
        }
    }

    fn realloc(&mut self, new_capacity: u64) {
        self.capacity = new_capacity
        let layout = Layout::new::<T>(self.capacity);  
        let new_data = allo
    }
}
