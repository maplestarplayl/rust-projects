use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}
// This is redudant because UnsafeCell already implements !Sync
// impl<T> !Sync for Cell<T> {}
impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }
    // if get returns a reference, in some cases, it will be unsafe to use it 
    // ex: after get, set will be unsafe since the reference is invalidated
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.value.get() }
    }
    pub fn set(&self, value: T) {
        unsafe { *self.value.get() = value };
    }
}
