use std::{iter::Peekable, cell::RefCell};

use crate::IndexedElement;

pub mod shifting_cell_group_dependency_incrementer;
pub mod round_robin_incrementer;
pub mod shifter_incrementer;
pub mod binary_density_incrementer;
pub mod binary_value_incrementer;
pub mod limited_incrementer;

pub trait Incrementer {
    type T;

    fn try_increment(&mut self) -> bool;
    fn get(&self) -> Vec<IndexedElement<Self::T>>;
    fn reset(&mut self);
    fn randomize(&mut self);
}

impl<T> Iterator for dyn Incrementer<T = T> {
    type Item = Vec<IndexedElement<T>>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.try_increment() {
            return Some(self.get());
        }
        return None;
    }
}
