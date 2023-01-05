use std::{rc::Rc, cell::RefCell};
use super::Shifter;

pub struct EncapsulatedShifter<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    current_shifter_index: Option<usize>,
    is_forward_at_least_once: bool,
    is_incremented_at_least_once: bool
}

impl<T> EncapsulatedShifter<T> {
    pub fn new(shifters: &Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        EncapsulatedShifter {
            shifters: shifters.clone(),
            current_shifter_index: None,
            is_forward_at_least_once: false,
            is_incremented_at_least_once: false
        }
    }
}
impl<T> Shifter for EncapsulatedShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            if self.shifters.len() == 0 {
                if !self.is_forward_at_least_once {
                    self.is_forward_at_least_once = true;
                    return true;
                }
                return false;
            }
            self.current_shifter_index = Some(0);
        }
        else {
            let mut current_shifter_index = self.current_shifter_index.unwrap();
            if current_shifter_index != self.shifters.len() {
                let is_current_shifter_try_forward_successful = self.shifters[current_shifter_index].borrow_mut().try_forward();
                if is_current_shifter_try_forward_successful {
                    return true;
                }
                current_shifter_index += 1;
                self.current_shifter_index = Some(current_shifter_index);
            }
            if current_shifter_index == self.shifters.len() {
                return false;
            }
        }
        return self.shifters[self.current_shifter_index.unwrap()].borrow_mut().try_forward();
    }
    fn try_backward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            if self.shifters.len() == 0 {
                if self.is_forward_at_least_once {
                    self.is_forward_at_least_once = false;
                    return true;
                }
                return false;
            }
            return false;
        }
        else {
            let mut current_shifter_index = self.current_shifter_index.unwrap();
            let is_current_shifter_try_backward_successful = self.shifters[current_shifter_index].borrow_mut().try_backward();
            if is_current_shifter_try_backward_successful {
                return true;
            }
            if current_shifter_index == 0 {
                self.current_shifter_index = None;
                return false;
            }
            current_shifter_index -= 1;
            self.current_shifter_index = Some(current_shifter_index);
            return self.shifters[current_shifter_index].borrow_mut().try_backward();
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
            let is_current_shifter_try_increment_successful = self.shifters[current_shifter_index].borrow_mut().try_increment();
            return is_current_shifter_try_increment_successful;
        }
    }
    fn get(&self) -> Option<Rc<Self::T>> {
        if self.shifters.len() == 0 {
            return None;
        }
        else {
            let current_shifter_index = self.current_shifter_index.unwrap();
            let current_shifter_get_option = self.shifters[current_shifter_index].borrow_mut().get();
            return current_shifter_get_option;
        }
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