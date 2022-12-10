use bitvec::vec::BitVec;
use uuid::Uuid;


#[derive(Clone, Debug)]
pub struct Segment<TIdentifier: Clone + std::fmt::Debug> {
    id: TIdentifier,
    length: usize
}

impl<TIdentifier: Clone + std::fmt::Debug> Segment<TIdentifier> {
    pub fn new(id: TIdentifier, length: usize) -> Self {
        Segment {
            id: id,
            length: length
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocatedSegment<TIdentifier: Clone + std::fmt::Debug> {
    id: TIdentifier,
    position: usize,
    length: usize
}

impl<TIdentifier: Clone + std::fmt::Debug> LocatedSegment<TIdentifier> {
    pub fn new(id: TIdentifier, position: usize, length: usize) -> Self {
        LocatedSegment {
            id: id,
            position: position,
            length: length
        }
    }
}

#[derive(Clone, Debug)]
pub struct SegmentContainer<TIdentifier: Clone + std::fmt::Debug> {
    segments: Vec<Segment<TIdentifier>>
}

impl<TIdentifier: Clone + std::fmt::Debug> SegmentContainer<TIdentifier> {
    pub fn new(segments: Vec<Segment<TIdentifier>>) -> Self {
        SegmentContainer {
            segments: segments
        }
    }
    fn get_segment_location_permutations_within_bounding_length_and_padding_excluding_mask(segments: &Vec<Segment<TIdentifier>>, mask: &mut BitVec, length: usize, padding: usize, position_offset: usize) -> Vec<Vec<LocatedSegment<TIdentifier>>> {
        debug!("get all possible positions when mask {}, length {}, and position offset {}", mask, length, position_offset);
        let mut snapshots: Vec<Vec<LocatedSegment<TIdentifier>>> = Vec::new();
        for (segment_index, segment) in segments.iter().enumerate() {
            if mask[segment_index] {
                debug!("searching {:?} as A", segment.id);
                mask.set(segment_index, false);
                let mut other_segments_total_length = 0;
                for (other_segment_index, other_segment) in segments.iter().enumerate() {
                    if mask[other_segment_index] {
                        if other_segments_total_length == 0 {
                            other_segments_total_length += other_segment.length;
                        }
                        else {
                            other_segments_total_length += other_segment.length + padding;
                        }
                    }
                }
                debug!("other_segments_total_length: {}", other_segments_total_length);
                
                if other_segments_total_length == 0 {
                    let current_segment_position_maximum: usize;
                    if position_offset == 0 {
                        current_segment_position_maximum = length - segment.length;
                    }
                    else {
                        current_segment_position_maximum = 0;
                    }
                    debug!("creating snapshots of this single segment {} from 0 to ={}", segment_index, current_segment_position_maximum);
                    for current_segment_position in 0..=current_segment_position_maximum {
                        let mut snapshot: Vec<LocatedSegment<TIdentifier>> = Vec::new();
                        snapshot.push(LocatedSegment {
                            id: segment.id.clone(),
                            position: current_segment_position + position_offset,
                            length: segment.length
                        });
                        snapshots.push(snapshot);
                    }
                }
                else {
                    let creating_snapshots_uuid = Uuid::new_v4().to_string();
                    debug!("creating snapshots after getting other segment: {}", creating_snapshots_uuid);
                    debug!("iterating over other_segments_current_length from {} to ={}", other_segments_total_length, (length - segment.length - padding));
                    for other_segments_current_length in other_segments_total_length..=(length - segment.length - padding) {
                        let other_position_offset = length - other_segments_current_length;
                        let recursive_uuid = Uuid::new_v4().to_string();
                        debug!("recursively search other segments: {}", recursive_uuid);
                        let other_segments_all_possible_positions = SegmentContainer::get_segment_location_permutations_within_bounding_length_and_padding_excluding_mask(segments, mask, other_segments_current_length, padding, other_position_offset + position_offset);
                        debug!("recursively searched other segments: {}", recursive_uuid);
                        debug!("found {} other segment position snapshots", other_segments_all_possible_positions.len());
                        let current_segment_position_maximum: usize;
                        if position_offset == 0 {
                            current_segment_position_maximum = length - other_segments_current_length - segment.length - padding;
                        }
                        else {
                            current_segment_position_maximum = 0;
                        }
                        debug!("moving A relatively forward from 0 to {}", current_segment_position_maximum);
                        for current_segment_position in 0..=current_segment_position_maximum {
                            for other_segment_snapshot in other_segments_all_possible_positions.iter() {
                                let mut snapshot: Vec<LocatedSegment<TIdentifier>> = Vec::new();
                                snapshot.push(LocatedSegment {
                                    id: segment.id.clone(),
                                    position: current_segment_position + position_offset,
                                    length: segment.length
                                });
                                for other_segment_snapshot_segment in other_segment_snapshot.iter() {
                                    snapshot.push(other_segment_snapshot_segment.clone());
                                }
                                snapshots.push(snapshot);
                            }
                        }
                    }
                    debug!("created snapshots after getting other segments: {}", creating_snapshots_uuid);
                }

                mask.set(segment_index, true);
            }
        }
        snapshots
    }
    pub fn get_segment_location_permutations_within_bounding_length(&self, length: usize, padding: usize) -> Vec<Vec<LocatedSegment<TIdentifier>>> {
        let mut mask: BitVec = BitVec::new();
        for _ in 0..self.segments.len() {
            mask.push(true);
        }
        SegmentContainer::get_segment_location_permutations_within_bounding_length_and_padding_excluding_mask(&self.segments, &mut mask, length, padding, 0)
    }
}

#[cfg(test)]
mod segment_container_tests {
    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialize_located_segment() {
        init();

        let id: String = Uuid::new_v4().to_string();
        let located_segment = LocatedSegment::new(id.clone(), 1, 2);
        assert_eq!(id, located_segment.id);
        assert_eq!(1, located_segment.position);
        assert_eq!(2, located_segment.length);
    }

    #[rstest]
    fn initialize_segment_container() {
        init();

        let _segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("segment_0"), 2),
            Segment::new(String::from("segment_1"), 3)
        ]);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    fn get_all_possible_positions_within_bounding_length_with_no_segments_padding_one(#[case] bounding_length: usize) {
        init();
        
        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![]);
        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(bounding_length, 1);
        assert_eq!(0, permutations.len());
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn get_all_possible_positions_within_bounding_length_with_one_segment_size_one_padding_one(#[case] bounding_length: usize) {
        init();
        
        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("segment_0"), 1)
        ]);
        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(bounding_length, 1);
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
        
        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("segment_0"), 2)
        ]);
        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(bounding_length, 1);
        assert_eq!(bounding_length - 1, permutations.len());

        for (bounding_length_index, permutation) in std::iter::zip(0..bounding_length, permutations.iter()) {
            assert_eq!(1, permutation.len());
            assert_eq!(bounding_length_index, permutation[0].position);
        }
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_two_segments_size_one_bounds_three_padding_one() {
        init();

        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("segment_0"), 1),
            Segment::new(String::from("segment_1"), 1)
        ]);
        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(3, 1);
        assert_eq!(2, permutations.len());
        assert_eq!("segment_0", permutations[0][0].id);
        assert_eq!(0, permutations[0][0].position);
        assert_eq!(1, permutations[0][0].length);
        assert_eq!("segment_1", permutations[0][1].id);
        assert_eq!(2, permutations[0][1].position);
        assert_eq!(1, permutations[0][1].length);
        assert_eq!("segment_1", permutations[1][0].id);
        assert_eq!(0, permutations[1][0].position);
        assert_eq!(1, permutations[1][0].length);
        assert_eq!("segment_0", permutations[1][1].id);
        assert_eq!(2, permutations[1][1].position);
        assert_eq!(1, permutations[1][1].length);
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_two_segments_size_one_bounds_five_padding_two() {
        init();

        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("0"), 1),
            Segment::new(String::from("1"), 1)
        ]);

        // 0---1
        // -0--1
        // 0--1-
        // 1---0
        // -1--0
        // 1--0-

        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(5, 2);

        for snapshot in permutations.iter() {
            for print_index in 0..5 {
                let mut is_found = false;
                for located_segment in snapshot {
                    if print_index >= located_segment.position && print_index < located_segment.position + located_segment.length {
                        print!("{}", located_segment.id);
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

        assert_eq!(6, permutations.len());
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_two_segments_size_two_bounds_six_padding_one() {
        init();

        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("segment_0"), 2),
            Segment::new(String::from("segment_1"), 2)
        ]);

        // 00--11
        // -00-11
        // 00-11-
        // 11--00
        // -11-00
        // 11-00-

        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(6, 1);

        debug!("permutations: {:?}", permutations);

        assert_eq!(6, permutations.len());
        assert_eq!("segment_0", permutations[0][0].id);
        assert_eq!(0, permutations[0][0].position);
        assert_eq!(2, permutations[0][0].length);
        assert_eq!("segment_1", permutations[0][1].id);
        assert_eq!(4, permutations[0][1].position);
        assert_eq!(2, permutations[0][1].length);
        assert_eq!("segment_0", permutations[1][0].id);
        assert_eq!(1, permutations[1][0].position);
        assert_eq!(2, permutations[1][0].length);
        assert_eq!("segment_1", permutations[1][1].id);
        assert_eq!(4, permutations[1][1].position);
        assert_eq!(2, permutations[1][1].length);
        assert_eq!("segment_0", permutations[2][0].id);
        assert_eq!(0, permutations[2][0].position);
        assert_eq!(2, permutations[2][0].length);
        assert_eq!("segment_1", permutations[2][1].id);
        assert_eq!(3, permutations[2][1].position);
        assert_eq!(2, permutations[2][1].length);
        assert_eq!("segment_1", permutations[3][0].id);
        assert_eq!(0, permutations[3][0].position);
        assert_eq!(2, permutations[3][0].length);
        assert_eq!("segment_0", permutations[3][1].id);
        assert_eq!(4, permutations[3][1].position);
        assert_eq!(2, permutations[3][1].length);
        assert_eq!("segment_1", permutations[4][0].id);
        assert_eq!(1, permutations[4][0].position);
        assert_eq!(2, permutations[4][0].length);
        assert_eq!("segment_0", permutations[4][1].id);
        assert_eq!(4, permutations[4][1].position);
        assert_eq!(2, permutations[4][1].length);
        assert_eq!("segment_1", permutations[5][0].id);
        assert_eq!(0, permutations[5][0].position);
        assert_eq!(2, permutations[5][0].length);
        assert_eq!("segment_0", permutations[5][1].id);
        assert_eq!(3, permutations[5][1].position);
        assert_eq!(2, permutations[5][1].length);
    }

    #[rstest]
    fn get_all_possible_positions_within_bounding_length_with_three_segments_size_two_bounds_ten_padding_one() {
        init();

        let segment_container: SegmentContainer<String> = SegmentContainer::new(vec![
            Segment::new(String::from("0"), 2),
            Segment::new(String::from("1"), 2),
            Segment::new(String::from("2"), 2)
        ]);

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

        let permutations = segment_container.get_segment_location_permutations_within_bounding_length(10, 1);

        for snapshot in permutations.iter() {
            for print_index in 0..10 {
                let mut is_found = false;
                for located_segment in snapshot {
                    if print_index >= located_segment.position && print_index < located_segment.position + located_segment.length {
                        print!("{}", located_segment.id);
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
}