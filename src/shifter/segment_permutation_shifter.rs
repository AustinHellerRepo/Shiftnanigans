use std::{rc::Rc, collections::VecDeque};
use bitvec::vec::BitVec;
use crate::{IndexedElement, get_n_choose_k};

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

#[derive(Clone)]
pub struct SegmentPermutationShifter {
    segments: Vec<Rc<Segment>>,
    origin: (u8, u8),
    bounding_length: usize,
    is_horizontal: bool,
    padding: usize,
    is_swapping_permitted: bool,
    possible_locations: Vec<Rc<(u8, u8)>>,
    current_mask: BitVec,
    current_segment_index_per_shift_index: Vec<usize>,
    current_initial_position_offset_per_shift_index: Vec<usize>,
    current_minimum_position_offset_per_shift_index: Vec<usize>,
    current_maximum_position_offset_per_shift_index: Vec<usize>,
    current_position_offset_per_shift_index: Vec<Option<usize>>,
    current_is_parent_ending: BitVec,
    is_shifted_outside: bool,
    segments_length: usize,
    starting_segment_index_per_shift_index: Vec<usize>,
    starting_initial_position_offset_per_shift_index: Vec<usize>,
    starting_minimum_position_offset_per_shift_index: Vec<usize>,
    starting_maximum_position_offset_per_shift_index: Vec<usize>,
    ending_segment_index_per_shift_index: Vec<usize>,
    ending_position_offset_per_shift_index: Vec<usize>,
    is_starting: bool,  // true if "starting" states are still being pulled from
    is_looped: bool,  // true if one cycle has been performed on the mask
    is_starting_equal_to_ending: bool,  // true if the starting and ending positions are the same
    is_starting_at_beginning: bool  // true if the segments starting in sequential order and flush to the left
}

impl SegmentPermutationShifter {
    pub fn new(segments: Vec<Rc<Segment>>, origin: (u8, u8), bounding_length: usize, is_horizontal: bool, padding: usize, is_swapping_permitted: bool) -> Self {
        let segments_length = segments.len();

        let mut current_mask: BitVec = BitVec::with_capacity(segments_length);
        current_mask.resize(segments.len(), false);

        let mut smallest_segment_length_option: Option<usize> = None;
        let mut smallest_bounding_length: usize = 0;
        for (segment_index, segment) in segments.iter().enumerate() {
            if smallest_segment_length_option.is_none() || smallest_segment_length_option.unwrap() > segment.length {
                smallest_segment_length_option = Some(segment.length);
            }
            if segment_index != 0 {
                smallest_bounding_length += padding;
            }
            smallest_bounding_length += segment.length;
        }

        let reduced_bounding_length_offset: usize;
        if let Some(smallest_segment_length) = smallest_segment_length_option {
            if smallest_segment_length != 0 {
                reduced_bounding_length_offset = smallest_segment_length - 1;
            }
            else {
                reduced_bounding_length_offset = 0;
            }
        }
        else {
            reduced_bounding_length_offset = 0;
        }

        let mut possible_locations: Vec<Rc<(u8, u8)>> = Vec::new();
        let mut current_possible_location = origin.clone();
        if is_horizontal {
            for _ in 0..(bounding_length - reduced_bounding_length_offset) {
                possible_locations.push(Rc::new(current_possible_location));
                current_possible_location.0 += 1;
            }
        }
        else {
            for _ in 0..(bounding_length - reduced_bounding_length_offset) {
                possible_locations.push(Rc::new(current_possible_location));
                current_possible_location.1 += 1;
            }
        }

        let mut starting_segment_index_per_shift_index: Vec<usize> = Vec::new();  // the "ending" state is always going to occur when the segment indexes are sequential with the shift indexes, but it may randomly be at the very end
        let mut starting_minimum_position_offset_per_shift_index: Vec<usize> = Vec::new();
        let mut starting_maximum_position_offset_per_shift_index: Vec<usize> = Vec::new();
        let starting_initial_position_offset_per_shift_index: Vec<usize>;
        let mut ending_segment_index_per_shift_index: Vec<usize> = Vec::new();
        let mut ending_position_offset_per_shift_index: Vec<usize> = Vec::new();

        // initialize "ending" objects
        {
            for shift_index in 0..segments_length {
                if is_swapping_permitted {
                    ending_segment_index_per_shift_index.push(segments_length - shift_index - 1);
                }
                else {
                    ending_segment_index_per_shift_index.push(shift_index);
                }
            }
            if is_swapping_permitted {
                ending_position_offset_per_shift_index.push(bounding_length - smallest_bounding_length);
                for shift_index in (1..segments_length).rev() {
                    let previous_segment_length = segments[shift_index].length + padding;
                    let previous_ending_position_offset = ending_position_offset_per_shift_index[segments_length - shift_index - 1];
                    ending_position_offset_per_shift_index.push(previous_ending_position_offset + previous_segment_length);
                }
            }
            else {
                ending_position_offset_per_shift_index.push(bounding_length - smallest_bounding_length);
                for shift_index in 1..segments_length {
                    let previous_segment_length = segments[shift_index - 1].length + padding;
                    let previous_ending_position_offset = ending_position_offset_per_shift_index[shift_index - 1];
                    ending_position_offset_per_shift_index.push(previous_ending_position_offset + previous_segment_length);
                }
            }
        }

        // initialize "starting" objects
        {
            for shift_index in 0..segments_length {
                starting_segment_index_per_shift_index.push(shift_index);
            }
            let next_minimum_position_offset = 0;
            let next_maximum_position_offset = bounding_length - smallest_bounding_length;
            starting_minimum_position_offset_per_shift_index.push(next_minimum_position_offset);
            starting_maximum_position_offset_per_shift_index.push(next_maximum_position_offset);
            for shift_index in 1..segments_length {
                let previous_segment_length = segments[shift_index - 1].length + padding;
                let previous_minimum_position_offset = starting_minimum_position_offset_per_shift_index[shift_index - 1];
                let previous_maximum_position_offset = starting_maximum_position_offset_per_shift_index[shift_index - 1];
                starting_minimum_position_offset_per_shift_index.push(previous_minimum_position_offset + previous_segment_length);
                starting_maximum_position_offset_per_shift_index.push(previous_maximum_position_offset + previous_segment_length);
            }
            starting_initial_position_offset_per_shift_index = starting_minimum_position_offset_per_shift_index.clone();
        }

        let mut is_starting_equal_to_ending: bool = true;  // TODO save this for when incrementing early while is_starting
        for shift_index in 0..segments_length {
            if starting_initial_position_offset_per_shift_index[shift_index] != ending_position_offset_per_shift_index[shift_index] {
                is_starting_equal_to_ending = false;
                break;
            }
            if starting_segment_index_per_shift_index[shift_index] != ending_segment_index_per_shift_index[shift_index] {
                is_starting_equal_to_ending = false;
                break;
            }
        }

        let is_starting_at_beginning = true;

        let mut segment_permutation_shifter = SegmentPermutationShifter {
            segments: segments,
            origin: origin,
            bounding_length: bounding_length,
            is_horizontal: is_horizontal,
            padding: padding,
            is_swapping_permitted: is_swapping_permitted,
            possible_locations: possible_locations,
            current_mask: current_mask,
            current_segment_index_per_shift_index: Vec::new(),
            current_initial_position_offset_per_shift_index: Vec::new(),
            current_minimum_position_offset_per_shift_index: Vec::new(),
            current_maximum_position_offset_per_shift_index: Vec::new(),
            current_position_offset_per_shift_index: Vec::new(),
            current_is_parent_ending: BitVec::new(),
            is_shifted_outside: false,
            segments_length: segments_length,
            starting_segment_index_per_shift_index: starting_segment_index_per_shift_index,
            starting_initial_position_offset_per_shift_index: starting_initial_position_offset_per_shift_index,
            starting_minimum_position_offset_per_shift_index: starting_minimum_position_offset_per_shift_index,
            starting_maximum_position_offset_per_shift_index: starting_maximum_position_offset_per_shift_index,
            ending_segment_index_per_shift_index: ending_segment_index_per_shift_index,
            ending_position_offset_per_shift_index: ending_position_offset_per_shift_index,
            is_starting: true,
            is_looped: false,
            is_starting_equal_to_ending: is_starting_equal_to_ending,
            is_starting_at_beginning: is_starting_at_beginning
        };

        if segment_permutation_shifter.is_initially_looped() {
            segment_permutation_shifter.is_looped = true;
        }

        return segment_permutation_shifter;
    }
    fn is_initially_looped(&self) -> bool {
        return self.is_starting_at_beginning;

        let mut is_looped = false;  // TODO remove is_looped logic completely or replace with loop iteration count
        if self.is_starting_equal_to_ending {
            // there is only one state
            is_looped = true;
        }
        else if self.is_starting_at_beginning {
            is_looped = true;
        }
        else if !self.is_swapping_permitted && self.is_starting_at_beginning {
            // the segments are already in the correct order in this non-randomized state
            is_looped = true;
        }
        else if self.segments_length == 1 && self.is_starting_at_beginning {
            // the single segment is going to march through each position
            // TODO determine if this should be generalized for multiple segments that start at their left-most position offset
            is_looped = true;
        }
        return is_looped;
    }
}

