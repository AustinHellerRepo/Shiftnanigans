use std::{rc::Rc, cell::RefCell};
use super::Incrementer;

pub struct RoundRobinIncrementer<T> {
    incrementers: Vec<Rc<RefCell<dyn Incrementer<T = T>>>>,
    current_available_indexes: Vec<usize>,
    current_available_indexes_index: usize,
    is_completed: bool
}

impl<T> RoundRobinIncrementer<T> {
    pub fn new(incrementers: Vec<Rc<RefCell<dyn Incrementer<T = T>>>>) -> Self {
        let mut current_available_indexes: Vec<usize> = Vec::new();
        let is_completed = incrementers.len() == 0;
        if !is_completed {
            for index in 0..incrementers.len() {
                current_available_indexes.push(index);
            }
        }
        RoundRobinIncrementer {
            incrementers: incrementers,
            current_available_indexes: current_available_indexes,
            current_available_indexes_index: 0,
            is_completed: is_completed
        }
    }
}

impl<T> Incrementer for RoundRobinIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {
        if self.is_completed {
            return false;
        }
        let mut incrementer_index: usize = self.current_available_indexes[self.current_available_indexes_index];
        while !self.incrementers[incrementer_index].borrow_mut().try_increment() {
            self.current_available_indexes.remove(self.current_available_indexes_index);
            if self.current_available_indexes.len() == 0 {
                self.is_completed = true;
                return false;
            }
            if self.current_available_indexes_index == self.current_available_indexes.len() {
                self.current_available_indexes_index = 0;
            }
            incrementer_index = self.current_available_indexes[self.current_available_indexes_index];
        }
        return true;
    }
    fn get(&self) -> Vec<crate::IndexedElement<Self::T>> {
        let incrementer_index: usize = self.current_available_indexes[self.current_available_indexes_index];
        return self.incrementers[incrementer_index].borrow().get();
    }
    fn reset(&mut self) {
        self.is_completed = self.incrementers.len() == 0;
        if !self.is_completed {
            self.current_available_indexes.clear();
            self.current_available_indexes_index = 0;
            for index in 0..self.incrementers.len() {
                self.current_available_indexes.push(index);
                self.incrementers[index].borrow_mut().reset();
            }
        }
    }
    fn randomize(&mut self) {
        for incrementer in self.incrementers.iter() {
            incrementer.borrow_mut().randomize();
        }
        fastrand::shuffle(&mut self.incrementers);
    }
}