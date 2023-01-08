use std::{rc::Rc, cell::RefCell};
use crate::IndexedElement;

use super::{Shifter};

pub struct EncapsulatedShifter<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    current_shifter_index: Option<usize>,
    index_offset_per_shifter: Vec<usize>,
    shifters_segments_length_total: usize
}

impl<T> EncapsulatedShifter<T> {
    pub fn new(shifters: &Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        let mut index_offset_per_shifter: Vec<usize> = Vec::new();
        let mut current_index_offset: usize = 0;
        for shifter in shifters.iter() {
            index_offset_per_shifter.push(current_index_offset);
            current_index_offset += shifter.borrow().length();
        }
        EncapsulatedShifter {
            shifters: shifters.clone(),
            current_shifter_index: None,
            index_offset_per_shifter: index_offset_per_shifter,
            shifters_segments_length_total: current_index_offset
        }
    }
}

impl<T> Shifter for EncapsulatedShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            if self.shifters.len() == 0 {
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
            return false;
        }
        else {
            let mut current_shifter_index = self.current_shifter_index.unwrap();
            if current_shifter_index != self.shifters.len() {
                let is_current_shifter_try_backward_successful = self.shifters[current_shifter_index].borrow_mut().try_backward();
                if is_current_shifter_try_backward_successful {
                    return true;
                }
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
        let current_shifter_index = self.current_shifter_index.unwrap();
        let is_current_shifter_try_increment_successful = self.shifters[current_shifter_index].borrow_mut().try_increment();
        return is_current_shifter_try_increment_successful;
    }
    fn get(&self) -> IndexedElement<Self::T> {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let mut current_shifter_get = self.shifters[current_shifter_index].borrow_mut().get();
        current_shifter_get.index += self.index_offset_per_shifter[current_shifter_index];
        return current_shifter_get;
    }
    fn length(&self) -> usize {
        return self.shifters_segments_length_total;
    }
}

#[cfg(test)]
mod encapsulated_shifter_tests {
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
    
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (i32, i32)>>>> = Vec::new();
        let mut encapsulated_shifter = EncapsulatedShifter::new(&shifters);

