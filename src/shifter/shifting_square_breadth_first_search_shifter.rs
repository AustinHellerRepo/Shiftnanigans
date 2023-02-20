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
pub struct ShiftingSquareBreadthFirstSearchShifter<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    length_per_shifter_index: Vec<usize>,
    current_indexed_elements: Vec<IndexedElement<T>>,
    current_shifter_index: Option<usize>,
    length: usize
}

impl<T> ShiftingSquareBreadthFirstSearchShifter<T> {
    pub fn new(shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        let mut length_per_shifter_index: Vec<usize> = Vec::new();
        let mut length = 0;
        for shifter in shifters.iter() {
            let borrowed_shifter = shifter.borrow();
            let shifter_length = borrowed_shifter.get_length();
            length_per_shifter_index.push(shifter_length);
            length += shifter_length;
        }
        ShiftingSquareBreadthFirstSearchShifter {
            shifters: shifters,
            length_per_shifter_index: length_per_shifter_index,
            current_indexed_elements: Vec::new(),
            current_shifter_index: None,
            length: length
        }
    }
}

impl<T> Shifter for ShiftingSquareBreadthFirstSearchShifter<T> {
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