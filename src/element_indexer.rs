use std::{rc::Rc, collections::BTreeMap};

use crate::{segment_container::{SegmentPermutationIncrementer}, index_incrementer::IndexIncrementer};

#[derive(Debug)]
pub struct IndexedElement<'a, T> {
    pub index: usize,
    pub element: &'a T
}

pub trait ElementIndexer {
    type T;
    fn try_get_next_indexed_elements(&mut self) -> Option<Vec<IndexedElement<Self::T>>>;
    fn reset(&mut self);
}

pub struct SegmentPermutationIncrementerElementIndexer<'a> {
    element_indexes: &'a Vec<usize>,
    segment_permutation_incrementer: SegmentPermutationIncrementer<'a>,
    origin_location: (i32, i32),
    is_horizontal: bool,
    calculated_location_per_position: BTreeMap<usize, (i32, i32)>
}

impl<'a> ElementIndexer for SegmentPermutationIncrementerElementIndexer<'a> {
    type T = (i32, i32);

    fn try_get_next_indexed_elements(&mut self) -> Option<Vec<IndexedElement<Self::T>>> {
        let segment_location_permutations_option = self.segment_permutation_incrementer.try_get_next_segment_location_permutations();
        if let Some(segment_location_permutations) = segment_location_permutations_option {
            let mut indexed_elements: Vec<IndexedElement<(i32, i32)>> = Vec::new();
            for segment_location in segment_location_permutations.iter() {
                let is_calculated_location_cached = self.calculated_location_per_position.contains_key(&segment_location.position);
                if !is_calculated_location_cached {
                    let element: (i32, i32);
                    if self.is_horizontal {
                        element = (self.origin_location.0 + segment_location.position as i32, self.origin_location.1);
                    }
                    else {
                        element = (self.origin_location.0, self.origin_location.1 + segment_location.position as i32);
                    }
                    self.calculated_location_per_position.insert(segment_location.position, element);
                }
            }
            for segment_location in segment_location_permutations.into_iter() {
                indexed_elements.push(IndexedElement {
                    index: self.element_indexes[segment_location.segment_index],
                    element: self.calculated_location_per_position.get(&segment_location.position).unwrap()
                });
            }
            return Some(indexed_elements);
        }
        return None;
    }
    fn reset(&mut self) {
        self.segment_permutation_incrementer.reset();
    }
}

impl<'a> SegmentPermutationIncrementerElementIndexer<'a> {
    pub fn new(element_indexes: &'a Vec<usize>, segment_permutation_incrementer: SegmentPermutationIncrementer<'a>, origin_location: (i32, i32), is_horizontal: bool) -> Self {

        if element_indexes.len() != segment_permutation_incrementer.get_segments_length() {
            panic!("Unexpected mismatch of lengths between element_indexes {} and segment_permutation_incrementer {}.", element_indexes.len(), segment_permutation_incrementer.get_segments_length());
        }

        SegmentPermutationIncrementerElementIndexer {
            element_indexes: element_indexes,
            segment_permutation_incrementer: segment_permutation_incrementer,
            origin_location: origin_location,
            is_horizontal: is_horizontal,
            calculated_location_per_position: BTreeMap::new()
        }
    }
}

pub struct IndexIncrementerElementIndexer<'a, TElement> {
    element_indexes: &'a Vec<usize>,
    index_incrementer: IndexIncrementer,
    locations_per_element: Vec<Vec<TElement>>,
    is_last_increment_successful: bool
}

impl<'a, TElement> ElementIndexer for IndexIncrementerElementIndexer<'a, TElement> {
    type T = TElement;

    fn try_get_next_indexed_elements(&mut self) -> Option<Vec<IndexedElement<Self::T>>> {
        if !self.is_last_increment_successful {
            return None;
        }
        let location_index_per_element_index = self.index_incrementer.get();
        self.is_last_increment_successful = self.index_incrementer.try_increment();

        let mut indexed_elements: Vec<IndexedElement<TElement>> = Vec::new();
        for (element_index, location_index) in location_index_per_element_index.iter().enumerate() {
            let locations = &self.locations_per_element[element_index];
            let element = &locations[location_index.unwrap()];
            indexed_elements.push(IndexedElement {
                index: self.element_indexes[element_index],
                element: element
            });
        }
        return Some(indexed_elements);
    }
    fn reset(&mut self) {
        self.index_incrementer.reset();
    }
}

