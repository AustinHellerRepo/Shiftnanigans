use crate::IndexedElement;

pub mod shifting_cell_group_dependency_incrementer;
pub mod round_robin_incrementer;
pub mod square_breadth_first_search_incrementer;
pub mod paired_square_breadth_first_search_incrementer;
pub mod shifter_incrementer;
pub mod binary_density_incrementer;

pub trait Incrementer {
    type T;

    fn try_increment(&mut self) -> bool;
    fn get(&self) -> Vec<IndexedElement<Self::T>>;
    fn reset(&mut self);
    fn randomize(&mut self);
}