use std::{rc::Rc, cell::RefCell};

use bitvec::{vec::BitVec, bits};
use uuid::Uuid;

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

#[derive(Debug)]
struct RecursiveSegmentPermutationIncrementer {
    segments: Rc<Vec<Segment>>,
    bounding_length: usize,
    padding: usize,
    position_offset: usize,
    depth: usize,
    // iterate over mask
    current_mask: Rc<RefCell<BitVec>>,  // TODO check if having the mask as a property is more efficient than passing it in to try_get_next_snapshot as a mutable reference
    current_segment_index: usize,
    // iterate over other segment's length
    current_other_segments_current_length: usize,
    current_other_segments_maximum_inclusive_length: usize,
    // iterate over other segment's snapshot
    current_other_segments_recursive_segment_permutation_incrementer: Box<Option<RecursiveSegmentPermutationIncrementer>>,
    current_other_segments_snapshot: Vec<LocatedSegment>,
    // iterate over current segment's position
    current_segment_position: Option<usize>,
    current_segment_position_maximum_inclusive: usize
}

impl RecursiveSegmentPermutationIncrementer {
    fn new(segments: Rc<Vec<Segment>>, bounding_length: usize, padding: usize, current_mask: Rc<RefCell<BitVec>>, position_offset: usize, depth: usize) -> Self {

        //println!("{:?}: new RecursiveSegmentPermutationIncrementer: {:?}", depth, position_offset);

        // TODO consider refactoring such that all "current" properties are options that then can be initialized during the first call to try_get_next_snapshot

        let mut current_segment_index: usize = 0;
        while current_segment_index < segments.len() && !current_mask.borrow()[current_segment_index] {
            current_segment_index += 1;
        }
        if current_segment_index == segments.len() {
            panic!("Unexpected lack of available positions in mask at this depth of recursion.");
        }
        current_mask.borrow_mut().set(current_segment_index, false);

        let mut other_segments_total_length = 0;
        for (other_segment_index, other_segment) in segments.iter().enumerate() {
            if current_mask.borrow()[other_segment_index] {
                if other_segments_total_length == 0 {
                    other_segments_total_length += other_segment.length;
                }
                else {
                    other_segments_total_length += other_segment.length + padding;
                }
            }
        }

        let current_other_segments_current_length = other_segments_total_length;

        let current_other_segments_maximum_inclusive_length: usize;
        if other_segments_total_length == 0 {
            current_other_segments_maximum_inclusive_length = 0;
        }
        else {
            current_other_segments_maximum_inclusive_length = bounding_length - segments[current_segment_index].length - padding;
        }
    
        let mut current_other_segments_recursive_segment_permutation_incrementer: Box<Option<RecursiveSegmentPermutationIncrementer>>;
        let current_other_segments_snapshot: Vec<LocatedSegment>;

        if current_other_segments_current_length == 0 {
            // there are no other segments to iterate over
            current_other_segments_recursive_segment_permutation_incrementer = Box::new(None);
            current_other_segments_snapshot = vec![];
        }
        else {
            let other_position_offset = bounding_length - current_other_segments_current_length;

            //println!("{:?}: other_position_offset {:?} = bounding_length {:?} - current_other_segments_current_length {:?}", depth, other_position_offset, bounding_length, current_other_segments_current_length);

            current_other_segments_recursive_segment_permutation_incrementer = Box::new(Some(RecursiveSegmentPermutationIncrementer::new(
                segments.clone(),
                current_other_segments_current_length,
                padding,
                current_mask.clone(),
                position_offset + other_position_offset,
                depth + 1
            )));
            let other_segments_recursive_segment_permutation_incrementer = current_other_segments_recursive_segment_permutation_incrementer.as_mut().as_mut().unwrap();
            current_other_segments_snapshot = other_segments_recursive_segment_permutation_incrementer.try_get_next_snapshot().expect("The newly created RecursiveSegmentPermutationIncrementer should have at least one valid state within it.");
        }

        RecursiveSegmentPermutationIncrementer {
            segments: segments,
            bounding_length: bounding_length,
            padding: padding,
            position_offset: position_offset,
            current_mask: current_mask,
            current_segment_index: current_segment_index,
            current_other_segments_current_length: current_other_segments_current_length,
            current_other_segments_maximum_inclusive_length: current_other_segments_maximum_inclusive_length,
            current_other_segments_recursive_segment_permutation_incrementer: current_other_segments_recursive_segment_permutation_incrementer,
            current_other_segments_snapshot: current_other_segments_snapshot,
            current_segment_position: None,
            current_segment_position_maximum_inclusive: 0,
            depth: depth
        }
    }
    fn try_get_next_snapshot(&mut self) -> Option<Vec<LocatedSegment>> {

        // try to increment the current other segment current length

        // start at first snapshot or roll over to next current_segment_position

        // if current_segment_position increment would take it outside of bounds...
        if self.current_segment_position.is_none() || self.current_segment_position.unwrap() + 1 > self.current_segment_position_maximum_inclusive {

            if !self.current_segment_position.is_none() && self.current_segment_position.unwrap() + 1 > self.current_segment_position_maximum_inclusive {

                let is_next_other_segments_current_length_required: bool;
                if self.current_other_segments_recursive_segment_permutation_incrementer.is_none() {
                    // this was the leaf node of the recursive algorithm
                    is_next_other_segments_current_length_required = true;
                }
                else {
                    let other_segments_recursive_segment_permutation_incrementer = self.current_other_segments_recursive_segment_permutation_incrementer.as_mut().as_mut().unwrap();
                    let current_other_segments_snapshot_option = other_segments_recursive_segment_permutation_incrementer.try_get_next_snapshot();

                    if let Some(current_other_segments_snapshot) = current_other_segments_snapshot_option {
                        self.current_other_segments_snapshot = current_other_segments_snapshot;
                        is_next_other_segments_current_length_required = false;
                    }
                    else {
                        is_next_other_segments_current_length_required = true;
                    }
                }

                if is_next_other_segments_current_length_required {

                    // if current_other_segments_current_length increment would take it outside of bounds...
                    if self.current_other_segments_current_length + 1 > self.current_other_segments_maximum_inclusive_length {
                                
                        // increment the current segment index
                        self.current_mask.borrow_mut().set(self.current_segment_index, true);
                        self.current_segment_index += 1;
                        while self.current_segment_index < self.segments.len() && !self.current_mask.borrow()[self.current_segment_index] {
                            self.current_segment_index += 1;
                        }
                        if self.current_segment_index == self.segments.len() {
                            return None;
                        }
                        self.current_mask.borrow_mut().set(self.current_segment_index, false);

                        let mut other_segments_total_length = 0;
                        for (other_segment_index, other_segment) in self.segments.iter().enumerate() {
                            if self.current_mask.borrow()[other_segment_index] {
                                if other_segments_total_length == 0 {
                                    other_segments_total_length += other_segment.length;
                                }
                                else {
                                    other_segments_total_length += other_segment.length + self.padding;
                                }
                            }
                        }

                        self.current_other_segments_current_length = other_segments_total_length;
                        self.current_other_segments_maximum_inclusive_length = self.bounding_length - self.segments[self.current_segment_index].length - self.padding;
                    }
                    else {
                        self.current_other_segments_current_length += 1;
                    }

                    if self.current_other_segments_current_length == 0 {
                        // there are no other segments to iterate over
                        self.current_other_segments_recursive_segment_permutation_incrementer = Box::new(None);
                        self.current_other_segments_snapshot = vec![];
                    }
                    else {
                        let other_position_offset = self.bounding_length - self.current_other_segments_current_length;

                        self.current_other_segments_recursive_segment_permutation_incrementer = Box::new(Some(RecursiveSegmentPermutationIncrementer::new(
                            self.segments.clone(),
                            self.current_other_segments_current_length,
                            self.padding,
                            self.current_mask.clone(),
                            self.position_offset + other_position_offset,
                            self.depth + 1
                        )));
                        let other_segments_recursive_segment_permutation_incrementer = self.current_other_segments_recursive_segment_permutation_incrementer.as_mut().as_mut().unwrap();
                        self.current_other_segments_snapshot = other_segments_recursive_segment_permutation_incrementer.try_get_next_snapshot().expect("The newly created RecursiveSegmentPermutationIncrementer should have at least one valid state within it.");
                    }
                }
            }
            
            self.current_segment_position = Some(0);

            if self.depth == 0 {
                if self.current_other_segments_current_length == 0 {
                    self.current_segment_position_maximum_inclusive = self.bounding_length - self.segments[self.current_segment_index].length;
                }
                else {
                    self.current_segment_position_maximum_inclusive = self.bounding_length - self.current_other_segments_current_length - self.segments[self.current_segment_index].length - self.padding;
                }                
            }
            else {
                self.current_segment_position_maximum_inclusive = 0;
            }
        }
        else {
            self.current_segment_position = Some(self.current_segment_position.unwrap() + 1);
        }

        let mut snapshot: Vec<LocatedSegment> = Vec::new();
        snapshot.push(LocatedSegment {
            segment_index: self.current_segment_index,
            position: self.current_segment_position.unwrap() + self.position_offset
        });
        // TODO consider doing snapshot.extend here (or something like it)
        //for other_segment_snapshot_segment in self.current_other_segments_snapshot.drain(0..) {
        //    snapshot.push(other_segment_snapshot_segment.clone());
        //}
        for other_segment_snapshot_segment in self.current_other_segments_snapshot.iter() {
            snapshot.push(other_segment_snapshot_segment.clone());
        }
        debug!("{}: snapshot: {:?}", self.depth, snapshot);
        return Some(snapshot);
    }
}

