use std::{cell::RefCell, rc::Rc};
use crate::IndexedElement;
use super::{singular_shifter::SingularShifter, Shifter};

/// Purpose:
///     To increment each shifter forward, resetting as need be, to ensure that the earliest states of each shifter are attempted before the last states
///     Example:
///         0  0  0
///         1  0  0
///         0  1  0
///         0  0  1
///         1  1  0
///         1  0  1
///         0  1  1
///         1  1  1
///         2  0  0
///         0  2  0
///         0  0  2
///         2  1  0
///         1  2  0
///         1  0  2
///         2  0  1
///         0  2  1
///         0  1  2
///         2  1  1
///         1  2  1
///         1  1  2
///         2  2  0
///         2  0  2
///         0  2  2
///         2  2  1
///         2  1  2
///         1  2  2
///         2  2  2
/// 
///     Square algorithm
///         0  0  0
///         1  0  0
///         0  1  0
///         0  0  1
///         1  1  0
///         1  0  1
///         0  1  1
///         1  1  1
///         2  0  0
///         2  1  0
///         2  0  1
///         2  1  1
///         0  2  0
///         1  2  0
///         0  2  1
///         1  2  1
///         0  0  2
///         1  0  2
///         0  1  2
///         1  1  2
///         2  2  0
///         2  2  1
///         2  0  2
///         2  1  2
///         0  2  2
///         1  2  2
///         2  2  2
/// 
pub struct BreadthFirstSearchShifter<T> {
    shifters: Vec<SingularShifter<T>>,
    length_per_shifter_index: Vec<usize>
}

impl<T> BreadthFirstSearchShifter<T> {
    pub fn new(shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        let mut length_per_shifter_index: Vec<usize> = Vec::new();
        let mut singular_shifters: Vec<SingularShifter<T>> = Vec::new();
        for shifter in shifters {
            length_per_shifter_index.push(shifter.borrow().get_length());
            let singular_shifter = SingularShifter::new(shifter);
            singular_shifters.push(singular_shifter);
        }
        BreadthFirstSearchShifter {
            shifters: singular_shifters,
            length_per_shifter_index: length_per_shifter_index
        }
    }
}

impl<T> Shifter for BreadthFirstSearchShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        todo!();
    }
    fn try_backward(&mut self) -> bool {
        todo!();
    }
    fn try_increment(&mut self) -> bool {
        todo!();
    }
    fn get_indexed_element(&self) -> IndexedElement<Self::T> {
        todo!();
    }
    fn get_length(&self) -> usize {
        todo!();
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        todo!();
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        todo!();
    }
    fn randomize(&mut self) {
        todo!();
    }
}