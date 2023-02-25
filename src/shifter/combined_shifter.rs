use std::{rc::Rc, cell::RefCell};
use crate::IndexedElement;

use super::{Shifter};

#[derive(Clone)]
pub struct CombinedShifter<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    state_index_mapping_per_shifter_index: Vec<Vec<usize>>,
    possible_states: Vec<Rc<T>>,
    current_shifter_index: Option<usize>,
    index_offset_per_shifter: Vec<usize>,
    shifters_segments_length_total: usize,
    is_shifter_order_preserved_on_randomize: bool
}

impl<T: PartialEq> CombinedShifter<T> {
    pub fn new(shifters: &Vec<Rc<RefCell<dyn Shifter<T = T>>>>, is_shifter_order_preserved_on_randomize: bool) -> Self {
        // TODO determine how to share this functionality between CombinedShifter and ShiftingSquareBreadthFirstSearchShifter
        let mut index_offset_per_shifter: Vec<usize> = Vec::new();
        let mut current_index_offset: usize = 0;
        let mut state_index_mapping_per_shifter_index: Vec<Vec<usize>> = Vec::new();
        let mut possible_states: Vec<Rc<T>> = Vec::new();
        for shifter in shifters.iter() {
            index_offset_per_shifter.push(current_index_offset);
            let mut state_index_mapping: Vec<usize> = Vec::new();
            let current_shifter_possible_states = shifter.borrow().get_states();
            for current_shifter_state in current_shifter_possible_states.iter() {
                let mut existing_state_index: usize = 0;
                let mut is_existing: bool = false;
                for (possible_state_index, possible_state) in possible_states.iter().enumerate() {
                    if possible_state == current_shifter_state {
                        existing_state_index = possible_state_index;
                        is_existing = true;
                        break;
                    }
                }
                if !is_existing {
                    existing_state_index = possible_states.len();
                    possible_states.push(current_shifter_state.clone());
                }
                state_index_mapping.push(existing_state_index);
            }
            state_index_mapping_per_shifter_index.push(state_index_mapping);
            current_index_offset += shifter.borrow().get_length();
        }
        CombinedShifter {
            shifters: shifters.clone(),
            state_index_mapping_per_shifter_index: state_index_mapping_per_shifter_index,
            possible_states: possible_states,
            current_shifter_index: None,
            index_offset_per_shifter: index_offset_per_shifter,
            shifters_segments_length_total: current_index_offset,
            is_shifter_order_preserved_on_randomize: is_shifter_order_preserved_on_randomize
        }
    }
}

impl<T> Shifter for CombinedShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            if self.shifters.len() == 0 {
                return false;
            }
            self.current_shifter_index = Some(0);
        }
        let mut current_shifter_index = self.current_shifter_index.unwrap();
        while current_shifter_index != self.shifters.len() {
            let is_current_shifter_try_forward_successful = self.shifters[current_shifter_index].borrow_mut().try_forward();
            if is_current_shifter_try_forward_successful {
                return true;
            }
            current_shifter_index += 1;
            self.current_shifter_index = Some(current_shifter_index);
        }
        return current_shifter_index != self.shifters.len();
    }
    fn try_backward(&mut self) -> bool {
        if self.current_shifter_index.is_none() || self.shifters.len() == 0 {
            return false;
        }
        if self.current_shifter_index.unwrap() == self.shifters.len() {
            self.current_shifter_index = Some(self.current_shifter_index.unwrap() - 1);
        }
        while self.current_shifter_index.is_some() {
            let is_current_shifter_try_backward_successful = self.shifters[self.current_shifter_index.unwrap()].borrow_mut().try_backward();
            if is_current_shifter_try_backward_successful {
                return true;
            }
            if self.current_shifter_index.unwrap() == 0 {
                self.current_shifter_index = None;
            }
            else {
                self.current_shifter_index = Some(self.current_shifter_index.unwrap() - 1);
            }
        }
        return self.current_shifter_index.is_some();
    }
    fn try_increment(&mut self) -> bool {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let is_current_shifter_try_increment_successful = self.shifters[current_shifter_index].borrow_mut().try_increment();
        return is_current_shifter_try_increment_successful;
    }
    /*fn try_decrement(&mut self) -> bool {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let is_current_shifter_try_decrement_successful = self.shifters[current_shifter_index].borrow_mut().try_decrement();
        return is_current_shifter_try_decrement_successful;
    }*/
    fn get_indexed_element(&self) -> IndexedElement<Self::T> {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let mut indexed_element = self.shifters[current_shifter_index].borrow().get_indexed_element();
        indexed_element.index += self.index_offset_per_shifter[current_shifter_index];
        return indexed_element;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let (mut element_index, mut state_index) = self.shifters[current_shifter_index].borrow().get_element_index_and_state_index();
        element_index += self.index_offset_per_shifter[current_shifter_index];
        state_index = self.state_index_mapping_per_shifter_index[current_shifter_index][state_index];
        return (element_index, state_index);
    }
    fn get_length(&self) -> usize {
        return self.shifters_segments_length_total;
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_states.clone();
    }
    fn randomize(&mut self) {
        // TODO determine if this misorders indexes - should a mapper be used and randomized instead?
        for shifter in self.shifters.iter() {
            shifter.borrow_mut().randomize();
        }
        if !self.is_shifter_order_preserved_on_randomize {
            fastrand::shuffle(&mut self.shifters);
        }
    }
}

