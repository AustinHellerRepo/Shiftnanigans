use std::{rc::Rc, cell::RefCell};
use crate::{shifter::Shifter, IndexedElement};
use super::Incrementer;

pub struct PairedSquareBreadthFirstSearchIncrementer<T> {
    incrementers: Vec<Rc<RefCell<dyn Incrementer<T = T>>>>,
    current_size: Option<usize>,
    current_x: Option<usize>,
    current_y: Option<usize>,
    current_indexed_elements_per_incrementer_index: Vec<Vec<IndexedElement<T>>>,
    is_completed: bool
}

impl<T> PairedSquareBreadthFirstSearchIncrementer<T> {
    pub fn new(incrementers: (Rc<RefCell<dyn Incrementer<T = T>>>, Rc<RefCell<dyn Incrementer<T = T>>>)) -> Self {
        let indexed_elements_per_incrementer_index: Vec<Vec<IndexedElement<T>>> = Vec::with_capacity(2);
        PairedSquareBreadthFirstSearchIncrementer {
            incrementers: vec![incrementers.0, incrementers.1],
            current_size: None,
            current_x: None,
            current_y: None,
            current_indexed_elements_per_incrementer_index: indexed_elements_per_incrementer_index,
            is_completed: false
        }
    }
}

impl<T> Incrementer for PairedSquareBreadthFirstSearchIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {

        if self.is_completed {
            return false;
        }

        if self.current_size.is_none() {

            for incrementer in self.incrementers.iter() {
                let mut borrowed_incrementer = incrementer.borrow_mut();
                if !borrowed_incrementer.try_increment() {
                    self.current_indexed_elements_per_incrementer_index.clear();
                    self.is_completed = true;
                    return false;
                }
                let indexed_elements = borrowed_incrementer.get();
                self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
            }
            self.current_size = Some(1);
            self.current_x = Some(0);
            self.current_y = Some(0);
            return true;
        }

        // if the previous step was one that ended in a square shape
        if self.current_x.unwrap() == self.current_y.unwrap() {

            // remove previous indexed elements
            self.current_indexed_elements_per_incrementer_index.clear();

            // try to move x once to the right
            if self.incrementers[0].borrow_mut().try_increment() {
                self.incrementers[1].borrow_mut().reset();
                if !self.incrementers[1].borrow_mut().try_increment() {
                    panic!("Unexpectedly failed to increment y incrementer after succeeding before.");
                }
                self.current_size = Some(self.current_size.unwrap() + 1);
                self.current_x = Some(self.current_x.unwrap() + 1);
                self.current_y = Some(0);

                for incrementer in self.incrementers.iter() {
                    let borrowed_incrementer = incrementer.borrow();
                    let indexed_elements = borrowed_incrementer.get();
                    self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
                }

                return true;
            }

            // x was not able to move to the new size, so reset x and move y to the new size, starting a tall rectangle
            if self.incrementers[1].borrow_mut().try_increment() {
                self.incrementers[0].borrow_mut().reset();
                if !self.incrementers[0].borrow_mut().try_increment() {
                    panic!("Unexpectedly failed to increment x incrementers after succeeding before.");
                }
                self.current_size = Some(self.current_size.unwrap() + 1);
                self.current_x = Some(0);
                self.current_y = Some(self.current_y.unwrap() + 1);

                for incrementer in self.incrementers.iter() {
                    let borrowed_incrementer = incrementer.borrow();
                    let indexed_elements = borrowed_incrementer.get();
                    self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
                }

                return true;
            }

            // reached the bounds of what is a square area
            self.is_completed = true;
            return false;
        }

        // if x is to the far right, we need to increment the y downward
        if self.current_x.unwrap() == self.current_size.unwrap() - 1 {
            // if y is one step away from the far bottom, then we need to start y at the bottom and move once to the right
            if self.current_y.unwrap() == self.current_size.unwrap() - 2 {
                self.current_indexed_elements_per_incrementer_index.clear();
                if self.incrementers[1].borrow_mut().try_increment() {
                    self.incrementers[0].borrow_mut().reset();
                    if !self.incrementers[0].borrow_mut().try_increment() {
                        panic!("Unexpectedly failed to increment x incrementer after succeeding before.");
                    }
                    for incrementer in self.incrementers.iter() {
                        let borrowed_incrementer = incrementer.borrow();
                        let indexed_elements = borrowed_incrementer.get();
                        self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
                    }
                    // the size is the same, we're just making our way along the bottom of the square now
                    self.current_x = Some(0);
                    self.current_y = Some(self.current_y.unwrap() + 1);
                    return true;
                }
                
                // we have reached the end of what y is capable of, but maybe not x
                // we should increment x and reset y, moving along a wide rectangle
                if self.incrementers[0].borrow_mut().try_increment() {
                    self.incrementers[1].borrow_mut().reset();
                    if !self.incrementers[1].borrow_mut().try_increment() {
                        panic!("Unexpectedly failed to increment y incrementer after succeeding before.");
                    }
                    for incrementer in self.incrementers.iter() {
                        let borrowed_incrementer = incrementer.borrow();
                        let indexed_elements = borrowed_incrementer.get();
                        self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
                    }
                    self.current_size = Some(self.current_size.unwrap() + 1);
                    self.current_x = Some(self.current_x.unwrap() + 1);
                    self.current_y = Some(0);
                    return true;
                }

                // the rectangle of x width and (x - 1) height has been completed
                self.is_completed = true;
                return false;
            }
            else {
                self.current_indexed_elements_per_incrementer_index.pop();
                if self.incrementers[1].borrow_mut().try_increment() {
                    let indexed_elements = self.incrementers[1].borrow().get();
                    self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
                    self.current_y = Some(self.current_y.unwrap() + 1);

                    return true;
                }
                else {
                    // we are currently in a rectangle with a larger width than height
                    // we have arrived beyond the bottom, so increase the size, increase x, and reset y
                    self.current_indexed_elements_per_incrementer_index.pop();
                    if self.incrementers[0].borrow_mut().try_increment() {
                        self.incrementers[1].borrow_mut().reset();
                        if !self.incrementers[1].borrow_mut().try_increment() {
                            panic!("Unexpectedly failed to increment y incrementer after already having succeeded before.");
                        }
                        self.current_size = Some(self.current_size.unwrap() + 1);
                        self.current_x = Some(self.current_x.unwrap() + 1);
                        self.current_y = Some(0);

                        for incrementer in self.incrementers.iter() {
                            let borrowed_incrementer = incrementer.borrow();
                            let indexed_elements = borrowed_incrementer.get();
                            self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
                        }

                        return true;
                    }
                    else {
                        // reached the end of the rightward rectangle
                        self.current_indexed_elements_per_incrementer_index.clear();
                        self.is_completed = true;
                        return false;
                    }
                }
            }
        }

