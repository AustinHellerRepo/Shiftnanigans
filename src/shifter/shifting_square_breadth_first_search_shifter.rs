use std::{cell::RefCell, rc::Rc};
use crate::IndexedElement;
use super::{Shifter, scaling_square_breadth_first_search_shifter::ScalingSquareBreadthFirstSearchShifter};

/// Purpose:
///     To increment each shifter forward, resetting as need be, to ensure that the earliest states of each shifter are attempted before the last states
///     Example:
///         0  0  0
///         1  0  0
///         0  1  0
///         0  0  1
///         1  1  0
///         1  0  1
///         0  1  1
///         1  1  1
///         2  0  0
///         0  2  0
///         0  0  2
///         2  1  0
///         1  2  0
///         1  0  2
///         2  0  1
///         0  2  1
///         0  1  2
///         2  1  1
///         1  2  1
///         1  1  2
///         2  2  0
///         2  0  2
///         0  2  2
///         2  2  1
///         2  1  2
///         1  2  2
///         2  2  2
/// 
///     Square algorithm
///         0  0  0
///         1  0  0
///         0  1  0
///         0  0  1
///         1  1  0
///         1  0  1
///         0  1  1
///         1  1  1
///         2  0  0
///         2  1  0
///         2  0  1
///         2  1  1
///         0  2  0
///         1  2  0
///         0  2  1
///         1  2  1
///         0  0  2
///         1  0  2
///         0  1  2
///         1  1  2
///         2  2  0
///         2  2  1
///         2  0  2
///         2  1  2
///         0  2  2
///         1  2  2
///         2  2  2
/// 
pub struct ShiftingSquareBreadthFirstSearchShifter<T> {
    shifters: Vec<Box<dyn Shifter<T = T>>>,
    element_index_offset_per_shifter_index: Vec<usize>,
    current_global_shift_index: Option<usize>,
    current_shifter_index: Option<usize>,
    current_shift_index_per_shifter_index: Vec<usize>,
    current_state_index_per_shift_index_per_shifter_index: Vec<Vec<Option<usize>>>,
    scaling_square_breadth_first_search_shifter: ScalingSquareBreadthFirstSearchShifter,
    length: usize,
    possible_states: Vec<Rc<T>>,
    state_index_mapping_per_shifter_index: Vec<Vec<usize>>,
    is_shifter_order_preserved_on_randomize: bool
}

impl<T: PartialEq> ShiftingSquareBreadthFirstSearchShifter<T> {
    pub fn new(shifters: Vec<Box<dyn Shifter<T = T>>>, is_shifter_order_preserved_on_randomize: bool) -> Self {
        let mut element_index_offset_per_shifter_index: Vec<usize> = Vec::new();
        let mut length = 0;
        let mut highest_shifter_state_length: usize = 0;
        let mut possible_states: Vec<Rc<T>> = Vec::new();
        let mut state_index_mapping_per_shifter_index: Vec<Vec<usize>> = Vec::new();
        for shifter in shifters.iter() {
            let borrowed_shifter = shifter;
            let shifter_length = borrowed_shifter.get_length();
            element_index_offset_per_shifter_index.push(length);
            length += shifter_length;
            // find the highest shifter state length in order to provide a maximum size for the scaling square breadth first search shifter
            let shifter_possible_states_length = borrowed_shifter.get_states().len();
            if shifter_possible_states_length > highest_shifter_state_length {
                highest_shifter_state_length = shifter_possible_states_length;
            }
            // determine the possible states mapping
            let mut state_index_mapping: Vec<usize> = Vec::new();
            let current_shifter_possible_states = borrowed_shifter.get_states();
            for current_shifter_possible_state in current_shifter_possible_states {
                'found_possible_state_index: {
                    for (possible_state_index, possible_state) in possible_states.iter().enumerate() {
                        if possible_state.as_ref() == current_shifter_possible_state.as_ref() {
                            state_index_mapping.push(possible_state_index);
                            break 'found_possible_state_index;
                        }
                    }
                    state_index_mapping.push(possible_states.len());
                    possible_states.push(current_shifter_possible_state);
                }
            }
            state_index_mapping_per_shifter_index.push(state_index_mapping);
        }
        if highest_shifter_state_length == 0 {
            // ensure that the ScalingSquareBreadthFirstSearchShifter can be initialized, but it will never be used
            // TODO do not create the ScalingSquareBreadthFirstSearchShifter
            highest_shifter_state_length = 1;
        }
        ShiftingSquareBreadthFirstSearchShifter {
            shifters: shifters,
            element_index_offset_per_shifter_index: element_index_offset_per_shifter_index,
            current_global_shift_index: None,
            current_shifter_index: None,
            current_shift_index_per_shifter_index: Vec::new(),
            current_state_index_per_shift_index_per_shifter_index: Vec::new(),
            scaling_square_breadth_first_search_shifter: ScalingSquareBreadthFirstSearchShifter::new(length, highest_shifter_state_length - 1),
            length: length,
            possible_states: possible_states,
            state_index_mapping_per_shifter_index: state_index_mapping_per_shifter_index,
            is_shifter_order_preserved_on_randomize: is_shifter_order_preserved_on_randomize
        }
    }
}

