use std::{rc::Rc, cell::RefCell};

use crate::{IndexedElement, shifter::Shifter};

use super::{Incrementer};


struct SquareBreadthFirstSearchIncrementer<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    current_square_index: Option<usize>,
    current_edge_index: Option<usize>,
    is_completed: bool,
    max_length: usize
}

impl<T> SquareBreadthFirstSearchIncrementer<T> {
    pub fn new(shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        let mut max_length: Option<usize> = None;
        for shifter in shifters.iter() {
            let length = shifter.borrow().get_length();
            if max_length.is_none() {
                max_length = Some(length);
            }
            else {
                max_length = Some(max_length.unwrap().max(length));
            }
        }
        if max_length.is_none() {
            max_length = Some(0);
        }
        SquareBreadthFirstSearchIncrementer {
            shifters: shifters,
            current_square_index: None,
            current_edge_index: None,
            is_completed: max_length.unwrap() == 0,
            max_length: max_length.unwrap()
        }
    }
}

impl<T> Incrementer for SquareBreadthFirstSearchIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {


        // if the iterator is in a terminal state
        //      return false
        // if the iterator is unstarted
        //      set the current square index to zero
        // if the current dimension is completed (default)
        //      if the current dimension index is none
        //          set the current dimension index to zero
        //      else
        //          increment the current dimension index
        //          if the current dimension index is the self.shifters.len()
        //              set the iterator to a terminal state
        //              return false
        //      move all of the shifters to their first shift index
        //      increment the current dimension index's shifter to the current square index state
        // else
        //      // TODO increment state of non-dimension shifters

        if self.is_completed {
            return false;
        }
        if self.current_square_index.is_none() {
            self.current_square_index = Some(0);
            self.current_edge_index = Some(0);
            for shifter in self.shifters.iter() {
                let borrowed_shifter = shifter.borrow_mut();
            }
        }
        return true;
    }
    fn get(&self) -> Vec<IndexedElement<T>> {
        todo!();
    }
    fn reset(&mut self) {
        todo!();
    }
    fn randomize(&mut self) {
        todo!();
    }
}