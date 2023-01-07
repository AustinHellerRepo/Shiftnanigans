use std::{collections::VecDeque, rc::Rc};

use crate::IndexedElement;

use super::{Shifter};

pub struct IndexShifter<T> {
    current_shift_index: Option<usize>,
    current_state_index_per_shift_index: VecDeque<Option<usize>>,
    is_incremented_at_least_once_per_shift_index: VecDeque<bool>,
    states_per_shift_index: Vec<Vec<Rc<T>>>,
    shifts_length: usize
}

impl<T> IndexShifter<T> {
    pub fn new(states_per_shift_index: &Vec<Vec<Rc<T>>>) -> Self {
        let shifts_length: usize = states_per_shift_index.len();
        IndexShifter {
            current_shift_index: None,
            current_state_index_per_shift_index: VecDeque::new(),
            is_incremented_at_least_once_per_shift_index: VecDeque::new(),
            states_per_shift_index: states_per_shift_index.clone(),
            shifts_length: shifts_length
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
        if self.current_shift_index.is_none() {
            return false;
        }
        else if self.current_shift_index.unwrap() == self.states_per_shift_index.len() {
            return false;
        }
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
    fn get(&self) -> IndexedElement<T> {
        let current_shift_index = self.current_shift_index.unwrap();
        let current_state_index = self.current_state_index_per_shift_index[current_shift_index].unwrap();
        let element = self.states_per_shift_index[current_shift_index][current_state_index].clone();
        return IndexedElement::new(element, current_shift_index);
    }
    fn length(&self) -> usize {
        return self.shifts_length;
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

    #[rstest]
    #[case(1, 1)]
    #[case(1, 2)]
    #[case(2, 1)]
    #[case(2, 2)]
    #[case(3, 1)]
    #[case(3, 2)]
    #[case(3, 3)]
    #[case(1, 3)]
    #[case(2, 3)]
    fn shift_through_different_states(#[case] states_total: usize, #[case] shifts_total: usize) {
        init();

        let mut states_per_shift_index: Vec<Vec<Rc<(i32, i32)>>> = Vec::new();
        for shift_index in 0..shifts_total {
            let mut states: Vec<Rc<(i32, i32)>> = Vec::new();
            for state_index in 0..states_total {
                states.push(Rc::new((state_index as i32, shift_index as i32)));
            }
            states_per_shift_index.push(states);
        }
        let mut index_shifter = IndexShifter::new(&states_per_shift_index);
        for _ in 0..10 {
            assert!(!index_shifter.try_backward());
            for index in 0..(shifts_total * states_total) {
                debug!("index: {:?}", index);
                if index % states_total == 0 {
                    assert!(!index_shifter.try_increment());
                    assert!(index_shifter.try_forward());
                }
                assert!(index_shifter.try_increment());
                let get = index_shifter.get();
                assert_eq!(index as i32 % states_total as i32, get.element.0);
                assert_eq!(index as i32 / states_total as i32, get.element.1);
            }
            assert!(!index_shifter.try_forward());
            for index in 0..shifts_total {
                debug!("index: {:?}", index);
                assert!(index_shifter.try_backward());
            }
        }
        assert!(!index_shifter.try_backward());
    }
}
