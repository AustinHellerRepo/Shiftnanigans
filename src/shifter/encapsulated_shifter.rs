use std::{rc::Rc, cell::RefCell};
use super::Shifter;

pub struct EncapsulatedShifter<T> {
    shifters: Vec<Rc<RefCell<dyn Shifter<T = T>>>>,
    current_shifter_index: Option<usize>
}

impl<T> EncapsulatedShifter<T> {
    pub fn new(shifters: &Vec<Rc<RefCell<dyn Shifter<T = T>>>>) -> Self {
        EncapsulatedShifter {
            shifters: shifters.clone(),
            current_shifter_index: None
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
            let is_current_shifter_try_backward_successful = self.shifters[current_shifter_index].borrow_mut().try_backward();
            if is_current_shifter_try_backward_successful {
                return true;
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
    fn get(&self) -> Rc<Self::T> {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let current_shifter_get = self.shifters[current_shifter_index].borrow_mut().get();
        return current_shifter_get;
    }
}

#[cfg(test)]
mod encapsulated_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use crate::shifter::index_shifter::IndexShifter;

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
        assert_eq!(&(1, 1), encapsulated_shifter.get().as_ref());
        assert!(encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(10, 10), encapsulated_shifter.get().as_ref());
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(encapsulated_shifter.try_increment());
        assert_eq!(&(11, 11), encapsulated_shifter.get().as_ref());
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // no more states for the 2nd shift
        assert!(encapsulated_shifter.try_backward());  // move back to the 1st shift
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd state for the 1st shift
        assert_eq!(&(2, 2), encapsulated_shifter.get().as_ref());
        assert!(encapsulated_shifter.try_forward());  // move to the 2nd shifter
        assert!(encapsulated_shifter.try_increment());  // pull the 1st state for the 2nd shift
        assert_eq!(&(10, 10), encapsulated_shifter.get().as_ref());
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(encapsulated_shifter.try_increment());  // pull the 2nd state for the 2nd shift
        assert_eq!(&(11, 11), encapsulated_shifter.get().as_ref());
        assert!(!encapsulated_shifter.try_forward());
        assert!(encapsulated_shifter.try_backward());  // move back to the 2nd shift
        assert!(!encapsulated_shifter.try_increment());  // no more states for the 2nd shift
        assert!(encapsulated_shifter.try_backward());  // move back to the 1st shift
        assert!(!encapsulated_shifter.try_increment());  // no more states in 1st shift
        assert!(!encapsulated_shifter.try_backward());  // done
    }
}