impl<'a, TElement> IndexIncrementerElementIndexer<'a, TElement> {
    pub fn new(element_indexes: &'a Vec<usize>, locations_per_element: Vec<Vec<TElement>>) -> Self {

        if locations_per_element.len() != element_indexes.len() {
            panic!("Unexpected mismatch of lengths between element_indexes {} and locations_per_element {}.", element_indexes.len(), locations_per_element.len());
        }

        let index_incrementer = IndexIncrementer::from_vector_of_vector(&locations_per_element);
        IndexIncrementerElementIndexer {
            element_indexes: element_indexes,
            index_incrementer: index_incrementer,
            locations_per_element: locations_per_element,
            is_last_increment_successful: true
        }
    }
}

pub struct ElementIndexerIncrementer<'a, T> {
    element_indexers: Vec<Box<dyn ElementIndexer<T = T>>>,
    previous_indexed_elements: Vec<Option<Rc<Vec<IndexedElement<'a, T>>>>>
}

impl<'a, T> ElementIndexerIncrementer<'a, T> {
    pub fn new(element_indexers: Vec<Box<dyn ElementIndexer<T = T>>>) -> Self {
        let mut previous_indexed_elements: Vec<Option<Rc<Vec<IndexedElement<T>>>>> = Vec::new();
        for _ in element_indexers.iter() {
            previous_indexed_elements.push(None);
        }
        ElementIndexerIncrementer {
            element_indexers: element_indexers,
            previous_indexed_elements: previous_indexed_elements
        }
    }
    pub fn try_get_next_indexed_elements_per_element_indexer(&mut self) -> Option<Vec<Rc<Vec<IndexedElement<T>>>>>{
        let mut indexed_elements_per_element_indexer: Vec<Rc<Vec<IndexedElement<T>>>> = Vec::new();
        let mut is_previous_element_indexer_incremented: bool = false;
        let mut is_last_element_indexer_cycled: bool = self.element_indexers.is_empty();
        let mut element_indexer_index: usize = 0;
        let element_indexers_length: usize = self.element_indexers.len();
        while element_indexer_index < element_indexers_length {
            if self.previous_indexed_elements[element_indexer_index].is_none() {
                let indexed_elements = self.element_indexers[element_indexer_index].try_get_next_indexed_elements().unwrap();
                self.previous_indexed_elements[element_indexer_index] = Some(Rc::new(indexed_elements));
            }
        }
        if is_last_element_indexer_cycled {
            None
        }
        else {
            Some(indexed_elements_per_element_indexer)
        }
    }
}

#[cfg(test)]
mod element_indexer_tests {
    use crate::segment_container::Segment;

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[should_panic]
    fn initialize_segment_permutation_incrementer_element_indexer_without_initializing_element_indexes(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let element_indexes: Vec<usize> = Vec::new();
        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (1, 2);
        let is_horizontal: bool = true;

        let _: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(&element_indexes, segment_permutation_incrementer, origin_location, is_horizontal);
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    fn initialize_segment_permutation_incrementer_element_indexer_with_initialized_element_indexes(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let mut element_indexes: Vec<usize> = Vec::new();
        for element_index in 0..segments_total {
            element_indexes.push(element_index * element_index);
        }

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (1, 2);
        let is_horizontal: bool = true;

        let _: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(&element_indexes, segment_permutation_incrementer, origin_location, is_horizontal);
    }

    #[rstest]
    #[case(3)]
    fn get_elements_from_specific_segment_permutation_incrementer_element_indexer(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let mut element_indexes: Vec<usize> = Vec::new();
        for element_index in 0..segments_total {
            element_indexes.push(element_index * element_index);
        }

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (10, 100);
        let is_horizontal: bool = true;

        let mut element_indexer: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(&element_indexes, segment_permutation_incrementer, origin_location, is_horizontal);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_some());

        let indexed_elements = indexed_elements_option.unwrap();
        assert_eq!(3, indexed_elements.len());
        assert_eq!(0, indexed_elements[0].index);
        assert_eq!(&(10, 100), indexed_elements[0].element);
        assert_eq!(1, indexed_elements[1].index);
        assert_eq!(&(12, 100), indexed_elements[1].element);
        assert_eq!(4, indexed_elements[2].index);
        assert_eq!(&(15, 100), indexed_elements[2].element);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_some());