impl<T> Shifter for ShiftingSquareBreadthFirstSearchShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.current_global_shift_index.is_none() {
            let mut current_shifter_index = 0;
            if self.scaling_square_breadth_first_search_shifter.try_forward() {
                while !self.shifters[current_shifter_index].try_forward() {
                    current_shifter_index += 1;
                    if current_shifter_index == self.shifters.len() {
                        self.current_shifter_index = Some(current_shifter_index);
                        return false;
                    }
                    self.current_state_index_per_shift_index_per_shifter_index.push(Vec::new());
                    self.current_shift_index_per_shifter_index.push(0);
                }
                self.current_state_index_per_shift_index_per_shifter_index.push(vec![None]);
                self.current_shift_index_per_shifter_index.push(0);
                self.current_global_shift_index = Some(0);
                self.current_shifter_index = Some(current_shifter_index);
                return true;
            }
            self.current_shifter_index = Some(current_shifter_index);
            return false;
        }
        if self.current_global_shift_index.unwrap() == self.length {
            return false;
        }
        let mut current_shifter_index = self.current_shifter_index.unwrap();
        let mut is_shifter_incremented = false;
        while !self.shifters[current_shifter_index].try_forward() {
            is_shifter_incremented = true;
            current_shifter_index += 1;
            if current_shifter_index == self.shifters.len() {
                self.current_shifter_index = Some(current_shifter_index);
                return false;
            }
            self.current_state_index_per_shift_index_per_shifter_index.push(Vec::new());
            self.current_shift_index_per_shifter_index.push(0);
        }
        if is_shifter_incremented {
            self.current_shifter_index = Some(current_shifter_index);
        }
        else {
            self.current_shift_index_per_shifter_index[current_shifter_index] += 1;
        }
        if !self.scaling_square_breadth_first_search_shifter.try_forward() {
            panic!("Unexpectedly failed to move scaling square breadth first search shifter forward.");
        }
        self.current_global_shift_index = Some(self.current_global_shift_index.unwrap() + 1);
        self.current_state_index_per_shift_index_per_shifter_index[current_shifter_index].push(None);
        return true;
    }
    fn try_backward(&mut self) -> bool {
        if self.current_shifter_index.is_none() {
            return false;
        }
        let mut current_shifter_index = self.current_shifter_index.unwrap();
        if current_shifter_index == self.shifters.len() {
            if current_shifter_index == 0 {
                self.current_shifter_index = None;
                return false;
            }
            current_shifter_index -= 1;
            while !self.shifters[current_shifter_index].try_backward() {
                self.current_shift_index_per_shifter_index.pop();
                self.current_state_index_per_shift_index_per_shifter_index.pop();
                if current_shifter_index == 0 {
                    self.current_shifter_index = None;
                    return false;
                }
                current_shifter_index -= 1;
            }
            self.current_shifter_index = Some(current_shifter_index);
            return true;
        }
        self.current_state_index_per_shift_index_per_shifter_index[current_shifter_index].pop();
        let mut is_shifter_decremented = false;
        while !self.shifters[current_shifter_index].try_backward() {
            self.current_state_index_per_shift_index_per_shifter_index.pop();
            self.current_shift_index_per_shifter_index.pop();
            if current_shifter_index == 0 {
                if self.scaling_square_breadth_first_search_shifter.try_backward() {
                    panic!("Unexpectedly successful backward of internal scaling square breadth first search shifter when at beginning.");
                }
                self.current_global_shift_index = None;
                self.current_shifter_index = None;
                return false;
            }
            is_shifter_decremented = true;
            current_shifter_index -= 1;
        }
        if is_shifter_decremented {
            self.current_shifter_index = Some(current_shifter_index);
        }
        else {
            self.current_shift_index_per_shifter_index[current_shifter_index] -= 1;
        }
        if !self.scaling_square_breadth_first_search_shifter.try_backward() {
            panic!("Unexpectedly failed to move scaling square breadth first search shifter backward.");
        }
        self.current_global_shift_index = Some(self.current_global_shift_index.unwrap() - 1);
        return true;
    }
    fn try_increment(&mut self) -> bool {
        // if we are currently at a shifter
        //      if able to increment the internal scaling square breadth first search shifter
        //          get the current scaling index
        //          if the current shift index is none
        //              set the current shift index to zero
        //          else
        //              set the current shift index via incrementing by one
        //          if able to increment the current shifter's shift to match the current scaling index
        //              return true
        //          return false
        //      return false
        // return false

        if let Some(current_shifter_index) = self.current_shifter_index {
            let current_shifter = &mut self.shifters[current_shifter_index];
            let current_state_index_per_shift_index = &mut self.current_state_index_per_shift_index_per_shifter_index[current_shifter_index];
            let current_shift_index = self.current_shift_index_per_shifter_index[current_shifter_index];
            while self.scaling_square_breadth_first_search_shifter.try_increment() {
                let current_scaling_index = self.scaling_square_breadth_first_search_shifter.get_scaling_index();
                let mut current_state_option = current_state_index_per_shift_index[current_shift_index];
                if let Some(current_state_index) = current_state_option {
                    if current_state_index > current_scaling_index {
                        // the current state index is above the scaling index, so reset and move back to the 0th state
                        if self.current_global_shift_index.unwrap() != 0 {
                            panic!("Unexpected state index above scaling index when not at the beginning.");
                        }
                        current_shifter.reset();
                        if !current_shifter.try_forward() {
                            panic!("Unexpectedly failed to move first shifter with shifts forward when reseting due to current shift index exceeding scaling index.");
                        }
                        /*self.shifters[current_shifter_index].reset();
                        if !self.shifters[current_shifter_index].try_forward() {
                            panic!("Unexpectedly failed to move first shifter with shifts forward when reseting due to current shift index exceeding scaling index.");
                        }*/
                        current_state_option = None;
                    }
                }
                // was getting 85.6
                'match_scaling_index: {
                    let mut current_state_index;
                    if let Some(temp_current_state_index) = current_state_option {
                        current_state_index = temp_current_state_index;
                    }
                    else {
                        if !current_shifter.try_increment() {
                            break 'match_scaling_index;
                        }
                        current_state_index = 0;
                    }
                    while current_state_index < current_scaling_index {
                        if !current_shifter.try_increment() {
                            // failed to increment the current shifter to match the scaling index
                            current_state_index_per_shift_index[current_shift_index] = Some(current_state_index);
                            break 'match_scaling_index;
                        }
                        current_state_index += 1;
                    }
                    current_state_index_per_shift_index[current_shift_index] = Some(current_state_index);
                    return true;
                }
            }
            return false;
        }
        return false;
    }
    fn get_indexed_element(&self) -> IndexedElement<Self::T> {
        if let Some(current_shifter_index) = self.current_shifter_index {
            let mut indexed_element = self.shifters[current_shifter_index].get_indexed_element();
            indexed_element.index += self.element_index_offset_per_shifter_index[current_shifter_index];
            return indexed_element;
        }
        panic!("Unexpected attempt to get the indexed element when not in a valid state.");
    }
    fn get_length(&self) -> usize {
        return self.length;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        let current_shifter_index = self.current_shifter_index.unwrap();
        let (mut element_index, mut state_index) = self.shifters[current_shifter_index].get_element_index_and_state_index();
        element_index += self.element_index_offset_per_shifter_index[current_shifter_index];
        state_index = self.state_index_mapping_per_shifter_index[current_shifter_index][state_index];
        return (element_index, state_index);
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_states.clone();
    }
    fn randomize(&mut self) {
        // TODO determine if this misorders indexes - should a mapper be used and randomized instead?
        for shifter in self.shifters.iter_mut() {
            shifter.randomize();
        }
        if !self.is_shifter_order_preserved_on_randomize {
            fastrand::shuffle(&mut self.shifters);
        }
    }
}

