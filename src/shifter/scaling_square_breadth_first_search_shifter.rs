use std::rc::Rc;
use bitvec::vec::BitVec;
use crate::{IndexedElement, incrementer::{binary_density_incrementer::BinaryDensityIncrementer, Incrementer}};
use super::Shifter;

// Purpose:
// This represents an IndexShifter of sorts that expands the depth of the element indexes gradually.
// Example:
// The index of each person's favorite toy is sorted lowest index to highest index. The result is that the elements returned first are always the most favorites first.
pub struct ScalingSquareBreadthFirstSearchShifter {
    length: usize,
    maximum_scale: usize,
    current_scale: usize,
    binary_density_incrementer: BinaryDensityIncrementer,
    current_binary_density_mask: BitVec,
    current_scale_per_index: Vec<Option<usize>>,
    current_index: Option<usize>,
    possible_states: Vec<Rc<usize>>
}

impl ScalingSquareBreadthFirstSearchShifter {
    pub fn new(length: usize, maximum_scale: usize) -> Self {
        let mut possible_states: Vec<Rc<usize>> = Vec::new();
        for index in 0..=maximum_scale {
            possible_states.push(Rc::new(index));
        }
        ScalingSquareBreadthFirstSearchShifter {
            length: length,
            maximum_scale: maximum_scale,
            current_scale: 0,
            binary_density_incrementer: BinaryDensityIncrementer::new(length),
            current_binary_density_mask: BitVec::repeat(false, length),
            current_scale_per_index: Vec::new(),
            current_index: None,
            possible_states: possible_states
        }
    }
    pub fn get_scaling_index(&self) -> usize {
        // TODO cache all possible indexes to reduce memory footprint
        let current_index = self.current_index.unwrap();
        return self.current_scale_per_index[current_index].unwrap();
    }
}

impl Shifter for ScalingSquareBreadthFirstSearchShifter {
    type T = usize;

    fn try_forward(&mut self) -> bool {
        if self.length == 0 {
            return false;
        }
        if self.current_index.is_none() {
            if !self.binary_density_incrementer.try_increment() {
                panic!("Unexpectedly failed to do the initial increment of the binary density incrementer.");
            }
            // if the binary density incrementer would have any influence, since the all-zero state should only exist for when the self.current_scale equals zero
            if self.current_scale != 0 {
                if !self.binary_density_incrementer.try_increment() {
                    panic!("Unexpectedly failed to do the second increment of the binary density incrementer.");
                }
            }
            self.current_index = Some(0);
            self.current_scale_per_index.push(None);
            // store the next binary density mask
            self.current_binary_density_mask = BitVec::repeat(false, self.length);
            for indexed_element in self.binary_density_incrementer.get() {
                if *indexed_element.element.as_ref() {
                    self.current_binary_density_mask.set(indexed_element.index, true);
                }
            }
            return true;
        }
        if self.current_index.unwrap() == self.length {
            return false;
        }
        self.current_index = Some(self.current_index.unwrap() + 1);
        if self.current_index.unwrap() == self.length {
            return false;
        }
        self.current_scale_per_index.push(None);
        return true;
    }
    fn try_backward(&mut self) -> bool {
        if self.length == 0 {
            return false;
        }
        if self.current_index.is_none() {
            return false;
        }
        let current_index = self.current_index.unwrap();
        if current_index != self.length {
            self.current_scale_per_index.pop();
        }
        if current_index == 0 {
            self.current_index = None;
            self.current_scale = 0;
            self.binary_density_incrementer.reset();
            return false;
        }
        self.current_index = Some(current_index - 1);
        return true;
    }
    fn try_increment(&mut self) -> bool {
        if self.current_index.is_none() {
            return false;
        }
        let current_index = self.current_index.unwrap();
        // if this the first increment at this index
        if self.current_scale_per_index[current_index].is_none() {
            debug!("try_increment: setting current_scale_per_index at {:?} based on current_scale {:?} and current_binary_density_mask {:?}.", current_index, self.current_scale, self.current_binary_density_mask);
            if self.current_binary_density_mask[current_index] {
                self.current_scale_per_index[current_index] = Some(self.current_scale);
            }
            else {
                self.current_scale_per_index[current_index] = Some(0);
            }
            return true;
        }
        // if the current index is at the limit
        let current_scale = self.current_scale_per_index[current_index].unwrap();
        // if the current_scale represents a binary density point or is not a binary density point that has reached the end
        if current_scale == self.current_scale || current_scale == self.current_scale - 1 {
            // is the current index the first index, permitting an increment of the binary density mask
            if current_index == 0 {
                self.current_scale_per_index.pop();
                // if we are at the end of this self.current_shift
                // need to increment the binary density and resize self.current_scale_per_index
                if self.current_scale == 0 || !self.binary_density_incrementer.try_increment() {
                    if self.current_scale == self.maximum_scale {
                        // reached the end
                        return false;
                    }
                    self.current_scale += 1;
                    self.binary_density_incrementer.reset();
                    if !self.binary_density_incrementer.try_increment() {
                        panic!("Unexpectedly failed to perform initial increment of binary density incrementer after previously succeeding.");
                    }
                    if !self.binary_density_incrementer.try_increment() {
                        panic!("Unexpectedly failed to perform second increment of binary density incrementer after previously succeeding.");
                    }
                    // fall through and collect the updated mask and set the 0th self.current_scale_per_index
                }
                // reset self.current_binary_density_mask
                self.current_binary_density_mask = BitVec::repeat(false, self.length);
                for indexed_element in self.binary_density_incrementer.get() {
                    if *indexed_element.element.as_ref() {
                        self.current_binary_density_mask.set(indexed_element.index, true);
                    }
                }
                // initialize the 0th element
                if self.current_binary_density_mask[0] {
                    self.current_scale_per_index.push(Some(self.current_scale));
                }
                else {
                    self.current_scale_per_index.push(Some(0));
                }
                return true;
            }
            // the current index has reached the maximum permitted
            return false;
        }
        // the current index is not at the maximum scale yet
        self.current_scale_per_index[current_index] = Some(current_scale + 1);
        return true;
    }
    fn get_indexed_element(&self) -> IndexedElement<Self::T> {
        // TODO cache all possible indexes to reduce memory footprint
        let current_index = self.current_index.unwrap();
        let current_scale = self.current_scale_per_index[current_index].unwrap();
        return IndexedElement::new(self.possible_states[current_scale].clone(), current_index);
    }
    fn get_length(&self) -> usize {
        return self.length;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        let current_index = self.current_index.unwrap();
        return (current_index, self.current_scale_per_index[current_index].unwrap());
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_states.clone();
    }
    fn randomize(&mut self) {
        todo!();
    }
}

#[cfg(test)]
mod scaling_square_breadth_first_search_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn zero_length_zero_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(0, 0);
        assert!(!shifter.try_forward());
    }

    #[rstest]
    fn one_length_zero_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(1, 0);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn one_length_one_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(1, 1);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn one_length_two_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(1, 2);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_length_zero_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(2, 0);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_length_one_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(2, 1);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn three_length_two_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(3, 2);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_length_three_scale() {
        init();

        let mut shifter = ScalingSquareBreadthFirstSearchShifter::new(2, 3);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&3, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&0, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&3, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&1, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&3, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&2, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&3, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&3, indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&3, indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }
}