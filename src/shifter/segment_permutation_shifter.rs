use std::{rc::Rc, collections::VecDeque};
use bitvec::vec::BitVec;
use crate::IndexedElement;

use super::{Shifter};

/// This struct is an unfixed line segment.
#[derive(Clone, Debug)]
pub struct Segment {
    length: usize
}

impl Segment {
    pub fn new(length: usize) -> Self {
        Segment {
            length: length
        }
    }
}

/// This struct is a fixed line segment.
#[derive(Clone, Debug)]
pub struct LocatedSegment {
    pub segment_index: usize,
    pub position: usize
}

impl LocatedSegment {
    pub fn new(segment_index: usize, position: usize) -> Self {
        LocatedSegment {
            segment_index: segment_index,
            position: position
        }
    }
}

pub struct SegmentPermutationShifter {
    segments: Vec<Rc<Segment>>,
    origin: (u8, u8),
    is_horizontal: bool,
    padding: usize,
    possible_locations: Vec<Rc<(u8, u8)>>,
    current_mask: BitVec,
    current_segment_index_per_shift_index: VecDeque<usize>,
    current_bounding_length_per_shift_index: VecDeque<usize>,
    current_maximum_bounding_length_per_shift_index: VecDeque<usize>,
    current_minimum_bounding_length_per_shift_index: VecDeque<usize>,
    current_initial_position_offset_per_shift_index: VecDeque<usize>,
    current_position_offset_per_shift_index: VecDeque<Option<usize>>,
    current_remaining_maximum_bounding_length: usize,
    current_remaining_minimum_bounding_length: usize,
    is_shifted_outside: bool,
    segments_length: usize
}

impl SegmentPermutationShifter {
    pub fn new(segments: Vec<Rc<Segment>>, origin: (u8, u8), bounding_length: usize, is_horizontal: bool, padding: usize) -> Self {
        let segments_length = segments.len();

        let mut current_mask: BitVec = BitVec::with_capacity(segments_length);
        current_mask.resize(segments.len(), false);

        let mut current_remaining_minimum_bounding_length: usize = 0;
        for (segment_index, segment) in segments.iter().enumerate() {
            if segment_index != 0 {
                current_remaining_minimum_bounding_length += padding;
            }
            current_remaining_minimum_bounding_length += segment.length;
        }

        let mut possible_locations: Vec<Rc<(u8, u8)>> = Vec::new();
        let mut current_possible_location = origin.clone();
        if is_horizontal {
            for _ in 0..bounding_length {
                possible_locations.push(Rc::new(current_possible_location));
                current_possible_location.0 += 1;
            }
        }
        else {
            for _ in 0..bounding_length {
                possible_locations.push(Rc::new(current_possible_location));
                current_possible_location.1 += 1;
            }
        }

        SegmentPermutationShifter {
            segments: segments,
            origin: origin,
            is_horizontal: is_horizontal,
            padding: padding,
            possible_locations: possible_locations,
            current_mask: current_mask,
            current_segment_index_per_shift_index: VecDeque::new(),
            current_bounding_length_per_shift_index: VecDeque::new(),
            current_maximum_bounding_length_per_shift_index: VecDeque::new(),
            current_minimum_bounding_length_per_shift_index: VecDeque::new(),
            current_initial_position_offset_per_shift_index: VecDeque::new(),
            current_position_offset_per_shift_index: VecDeque::new(),
            current_remaining_maximum_bounding_length: bounding_length,
            current_remaining_minimum_bounding_length: current_remaining_minimum_bounding_length,
            is_shifted_outside: false,
            segments_length: segments_length
        }
    }
}

impl Shifter for SegmentPermutationShifter {
    type T = (u8, u8);