        let indexed_elements = indexed_elements_option.unwrap();
        assert_eq!(3, indexed_elements.len());
        assert_eq!(0, indexed_elements[0].index);
        assert_eq!(&(10, 100), indexed_elements[0].element);
        assert_eq!(4, indexed_elements[1].index);
        assert_eq!(&(12, 100), indexed_elements[1].element);
        assert_eq!(1, indexed_elements[2].index);
        assert_eq!(&(16, 100), indexed_elements[2].element);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_some());

        let indexed_elements = indexed_elements_option.unwrap();
        assert_eq!(3, indexed_elements.len());
        assert_eq!(1, indexed_elements[0].index);
        assert_eq!(&(10, 100), indexed_elements[0].element);
        assert_eq!(0, indexed_elements[1].index);
        assert_eq!(&(13, 100), indexed_elements[1].element);
        assert_eq!(4, indexed_elements[2].index);
        assert_eq!(&(15, 100), indexed_elements[2].element);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_some());

        let indexed_elements = indexed_elements_option.unwrap();
        assert_eq!(3, indexed_elements.len());
        assert_eq!(1, indexed_elements[0].index);
        assert_eq!(&(10, 100), indexed_elements[0].element);
        assert_eq!(4, indexed_elements[1].index);
        assert_eq!(&(13, 100), indexed_elements[1].element);
        assert_eq!(0, indexed_elements[2].index);
        assert_eq!(&(17, 100), indexed_elements[2].element);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_some());

        let indexed_elements = indexed_elements_option.unwrap();
        assert_eq!(3, indexed_elements.len());
        assert_eq!(4, indexed_elements[0].index);
        assert_eq!(&(10, 100), indexed_elements[0].element);
        assert_eq!(0, indexed_elements[1].index);
        assert_eq!(&(14, 100), indexed_elements[1].element);
        assert_eq!(1, indexed_elements[2].index);
        assert_eq!(&(16, 100), indexed_elements[2].element);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_some());

        let indexed_elements = indexed_elements_option.unwrap();
        assert_eq!(3, indexed_elements.len());
        assert_eq!(4, indexed_elements[0].index);
        assert_eq!(&(10, 100), indexed_elements[0].element);
        assert_eq!(1, indexed_elements[1].index);
        assert_eq!(&(14, 100), indexed_elements[1].element);
        assert_eq!(0, indexed_elements[2].index);
        assert_eq!(&(17, 100), indexed_elements[2].element);

        let indexed_elements_option = element_indexer.try_get_next_indexed_elements();
        println!("indexed_elements_option: {:?}", indexed_elements_option);
        assert!(indexed_elements_option.is_none());
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[case(10)]
    fn get_elements_from_segment_permutation_incrementer_element_indexer(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let mut element_indexes: Vec<usize> = Vec::new();
        for element_index in 0..segments_total {
            element_indexes.push(element_index * element_index);
        }

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (1, 2);
        let is_horizontal: bool = true;

        let mut element_indexer: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(&element_indexes, segment_permutation_incrementer, origin_location, is_horizontal);

        let mut is_successful = true;
        let mut iterations_total = 0;
        while is_successful {
            is_successful = element_indexer.try_get_next_indexed_elements().is_some();
            if is_successful {
                iterations_total += 1;
            }
        }

        let mut expected_iterations_total = 1;
        for segment_index in 0..segments_total {
            expected_iterations_total *= segment_index + 1;
        }
        assert_eq!(expected_iterations_total, iterations_total);
    }
    #[rstest]
    fn get_element_indexer_incrementer_zero_element_indexers() {
        let element_indexers: Vec<Box<dyn ElementIndexer<T = String>>> = Vec::new();
        let mut element_indexer_incrementer = ElementIndexerIncrementer::new(element_indexers);
        let indexed_elements_per_element_indexer = element_indexer_incrementer.try_get_next_indexed_elements_per_element_indexer();
        assert!(indexed_elements_per_element_indexer.is_none());
    }
    #[rstest]
    fn get_element_indexer_incrementer_one_element_indexer_index_incrementer() {
        let element_indexers: Vec<Box<dyn ElementIndexer<T = String>>> = Vec::new();
        element_indexers.push(Box::new(IndexIncrementerElementIndexer::new(
            
        )));
        let mut element_indexer_incrementer = ElementIndexerIncrementer::new(element_indexers);
        let indexed_elements_per_element_indexer = element_indexer_incrementer.try_get_next_indexed_elements_per_element_indexer();
        assert!(indexed_elements_per_element_indexer.is_none());
    }
}