        // the process is iterating over the bottom towards the right corner
        if self.incrementers[0].borrow_mut().try_increment() {
            let indexed_elements = self.incrementers[0].borrow().get();
            self.current_indexed_elements_per_incrementer_index[0] = indexed_elements;
            self.current_x = Some(self.current_x.unwrap() + 1);

            return true;
        }

        // this is a rectangle that is taller than it is wide
        // move down and reset the x to the left
        self.current_indexed_elements_per_incrementer_index.clear();
        if self.incrementers[1].borrow_mut().try_increment() {
            self.incrementers[0].borrow_mut().reset();
            if !self.incrementers[0].borrow_mut().try_increment() {
                panic!("Unexpectedly failed to increment x incrementer after succeeding before.");
            }
            self.current_size = Some(self.current_size.unwrap() + 1);
            self.current_x = Some(0);
            self.current_y = Some(self.current_y.unwrap() + 1);

            for incrementer in self.incrementers.iter() {
                let indexed_elements = incrementer.borrow().get();
                self.current_indexed_elements_per_incrementer_index.push(indexed_elements);
            }

            return true;
        }

        // the tall rectangle has been fully discovered

        self.is_completed = true;
        return false;
    }
    fn reset(&mut self) {
        for incrementer in self.incrementers.iter() {
            incrementer.borrow_mut().reset();
        }
        self.current_size = None;
        self.current_x = None;
        self.current_y = None;
        self.current_indexed_elements_per_incrementer_index.clear();
        self.is_completed = false;
    }
    fn get(&self) -> Vec<IndexedElement<Self::T>> {
        return self.current_indexed_elements_per_incrementer_index[0]
            .iter()
            .cloned()
            .chain(
                self.current_indexed_elements_per_incrementer_index[1]
                .iter()
                .cloned()
            )
            .collect();

        let mut combined_indexed_elements: Vec<IndexedElement<T>> = Vec::new();
        for indexed_element in self.current_indexed_elements_per_incrementer_index[0].iter() {
            let mapped_indexed_element = IndexedElement::new(indexed_element.element.clone(), indexed_element.index);
            combined_indexed_elements.push(mapped_indexed_element);
        }
        let indexed_element_index_offset = self.current_indexed_elements_per_incrementer_index[0].len();
        for indexed_element in self.current_indexed_elements_per_incrementer_index[1].iter() {
            let mapped_indexed_element = IndexedElement::new(indexed_element.element.clone(), indexed_element.index + indexed_element_index_offset);
            combined_indexed_elements.push(mapped_indexed_element);
        }
        return combined_indexed_elements;
    }
    fn randomize(&mut self) {
        self.incrementers[0].borrow_mut().randomize();
        self.incrementers[1].borrow_mut().randomize();
    }
}

#[cfg(test)]
mod paired_square_breadth_first_search_incrementer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeSet};

    use crate::shifter::segment_permutation_shifter::{SegmentPermutationShifter, Segment};
    use crate::incrementer::shifter_incrementer::{ShifterIncrementer};

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

        let mut paired_incrementer = PairedSquareBreadthFirstSearchIncrementer::new(
            (
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
                    0
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
                    2
                )))
            )
        );

        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 200), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 202), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 200), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 202), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 200), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 203), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 200), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 203), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 200), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 202), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 200), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 203), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 201), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 203), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 201), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 203), indexed_elements[3].element.as_ref());
        }
        assert!(paired_incrementer.try_increment());
        {
            let indexed_elements = paired_incrementer.get();
            assert_eq!(4, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(20, 201), indexed_elements[2].element.as_ref());
            assert_eq!(3, indexed_elements[3].index);
            assert_eq!(&(20, 203), indexed_elements[3].element.as_ref());
        }
        assert!(!paired_incrementer.try_increment());
    }
}