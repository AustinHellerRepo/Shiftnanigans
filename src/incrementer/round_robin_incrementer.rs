use std::{rc::Rc, cell::RefCell};
use super::Incrementer;

pub struct RoundRobinIncrementer<T> {
    incrementers: Vec<Rc<RefCell<dyn Incrementer<T = T>>>>,
    current_available_indexes: Vec<usize>,
    current_available_indexes_index: Option<usize>,
    is_completed: bool
}

impl<T> RoundRobinIncrementer<T> {
    pub fn new(incrementers: Vec<Rc<RefCell<dyn Incrementer<T = T>>>>) -> Self {
        let mut current_available_indexes: Vec<usize> = Vec::new();
        let is_completed = incrementers.len() == 0;
        if !is_completed {
            for index in 0..incrementers.len() {
                current_available_indexes.push(index);
            }
        }
        RoundRobinIncrementer {
            incrementers: incrementers,
            current_available_indexes: current_available_indexes,
            current_available_indexes_index: None,
            is_completed: is_completed
        }
    }
}

impl<T> Incrementer for RoundRobinIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {
        if self.is_completed {
            return false;
        }
        if let Some(mut current_available_indexes_index) = self.current_available_indexes_index {
            current_available_indexes_index += 1;
            if current_available_indexes_index == self.current_available_indexes.len() {
                self.current_available_indexes_index = Some(0);
            }
            else {
                self.current_available_indexes_index = Some(current_available_indexes_index);
            }
        }
        else {
            self.current_available_indexes_index = Some(0);
        }
        let mut incrementer_index: usize = self.current_available_indexes[self.current_available_indexes_index.unwrap()];
        while !self.incrementers[incrementer_index].borrow_mut().try_increment() {
            debug!("removing incrementer {incrementer_index}");
            self.current_available_indexes.remove(self.current_available_indexes_index.unwrap());
            if self.current_available_indexes.len() == 0 {
                debug!("removed all incrementers");
                self.is_completed = true;
                return false;
            }
            if self.current_available_indexes_index.unwrap() == self.current_available_indexes.len() {
                self.current_available_indexes_index = Some(0);
            }
            incrementer_index = self.current_available_indexes[self.current_available_indexes_index.unwrap()];
        }
        return true;
    }
    fn get(&self) -> Vec<crate::IndexedElement<Self::T>> {
        let incrementer_index: usize = self.current_available_indexes[self.current_available_indexes_index.unwrap()];
        let indexed_elements = self.incrementers[incrementer_index].borrow().get();
        let mut offset_indexed_elements: Vec<crate::IndexedElement<T>> = Vec::new();
        for indexed_element in indexed_elements {
            //let offset_indexed_element: crate::IndexedElement<T> = crate::IndexedElement::new(indexed_element.element, indexed_element.index + self.current_indexed_element_index_offset);
            offset_indexed_elements.push(indexed_element);
        }
        return offset_indexed_elements;
    }
    fn reset(&mut self) {
        self.is_completed = self.incrementers.len() == 0;
        if !self.is_completed {
            self.current_available_indexes.clear();
            self.current_available_indexes_index = None;
            for index in 0..self.incrementers.len() {
                self.current_available_indexes.push(index);
                self.incrementers[index].borrow_mut().reset();
            }
        }
    }
    fn randomize(&mut self) {
        for incrementer in self.incrementers.iter() {
            incrementer.borrow_mut().randomize();
        }
        fastrand::shuffle(&mut self.incrementers);
    }
}

#[cfg(test)]
mod round_robin_incrementer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeSet};

    use crate::{incrementer::shifter_incrementer::ShifterIncrementer, shifter::segment_permutation_shifter::{SegmentPermutationShifter, Segment}};

    use super::*;
    use bitvec::{bits, vec::BitVec};
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn two_shifter_incrementers() {
        let mut round_robin_incrementer = RoundRobinIncrementer::new(vec![
            Rc::new(RefCell::new(ShifterIncrementer::new(
                Rc::new(RefCell::new(SegmentPermutationShifter::new(
                    vec![
                        Rc::new(Segment::new(1)),
                        Rc::new(Segment::new(1))
                    ],
                    (10, 100),
                    4,
                    true,
                    1,
                    false
                ))),
                vec![0, 1]
            ))),
            Rc::new(RefCell::new(ShifterIncrementer::new(
                Rc::new(RefCell::new(SegmentPermutationShifter::new(
                    vec![
                        Rc::new(Segment::new(1)),
                        Rc::new(Segment::new(1))
                    ],
                    (20, 200),
                    4,
                    false,
                    1,
                    false
                ))),
                vec![2, 3]
            )))
        ]);

        assert!(round_robin_incrementer.try_increment());
        {
            let indexed_elements = round_robin_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
        }
        assert!(round_robin_incrementer.try_increment());
        {
            let indexed_elements = round_robin_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(2, indexed_elements[0].index);
            assert_eq!(&(20, 200), indexed_elements[0].element.as_ref());
            assert_eq!(3, indexed_elements[1].index);
            assert_eq!(&(20, 202), indexed_elements[1].element.as_ref());
        }
        assert!(round_robin_incrementer.try_increment());
        {
            let indexed_elements = round_robin_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
        }
        assert!(round_robin_incrementer.try_increment());
        {
            let indexed_elements = round_robin_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(2, indexed_elements[0].index);
            assert_eq!(&(20, 200), indexed_elements[0].element.as_ref());
            assert_eq!(3, indexed_elements[1].index);
            assert_eq!(&(20, 203), indexed_elements[1].element.as_ref());
        }
        assert!(round_robin_incrementer.try_increment());
        {
            let indexed_elements = round_robin_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
        }
        assert!(round_robin_incrementer.try_increment());
        {
            let indexed_elements = round_robin_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(2, indexed_elements[0].index);
            assert_eq!(&(20, 201), indexed_elements[0].element.as_ref());
            assert_eq!(3, indexed_elements[1].index);
            assert_eq!(&(20, 203), indexed_elements[1].element.as_ref());
        }
        assert!(!round_robin_incrementer.try_increment());
    }
}