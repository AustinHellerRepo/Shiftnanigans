use std::rc::Rc;
use bitvec::vec::BitVec;
use crate::IndexedElement;
use super::Incrementer;
pub struct FixedBinaryDensityIncrementer {
    length: usize,
    density: usize,
    current_state: BitVec,
    current_ones_total: usize,
    is_started: bool
}

impl FixedBinaryDensityIncrementer {
    pub fn new(density: usize, remaining_length: usize) -> Self {
        let current_state = {
            // start with the density on the left and the remaining are false
            let mut left = BitVec::repeat(true, density);
            let mut right: BitVec = BitVec::repeat(false, remaining_length);
            left.append(&mut right);
            left
        };
        FixedBinaryDensityIncrementer {
            length: density + remaining_length,
            density,
            current_state,
            current_ones_total: density,
            is_started: false
        }
    }
}

impl Incrementer for FixedBinaryDensityIncrementer {
    type T = bool;

    fn try_increment(&mut self) -> bool {
        if self.current_ones_total != self.density || self.length == 0 {
            return false;
        }
        if !self.is_started {
            self.is_started = true;
            return true;
        }
        if self.is_started && self.current_ones_total == 0 {
            // we would need to change the density, so the current value is invalid
            self.current_ones_total += 1;
            return false;
        }
        let mut is_zero_found = false;
        let mut rightmost_zero_before_one_index_option: Option<usize> = None;
        for index in (0..self.length).rev() {
            if self.current_state[index] {
                if is_zero_found && rightmost_zero_before_one_index_option.is_none() {
                    rightmost_zero_before_one_index_option = Some(index + 1);
                    break;
                }
            }
            else {
                is_zero_found = true;
            }
        }
        if let Some(index) = rightmost_zero_before_one_index_option {
            // move the rightmost one to the right since we've guaranteed that it was empty
            self.current_state.set(index - 1, false);
            self.current_state.set(index, true);

            // move all of the ones built up on the right side of the structure back to the point where we just moved a bit over
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
            // we would need to change the density, so the current value is invalid
            self.current_ones_total += 1;
            return false;
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
        self.current_state = {
            // start with the density on the left and the remaining are false
            let mut left = BitVec::repeat(true, self.density);
            let mut right: BitVec = BitVec::repeat(false, self.length - self.density);
            left.append(&mut right);
            left
        };
        self.current_ones_total = self.density;
        self.is_started = false;
    }
    fn randomize(&mut self) {
        todo!();
    }
}

impl Iterator for FixedBinaryDensityIncrementer {
    type Item = Vec<IndexedElement<bool>>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.try_increment() {
            return Some(self.get());
        }
        return None;
    }
}

#[cfg(test)]
mod fixed_binary_density_incrementer {
    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn zero_bits() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(0, 0);
        for _ in 0..10 {
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn single_bit_with_density_zero() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(0, 1);
        for _ in 0..10 {
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(1, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&false, indexed_elements[0].element.as_ref());
            }
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn single_bit_with_density_one() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(1, 0);
        for _ in 0..10 {
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
    fn two_bits_with_density_zero() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(0, 2);
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
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn two_bits_with_density_one() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(1, 1);
        for _ in 0..10 {
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
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn two_bits_with_density_two() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(2, 0);
        for _ in 0..10 {
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
    fn three_bits_with_density_one() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(1, 2);
        for _ in 0..10 {
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
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn four_bits_with_density_two() {
        let mut incrementer = FixedBinaryDensityIncrementer::new(2, 2);
        for _ in 0..10 {
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
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }
}