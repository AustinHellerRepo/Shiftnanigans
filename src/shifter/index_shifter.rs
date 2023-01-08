use std::{collections::VecDeque, rc::Rc};

use crate::IndexedElement;

use super::{Shifter};

pub struct IndexShifter<T> {
    current_shift_index: Option<usize>,
    current_state_index_per_shift_index: VecDeque<Option<usize>>,
    is_incremented_at_least_once_per_shift_index: VecDeque<bool>,
    possible_states: Vec<Rc<T>>,
    state_indexes_per_shift_index: Vec<Vec<usize>>,
    shifts_length: usize
}

impl<T: PartialEq> IndexShifter<T> {
    pub fn new(states_per_shift_index: &Vec<Vec<Rc<T>>>) -> Self {
        let shifts_length: usize = states_per_shift_index.len();
        let mut possible_states: Vec<Rc<T>> = Vec::new();
        let mut state_indexes_per_shift_index: Vec<Vec<usize>> = Vec::new();
        for states in states_per_shift_index.iter() {
            let mut state_indexes: Vec<usize> = Vec::new();
            for state in states.iter() {
                let mut state_index: usize = 0;
                let mut is_existing = false;
                for (existing_state_index, existing_state) in possible_states.iter().enumerate() {
                    if existing_state == state {
                        state_index = existing_state_index;
                        is_existing = true;
                        break;
                    }
                }
                if !is_existing {
                    state_index = possible_states.len();
                    possible_states.push(state.clone());
                }
                state_indexes.push(state_index);
            }
            state_indexes_per_shift_index.push(state_indexes);
        }
        IndexShifter {
            current_shift_index: None,
            current_state_index_per_shift_index: VecDeque::new(),
            is_incremented_at_least_once_per_shift_index: VecDeque::new(),
            possible_states: possible_states,
            state_indexes_per_shift_index: state_indexes_per_shift_index,
            shifts_length: shifts_length
        }
    }
}

impl<T> Shifter for IndexShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_shift_index.is_none() {
            if self.shifts_length == 0 {
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
            if current_shift_index == self.shifts_length {
                return false;
            }
            else {
                let next_shift_index = current_shift_index + 1;
                self.current_shift_index = Some(next_shift_index);
                if next_shift_index == self.shifts_length {
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
            if current_shift_index != self.shifts_length {
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
        else if self.current_shift_index.unwrap() == self.shifts_length {
            return false;
        }
        let current_shift_index = self.current_shift_index.unwrap();
        let current_state_index_option = self.current_state_index_per_shift_index[current_shift_index];
        if current_state_index_option.is_none() {
            if self.state_indexes_per_shift_index[current_shift_index].len() == 0 {
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
            if current_state_index == self.state_indexes_per_shift_index[current_shift_index].len() {
                return false;
            }
            else {
                let next_state_index = current_state_index + 1;
                self.current_state_index_per_shift_index[current_shift_index] = Some(next_state_index);
                if next_state_index == self.state_indexes_per_shift_index[current_shift_index].len() {
                    return false;
                }
                return true;
            }
        }
    }
    fn get_indexed_element(&self) -> IndexedElement<T> {
        let (element_index, state_index) = self.get_element_index_and_state_index();
        let element = self.possible_states[state_index].clone();
        return IndexedElement::new(element, element_index);
    }
    fn get_length(&self) -> usize {
        return self.shifts_length;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        let current_shift_index = self.current_shift_index.unwrap();
        let current_state_index = self.current_state_index_per_shift_index[current_shift_index].unwrap();
        return (current_shift_index, self.state_indexes_per_shift_index[current_shift_index][current_state_index]);
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_states.clone();
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
                let indexed_element = index_shifter.get_indexed_element();
                assert_eq!(index as i32 % states_total as i32, indexed_element.element.0);
                assert_eq!(index as i32 / states_total as i32, indexed_element.element.1);
                let (element_index, state_index) = index_shifter.get_element_index_and_state_index();
                assert_eq!(element_index, indexed_element.index);
                let state = index_shifter.get_states()[state_index].clone();
                assert_eq!(state, indexed_element.element);
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
