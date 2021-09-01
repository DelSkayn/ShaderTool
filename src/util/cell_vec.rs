use std::cell::UnsafeCell;

/// A version of vector which is has only immutable operations
/// All of which are save because you are not allowed to obtain a reference to an internal value.
/// Offcourse this vector does not implement Sync
pub struct CellVec<T>(UnsafeCell<Vec<T>>);

unsafe impl<T: Send> Send for CellVec<T> {}

impl<T: Clone> CellVec<T> {
    pub fn new() -> Self {
        CellVec(UnsafeCell::new(Vec::new()))
    }

    pub fn push(&self, value: T) {
        unsafe { (*self.0.get()).push(value) }
    }

    pub fn pop(&self) -> Option<T> {
        unsafe { (*self.0.get()).pop() }
    }

    pub fn get(&self, index: usize) -> T {
        unsafe { (*self.0.get())[index].clone() }
    }

    pub fn clear(&self) {
        unsafe { (*self.0.get()).clear() }
    }
}