impl Shifter for SegmentPermutationShifter {
    type T = (u8, u8);

    fn try_forward(&mut self) -> bool {
        // if mask is full
        //      set already outside to true
        //      return false
        // if is starting
        //      set mask based on "starting" segment index per shift index
        //      set "current" states based on "starting" states
        //      push current position as None, requiring increment to set to initial position
        // else
        //      search for the next available unmasked index for the segment index of this shift index
        //      calculate minimum and maximum position offsets
        //      clone minimum into initial position offsets

        if self.current_mask.first_zero().is_none() {
            self.is_shifted_outside = true;
            return false;
        }
        let shift_index = self.current_segment_index_per_shift_index.len();
        if self.is_starting {
            let segment_index = self.starting_segment_index_per_shift_index[shift_index];
            self.current_mask.set(segment_index, true);
            self.current_segment_index_per_shift_index.push(segment_index);
            self.current_minimum_position_offset_per_shift_index.push(self.starting_minimum_position_offset_per_shift_index[shift_index]);
            self.current_maximum_position_offset_per_shift_index.push(self.starting_maximum_position_offset_per_shift_index[shift_index]);
            self.current_initial_position_offset_per_shift_index.push(self.starting_initial_position_offset_per_shift_index[shift_index]);
        }
        else {
            for mask_index in 0..self.segments_length {
                if !self.current_mask[mask_index] {
                    self.current_mask.set(mask_index, true);
                    self.current_segment_index_per_shift_index.push(mask_index);
                    let minimum_position_offset;
                    let maximum_position_offset;
                    if shift_index == 0 {
                        minimum_position_offset = 0;
                        maximum_position_offset = self.starting_maximum_position_offset_per_shift_index[0];
                    }
                    else {
                        let previous_segment_index = self.current_segment_index_per_shift_index[shift_index - 1];
                        let previous_segment_length = self.segments[previous_segment_index].length + self.padding;
                        minimum_position_offset = self.current_position_offset_per_shift_index[shift_index - 1].unwrap() + previous_segment_length;
                        maximum_position_offset = self.current_maximum_position_offset_per_shift_index[shift_index - 1] + previous_segment_length;
                    }
                    self.current_minimum_position_offset_per_shift_index.push(minimum_position_offset);
                    self.current_maximum_position_offset_per_shift_index.push(maximum_position_offset);
                    self.current_initial_position_offset_per_shift_index.push(minimum_position_offset);
                    break;
                }
            }
        }
        self.current_position_offset_per_shift_index.push(None);
        self.current_is_parent_ending.push(false);
        return true;
    }
    fn try_backward(&mut self) -> bool {
        // if mask is empty
        //      return false
        // reset mask position for current segment index
        // clear state for current segment index
        // if no mask positions selected
        //      return false
        // return true

        self.is_starting = false;
        if self.is_shifted_outside {
            self.is_shifted_outside = false;
            return self.segments_length != 0;
        }
        if self.current_mask.first_one().is_none() {
            return false;
        }

        let segment_index = self.current_segment_index_per_shift_index.pop().unwrap();
        self.current_mask.set(segment_index, false);
        self.current_minimum_position_offset_per_shift_index.pop();
        self.current_maximum_position_offset_per_shift_index.pop();
        self.current_initial_position_offset_per_shift_index.pop();
        self.current_position_offset_per_shift_index.pop();
        self.current_is_parent_ending.pop();

        if self.current_mask.first_one().is_none() {
            self.is_starting = true;
            self.is_looped = self.is_initially_looped();
            return false;
        }
        return true;
    }
    fn try_increment(&mut self) -> bool {
        // if this is a fresh forward
        //      set the state to the expected initial state for this shift index
        //      if is looped and parent is ending for previous shift index and the current state is the ending state for this shift index
        //          set parent is ending for this shift index
        //      return true
        // if parent is ending for this shift index
        //      return false
        // if the remaining bounding length is the minimum bounding length
        //      if swapping is not permitted
        //          if this is the first shift index
        //              initialize state to the initial state
        //              if current state is the ending state for this shift index
        //                  set parent is ending for this shift index   
        //              set is looped to true
        //              return true
        //          return false
        //      if successful in swapping the segment index for this shift index
        //          if current state is the ending state for this shift index
        //              set parent is ending for this shift index
        //          return true
        //      return false
        // reduce the remaining bounding length
        // increment the position offset
        // if is looped and parent is ending for previous shift index and the current state is the ending state for this shift index
        //      set parent is ending for this shift index
        // return true

        let shift_index = self.current_position_offset_per_shift_index.len() - 1;
        if self.current_position_offset_per_shift_index[shift_index].is_none() {
            let current_position_offset = self.current_initial_position_offset_per_shift_index[shift_index];
            self.current_position_offset_per_shift_index[shift_index] = Some(current_position_offset);
            if self.is_looped {
                if shift_index == 0 || self.current_is_parent_ending[shift_index - 1] {
                    if self.ending_position_offset_per_shift_index[shift_index] == current_position_offset && self.ending_segment_index_per_shift_index[shift_index] == self.current_segment_index_per_shift_index[shift_index] {
                        self.current_is_parent_ending.set(shift_index, true);
                    }
                }
            }
            return true;
        }
        self.is_starting = false;
        if self.current_is_parent_ending[shift_index] {
            return false;
        }
        if self.current_position_offset_per_shift_index[shift_index].unwrap() == self.current_maximum_position_offset_per_shift_index[shift_index] {
            if !self.is_swapping_permitted {
                if shift_index == 0 {
                    // TODO check to see if we need to loop back to the first state because this shifter was randomized

                    // loop back to the first state
                    self.current_initial_position_offset_per_shift_index[0] = 0;
                    self.current_position_offset_per_shift_index[0] = Some(0);

                    if self.ending_position_offset_per_shift_index[0] == 0 {
                        self.current_is_parent_ending.set(0, true);
                    }
                    else {
                        self.current_is_parent_ending.set(0, false);
                    }

                    self.is_looped = true;
                    debug!("try_increment: looping back to start");
                    return true;
                }
                return false;
            }
            let segment_index = self.current_segment_index_per_shift_index[shift_index];
            for next_segment_index in (segment_index + 1)..self.segments_length {
                if !self.current_mask[next_segment_index] {
                    self.current_mask.set(segment_index, false);
                    self.current_mask.set(next_segment_index, true);
                    self.current_segment_index_per_shift_index[shift_index] = next_segment_index;
                    self.current_position_offset_per_shift_index[shift_index] = Some(self.current_minimum_position_offset_per_shift_index[shift_index]);
                    if self.is_looped {
                        if shift_index == 0 || self.current_is_parent_ending[shift_index - 1] {
                            if self.ending_position_offset_per_shift_index[shift_index] == self.current_position_offset_per_shift_index[shift_index].unwrap() && self.ending_segment_index_per_shift_index[shift_index] == self.current_segment_index_per_shift_index[shift_index] {
                                self.current_is_parent_ending.set(shift_index, true);
                            }
                        }
                    }
                    return true;
                }
            }
            // at this point the swapped segments are all in reverse order and need to loop once back at 0th shift
            if shift_index == 0 {
                // loop back to the first state
                self.current_mask.set(segment_index, false);
                self.current_mask.set(0, true);
                self.current_segment_index_per_shift_index[0] = 0;
                self.current_initial_position_offset_per_shift_index[0] = 0;
                self.current_position_offset_per_shift_index[0] = Some(0);

                if self.ending_position_offset_per_shift_index[0] == 0 {
                    self.current_is_parent_ending.set(0, true);
                }
                else {
                    self.current_is_parent_ending.set(0, false);
                }

                self.is_looped = true;
                debug!("try_increment: looping back to start");
                return true;
            }
            return false;
        }
        self.current_position_offset_per_shift_index[shift_index] = Some(self.current_position_offset_per_shift_index[shift_index].unwrap() + 1);
        if self.is_looped {
            if shift_index == 0 || self.current_is_parent_ending[shift_index - 1] {
                if self.ending_position_offset_per_shift_index[shift_index] == self.current_position_offset_per_shift_index[shift_index].unwrap() && self.ending_segment_index_per_shift_index[shift_index] == self.current_segment_index_per_shift_index[shift_index] {
                    self.current_is_parent_ending.set(shift_index, true);
                }
            }
        }
        return true;
    }
    /*fn try_decrement(&mut self) -> bool {
        let shift_index = self.current_position_offset_per_shift_index.len() - 1;
        if self.current_position_offset_per_shift_index[shift_index].is_none() {
            // have not incremented yet
            return false;
        }

        todo!();
    }*/
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
        let current_position_offset = self.current_position_offset_per_shift_index.last().unwrap().unwrap();
        let current_segment_index = *self.current_segment_index_per_shift_index.last().unwrap();
        return (current_segment_index, current_position_offset);
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_locations.clone();
    }
    fn get_length(&self) -> usize {
        return self.segments_length;
    }
    fn randomize(&mut self) {
        if self.is_starting_equal_to_ending {
            // there is no reason to perform any randomization since there is only one state
            return;
        }
        if self.is_swapping_permitted {
            fastrand::shuffle(&mut self.ending_segment_index_per_shift_index);
        }
        // TODO start the "current" properties in a randomized state
        //
        //  1   1   1   1   1   1   1   1
        //      1   2   3   4   5   6   7
        //          1   3   6   10  15  21
        //              1   4   10  20  35
        //                  1   5   15  35
        //                      1   6   21
        //                          1   7
        //                              1         
        // f(x) = n! / ((n - k)! * k!)
        //  n   k   f(x)
        //  1   1   1
        //  2   1   2
        //  3   1   3
        //  .   .   .
        //  2   2   1
        //  3   2   3
        //  4   2   6   
        //
        //  S   B   P   n   k   f(x)
        //  1   1   1   
        //  1   2   2   
        //  1   3   3
        //  .   .   .
        //  2   2   1
        //  2   3   3
        //  2   4   6
        //  2   5   10
        //  .   .   .
        //  3   3   1
        //  3   4   4
        //  3   5   10
        //  3   6   20           

        // algorith for starting at random state
        //      consider each segment as having length of one (consider as "original" segment), ignore the padding, and each remaining bounding length as a segment of length one (consider as "empty" segment)
        //      sort segments randomly within vector
        //      initialize position to zero
        //      increment through vector
        //          if current segment is an "original" segment
        //              if not first "original" segment found
        //                  increment position by padding
        //              record segment position
        //              increment position by segment length
        //          else
        //              increment position by one

        let mut is_original_segment_list: BitVec = BitVec::repeat(true, self.segments_length);
        let mut remaining_bounding_length = self.bounding_length;
        for segment_index in 0..self.segments_length {
            if segment_index != 0 {
                remaining_bounding_length -= self.padding;
            }
            let mapped_segment_index = self.ending_segment_index_per_shift_index[segment_index];
            remaining_bounding_length -= self.segments[mapped_segment_index].length;
        }
        is_original_segment_list.resize(self.segments_length + remaining_bounding_length, false);
        debug!("randomize: before shuffle: {:?}", is_original_segment_list);
        {
            // implementation from shuffle in https://github.com/smol-rs/fastrand/blob/master/src/lib.rs
            for bit_index in 1..is_original_segment_list.len() {
                let other_bit_index = fastrand::usize(..=bit_index);
                if bit_index != other_bit_index {
                    is_original_segment_list.swap(bit_index, other_bit_index);
                }
            }
        }
        debug!("randomize: after shuffle: {:?}", is_original_segment_list);
        let mut current_position_index = 0;
        let mut current_segment_index = 0;
        self.ending_position_offset_per_shift_index.clear();
        for segment_list_index in 0..(self.segments_length + remaining_bounding_length) {
            if is_original_segment_list[segment_list_index] {
                if current_segment_index != 0 {
                    current_position_index += self.padding;
                }
                self.ending_position_offset_per_shift_index.push(current_position_index);
                let mapped_current_segment_index = self.ending_segment_index_per_shift_index[current_segment_index];
                current_position_index += self.segments[mapped_current_segment_index].length;
                current_segment_index += 1;
            }
            else {
                current_position_index += 1;
            }
        }

        // at this point all of the ending positions are known

        // TODO fill out the "starting" states
        self.starting_minimum_position_offset_per_shift_index.clear();
        self.starting_maximum_position_offset_per_shift_index.clear();
        self.starting_initial_position_offset_per_shift_index = self.ending_position_offset_per_shift_index.clone();
        self.starting_segment_index_per_shift_index = self.ending_segment_index_per_shift_index.clone();

        let mut current_minimum_position_offset = 0;
        let mut current_maximum_position_offset: Option<usize> = None;
        for segment_index in 0..self.segments_length {
            self.starting_minimum_position_offset_per_shift_index.push(current_minimum_position_offset);
            if current_maximum_position_offset.is_none() {
                let mut minimum_bounding_length = 0;
                for other_segment_index in (segment_index + 1)..self.segments_length {
                    if minimum_bounding_length != 0 {
                        minimum_bounding_length += self.padding;
                    }
                    minimum_bounding_length += self.segments[self.starting_segment_index_per_shift_index[other_segment_index]].length;
                }
                if minimum_bounding_length != 0 {
                    minimum_bounding_length += self.padding;
                }
                minimum_bounding_length += self.segments[self.starting_segment_index_per_shift_index[segment_index]].length;
                let maximum_position_offset = (self.bounding_length - current_minimum_position_offset) - minimum_bounding_length;
                debug!("randomize: maximum_position_offset: {:?}", maximum_position_offset);
                current_maximum_position_offset = Some(maximum_position_offset);
            }
            self.starting_maximum_position_offset_per_shift_index.push(current_maximum_position_offset.unwrap());

            let segment_length = self.segments[self.starting_segment_index_per_shift_index[segment_index]].length + self.padding;
            current_minimum_position_offset = self.starting_initial_position_offset_per_shift_index[segment_index] + segment_length;
            current_maximum_position_offset = Some(current_maximum_position_offset.unwrap() + segment_length);
        }

        // try to move the starting state forward one iteration or swap masks or fully reset
        
        let mut shift_index = self.segments_length - 1;
        let mut is_still_trying_earlier_shift_indexes = true;
        let mut mask: BitVec = BitVec::repeat(true, self.segments_length);
        let mut recalculate_position_offsets_from_shift_index: Option<usize> = None;
        'looking_for_position: {
            while is_still_trying_earlier_shift_indexes {
                if self.starting_initial_position_offset_per_shift_index[shift_index] != self.starting_maximum_position_offset_per_shift_index[shift_index] {
                    // this shift index can move forward
                    self.starting_initial_position_offset_per_shift_index[shift_index] += 1;
                    if shift_index + 1 < self.segments_length {
                        recalculate_position_offsets_from_shift_index = Some(shift_index + 1);
                    }
                    break 'looking_for_position;
                }
                if self.is_swapping_permitted {
                    mask.set(self.starting_segment_index_per_shift_index[shift_index], false);
                    for next_mask_index in (self.starting_segment_index_per_shift_index[shift_index] + 1)..self.segments_length {
                        if !mask[next_mask_index] {
                            // found a valid next mask for this shift index
                            mask.set(next_mask_index, true);
                            self.starting_segment_index_per_shift_index[shift_index] = next_mask_index;
                            recalculate_position_offsets_from_shift_index = Some(shift_index);
                            shift_index += 1;
                            for other_mask_index in 0..self.segments_length {
                                if !mask[other_mask_index] {
                                    mask.set(other_mask_index, true);
                                    self.starting_segment_index_per_shift_index[shift_index] = other_mask_index;
                                    shift_index += 1;
                                }
                            }
                            break 'looking_for_position;
                        }
                    }
                }
                if shift_index == 0 {
                    // none of the positions can move where they are right now
                    for mask_index in 0..self.segments_length {
                        self.starting_segment_index_per_shift_index[mask_index] = mask_index;
                    }
                    recalculate_position_offsets_from_shift_index = Some(0);
                    break 'looking_for_position;
                }
                shift_index -= 1;
            }
        }
        if recalculate_position_offsets_from_shift_index.is_some() {
            // recalculate minimum and maximum position offsets and set the initial position offset to the minimum
            let shift_index = recalculate_position_offsets_from_shift_index.unwrap();
            let mut next_minimum_position_offset;
            let mut next_maximum_position_offset;
            let mut previous_segment_length;

            if shift_index == 0 {
                next_minimum_position_offset = 0;

                // determine what the next maximum position offset is
                next_maximum_position_offset = 0;
                for shift_index in 0..self.segments_length {
                    if shift_index != 0 {
                        next_maximum_position_offset += self.padding;
                    }
                    let segment_index = self.starting_segment_index_per_shift_index[shift_index];
                    next_maximum_position_offset += self.segments[segment_index].length;
                }
                next_maximum_position_offset = self.bounding_length - next_maximum_position_offset;
            }
            else {
                let previous_shift_index = shift_index - 1;
                {
                    let segment_index = self.starting_segment_index_per_shift_index[previous_shift_index];
                    previous_segment_length = self.segments[segment_index].length + self.padding;
                }

                next_minimum_position_offset = self.starting_initial_position_offset_per_shift_index[previous_shift_index] + previous_segment_length;
                next_maximum_position_offset = self.starting_maximum_position_offset_per_shift_index[previous_shift_index] + previous_segment_length;
            }

            let segment_index = self.starting_segment_index_per_shift_index[shift_index];
            previous_segment_length = self.segments[segment_index].length + self.padding;
            
            for next_shift_index in shift_index..self.segments_length {
                debug!("randomize: next_minimum_position_offset {:?} for next_shift_index {:?}", next_minimum_position_offset, next_shift_index);
                debug!("randomize: next_maximum_position_offset {:?} for next_shift_index {:?}", next_maximum_position_offset, next_shift_index);
                self.starting_minimum_position_offset_per_shift_index[next_shift_index] = next_minimum_position_offset;
                self.starting_maximum_position_offset_per_shift_index[next_shift_index] = next_maximum_position_offset;
                self.starting_initial_position_offset_per_shift_index[next_shift_index] = next_minimum_position_offset;
                next_minimum_position_offset += previous_segment_length;
                next_maximum_position_offset += previous_segment_length;
                let segment_index = self.starting_segment_index_per_shift_index[next_shift_index];
                previous_segment_length = self.segments[segment_index].length + self.padding;
            }
        }
        // determine if we happened to end up with the starting positions and segment order at the very beginning
        'check_starting_at_beginning: {
            for segment_index in 0..self.segments_length {
                if self.ending_segment_index_per_shift_index[segment_index] != segment_index {
                    self.is_starting_at_beginning = false;
                    break 'check_starting_at_beginning;
                }
            }
            let mut previous_position_offset = 0;
            for shift_index in 0..self.segments_length {
                if self.starting_initial_position_offset_per_shift_index[shift_index] != previous_position_offset {
                    self.is_starting_at_beginning = false;
                    break 'check_starting_at_beginning;
                }
                previous_position_offset += self.segments[shift_index].length + self.padding;
            }
            self.is_starting_at_beginning = true;
        }
        // set is_looped based on loop criteria
        self.is_looped = self.is_initially_looped();
    }
}

