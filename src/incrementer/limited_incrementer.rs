use std::{rc::Rc, cell::RefCell};

use crate::IndexedElement;

use super::Incrementer;


pub struct LimitedIncrementer<T> {
    incrementer: Box<dyn Incrementer<T = T>>,
    length: usize,
    current_index: Option<usize>,
    is_completed: bool
}

impl<T> LimitedIncrementer<T> {
    pub fn new(incrementer: Box<dyn Incrementer<T = T>>, length: usize) -> Self {
        LimitedIncrementer {
            incrementer: incrementer,
            length: length,
            current_index: None,
            is_completed: length == 0
        }
    }
}

impl<T> Incrementer for LimitedIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {
        if self.is_completed {
            return false;
        }
        if let Some(current_index) = self.current_index {
            let next_index = current_index + 1;
            if next_index == self.length {
                self.is_completed = true;
                return false;
            }
            self.current_index = Some(next_index);
        }
        else {
            self.current_index = Some(0);
        }
        return self.incrementer.try_increment();
    }
    fn get(&self) -> Vec<IndexedElement<Self::T>> {
        return self.incrementer.get();
    }
    fn reset(&mut self) {
        self.incrementer.reset();
        self.is_completed = self.length == 0;
        self.current_index = None;
    }
    fn randomize(&mut self) {
        self.incrementer.randomize();
    }
}

#[cfg(test)]
mod limited_incrementer_tests {
    use rstest::rstest;


    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn limit_one_with_possible_one() {
        init();

        // create an ShifterIncrementer with an IndexShifter with one possible value
    }
}