#[cfg(test)]
mod shifting_square_breadth_first_search_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use crate::{shifter::{segment_permutation_shifter::{SegmentPermutationShifter, Segment}, index_shifter::IndexShifter}, incrementer::{shifter_incrementer::ShifterIncrementer, Incrementer}};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn no_shifters() {
        init();

        let mut shifter: ShiftingSquareBreadthFirstSearchShifter<(u8, u8)> = ShiftingSquareBreadthFirstSearchShifter::new(vec![
        ], true);
        for _ in 0..10 {
            assert!(!shifter.try_forward());
        }
    }

    #[rstest]
    fn two_shifters_separate_segment_permutation_shifters() {
        init();

        let mut shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![
            Box::new(SegmentPermutationShifter::new(vec![
                Rc::new(Segment::new(1)),
                Rc::new(Segment::new(1))
            ], (10, 100), 4, true, 1, false)),
            Box::new(SegmentPermutationShifter::new(vec![
                Rc::new(Segment::new(1)),
                Rc::new(Segment::new(1))
            ], (20, 200), 4, false, 1, false))
        ], true);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());  // back to the 3th index
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());  // back to the 2th index
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());  // back to the 1th index
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());  // back to the 0th index
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 1 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 0 1 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 0 0 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 1 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 0 1 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 0 0 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 1 1 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 1 0 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(3, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 0 1 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 1 1 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 1 0 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 0 1 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 0 1 1 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // scaling is 1 1 1 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // the maximum scale is 4, but the internal shifters max out at 1, so we have to burn through the instances when the 0th shift will be the 0th state and the 1th shift will be the next maximum scaling index
            assert!(shifter.try_increment());  // 0 2 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 2 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 2 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 2 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 2 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 0 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 1 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 0 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 1 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 0 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 1 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // burning through 2 2 0 0, 2 0 2 0, and 2 0 0 2
            assert!(shifter.try_increment());  // 0 2 2 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 2 2 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 2 0 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 2 0 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 2 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 2 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 2 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // burning through 2 2 2 0, 2 2 0 2, and 2 0 2 2
            assert!(shifter.try_increment());  // 0 2 2 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 2 2 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // the maximum scale is 4, but the internal shifters max out at 1, so we have to burn through the instances when the 0th shift will be the 0th state and the 1th shift will be the next maximum scaling index
            assert!(shifter.try_increment());  // 0 3 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 3 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 3 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 3 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 3 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 0 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 1 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 0 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 1 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 0 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 1 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // burning through 3 3 0 0, 3 0 3 0, and 3 0 0 3
            assert!(shifter.try_increment());  // 0 3 3 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 3 3 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 3 0 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 3 0 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 0 3 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1 3 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0 3 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // burning through 3 3 3 0, 3 3 0 3, and 3 0 3 3
            assert!(shifter.try_increment());  // 0 3 3 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 3 3 3
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_shifters_overlapping_segment_permutation_shifters() {
        init();

        let mut shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![
            Box::new(SegmentPermutationShifter::new(vec![
                Rc::new(Segment::new(1))
            ], (10, 100), 3, true, 1, false)),
            Box::new(SegmentPermutationShifter::new(vec![
                Rc::new(Segment::new(1))
            ], (11, 99), 3, false, 1, false))
        ], true);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());  // 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 99), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 99), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 2 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 99), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 2 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 0 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 101), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 1 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(11, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 101), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());  // 2 2
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(11, 101), indexed_element.element.as_ref());
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
    fn two_shifters_skewed_by_one_index_shifters() {
        let mut shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![
            Box::new(IndexShifter::new(&vec![
                vec![
                    Rc::new((0 as u8, 0 as u8))
                ]
            ])),
            Box::new(IndexShifter::new(&vec![
                vec![
                    Rc::new((2 as u8, 1 as u8)),
                    Rc::new((1 as u8, 1 as u8))
                ]
            ]))
        ], true);

        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());  // 0 0
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(0, 0), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(2, 1), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            // burning past 1 0
            assert!(shifter.try_increment());  // 0 1
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(0, 0), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(1, 1), indexed_element.element.as_ref());
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
    fn one_specific_randomized_segment_permutation_shifter_iterating_completely() {
        // the randomized segment permutation shifter requires a loop event
        fastrand::seed(11);
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(
            vec![
                Rc::new(Segment::new(1)),
                Rc::new(Segment::new(1))
            ],
            (10, 100),
            5,
            true,
            1,
            false
        );
        segment_permutation_shifter.randomize();
        // verify that the SegmentPermutationShifter is in the expected random state
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        {
            let indexed_element = segment_permutation_shifter.get_indexed_element();
            assert_eq!(0, indexed_element.index);
            assert_eq!(&(10, 100), indexed_element.element.as_ref());
        }
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        {
            let indexed_element = segment_permutation_shifter.get_indexed_element();
            assert_eq!(1, indexed_element.index);
            assert_eq!(&(13, 100), indexed_element.element.as_ref());
        }
        segment_permutation_shifter.reset();
        // iterate over the ShiftingSquareBreadthFirstSearchShifter
        let shifter = ShiftingSquareBreadthFirstSearchShifter::new(
            vec![
                Box::new(segment_permutation_shifter)
            ],
            true
        );
        let mut incrementer = ShifterIncrementer::new(Box::new(shifter), vec![0, 1]);
        for _ in 0..10 {
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&(14, 100), indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&(14, 100), indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&(12, 100), indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&(14, 100), indexed_elements[1].element.as_ref());
            }
            assert!(incrementer.try_increment());
            {
                let indexed_elements = incrementer.get();
                assert_eq!(2, indexed_elements.len());
                assert_eq!(0, indexed_elements[0].index);
                assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
                assert_eq!(1, indexed_elements[1].index);
                assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
            }
            assert!(!incrementer.try_increment());
            incrementer.reset();
        }
    }

    #[rstest]
    fn one_randomly_chosen_randomized_segment_permutation_shifter_iterating_completely() {
        // the randomized segment permutation shifter requires a loop event
        for _ in 0..20 {
            let mut segment_permutation_shifter = SegmentPermutationShifter::new(
                vec![
                    Rc::new(Segment::new(1)),
                    Rc::new(Segment::new(1))
                ],
                (10, 100),
                5,
                true,
                1,
                false
            );
            segment_permutation_shifter.randomize();
            // iterate over the ShiftingSquareBreadthFirstSearchShifter
            let shifter = ShiftingSquareBreadthFirstSearchShifter::new(
                vec![
                    Box::new(segment_permutation_shifter)
                ],
                true
            );
            let mut incrementer = ShifterIncrementer::new(Box::new(shifter), vec![0, 1]);
            for _ in 0..10 {
                assert!(incrementer.try_increment());
                assert!(incrementer.try_increment());
                assert!(incrementer.try_increment());
                assert!(incrementer.try_increment());
                assert!(incrementer.try_increment());
                assert!(incrementer.try_increment());
                assert!(!incrementer.try_increment());
                incrementer.reset();
            }
        }
    }

}