use crate::{segment_container::{SegmentPermutationIncrementer}, index_incrementer::IndexIncrementer};

pub struct IndexedElement<T> {
    pub index: usize,
    pub element: T
}

pub trait ElementIndexer {
    type T;
    fn try_get_next_elements(&mut self) -> Option<Vec<IndexedElement<Self::T>>>;
}

pub struct SegmentPermutationIncrementerElementIndexer<'a> {
    element_indexes: &'a Vec<usize>,
    segment_permutation_incrementer: SegmentPermutationIncrementer<'a>,
    origin_location: (i32, i32),
    is_horizontal: bool
}

impl<'a> ElementIndexer for SegmentPermutationIncrementerElementIndexer<'a> {
    type T = (i32, i32);

    fn try_get_next_elements(&mut self) -> Option<Vec<IndexedElement<Self::T>>> {
        let segment_location_permutations_option = self.segment_permutation_incrementer.try_get_next_segment_location_permutations();
        if let Some(segment_location_permutations) = segment_location_permutations_option {
            let mut indexed_elements: Vec<IndexedElement<(i32, i32)>> = Vec::new();
            for segment_location in segment_location_permutations.into_iter() {
                let element: (i32, i32);
                if self.is_horizontal {
                    element = (self.origin_location.0 + segment_location.position as i32, self.origin_location.1);
                }
                else {
                    element = (self.origin_location.0, self.origin_location.1 + segment_location.position as i32);
                }
                indexed_elements.push(IndexedElement {
                    index: self.element_indexes[segment_location.segment_index],
                    element: element
                });
            }
            return Some(indexed_elements);
        }
        return None;
    }
}

impl<'a> SegmentPermutationIncrementerElementIndexer<'a> {
    pub fn new(element_indexes: &'a Vec<usize>, segment_permutation_incrementer: SegmentPermutationIncrementer<'static>, origin_location: (i32, i32), is_horizontal: bool) -> Self {

        if element_indexes.len() != segment_permutation_incrementer.get_segments_length() {
            panic!("Unexpected mismatch of lengths between element_indexes {} and segment_permutation_incrementer {}.", element_indexes.len(), segment_permutation_incrementer.get_segments_length());
        }

        SegmentPermutationIncrementerElementIndexer {
            element_indexes: element_indexes,
            segment_permutation_incrementer: segment_permutation_incrementer,
            origin_location: origin_location,
            is_horizontal: is_horizontal
        }
    }
}

pub struct IndexIncrementerElementIndexer<'a> {
    element_indexes: &'a Vec<usize>,
    index_incrementer: IndexIncrementer,
    locations_per_element: Vec<Vec<(i32, i32)>>,
    is_last_increment_successful: bool
}

impl<'a> ElementIndexer for IndexIncrementerElementIndexer<'a> {
    type T = (i32, i32);

    fn try_get_next_elements(&mut self) -> Option<Vec<IndexedElement<Self::T>>> {
        if !self.is_last_increment_successful {
            return None;
        }
        let location_index_per_element_index = self.index_incrementer.get();
        self.is_last_increment_successful = self.index_incrementer.try_increment();

        let mut indexed_elements: Vec<IndexedElement<(i32, i32)>> = Vec::new();
        for (element_index, location_index) in location_index_per_element_index.iter().enumerate() {
            let locations = &self.locations_per_element[element_index];
            let element = locations[location_index.unwrap()];
            indexed_elements.push(IndexedElement {
                index: self.element_indexes[element_index],
                element: element
            });
        }
        return Some(indexed_elements);
    }
}

impl<'a> IndexIncrementerElementIndexer<'a> {
    pub fn new(element_indexes: &'a Vec<usize>, locations_per_element: Vec<Vec<(i32, i32)>>) -> Self {

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

#[cfg(test)]
mod element_indexer_tests {
    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialize_segment_permutation_incrementer_element_indexer() {
        init();

        todo!();
    }
}