    fn try_forward(&mut self) -> bool {
        // calculate bounding length for the next "other segments"
        // calculate position of chosen segment in "other segments"

        if self.is_shifted_outside {
            debug!("try_forward: already shifted outside");
            return false;
        }
        else {
            if self.current_mask.count_zeros() == 0 {
                self.is_shifted_outside = true;
                debug!("try_forward: discovered that there are no zeros, so shifting outside");
                return false;
            }
            else {
                for mask_index in 0..self.segments.len() {
                    if !self.current_mask[mask_index] {
                        // choose this segment index
                        self.current_mask.set(mask_index, true);
                        // specify that this shift index maps to this segment index
                        self.current_segment_index_per_shift_index.push_back(mask_index);
                        // store the starting and ending bounding length for this shift index
                        self.current_bounding_length_per_shift_index.push_back(self.current_remaining_maximum_bounding_length);
                        self.current_maximum_bounding_length_per_shift_index.push_back(self.current_remaining_maximum_bounding_length);
                        self.current_minimum_bounding_length_per_shift_index.push_back(self.current_remaining_minimum_bounding_length);
                        debug!("try_forward: storing {:?} and {:?} as max and min bounding length", self.current_remaining_maximum_bounding_length, self.current_remaining_minimum_bounding_length);
                        if self.current_mask.count_ones() == 1 {
                            self.current_initial_position_offset_per_shift_index.push_back(0);
                        }
                        else {
                            let previous_shift_index = self.current_initial_position_offset_per_shift_index.len() - 1;
                            let previous_segment_index = self.current_segment_index_per_shift_index[previous_shift_index];
                            let previous_segment_length = self.segments[previous_segment_index].length;
                            let previous_position_offset = self.current_position_offset_per_shift_index[previous_shift_index].unwrap();
                            let current_initial_position = previous_position_offset + previous_segment_length + self.padding;
                            self.current_initial_position_offset_per_shift_index.push_back(current_initial_position);
                        }
                        self.current_position_offset_per_shift_index.push_back(None);

                        let segment_length = self.segments[mask_index].length;
                        self.current_remaining_maximum_bounding_length -= segment_length;
                        self.current_remaining_minimum_bounding_length -= segment_length;
                        if self.current_mask.count_zeros() != 0 {
                            self.current_remaining_maximum_bounding_length -= self.padding;
                            self.current_remaining_minimum_bounding_length -= self.padding;
                        }
                        debug!("try_forward: found the next segment at index {:?}", mask_index);
                        return true;
                    }
                }
                panic!("Unexpected missing mask opening.");
            }
        }
    }
    fn try_backward(&mut self) -> bool {
        if self.current_mask.count_ones() != 0 {
            if self.is_shifted_outside {
                self.is_shifted_outside = false;
                debug!("try_backward: shifted outside");
            }
            else {
                let current_segment_index = self.current_segment_index_per_shift_index.pop_back().unwrap();
                self.current_mask.set(current_segment_index, false);
                self.current_bounding_length_per_shift_index.pop_back();
                self.current_remaining_maximum_bounding_length = self.current_maximum_bounding_length_per_shift_index.pop_back().unwrap();
                self.current_remaining_minimum_bounding_length = self.current_minimum_bounding_length_per_shift_index.pop_back().unwrap();
                self.current_initial_position_offset_per_shift_index.pop_back();
                self.current_position_offset_per_shift_index.pop_back();
                debug!("try_backward: moved to previous state with mask {:?}", self.current_mask);
            }
        }
        if self.current_mask.count_ones() == 0 {
            debug!("try_backward: nothing is selected, so cannot increment");
            return false;
        }
        debug!("try_backward: at least one thing is selected");
        return true;
    }
    fn try_increment(&mut self) -> bool {
        let current_shift_index = self.current_segment_index_per_shift_index.len() - 1;
        if self.current_position_offset_per_shift_index[current_shift_index].is_none() {
            debug!("try_increment: incrementing to initial position");
            self.current_position_offset_per_shift_index[current_shift_index] = Some(self.current_initial_position_offset_per_shift_index[current_shift_index]);
            return true;
        }
        else {
            let current_bounding_length = self.current_bounding_length_per_shift_index[current_shift_index];
            if current_bounding_length == self.current_minimum_bounding_length_per_shift_index[current_shift_index] {
                debug!("try_increment: position is at last location already, so trying to increment segment");
                let current_segment_index = self.current_segment_index_per_shift_index[current_shift_index];
                for next_segment_index in (current_segment_index + 1)..self.segments.len() {
                    if !self.current_mask[next_segment_index] {
                        // found the next mask index
                        self.current_mask.set(current_segment_index, false);
                        self.current_mask.set(next_segment_index, true);
                        self.current_segment_index_per_shift_index[current_shift_index] = next_segment_index;
                        let next_segment_length = self.segments[next_segment_index].length;
                        debug!("try_increment: current remaining max/min bounding length of {:?}/{:?}", self.current_remaining_maximum_bounding_length, self.current_remaining_minimum_bounding_length);
                        self.current_bounding_length_per_shift_index[current_shift_index] = self.current_maximum_bounding_length_per_shift_index[current_shift_index];
                        self.current_remaining_maximum_bounding_length = self.current_maximum_bounding_length_per_shift_index[current_shift_index] - next_segment_length;
                        self.current_remaining_minimum_bounding_length = self.current_minimum_bounding_length_per_shift_index[current_shift_index] - next_segment_length;
                        if self.current_mask.count_zeros() != 0 {
                            self.current_remaining_maximum_bounding_length -= self.padding;
                            self.current_remaining_minimum_bounding_length -= self.padding;
                        }
                        debug!("try_increment: storing {:?} max bounding length with mask {:?}", self.current_bounding_length_per_shift_index[current_shift_index], self.current_mask);
                        self.current_position_offset_per_shift_index[current_shift_index] = Some(self.current_initial_position_offset_per_shift_index[current_shift_index]);
                        debug!("try_increment: found next segment {:?}", next_segment_index);
                        return true;
                    }
                }
                debug!("try_increment: failed to find next segment in mask");
                return false;
            }
            else {
                self.current_bounding_length_per_shift_index[current_shift_index] = current_bounding_length - 1;
                self.current_position_offset_per_shift_index[current_shift_index] = Some(self.current_position_offset_per_shift_index[current_shift_index].unwrap() + 1);
                self.current_remaining_maximum_bounding_length -= 1;
                debug!("try_increment: position moved to the right");
                return true;
            }
        }
    }
    fn get_indexed_element(&self) -> IndexedElement<(u8, u8)> {
        let (current_segment_index, current_position_offset) = self.get_element_index_and_state_index();
        let position: Rc<(u8, u8)>;
        if self.is_horizontal {
            position = Rc::new((self.origin.0 + current_position_offset as u8, self.origin.1));
        }
        else {
            position = Rc::new((self.origin.0, self.origin.1 + current_position_offset as u8));
        }
        return IndexedElement::new(position, current_segment_index);
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        let current_position_offset = self.current_position_offset_per_shift_index.back().unwrap().unwrap();
        let current_segment_index = *self.current_segment_index_per_shift_index.back().unwrap();
        return (current_segment_index, current_position_offset);
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_locations.clone();
    }
    fn get_length(&self) -> usize {
        return self.segments_length;
    }
    fn randomize(&mut self) {
        fastrand::shuffle(&mut self.segments);
    }
}

