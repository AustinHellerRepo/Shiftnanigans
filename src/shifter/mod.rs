pub mod index_shifter;
pub mod segment_permutation_shifter;
pub mod combined_shifter;
pub mod singular_shifter;
pub mod shifting_square_breadth_first_search_shifter;
pub mod scaling_square_breadth_first_search_shifter;
pub mod hyper_graph_cliche_shifter;
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
    //fn try_decrement(&mut self) -> bool;  // TODO implement
    // returns the current indexed element such that the IndexedElement.index is the same as element_index() and the IndexedElement.element is the same as states()[state_index()]
    fn get_indexed_element(&self) -> IndexedElement<Self::T>;
    // returns the number of shifts, so the number of valid forward movements
    fn get_length(&self) -> usize;
    // returns the current element index and current state index which can be used against the states()
    fn get_element_index_and_state_index(&self) -> (usize, usize);
    // returns the distinct states possible from this shifter
    fn get_states(&self) -> Vec<Rc<Self::T>>;
    fn randomize(&mut self);

    fn reset(&mut self) {
        while self.try_backward() {
            // move back again
        }
    }
}