#[derive(Debug)]
pub struct SegmentPermutationIncrementer {
    segments: Rc<Vec<Segment>>,
    bounding_length: usize,
    padding: usize,
    recursive_segment_permutation_incrementer: Option<RecursiveSegmentPermutationIncrementer>
}

impl SegmentPermutationIncrementer {
    #[time_graph::instrument]
    pub fn new(segments: Rc<Vec<Segment>>, bounding_length: usize, padding: usize) -> Self {
        
        let recursive_segment_permutation_incrementer: Option<RecursiveSegmentPermutationIncrementer>;
        if segments.len() == 0 {
            recursive_segment_permutation_incrementer = None;
        }
        else {
            let mut mask = BitVec::with_capacity(segments.len());
            mask.resize(segments.len(), true);
            let wrapped_mask = Rc::new(RefCell::new(mask));
            recursive_segment_permutation_incrementer = Some(RecursiveSegmentPermutationIncrementer::new(segments.clone(), bounding_length, padding, wrapped_mask, 0, 0));
        }

        SegmentPermutationIncrementer {
            segments: segments,
            bounding_length: bounding_length,
            padding: padding,
            recursive_segment_permutation_incrementer: recursive_segment_permutation_incrementer
        }
    }
    #[time_graph::instrument]
    pub fn try_get_next_segment_location_permutations(&mut self) -> Option<Vec<LocatedSegment>> {
        if self.recursive_segment_permutation_incrementer.is_none() {
            None
        }
        else {
            self.recursive_segment_permutation_incrementer.as_mut().unwrap().try_get_next_snapshot()
        }
    }
    pub fn get_segments_length(&self) -> usize {
        self.segments.len()
    }
    pub fn reset(&mut self) {
        let mut mask = BitVec::with_capacity(self.segments.len());
        mask.resize(self.segments.len(), true);
        let wrapped_mask = Rc::new(RefCell::new(mask));
        self.recursive_segment_permutation_incrementer = Some(RecursiveSegmentPermutationIncrementer::new(self.segments.clone(), self.bounding_length, self.padding, wrapped_mask, 0, 0));
    }
}

