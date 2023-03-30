use std::rc::Rc;
use bitvec::vec::BitVec;
use crate::IndexedElement;
use super::Incrementer;
pub struct BinaryValueIncrementer {
    length: usize,
    current_state: BitVec,
    is_started: bool
}

impl BinaryValueIncrementer {
    pub fn new(length: usize) -> Self {
        BinaryValueIncrementer {
            length: length,
            current_state: BitVec::repeat(false, length),
            is_started: false
        }
    }
}

impl Incrementer for BinaryValueIncrementer {
    type T = bool;

    fn try_increment(&mut self) -> bool {
        if self.current_state.count_ones() == self.length {
            return false;
        }
        if !self.is_started {
            self.is_started = true;
            return true;
        }
        for index in 0..self.length {
            if self.current_state[index] {
                self.current_state.set(index, false);
            }
            else {
                self.current_state.set(index, true);
                break;
            }
        }
        return true;
    }
    fn get(&self) -> Vec<IndexedElement<Self::T>> {
        let mut indexed_elements = Vec::new();
        for index in 0..self.length {
            let indexed_element: IndexedElement<Self::T> = IndexedElement::new(Rc::new(self.current_state[index]), index);
            indexed_elements.push(indexed_element);
        }
        return indexed_elements;
    }
    fn reset(&mut self) {
        self.current_state = BitVec::repeat(false, self.length);
        self.is_started = false;
    }
    fn randomize(&mut self) {
        todo!();
    }
}

impl Iterator for BinaryValueIncrementer {
    type Item = Vec<IndexedElement<bool>>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.try_increment() {
            return Some(self.get());
        }
        return None;
    }
}

#[cfg(test)]
mod binary_value_incrementer {
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
        let mut incrementer = BinaryValueIncrementer::new(0);
        for _ in 0..10 {
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn single_bit() {
        let mut incrementer = BinaryValueIncrementer::new(1);
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
        let mut incrementer = BinaryValueIncrementer::new(2);
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
        let mut incrementer = BinaryValueIncrementer::new(3);
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
        let mut incrementer = BinaryValueIncrementer::new(4);
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