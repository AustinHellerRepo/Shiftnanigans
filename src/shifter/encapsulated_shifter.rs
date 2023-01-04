use std::{rc::Rc, cell::RefCell};
use super::Shifter;

pub struct EncapsulatedShifter<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    current_shifter_index: Option<usize>,
    is_incremented_at_least_once: bool
}

impl<T> EncapsulatedShifter<T> {
    pub fn new(shifters: &Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        EncapsulatedShifter {
            shifters: shifters.clone(),
            current_shifter_index: None,
            is_incremented_at_least_once: false
        }
    }
}
impl<T> Shifter for EncapsulatedShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            self.current_shifter_index = Some(0);
            return true;
        }
        else {
            let current_shifter_index = self.current_shifter_index.unwrap();
            if current_shifter_index == self.shifters.len() {
                return false;
            }
            else {
                let next_shifter_index = current_shifter_index + 1;
                self.current_shifter_index = Some(next_shifter_index);
                if next_shifter_index == self.shifters.len() {
                    return false;
                }
                return true;
            }
        }
    }
    fn try_backward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            return false;
        }
        else {
            let current_shifter_index = self.current_shifter_index.unwrap();
            if current_shifter_index == 0 {
                self.current_shifter_index = None;
                return false;
            }
            else {
                self.current_shifter_index = Some(current_shifter_index - 1);
                return true;
            }
        }
    }
    fn try_increment(&mut self) -> bool {
        if self.shifters.len() == 0 {
            if !self.is_incremented_at_least_once {
                self.is_incremented_at_least_once = true;
                return true;
            }
            else {
                return false;
            }
        }
        else {
            let current_shifter_index = self.current_shifter_index.unwrap();
            let is_increment_successful = self.shifters[current_shifter_index].borrow_mut().try_increment();
            return is_increment_successful;
        }
    }
    fn get(&self) -> Option<Rc<Self::T>> {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let get_option = self.shifters[current_shifter_index].borrow_mut().get();
        return get_option;
    }
}

#[cfg(test)]
mod encapsulated_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialized_no_shifters() {
        init();
    
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (i32, i32)>>>> = Vec::new();
        let _ = EncapsulatedShifter::new(&shifters);
    }
}