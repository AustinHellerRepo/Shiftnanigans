use std::rc::Rc;
use bitvec::vec::BitVec;
use super::Incrementer;

pub struct BinaryDensityIncrementer {
    length: usize,
    current_state: BitVec,
    current_ones_total: usize,
    is_started: bool
}

impl BinaryDensityIncrementer {
    pub fn new(length: usize) -> Self {
        BinaryDensityIncrementer {
            length: length,
            current_state: BitVec::repeat(false, length),
            current_ones_total: 0,
            is_started: false
        }
    }
}

impl Incrementer for BinaryDensityIncrementer {
    type T = bool;

    fn try_increment(&mut self) -> bool {
        if self.current_ones_total == self.length {
            return false;
        }
        if !self.is_started {
            self.is_started = true;
            return true;
        }
        if self.is_started && self.current_ones_total == 0 {
            self.current_state.set(0, true);
            self.current_ones_total = 1;
            return true;
        }
        let mut is_zero_found = false;
        let mut rightmost_zero_before_one_index_option: Option<usize> = None;
        for index in (0..self.length).rev() {
            if self.current_state[index] {
                if is_zero_found && rightmost_zero_before_one_index_option.is_none() {
                    rightmost_zero_before_one_index_option = Some(index + 1);
                }
            }
            else {
                is_zero_found = true;
            }
        }
        // if all of the ones have built up on the end of the state, we need to restart them on the left and add one
        if let Some(index) = rightmost_zero_before_one_index_option {
            self.current_state.set(index - 1, false);
            self.current_state.set(index, true);
            let mut zero_index = index + 1;
            let mut one_index = index + 1;
            while one_index < self.length && !self.current_state[one_index] {
                one_index += 1;
            }
            while one_index < self.length && !self.current_state[zero_index] {
                self.current_state.set(zero_index, true);
                self.current_state.set(one_index, false);
                zero_index += 1;
                one_index += 1;
            }
        }
        else {
            self.current_ones_total += 1;
            for index in 0..self.length {
                let state = index < self.current_ones_total;
                self.current_state.set(index, state);
            }
        }
        return true;
    }
    fn get(&self) -> Vec<crate::IndexedElement<Self::T>> {
        let mut indexed_elements = Vec::new();
        for index in 0..self.length {
            let indexed_element: crate::IndexedElement<Self::T> = crate::IndexedElement::new(Rc::new(self.current_state[index]), index);
            indexed_elements.push(indexed_element);
        }
        return indexed_elements;
    }
    fn reset(&mut self) {
        self.current_state = BitVec::repeat(false, self.length);
        self.current_ones_total = 0;
        self.is_started = false;
    }
    fn randomize(&mut self) {
        todo!();
    }
}

#[cfg(test)]
mod binary_density_incrementer {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeSet};

    use super::*;
    use bitvec::{bits, vec::BitVec};
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn zero_bits() {
        let mut incrementer = BinaryDensityIncrementer::new(0);
        for _ in 0..10 {
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn single_bit() {
        let mut incrementer = BinaryDensityIncrementer::new(1);
        for _ in 0..10 {
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(1, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(1, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
            }
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn two_bits() {
        let mut incrementer = BinaryDensityIncrementer::new(2);
        for _ in 0..10 {
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
            }
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn three_bits() {
        let mut incrementer = BinaryDensityIncrementer::new(3);
        for _ in 0..10 {
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(3, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
            }
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn four_bits() {
        let mut incrementer = BinaryDensityIncrementer::new(4);
        for _ in 0..10 {
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&false, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&false, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&false, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(4, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&true, indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&true, indexed_elements[1].element.as_ref());
                assert_eq!(2, indexed_elements[2].index);
                assert_eq!(&true, indexed_elements[2].element.as_ref());
                assert_eq!(3, indexed_elements[3].index);
                assert_eq!(&true, indexed_elements[3].element.as_ref());
            }
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }
}