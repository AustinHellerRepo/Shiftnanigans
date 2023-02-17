use std::{rc::Rc, cell::RefCell};
use crate::{shifter::Shifter, IndexedElement};
use super::Incrementer;

pub struct ShifterIncrementer<T> {
    shifter: Rc<RefCell<dyn Shifter<T = T>>>,
    is_started: bool,
    is_completed: bool,
    current_indexed_elements: Vec<IndexedElement<T>>,
    shifter_length: usize
}

impl<T> ShifterIncrementer<T> {
    pub fn new(shifter: Rc<RefCell<dyn Shifter<T = T>>>) -> Self {
        let shifter_length = shifter.borrow().get_length();
        ShifterIncrementer {
            shifter: shifter,
            is_started: shifter_length == 0,
            is_completed: shifter_length == 0,
            current_indexed_elements: Vec::new(),
            shifter_length: shifter_length
        }
    }
}

impl<T> Incrementer for ShifterIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {
        if self.is_completed {
            return false;
        }
        let mut borrowed_shifter = self.shifter.borrow_mut();
        if !self.is_started {
            for _ in 0..self.shifter_length {
                if !borrowed_shifter.try_forward() {
                    self.is_completed = true;
                    return false;
                }
                if !borrowed_shifter.try_increment() {
                    panic!("Unexpectedly failed to increment shifter after moving forward to shift.");
                }
                let indexed_element = borrowed_shifter.get_indexed_element();
                self.current_indexed_elements.push(indexed_element);
            }
            return self.current_indexed_elements.len() != 0;
        }
        self.current_indexed_elements.pop();
        while self.current_indexed_elements.len() != self.shifter_length {
            if borrowed_shifter.try_increment() {
                let indexed_element = borrowed_shifter.get_indexed_element();
                self.current_indexed_elements.push(indexed_element);
                if self.current_indexed_elements.len() != self.shifter_length {
                    if !borrowed_shifter.try_forward() {
                        panic!("Unexpectedly failed to move forward when not at the end.");
                    }
                }
            }
            else {
                if self.current_indexed_elements.len() == 0 {
                    self.is_completed = true;
                    return false;
                }
                self.current_indexed_elements.pop();
                if !borrowed_shifter.try_backward() {
                    panic!("Unexpectedly failed to move backward when not at the beginning.");
                }
            }
        }
        return true;
    }
    fn get(&self) -> Vec<IndexedElement<Self::T>> {
        return self.current_indexed_elements.clone();
    }
    fn reset(&mut self) {
        self.shifter.borrow_mut().reset();
        self.is_started = false;
        self.is_completed = false;
        self.current_indexed_elements.clear();

    }
    fn randomize(&mut self) {
        self.shifter.borrow_mut().randomize();
    }
}

#[cfg(test)]
mod shifter_incrementer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeSet};

    use crate::shifter::segment_permutation_shifter::{SegmentPermutationShifter, Segment};

    use super::*;
    use bitvec::{bits, vec::BitVec};
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn two_segment_permutation_shifters() {
        init();

        let mut shifter_incrementer = ShifterIncrementer::new(Rc::new(RefCell::new(SegmentPermutationShifter::new(
            vec![
                Rc::new(Segment::new(1)),
                Rc::new(Segment::new(1))
            ],
            (10, 100),
            4,
            true,
            1,
            false
        ))));

        assert!(shifter_incrementer.try_increment());
        {
            let indexed_elements = shifter_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
        }
        assert!(shifter_incrementer.try_increment());
        {
            let indexed_elements = shifter_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
        }
        assert!(shifter_incrementer.try_increment());
        {
            let indexed_elements = shifter_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
        }
        assert!(!shifter_incrementer.try_increment());
    }
}