#[cfg(test)]
mod segment_permutation_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialized_no_segments() {
        init();
    
        let segments: Vec<Rc<Segment>> = Vec::new();
        let _ = SegmentPermutationShifter::new(segments, (10, 100), 5, true, 1);
    }
    
    #[rstest]
    #[case(vec![Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    fn shift_forward_and_backward_for_multiple_segments(#[case] segments: Vec<Rc<Segment>>, #[case] origin: (u8, u8), #[case] bounding_length: usize, #[case] is_horizontal: bool, #[case] padding: usize) {
        init();
        
        let segments_length = segments.len();
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, origin, bounding_length, is_horizontal, padding);
        for index in 0..10 {
            debug!("index: {:?}", index);
            assert!(!segment_permutation_shifter.try_backward());
            assert!(segment_permutation_shifter.try_forward());
            assert!(!segment_permutation_shifter.try_backward());
            for _ in 0..segments_length {
                assert!(segment_permutation_shifter.try_forward());
                assert!(segment_permutation_shifter.try_increment());
            }
            assert!(!segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_backward());
            assert!(!segment_permutation_shifter.try_forward());
            for _ in 0..segments_length {
                assert!(segment_permutation_shifter.try_backward());
            }
        }
    }

    #[rstest]
    #[case(vec![Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1))], (10, 100), 3, false, 1)]
    #[case(vec![Rc::new(Segment::new(2))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(2))], (10, 100), 3, false, 1)]
    #[case(vec![Rc::new(Segment::new(3))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(3))], (10, 100), 3, false, 1)]
    fn permutate_through_different_segments_one_segment(#[case] segments: Vec<Rc<Segment>>, #[case] origin: (u8, u8), #[case] bounding_length: usize, #[case] is_horizontal: bool, #[case] padding: usize) {
        init();
        
        let segment_length = segments[0].length;
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, origin, bounding_length, is_horizontal, padding);
        for index in 0..10 {
            debug!("index: {:?}", index);
            assert!(!segment_permutation_shifter.try_backward());
            assert!(segment_permutation_shifter.try_forward());
            assert!(!segment_permutation_shifter.try_backward());
            assert!(segment_permutation_shifter.try_forward());
            assert!(!segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_backward());
            assert!(!segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_backward());
        }
        for index in 0..=(bounding_length - segment_length) {
            assert!(segment_permutation_shifter.try_increment());
            let indexed_element = segment_permutation_shifter.get_indexed_element();
            if is_horizontal {
                assert_eq!(origin.0 + index as u8, indexed_element.element.0);
                assert_eq!(origin.1, indexed_element.element.1);
            }
            else {
                assert_eq!(origin.0, indexed_element.element.0);
                assert_eq!(origin.1 + index as u8, indexed_element.element.1);
            }
        }
        assert!(!segment_permutation_shifter.try_increment());
    }

    #[rstest]
    fn permutations_of_one_and_two_and_three_length_segments_with_one_padding_with_smallest_bounding_length() {
        init();

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(1)),
            Rc::new(Segment::new(2)),
            Rc::new(Segment::new(3))
        ];
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(
            segments,
            (10, 100),
            8,
            true,
            1
        );
        for index in 0..10 {
            let is_try_forward_at_end_required: bool = index % 2 == 0;
            assert!(segment_permutation_shifter.try_forward());  // move to 1st shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment at the 1st shift
            assert_eq!(&(10, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move to the 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment at the 2nd shift
            assert_eq!(&(12, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move to the 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment at the 3rd shift
            assert_eq!(&(15, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move past the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to the last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when all segments have been selected in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to the 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 2nd shift
            assert_eq!(&(12, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // moved forward to the 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment as the 3rd shift
            assert_eq!(&(16, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());
                assert!(segment_permutation_shifter.try_backward());
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when all segments have been selected in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to the 2nd shift
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when no other segments to find
            assert!(segment_permutation_shifter.try_backward());  // moved back to the 1st shift
            assert!(segment_permutation_shifter.try_increment());  // pulled the 2nd segment as the 1st shift
            assert_eq!(&(10, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move to 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 2nd shift
            assert_eq!(&(13, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move to 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 3rd shift
            assert_eq!(&(15, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // already at the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to the last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since there are no mask bits left
            assert!(segment_permutation_shifter.try_backward());  // move back to the 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 2nd shift
            assert_eq!(&(13, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move the to the 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 3rd shift
            assert_eq!(&(17, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move forward any further
                assert!(segment_permutation_shifter.try_backward());  // move back to the last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since there are no mask bits left
            assert!(segment_permutation_shifter.try_backward());  // move back to the 2nd shift
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since both the 1st and 3rd segment have already been tried
            assert!(segment_permutation_shifter.try_backward());  // move back to the 1st shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 1st shift
            assert_eq!(&(10, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move to 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 2nd shift
            assert_eq!(&(14, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // move to 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment as the 3rd shift
            assert_eq!(&(16, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move forward since already at the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to last shifter
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since nothing left in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to 2nd shifter
            assert!(segment_permutation_shifter.try_increment());  // pulled the 2nd segment as the 2nd shift
            assert_eq!(&(14, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            assert!(segment_permutation_shifter.try_forward());  // moved to 3rd shifter
            assert!(segment_permutation_shifter.try_increment());  // pulled the 1st segment as the 3rd shift
            assert_eq!(&(17, 100), segment_permutation_shifter.get_indexed_element().element.as_ref());
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move forward since already at the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since nothing left in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to 2nd shift
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since already tried 1st and 2nd segment
            assert!(segment_permutation_shifter.try_backward());  // moved back to 1st shift
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since already tried 1st, 2nd, and 3rd segment
            assert!(!segment_permutation_shifter.try_backward());  // cannot move backward since already at the beginning
        }
    }

    #[rstest]
    fn permutations_of_two_and_three_with_one_padding_with_one_open_space_bounding_length() {
        init();

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(2)),
            Rc::new(Segment::new(3))
        ];
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, (20, 200), 7, false, 1);
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 1st shift
        assert_eq!(&(20, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment as the 2nd shift
        assert_eq!(&(20, 203), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(20, 204), segment_permutation_shifter.get_indexed_element().element.as_ref());  // moved the 2nd segment down
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(!segment_permutation_shifter.try_increment());
        assert!(segment_permutation_shifter.try_backward());
        debug!("test: moving first segment in first shift forward");
        assert!(segment_permutation_shifter.try_increment());
        debug!("test: moved first segment");
        assert_eq!(&(20, 201), segment_permutation_shifter.get_indexed_element().element.as_ref());  // moved the 1st segment down
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(20, 204), segment_permutation_shifter.get_indexed_element().element.as_ref());  // found the 2nd segment already lower
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(!segment_permutation_shifter.try_increment());
        assert!(segment_permutation_shifter.try_backward());
        debug!("test: back to first shift, pulling second segment.");
        assert!(segment_permutation_shifter.try_increment());  // pull 2nd segment as 1st shift
        debug!("test: pulled second segment.");
        assert_eq!(&(20, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());  // pull 1st segment as 2nd shift
        assert_eq!(&(20, 204), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(segment_permutation_shifter.try_increment());  // move 1st segment over one space
        assert_eq!(&(20, 205), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(!segment_permutation_shifter.try_increment());  // nowhere else to move and no other segment to try
        assert!(segment_permutation_shifter.try_backward());  // move back to 1st shift
        assert!(segment_permutation_shifter.try_increment());  // move 2nd segment in 1st shift over one space
        assert_eq!(&(20, 201), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(segment_permutation_shifter.try_forward());  // move to the 2nd shift
        assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as 2nd shift
        assert_eq!(&(20, 205), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        {
            let (element_index, state_index) = segment_permutation_shifter.get_element_index_and_state_index();
            assert_eq!(element_index, segment_permutation_shifter.get_indexed_element().index);
            assert_eq!(segment_permutation_shifter.get_states()[state_index], segment_permutation_shifter.get_indexed_element().element);
        }
        assert!(!segment_permutation_shifter.try_increment());  // cannot increment since already moved forward to max and no other segments
        assert!(segment_permutation_shifter.try_backward());
        assert!(!segment_permutation_shifter.try_increment());  // cannot move 2nd segment in 1st shift any further and no other segments
        assert!(!segment_permutation_shifter.try_backward());  // moved back to outside start
    }
}
