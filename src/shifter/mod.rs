pub mod index_shifter;
pub mod segment_permutation_shifter;
pub mod encapsulated_shifter;
use std::rc::Rc;

use crate::IndexedElement;

/// Purpose:
///      To allow for shifting forward-and-backward across elements, incrementing their states individually
///      This would allow for optimizing on situations where states can be skipped immediately without needing to calculate deeper permutations

pub trait Shifter {
    type T;

    fn try_forward(&mut self) -> bool;
    fn try_backward(&mut self) -> bool;
    fn try_increment(&mut self) -> bool;
    fn get(&self) -> IndexedElement<Self::T>;
    fn length(&self) -> usize;

    fn reset(&mut self) {
        while self.try_backward() {
            // move back again
        }
    }
}
