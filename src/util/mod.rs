//! Implements general utility traits and datastructures
use std::{any::Any, rc::Rc};

pub mod cell_vec;
pub use cell_vec::CellVec;
pub mod slotmap;
pub use slotmap::SlotMap;

pub trait Downcast {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: Any> Downcast for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}
