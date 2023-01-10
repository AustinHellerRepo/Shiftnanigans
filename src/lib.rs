use std::rc::Rc;

pub mod incrementer;
// TODO bloom filter wrapper over hashset
pub mod shifter;
pub mod backup;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

pub mod cell_group_localizer;

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
    cells: Vec<(u8, u8)>,  // these should exist such that they can be added directly to location points
    cell_group_type_index: usize  // each type can have relationship attributes (detection location offsets, etc.)
}

pub struct LocatedCellGroup {
    cell_group_index: usize,
    location: Rc<(u8, u8)>
}