use std::rc::Rc;

use crate::IndexedElement;

pub mod shifting_cell_group_dependency_incrementer;

pub trait Incrementer {
    type T;

    fn try_increment(&mut self) -> bool;
    fn get(&self) -> Vec<IndexedElement<Self::T>>;
    fn reset(&mut self);
}