impl Iterator for SegmentPermutationIncrementer {
    type Item = Vec<LocatedSegment>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_get_next_segment_location_permutations()
    }
}

#[cfg(test)]
mod segment_permutation_incrementer_tests {
    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialize_located_segment() {
        init();

        let located_segment = LocatedSegment::new(2, 1);
        assert_eq!(2, located_segment.segment_index);
        assert_eq!(1, located_segment.position);
    }

    #[rstest]
    fn initialize_segment_permutation_incrementer_zero_segments() {
        init();

        let segments = vec![];

        let _: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(Rc::new(segments), 0, 0);
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(10)]
    fn initialize_segment_permutation_incrementer_more_than_zero_segments(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let smallest_bounding_length: usize = ((segments_total * (segments_total + 1)) / 2) as usize + (segments_total - 1);

        let _: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(Rc::new(segments), smallest_bounding_length, 1);
    }

    #[rstest]
    #[case(11)]
    fn initialize_segment_permutation_incrementer_large_number_of_segments(#[case] segments_total: usize) {
        init();

        time_graph::enable_data_collection(true);

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let smallest_bounding_length: usize = ((segments_total * (segments_total + 1)) / 2) as usize + (segments_total - 1);

        let _: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(Rc::new(segments), smallest_bounding_length, 1);

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[rstest]
    #[case(2)]
    fn get_segment_permutation_small_number_of_segments(#[case] segments_total: usize) {
        init();

        time_graph::enable_data_collection(true);

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let smallest_bounding_length: usize = ((segments_total * (segments_total + 1)) / 2) as usize + (segments_total - 1);

        let mut segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(Rc::new(segments), smallest_bounding_length, 1);

        let segment_location_permutations = segment_permutation_incrementer.try_get_next_segment_location_permutations();

        assert!(segment_location_permutations.is_some());

        println!("segment_location_permutations: {:?}", segment_location_permutations);

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[case(10)]
    fn get_segment_permutation_large_number_of_segments(#[case] segments_total: usize) {
        init();

        time_graph::enable_data_collection(true);

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let smallest_bounding_length: usize = ((segments_total * (segments_total + 1)) / 2) as usize + (segments_total - 1);

        println!("{:?}: smallest_bounding_length: {:?}", segments_total, smallest_bounding_length);

        let mut segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(Rc::new(segments), smallest_bounding_length, 1);

        let mut permutations_total = 0;
        let mut is_get_next_segment_location_permutation_successful = true;
        while is_get_next_segment_location_permutation_successful {
            let segment_location_permutations = segment_permutation_incrementer.try_get_next_segment_location_permutations();
            //println!("segment_location_permutations: {:?}", segment_location_permutations);

            if permutations_total == 0 {
                // the first one must succeed
                assert!(segment_location_permutations.is_some());
            }
            if segment_location_permutations.is_none() {
                is_get_next_segment_location_permutation_successful = false;
            }
            else {
                permutations_total += 1;
            }
        }

        println!("{:?}: permutations_total: {:?}", segments_total, permutations_total);

        println!("{}", time_graph::get_full_graph().as_dot());
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    fn get_all_possible_positions_within_bounding_length_with_no_segments_padding_one(#[case] bounding_length: usize) {
        init();
        
        let segments: Vec<Segment> = Vec::new();
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(Rc::new(segments), bounding_length, 1);
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();
        assert_eq!(0, permutations.len());
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn get_all_possible_positions_within_bounding_length_with_one_segment_size_one_padding_one(#[case] bounding_length: usize) {
        init();
        
        let segments: Vec<Segment> = vec![Segment::new(1)];
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            Rc::new(segments),
            bounding_length,
            1
        );
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();
        assert_eq!(bounding_length, permutations.len());

        for (bounding_length_index, permutation) in std::iter::zip(0..bounding_length, permutations.iter()) {
            assert_eq!(1, permutation.len());
            assert_eq!(bounding_length_index, permutation[0].position);
        }
    }

    #[rstest]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    fn get_all_possible_positions_within_bounding_length_with_one_segment_size_two_padding_one(#[case] bounding_length: usize) {
        init();
        
        let segments: Vec<Segment> = vec![Segment::new(2)];
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            Rc::new(segments),
            bounding_length,
            1
        );
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();
        assert_eq!(bounding_length - 1, permutations.len());

        for (bounding_length_index, permutation) in std::iter::zip(0..bounding_length, permutations.iter()) {
            assert_eq!(1, permutation.len());
            assert_eq!(bounding_length_index, permutation[0].position);
        }
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_two_segments_size_one_bounds_three_padding_one() {
        init();

        let segments: Vec<Segment> = vec![Segment::new(1), Segment::new(1)];
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            Rc::new(segments),
            3,
            1
        );
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();
        assert_eq!(2, permutations.len());
        assert_eq!(0, permutations[0][0].segment_index);
        assert_eq!(0, permutations[0][0].position);
        assert_eq!(1, permutations[0][1].segment_index);
        assert_eq!(2, permutations[0][1].position);
        assert_eq!(1, permutations[1][0].segment_index);
        assert_eq!(0, permutations[1][0].position);
        assert_eq!(0, permutations[1][1].segment_index);
        assert_eq!(2, permutations[1][1].position);
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_two_segments_size_one_bounds_five_padding_two() {
        init();

        let segments: Rc<Vec<Segment>> = Rc::new(vec![Segment::new(1), Segment::new(1)]);
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            segments.clone(),
            5,
            2
        );

        // 0---1
        // -0--1
        // 0--1-
        // 1---0
        // -1--0
        // 1--0-

        let mut permutations_total: usize = 0;
        for snapshot in segment_permutation_incrementer.into_iter() {
            permutations_total += 1;
            for print_index in 0..5 {
                let mut is_found = false;
                for located_segment in snapshot.iter() {
                    if print_index >= located_segment.position && print_index < located_segment.position + segments[located_segment.segment_index].length {
                        print!("{}", located_segment.segment_index);
                        is_found = true;
                        break;
                    }
                }
                if !is_found {
                    print!("-");
                }
            }
            println!("");
        }

        assert_eq!(6, permutations_total);
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_two_segments_size_two_bounds_six_padding_one() {
        init();

        let segments: Vec<Segment> = vec![Segment::new(2), Segment::new(2)];
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            Rc::new(segments),
            6,
            1
        );
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();

        // 00--11
        // -00-11
        // 00-11-
        // 11--00
        // -11-00
        // 11-00-

        debug!("permutations: {:?}", permutations);

        assert_eq!(6, permutations.len());
        assert_eq!(0, permutations[0][0].segment_index);
        assert_eq!(0, permutations[0][0].position);
        assert_eq!(1, permutations[0][1].segment_index);
        assert_eq!(4, permutations[0][1].position);
        assert_eq!(0, permutations[1][0].segment_index);
        assert_eq!(1, permutations[1][0].position);
        assert_eq!(1, permutations[1][1].segment_index);
        assert_eq!(4, permutations[1][1].position);
        assert_eq!(0, permutations[2][0].segment_index);
        assert_eq!(0, permutations[2][0].position);
        assert_eq!(1, permutations[2][1].segment_index);
        assert_eq!(3, permutations[2][1].position);
        assert_eq!(1, permutations[3][0].segment_index);
        assert_eq!(0, permutations[3][0].position);
        assert_eq!(0, permutations[3][1].segment_index);
        assert_eq!(4, permutations[3][1].position);
        assert_eq!(1, permutations[4][0].segment_index);
        assert_eq!(1, permutations[4][0].position);
        assert_eq!(0, permutations[4][1].segment_index);
        assert_eq!(4, permutations[4][1].position);
        assert_eq!(1, permutations[5][0].segment_index);
        assert_eq!(0, permutations[5][0].position);
        assert_eq!(0, permutations[5][1].segment_index);
        assert_eq!(3, permutations[5][1].position);
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_three_segments_size_two_bounds_ten_padding_one() {
        init();

        let segments: Rc<Vec<Segment>> = Rc::new(vec![Segment::new(2), Segment::new(2), Segment::new(2)]);
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            segments.clone(),
            10,
            1
        );
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();

        // 00---11-22
        // 00---22-11
        // -00--11-22
        // -00--22-11
        // --00-11-22
        // --00-22-11
        // 00--11--22
        // 00--11-22-
        // 00--22--11
        // 00--22-11-
        // -00-11--22
        // -00-11-22-
        // -00-22--11
        // -00-22-11-
        // 00-11---22
        // 00-11--22-
        // 00-11-22--
        // 00-22---11
        // 00-22--11-
        // 00-22-11--
        // 11---00-22
        // 11---22-00
        // -11--00-22
        // -11--22-00
        // --11-00-22
        // --11-22-00
        // 11--00--22
        // 11--00-22-
        // 11--22--00
        // 11--22-00-
        // -11-00--22
        // -11-00-22-
        // -11-22--00
        // -11-22-00-
        // 11-00---22
        // 11-00--22-
        // 11-00-22--
        // 11-22---00
        // 11-22--00-
        // 11-22-00--
        // 22---00-11
        // 22---11-00
        // -22--00-11
        // -22--11-00
        // --22-00-11
        // --22-11-00
        // 22--00--11
        // 22--00-11-
        // 22--11--00
        // 22--11-00-
        // -22-00--11
        // -22-00-11-
        // -22-11--00
        // -22-11-00-
        // 22-00---11
        // 22-00--11-
        // 22-00-11--
        // 22-11---00
        // 22-11--00-
        // 22-11-00--

        for snapshot in permutations.iter() {
            for print_index in 0..10 {
                let mut is_found = false;
                for located_segment in snapshot {
                    if print_index >= located_segment.position && print_index < located_segment.position + segments[located_segment.segment_index].length {
                        print!("{}", located_segment.segment_index);
                        is_found = true;
                        break;
                    }
                }
                if !is_found {
                    print!("-");
                }
            }
            println!("");
        }

        assert_eq!(60, permutations.len());

    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_ten_segments_size_incrementing_bounds_sixty_four_padding_one() {
        init();

        let segments = vec![
            Segment::new(1),
            Segment::new(2),
            Segment::new(3),
            Segment::new(4),
            Segment::new(5),
            Segment::new(6),
            Segment::new(7),
            Segment::new(8),
            Segment::new(9),
            Segment::new(10)
        ];

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(
            Rc::new(segments),
            64,
            1
        );
        let permutations = segment_permutation_incrementer.into_iter().collect::<Vec<Vec<LocatedSegment>>>();

        assert_eq!(60, permutations.len());
    }
}