        for _ in 0..10 {
            assert!(!encapsulated_shifter.try_forward());
        }
        for _ in 0..10 {
            assert!(!encapsulated_shifter.try_backward());
        }
        for _ in 0..10 {
            encapsulated_shifter.reset();
        }
    }

    #[rstest]
    fn permutations_one_shifter_index_shifter() {
        init();

        let states_per_shift: Vec<Vec<Rc<(i32, i32)>>> = vec![
            vec![
                Rc::new((1, 1)),
                Rc::new((2, 2))
            ],
            vec![
                Rc::new((10, 10)),
                Rc::new((11, 11))
            ]
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (i32, i32)>>>> = vec![
            Rc::new(RefCell::new(IndexShifter::new(&states_per_shift)))
        ];

        let mut encapsulated_shifter = EncapsulatedShifter::new(&shifters);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(1, 1), encapsulated_shifter.get().element.as_ref());
        assert_eq!(0, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(10, 10), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(11, 11), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // no more states for the 2nd shift
        assert!(encapsulated_shifter.try_backward());  // move back to the 1st shift
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd state for the 1st shift
        assert_eq!(&(2, 2), encapsulated_shifter.get().element.as_ref());
        assert_eq!(0, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());  // move to the 2nd shifter
        assert!(encapsulated_shifter.try_increment());  // pull the 1st state for the 2nd shift
        assert_eq!(&(10, 10), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd state for the 2nd shift
        assert_eq!(&(11, 11), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // no more states for the 2nd shift
        assert!(encapsulated_shifter.try_backward());  // move back to the 1st shift
        assert!(!encapsulated_shifter.try_increment());  // no more states in 1st shift
        assert!(!encapsulated_shifter.try_backward());  // done
    }

    #[rstest]
    fn permutations_one_shifter_segment_permutation_shifter() {
        init();

        let segments: Vec<Segment> = vec![
            Segment::new(1),
            Segment::new(1)
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (i32, i32)>>>> = vec![
            Rc::new(RefCell::new(SegmentPermutationShifter::new(Rc::new(segments), (30, 300), 2, true, 0)))
        ];
        let mut encapsulated_shifter = EncapsulatedShifter::new(&shifters);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(30, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(0, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(31, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // nothing left to increment to
        assert!(encapsulated_shifter.try_backward());  // move back to the 1st shift
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd segment as the 1st shift
        assert_eq!(&(30, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(31, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(0, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // nothing left to increment to
        assert!(encapsulated_shifter.try_backward());  // move back to the 1st shift
        assert!(!encapsulated_shifter.try_increment());  // nothing left to increment to
        assert!(!encapsulated_shifter.try_backward());  // done
    }

    #[rstest]
    fn permutations_two_shifters_segment_permutation_shifter_and_index_shifter() {

        let segments: Vec<Segment> = vec![
            Segment::new(1),
            Segment::new(1)
        ];
        let states_per_shift: Vec<Vec<Rc<(i32, i32)>>> = vec![
            vec![
                Rc::new((1, 1)),
                Rc::new((11, 11))
            ]
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (i32, i32)>>>> = vec![
            Rc::new(RefCell::new(SegmentPermutationShifter::new(Rc::new(segments), (30, 300), 2, true, 0))),
            Rc::new(RefCell::new(IndexShifter::new(&states_per_shift)))
        ];
        let mut encapsulated_shifter = EncapsulatedShifter::new(&shifters);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(30, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(0, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(31, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(1, 1), encapsulated_shifter.get().element.as_ref());
        assert_eq!(2, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd index for the 2nd shifter's 1st shift
        assert_eq!(&(11, 11), encapsulated_shifter.get().element.as_ref());
        assert_eq!(2, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());
        assert!(!encapsulated_shifter.try_increment());
        assert!(encapsulated_shifter.try_backward());  // move back to 1st shifter's 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // nowhere else to move
        assert!(encapsulated_shifter.try_backward());  // move back to 1st shifter's 1st shift
        assert!(encapsulated_shifter.try_increment());  // pull 2nd segment for 1st shifter's 1st shift
        assert_eq!(&(30, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(1, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());  // move to 1st shifter's 2nd shift
        assert!(encapsulated_shifter.try_increment());  // pull 1st segment for 1st shifter's 2nd shift
        assert_eq!(&(31, 300), encapsulated_shifter.get().element.as_ref());
        assert_eq!(0, encapsulated_shifter.get().index);
        assert!(encapsulated_shifter.try_forward());  // move to the 2nd shifter's 1st shift
        assert!(encapsulated_shifter.try_increment());  // pull the 1st index for the 2nd shifter's 1st shift
        assert_eq!(&(1, 1), encapsulated_shifter.get().element.as_ref());
        assert_eq!(2, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());  // nowhere else to go
        assert!(encapsulated_shifter.try_backward());  // move back to 2nd shifter's 1st shift
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd index for the 2nd shifter's 1st shift
        assert_eq!(&(11, 11), encapsulated_shifter.get().element.as_ref());
        assert_eq!(2, encapsulated_shifter.get().index);
        assert!(!encapsulated_shifter.try_forward());  // nowhere else to go
        assert!(encapsulated_shifter.try_backward());  // move back to 2nd shifter's 1st shift
        assert!(!encapsulated_shifter.try_increment());  // no other indexes
        assert!(encapsulated_shifter.try_backward());  // move back to 1st shifter's 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // nowhere else to move segment to
        assert!(encapsulated_shifter.try_backward());  // move back to 1st shifter's 1st shift
        assert!(!encapsulated_shifter.try_increment());  // nowhere to move segment to and no other segments to try
        assert!(!encapsulated_shifter.try_backward());  // done
    }
}