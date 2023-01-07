use std::rc::Rc;

pub mod incrementer;
// TODO bloom filter wrapper over hashset
pub mod shifter;
pub mod backup;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

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