#[cfg(test)]
mod combined_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use crate::shifter::{index_shifter::IndexShifter, segment_permutation_shifter::{SegmentPermutationShifter, Segment}};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn permutations_no_shifters() {
        init();
    
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = Vec::new();
        let mut combined_shifter = CombinedShifter::new(&shifters, false);

        for _ in 0..10 {
            assert!(!combined_shifter.try_forward());
        }
        for _ in 0..10 {
            assert!(!combined_shifter.try_backward());
        }
        for _ in 0..10 {
            combined_shifter.reset();
        }
    }

    #[rstest]
    fn permutations_one_shifter_index_shifter() {
        init();

        let states_per_shift: Vec<Vec<Rc<(u8, u8)>>> = vec![
            vec![
                Rc::new((1, 1)),
                Rc::new((2, 2))
            ],
            vec![
                Rc::new((10, 10)),
                Rc::new((11, 11))
            ]
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = vec![
            Rc::new(RefCell::new(IndexShifter::new(&states_per_shift)))
        ];

        let mut combined_shifter = CombinedShifter::new(&shifters, false);
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(1, 1), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, combined_shifter.get_indexed_element().index);
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(10, 10), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());  // move back to the 2nd shift
        assert!(combined_shifter.try_increment());
        assert_eq!(&(11, 11), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());  // move back to the 2nd shift
        assert!(!combined_shifter.try_increment());  // no more states for the 2nd shift
        assert!(combined_shifter.try_backward());  // move back to the 1st shift
        assert!(combined_shifter.try_increment());  // pull the 2nd state for the 1st shift
        assert_eq!(&(2, 2), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, combined_shifter.get_indexed_element().index);
        assert!(combined_shifter.try_forward());  // move to the 2nd shifter
        assert!(combined_shifter.try_increment());  // pull the 1st state for the 2nd shift
        assert_eq!(&(10, 10), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());  // move back to the 2nd shift
        assert!(combined_shifter.try_increment());  // pull the 2nd state for the 2nd shift
        assert_eq!(&(11, 11), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());  // move back to the 2nd shift
        assert!(!combined_shifter.try_increment());  // no more states for the 2nd shift
        assert!(combined_shifter.try_backward());  // move back to the 1st shift
        assert!(!combined_shifter.try_increment());  // no more states in 1st shift
        assert!(!combined_shifter.try_backward());  // done
    }

    #[rstest]
    fn permutations_one_shifter_segment_permutation_shifter() {
        init();

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(1)),
            Rc::new(Segment::new(1))
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = vec![
            Rc::new(RefCell::new(SegmentPermutationShifter::new(segments, (30, 255), 2, true, 0, true)))
        ];
        let mut combined_shifter = CombinedShifter::new(&shifters, false);
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(30, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, combined_shifter.get_indexed_element().index);
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(31, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());  // move back to the 2nd shift
        assert!(!combined_shifter.try_increment());  // nothing left to increment to
        assert!(combined_shifter.try_backward());  // move back to the 1st shift
        assert!(combined_shifter.try_increment());  // pull the 2nd segment as the 1st shift
        assert_eq!(&(30, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(31, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, combined_shifter.get_indexed_element().index);
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());  // move back to the 2nd shift
        assert!(!combined_shifter.try_increment());  // nothing left to increment to
        assert!(combined_shifter.try_backward());  // move back to the 1st shift
        assert!(!combined_shifter.try_increment());  // nothing left to increment to
        assert!(!combined_shifter.try_backward());  // done
    }

    #[rstest]
    fn permutations_two_shifters_segment_permutation_shifter_and_index_shifter() {

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(1)),
            Rc::new(Segment::new(1))
        ];
        let states_per_shift: Vec<Vec<Rc<(u8, u8)>>> = vec![
            vec![
                Rc::new((1, 1)),
                Rc::new((11, 11))
            ]
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = vec![
            Rc::new(RefCell::new(SegmentPermutationShifter::new(segments, (30, 255), 2, true, 0, true))),
            Rc::new(RefCell::new(IndexShifter::new(&states_per_shift)))
        ];
        let mut combined_shifter = CombinedShifter::new(&shifters, false);
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(30, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(31, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(combined_shifter.try_forward());
        assert!(combined_shifter.try_increment());
        assert_eq!(&(1, 1), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(2, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());
        assert!(combined_shifter.try_increment());  // pull the 2nd index for the 2nd shifter's 1st shift
        assert_eq!(&(11, 11), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(2, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(!combined_shifter.try_forward());
        assert!(combined_shifter.try_backward());
        assert!(!combined_shifter.try_increment());
        assert!(combined_shifter.try_backward());  // move back to 1st shifter's 2nd shift
        assert!(!combined_shifter.try_increment());  // nowhere else to move
        assert!(combined_shifter.try_backward());  // move back to 1st shifter's 1st shift
        assert!(combined_shifter.try_increment());  // pull 2nd segment for 1st shifter's 1st shift
        assert_eq!(&(30, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(combined_shifter.try_forward());  // move to 1st shifter's 2nd shift
        assert!(combined_shifter.try_increment());  // pull 1st segment for 1st shifter's 2nd shift
        assert_eq!(&(31, 255), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(combined_shifter.try_forward());  // move to the 2nd shifter's 1st shift
        assert!(combined_shifter.try_increment());  // pull the 1st index for the 2nd shifter's 1st shift
        assert_eq!(&(1, 1), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(2, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(!combined_shifter.try_forward());  // nowhere else to go
        assert!(combined_shifter.try_backward());  // move back to 2nd shifter's 1st shift
        assert!(combined_shifter.try_increment());  // pull the 2nd index for the 2nd shifter's 1st shift
        assert_eq!(&(11, 11), combined_shifter.get_indexed_element().element.as_ref());
        assert_eq!(2, combined_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = combined_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, combined_shifter.get_indexed_element().index);
            assert_eq!(combined_shifter.get_states()[state_index], combined_shifter.get_indexed_element().element);
        }
        assert!(!combined_shifter.try_forward());  // nowhere else to go
        assert!(combined_shifter.try_backward());  // move back to 2nd shifter's 1st shift
        assert!(!combined_shifter.try_increment());  // no other indexes
        assert!(combined_shifter.try_backward());  // move back to 1st shifter's 2nd shift
        assert!(!combined_shifter.try_increment());  // nowhere else to move segment to
        assert!(combined_shifter.try_backward());  // move back to 1st shifter's 1st shift
        assert!(!combined_shifter.try_increment());  // nowhere to move segment to and no other segments to try
        assert!(!combined_shifter.try_backward());  // done
    }
    /*fn decrement_incrementer() {
        todo!();
    }*/
}