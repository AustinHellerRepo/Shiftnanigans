use std::{rc::Rc, cell::RefCell};
use crate::IndexedElement;
use super::Shifter;

pub struct SingularShifter<T> {
    shifter: Rc<RefCell<dyn Shifter<T = T>>>,
    is_started: bool,
    is_completed: bool
}

impl<T> SingularShifter<T> {
    pub fn new(shifter: Rc<RefCell<dyn Shifter<T = T>>>) -> Self {
        SingularShifter {
            shifter: shifter,
            is_started: false,
            is_completed: false
        }
    }
    pub fn get_internal_shifter_length(&self) -> usize {
        // TODO determine if recursive check is needed if self.shifter is a SingularShifter
        return self.shifter.borrow().get_length();
    }
}

impl<T> Shifter for SingularShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.is_completed {
            return false;
        }
        if self.is_started {
            self.is_completed = true;
            return false;
        }
        self.is_started = true;
        if !self.shifter.borrow_mut().try_forward() {
            self.is_completed = true;
            return false;
        }
        return true;
    }
    fn try_backward(&mut self) -> bool {
        if self.is_completed {
            self.is_completed = false;
            self.shifter.borrow_mut().reset();
            return true;
        }
        if self.is_started {
            self.is_started = false;
            return false;
        }
        return false;
    }
    fn try_increment(&mut self) -> bool {
        if self.is_completed || !self.is_started {
            return false;
        }
        let mut shifter = self.shifter.borrow_mut();
        while !shifter.try_increment() {
            if !shifter.try_forward() {
                self.is_completed = true;
                return false;
            }
        }
        return true;
    }
    fn get_indexed_element(&self) -> IndexedElement<Self::T> {
        return self.shifter.borrow().get_indexed_element();
    }
    fn get_length(&self) -> usize {
        return 1;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        return self.shifter.borrow().get_element_index_and_state_index();
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.shifter.borrow().get_states();
    }
    fn randomize(&mut self) {
        self.shifter.borrow_mut().randomize();
    }
}