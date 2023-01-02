use std::{collections::VecDeque, rc::Rc};

use super::Shifter;

pub struct IndexShifter<T> {
    current_shift_index: Option<usize>,
    current_state_index_per_shift_index: VecDeque<Option<usize>>,
    is_incremented_at_least_once_per_shift_index: VecDeque<bool>,
    states_per_shift_index: Vec<Vec<Rc<T>>>
}

impl<T> IndexShifter<T> {
    pub fn new(states_per_shift_index: &Vec<Vec<Rc<T>>>) -> Self {
        IndexShifter {
            current_shift_index: None,
            current_state_index_per_shift_index: VecDeque::new(),
            is_incremented_at_least_once_per_shift_index: VecDeque::new(),
            states_per_shift_index: states_per_shift_index.clone()
        }
    }
}

impl<T> Shifter for IndexShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_shift_index.is_none() {
            if self.states_per_shift_index.len() == 0 {
                return false;
            }
            else {
                self.current_shift_index = Some(0);
                self.current_state_index_per_shift_index.push_back(None);
                self.is_incremented_at_least_once_per_shift_index.push_back(false);
                return true;
            }
        }
        else {
            let current_shift_index: usize = self.current_shift_index.unwrap();
            if current_shift_index == self.states_per_shift_index.len() {
                return false;
            }
            else {
                let next_shift_index = current_shift_index + 1;
                self.current_shift_index = Some(next_shift_index);
                if next_shift_index == self.states_per_shift_index.len() {
                    // if the shifter moves past the end, fail at this point so that when the next shifter fails to move backward, the idea is that you always try to have the previous shifter move backward too and then increment.
                    return false;
                }
                self.current_state_index_per_shift_index.push_back(None);
                self.is_incremented_at_least_once_per_shift_index.push_back(false);
                return true;
            }
        }
    }
    fn try_backward(&mut self) -> bool {
        if self.current_shift_index.is_none() {
            return false;
        }
        else {
            let current_shift_index = self.current_shift_index.unwrap();
            if current_shift_index != self.states_per_shift_index.len() {
                self.current_state_index_per_shift_index.pop_back();
            }
            if current_shift_index == 0 {
                self.current_shift_index = None;
                return false;
            }
            else {
                self.current_shift_index = Some(current_shift_index - 1);
                return true;
            }
        }
    }
    fn try_increment(&mut self) -> bool {
        let current_shift_index = self.current_shift_index.unwrap();
        let current_state_index_option = self.current_state_index_per_shift_index[current_shift_index];
        if current_state_index_option.is_none() {
            if self.states_per_shift_index[current_shift_index].len() == 0 {
                if !self.is_incremented_at_least_once_per_shift_index[current_shift_index] {
                    self.is_incremented_at_least_once_per_shift_index[current_shift_index] = true;
                    return true;
                }
                else {
                    return false;
                }
            }
            else {
                self.current_state_index_per_shift_index[current_shift_index] = Some(0);
                return true;
            }
        }
        else {
            let current_state_index = current_state_index_option.unwrap();
            if current_state_index == self.states_per_shift_index[current_shift_index].len() {
                return false;
            }
            else {
                let next_state_index = current_state_index + 1;
                self.current_state_index_per_shift_index[current_shift_index] = Some(next_state_index);
                if next_state_index == self.states_per_shift_index[current_shift_index].len() {
                    return false;
                }
                return true;
            }
        }
    }
    fn get(&self) -> Option<Rc<T>> {
        let current_shift_index = self.current_shift_index.unwrap();
        let current_state_index_option = self.current_state_index_per_shift_index[current_shift_index];
        if current_state_index_option.is_none() {
            return None;
        }
        else {
            return Some(self.states_per_shift_index[current_shift_index][current_state_index_option.unwrap()].clone());
        }
    }
}

#[cfg(test)]
mod index_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialized_no_states() {
        init();

        let states_per_shift_index: Vec<Vec<Rc<(i32, i32)>>> = Vec::new();
        let _ = IndexShifter::new(&states_per_shift_index);
    }
}
