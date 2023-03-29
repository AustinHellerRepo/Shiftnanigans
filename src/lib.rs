use std::rc::Rc;

pub mod incrementer;
// TODO bloom filter wrapper over hashset
pub mod shifter;
pub mod pixel_board;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[derive(Debug, PartialEq, Eq)]
pub struct IndexedElement<T> {
    element: Rc<T>,
    index: usize
}

impl<T> Clone for IndexedElement<T> {
    fn clone(&self) -> Self {
        return IndexedElement {
            element: Rc::clone(&self.element),
            index: self.index
        }
    }
}

impl<T> IndexedElement<T> {
    pub fn new(element: Rc<T>, index: usize) -> Self {
        IndexedElement {
            element: element,
            index: index
        }
    }
}

#[derive(Clone)]
pub struct CellGroup {
    cells: Vec<(u8, u8)>  // these should exist such that they can be added directly to location points
}

pub struct LocatedCellGroup {
    cell_group_index: usize,
    location: Rc<(u8, u8)>
}


fn get_n_choose_k(n: u64, k: u64) -> u64 {
    let mut permutations_total: u64 = 1;
    let mut denominator_remainder = k as u64;
    // calculate f(x) = n! / ((n-k)! * k!)
    // start with (k+1) as if n! was already divided by (n-k)!
    // divide out values of k! as they are discovered
    for f_k in (k + 1)..=n {
        permutations_total = permutations_total * f_k;
        if denominator_remainder > 1 && permutations_total > denominator_remainder && permutations_total % denominator_remainder == 0 {
            permutations_total /= denominator_remainder;
            denominator_remainder = denominator_remainder - 1;
        }
    }
    return permutations_total;
}