#[cfg(test)]
mod segment_permutation_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeMap};

    use crate::incrementer::{shifter_incrementer::ShifterIncrementer, Incrementer};

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
        let _ = SegmentPermutationShifter::new(segments, (10, 100), 5, true, 1, true);
    }
    
    #[rstest]
    #[case(vec![Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    #[case(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (10, 100), 3, true, 1)]
    fn shift_forward_and_backward_for_multiple_segments(#[case] segments: Vec<Rc<Segment>>, #[case] origin: (u8, u8), #[case] bounding_length: usize, #[case] is_horizontal: bool, #[case] padding: usize) {
        init();
        
        let segments_length = segments.len();
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, origin, bounding_length, is_horizontal, padding, true);
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
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, origin, bounding_length, is_horizontal, padding, true);
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
            println!("indexed element: {:?}", indexed_element);
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
            1,
            true
        );
        for index in 0..10 {
            let is_try_forward_at_end_required: bool = index % 2 == 0;
            assert!(segment_permutation_shifter.try_forward());  // move to 1st shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment at the 1st shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move to the 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment at the 2nd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move to the 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment at the 3rd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(15, 100), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when all segments have been selected in mask
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move past the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to the last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when all segments have been selected in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to the 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 2nd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(12, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // moved forward to the 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment as the 3rd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(16, 100), indexed_element.element.as_ref());
            }
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());
                assert!(segment_permutation_shifter.try_backward());
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when all segments have been selected in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to the 2nd shift
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment when no other segments to find
            assert!(segment_permutation_shifter.try_backward());  // moved back to the 1st shift
            assert!(segment_permutation_shifter.try_increment());  // pulled the 2nd segment as the 1st shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move to 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 2nd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move to 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 3rd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(15, 100), indexed_element.element.as_ref());
            }
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // already at the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to the last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since there are no mask bits left
            assert!(segment_permutation_shifter.try_backward());  // move back to the 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 2nd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(13, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move the to the 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 3rd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(17, 100), indexed_element.element.as_ref());
            }
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move forward any further
                assert!(segment_permutation_shifter.try_backward());  // move back to the last shift
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since there are no mask bits left
            assert!(segment_permutation_shifter.try_backward());  // move back to the 2nd shift
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since both the 1st and 3rd segment have already been tried
            assert!(segment_permutation_shifter.try_backward());  // move back to the 1st shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 3rd segment as the 1st shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(10, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move to 2nd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 1st segment as the 2nd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(14, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // move to 3rd shift
            assert!(segment_permutation_shifter.try_increment());  // pull the 2nd segment as the 3rd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(16, 100), indexed_element.element.as_ref());
            }
            if is_try_forward_at_end_required {
                assert!(!segment_permutation_shifter.try_forward());  // cannot move forward since already at the end
                assert!(segment_permutation_shifter.try_backward());  // moved back to last shifter
            }
            assert!(!segment_permutation_shifter.try_increment());  // cannot increment since nothing left in mask
            assert!(segment_permutation_shifter.try_backward());  // moved back to 2nd shifter
            assert!(segment_permutation_shifter.try_increment());  // pulled the 2nd segment as the 2nd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(14, 100), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());  // moved to 3rd shifter
            assert!(segment_permutation_shifter.try_increment());  // pulled the 1st segment as the 3rd shift
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(17, 100), indexed_element.element.as_ref());
            }
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
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, (20, 200), 7, false, 1, true);
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

    #[rstest]
    fn permutations_two_segments_two_and_three_length_and_eight_bounding_length_no_swapping_permitted() {
        init();

        // 22-333--
        // 22--333-
        // 22---333
        // -22-333-
        // -22--333
        // --22-333

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(2)),
            Rc::new(Segment::new(3))
        ];
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments, (20, 200), 8, true, 1, false);

        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(20, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(23, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        assert!(!segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_backward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(24, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        assert!(!segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_backward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(25, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        assert!(!segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_backward());
        assert!(!segment_permutation_shifter.try_increment());
        assert!(segment_permutation_shifter.try_backward());  // back to 0th shift
        assert!(segment_permutation_shifter.try_increment());  // increment to 1th position
        assert_eq!(&(21, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(24, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        assert!(!segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_backward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(25, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        assert!(!segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_backward());
        assert!(!segment_permutation_shifter.try_increment());
        assert!(segment_permutation_shifter.try_backward());  // back to the 0th shift
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(22, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(0, segment_permutation_shifter.get_indexed_element().index);
        assert!(segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_increment());
        assert_eq!(&(25, 200), segment_permutation_shifter.get_indexed_element().element.as_ref());
        assert_eq!(1, segment_permutation_shifter.get_indexed_element().index);
        assert!(!segment_permutation_shifter.try_forward());
        assert!(segment_permutation_shifter.try_backward());
        assert!(!segment_permutation_shifter.try_increment());
        assert!(segment_permutation_shifter.try_backward());  // back to the 0th shift
        assert!(!segment_permutation_shifter.try_increment());
        assert!(!segment_permutation_shifter.try_backward());  // done
    }

    #[rstest]
    fn permutations_randomly_two_segments_one_and_one_length_and_four_bounding_length_one_padding_no_swapping_permitted() {
        init();

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(1)),
            Rc::new(Segment::new(1))
        ];

        let mut permutations_per_scenario: [u32; 3] = [0, 0, 0];
        for _ in 0..20000 {

            let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments.clone(), (20, 200), 4, true, 1, false);
            segment_permutation_shifter.randomize();

            // verify that the next state after the ending position is the starting position
            assert!(!segment_permutation_shifter.is_starting_equal_to_ending);
            assert_eq!(0, segment_permutation_shifter.ending_segment_index_per_shift_index[0]);
            assert_eq!(1, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
            assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
            assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
            if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 0 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 2 {
                permutations_per_scenario[0] += 1;

                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(2, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 0 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 3 {
                permutations_per_scenario[1] += 1;

                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(3, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 1 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 3 {
                permutations_per_scenario[2] += 1;

                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(2, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(2, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
        }

        println!("permutations_per_scenario: {:?}", permutations_per_scenario);
        println!("0 / 1 = {}", (1.0 - permutations_per_scenario[0] as f32 / permutations_per_scenario[1] as f32).abs());
        println!("0 / 2 = {}", (1.0 - permutations_per_scenario[0] as f32 / permutations_per_scenario[2] as f32).abs());
        println!("1 / 2 = {}", (1.0 - permutations_per_scenario[1] as f32 / permutations_per_scenario[2] as f32).abs());
        assert!((1.0 - (permutations_per_scenario[0] as f32 / permutations_per_scenario[1] as f32)).abs() < 0.1);
        assert!((1.0 - (permutations_per_scenario[0] as f32 / permutations_per_scenario[2] as f32)).abs() < 0.1);
        assert!((1.0 - (permutations_per_scenario[1] as f32 / permutations_per_scenario[2] as f32)).abs() < 0.1);
    }

    #[rstest]
    fn permutations_randomly_two_segments_one_and_one_length_and_four_bounding_length_one_padding_swapping_permitted() {
        init();

        let segments: Vec<Rc<Segment>> = vec![
            Rc::new(Segment::new(1)),
            Rc::new(Segment::new(1))
        ];

        let mut permutations_per_scenario: [u32; 6] = [0, 0, 0, 0, 0, 0];
        for _ in 0..20000 {

            let mut segment_permutation_shifter = SegmentPermutationShifter::new(segments.clone(), (20, 200), 4, true, 1, true);
            segment_permutation_shifter.randomize();

            // verify that the next state after the ending position is the starting position
            assert!(!segment_permutation_shifter.is_starting_equal_to_ending);
            if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 0 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 2 && segment_permutation_shifter.ending_segment_index_per_shift_index[0] == 0 {
                permutations_per_scenario[0] += 1;

                assert_eq!(1, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(2, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 0 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 3 && segment_permutation_shifter.ending_segment_index_per_shift_index[0] == 0 {
                permutations_per_scenario[1] += 1;

                assert_eq!(1, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(3, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 1 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 3 && segment_permutation_shifter.ending_segment_index_per_shift_index[0] == 0 {
                permutations_per_scenario[2] += 1;

                assert_eq!(1, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
                assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(2, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(2, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 0 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 2 && segment_permutation_shifter.ending_segment_index_per_shift_index[0] == 1 {
                permutations_per_scenario[3] += 1;

                assert_eq!(0, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
                assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(2, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 0 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 3 && segment_permutation_shifter.ending_segment_index_per_shift_index[0] == 1 {
                permutations_per_scenario[4] += 1;

                assert_eq!(0, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
                assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(3, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
            else if segment_permutation_shifter.ending_position_offset_per_shift_index[0] == 1 && segment_permutation_shifter.ending_position_offset_per_shift_index[1] == 3 && segment_permutation_shifter.ending_segment_index_per_shift_index[0] == 1 {
                permutations_per_scenario[5] += 1;

                assert_eq!(0, segment_permutation_shifter.ending_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_segment_index_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_segment_index_per_shift_index[1]);
                assert_eq!(0, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[0]);
                assert_eq!(1, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[0]);
                assert_eq!(0, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[0]);
                assert_eq!(2, segment_permutation_shifter.starting_minimum_position_offset_per_shift_index[1]);
                assert_eq!(3, segment_permutation_shifter.starting_maximum_position_offset_per_shift_index[1]);
                assert_eq!(2, segment_permutation_shifter.starting_initial_position_offset_per_shift_index[1]);
            }
        }

        println!("permutations_per_scenario: {:?}", permutations_per_scenario);
        for scenario_index in 0..5 {
            for other_scenario_index in (scenario_index + 1)..6 {
                println!("{} / {} = {}", scenario_index, other_scenario_index, (1.0 - permutations_per_scenario[scenario_index] as f32 / permutations_per_scenario[other_scenario_index] as f32).abs());
                assert!((1.0 - (permutations_per_scenario[scenario_index] as f32 / permutations_per_scenario[other_scenario_index] as f32)).abs() < 0.1);
            }
        }
    }

    #[rstest]
    fn randomized_single_pixel_segment() {
        for bounding_length in 2..=6 as usize {
            let mut count_per_location: BTreeMap<(u8, u8), usize> = BTreeMap::new();
            for offset in 0..bounding_length as u8 {
                count_per_location.insert((20, 200 + offset), 0);
            }
            let iterations_total = 1000;
            for _ in 0..iterations_total {
                let mut segment_permutation_shifter = SegmentPermutationShifter::new(vec![Rc::new(Segment::new(1))], (20, 200), bounding_length, false, 1, true);
                segment_permutation_shifter.randomize();
                assert!(segment_permutation_shifter.try_forward());
                assert!(segment_permutation_shifter.try_increment());
                let previous_location;
                {
                    let indexed_element = segment_permutation_shifter.get_indexed_element();
                    assert_eq!(0, indexed_element.index);
                    let location: (u8, u8) = *indexed_element.element.as_ref();
                    count_per_location.insert(location, count_per_location[&location] + 1);
                    previous_location = location;
                }
                assert!(!segment_permutation_shifter.try_forward());
                assert!(segment_permutation_shifter.try_backward());
                for _ in 0..(bounding_length - 1) {
                    assert!(segment_permutation_shifter.try_increment());
                    {
                        let indexed_element = segment_permutation_shifter.get_indexed_element();
                        assert_eq!(0, indexed_element.index);
                        let location: (u8, u8) = *indexed_element.element.as_ref();
                        assert_ne!(previous_location, location);
                    }
                    assert!(!segment_permutation_shifter.try_forward());
                    assert!(segment_permutation_shifter.try_backward());
                }
                assert!(!segment_permutation_shifter.try_increment());
                assert!(!segment_permutation_shifter.try_backward());
            }

            println!("count_per_location: {:?}", count_per_location);
            for offset in 0..bounding_length as u8 {
                let location = (20, 200 + offset);
                let count = count_per_location[&location] as f32;
                let expected = iterations_total as f32 / bounding_length as f32;
                assert!((count - expected).abs() < (iterations_total as f32 / 10.0));
            }
        }
    }

    #[rstest]
    fn randomize_two_single_pixels_and_reset_before_end() {
        init();

        let bounding_length = 5;
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let mut segment_permutation_shifter = SegmentPermutationShifter::new(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (20, 200), bounding_length, false, 1, false);
            segment_permutation_shifter.randomize();
            let mut shifter_incrementer = ShifterIncrementer::new(Rc::new(RefCell::new(segment_permutation_shifter)), vec![0, 1]);
            assert!(shifter_incrementer.try_increment());
            {
                let indexed_elements = shifter_incrementer.get();
                println!("starting with {:?}", indexed_elements);
                let random_increments_total = fastrand::u8(0..6);
                println!("incrementing {random_increments_total} times");
                for _ in 0..random_increments_total {
                    println!("incrementing once");
                    assert!(shifter_incrementer.try_increment());
                    let current_indexed_elements = shifter_incrementer.get();
                    println!("currently with {:?}", current_indexed_elements);
                }
                shifter_incrementer.reset();
                assert!(shifter_incrementer.try_increment());
                let identical_indexed_elements = shifter_incrementer.get();
                println!("reset back to {:?}", identical_indexed_elements);
                assert_eq!(indexed_elements.len(), identical_indexed_elements.len());
                for (indexed_element, identical_indexed_element) in indexed_elements.iter().zip(identical_indexed_elements.iter()) {
                    assert_eq!(indexed_element, identical_indexed_element);
                }
            }
        }
    }

    #[rstest]
    fn nonrandom_two_single_pixels_but_only_first_shift_is_exhausted() {
        // forward, increment until burned through, !backward
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (20, 200), 5, false, 1, false);
        for _ in 0..10 {
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());
            assert!(!segment_permutation_shifter.try_backward());
        }
    }

    #[rstest]
    fn nonrandom_three_single_pixels_but_only_second_shift_is_exhausted() {
        // forward, increment, forward, increment until burned through, backward, increment until burned through, !backward
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (20, 200), 7, false, 1, false);
        for _ in 0..10 {
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 204), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());
            assert!(segment_permutation_shifter.try_backward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 204), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());
            assert!(segment_permutation_shifter.try_backward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 204), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());
            assert!(segment_permutation_shifter.try_backward());
            assert!(!segment_permutation_shifter.try_increment());
            assert!(!segment_permutation_shifter.try_backward());
        }
    }

    #[rstest]
    fn random_two_single_pixels_but_only_first_shift_is_exhausted_already_one_position_ahead() {
        // forward, increment until burned through, !backward
        // set seed such that the second shift starts one step ahead
        fastrand::seed(0);
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (20, 200), 5, false, 1, false);
        segment_permutation_shifter.randomize();
        // verify starting position
        {
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            segment_permutation_shifter.reset();
        }
        for _ in 0..10 {
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());
            assert!(!segment_permutation_shifter.try_backward());
        }
    }

    #[rstest]
    fn random_two_single_pixels_but_only_first_shift_is_exhausted_second_segment_already_one_position_ahead() {
        // forward, increment until burned through, !backward
        // set seed such that the second shift starts one step ahead
        fastrand::seed(2);
        let mut segment_permutation_shifter = SegmentPermutationShifter::new(vec![Rc::new(Segment::new(1)), Rc::new(Segment::new(1))], (20, 200), 5, false, 1, false);
        segment_permutation_shifter.randomize();
        // verify starting position
        {
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20, 203), indexed_element.element.as_ref());
            }
            segment_permutation_shifter.reset();
        }
        for _ in 0..10 {
            assert!(segment_permutation_shifter.try_forward());
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 201), indexed_element.element.as_ref());
            }
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 202), indexed_element.element.as_ref());
            }
            // one loop to permit getting the 200, 202 scenario
            assert!(segment_permutation_shifter.try_increment());
            {
                let indexed_element = segment_permutation_shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(20, 200), indexed_element.element.as_ref());
            }
            assert!(!segment_permutation_shifter.try_increment());
            assert!(!segment_permutation_shifter.try_backward());
        }
    }

    #[rstest]
    fn three_segments_in_large_bounding_length() {
        let segments_total = 3;

        let mut segments: Vec<Rc<Segment>> = Vec::new();
        for _ in 0..segments_total {
            segments.push(Rc::new(Segment::new(1)));
        }
        let mut segment_permutation_shifter_0 = SegmentPermutationShifter::new(segments.clone(), (0, 0), 255, false, 1, false);
        segment_permutation_shifter_0.randomize();
        let mut shifter_incrementer_0 = ShifterIncrementer::new(Rc::new(RefCell::new(segment_permutation_shifter_0)), vec![0]);
        let mut segment_permutation_shifter_1 = SegmentPermutationShifter::new(segments.clone(), (0, 0), 255, false, 1, false);
        segment_permutation_shifter_1.randomize();
        let mut shifter_incrementer_1 = ShifterIncrementer::new(Rc::new(RefCell::new(segment_permutation_shifter_1)), vec![0]);

        let mut iterations = 0;
        while shifter_incrementer_0.try_increment() {
            iterations += 1;
        }
        println!("iterations: {iterations}");
    }

    fn decrement_incrementer() {
        todo!();
    }
}
