use std::cell::UnsafeCell;
#[derive(Clone, Copy)]
enum RefState {
    Unshared,
    Shared(usize),
    Exclusive,
}
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: crate::cell::Cell<RefState>,
}

impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: crate::cell::Cell::new(RefState::Unshared),
        }
    }
    pub fn borrow(&self) -> Option<&T> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Shared(1));
                Some(unsafe {
                    &*self.value.get()
                })
            }
            RefState::Shared(n) => {
                self.state.set(RefState::Shared(n + 1));
                Some(unsafe {
                    &*self.value.get()
                })
            }
            RefState::Exclusive => panic!("already borrowed"),
        }
    }

    pub fn borrow_mut(&self) -> Option<&mut T> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Exclusive);
                Some(unsafe { &mut *self.value.get() })
            }
            RefState::Shared(_) | RefState::Exclusive => panic!("already borrowed"),
        }
    }
}
