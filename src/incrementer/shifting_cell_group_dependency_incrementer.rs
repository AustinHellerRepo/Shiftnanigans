// Purpose:
//      This struct will shift forward, always checking the new element state (the cell group location) against the existing element states from previous shift indexes (previous cell groups)
//      The idea is that once a bad pair of located cell groups are found, the current shifter can be pushed forward, skipping over any need to iterate over later shift indexes
//          A bonus is to store the earlier shift index, the earlier state (location), the current shift index, and the current state (location) so that when this pair is found again the current shift index can be incremented without having to do any real calculation
//              Implementation detail: maintain a Vec<Option<BTreeSet<TElement>>> where it is initialized to None for each shift index and the BTreeSet contains the ever-growing temp collection of bad states for that specific shift index
//                  It can be set back to None as each index is incremented across (from shift index 0 to n as each shift index state is found to be valid) since there's no need to look back
//                  It is filled from a master collection per shift index and state key of vectors of BTreeSets, filled as new bad pairs are discovered.

use std::{collections::VecDeque, rc::Rc, cell::RefCell};

use bitvec::vec::BitVec;

use crate::{shifter::{encapsulated_shifter::EncapsulatedShifter, Shifter}, IndexedElement};
use super::Incrementer;

#[derive(Clone)]
pub struct CellGroup {
    cells: Vec<(u8, u8)>,  // these should exist such that they can be added directly to location points
    cell_group_type_index: usize  // each type can have relationship attributes (detection location offsets, etc.)
}

pub struct LocatedCellGroup {
    cell_group_index: usize,
    location: Rc<(u8, u8)>
}

/// This struct specifies that "this" cell group location has "these" cell group location collections as dependencies such that if being at that location makes all of them invalid, then that location must be invalid
pub struct CellGroupDependency {
    cell_group_index_mapping: Vec<usize>,
    encapsulated_shifter: RefCell<EncapsulatedShifter<(u8, u8)>>
}

pub struct ShiftingCellGroupDependencyIncrementer {
    cell_groups: Vec<CellGroup>,
    cell_group_dependencies: Vec<CellGroupDependency>,
    current_cell_group_dependency_index: Option<usize>,
    current_locations: Vec<IndexedElement<(u8, u8)>>,
    current_element_index_and_state_index_pairs: Vec<(usize, usize)>,
    current_elements_total: usize,
    current_states_total: usize,
    current_is_checked: BitVec,
    current_is_valid: BitVec,
    current_states: Vec<Rc<(u8, u8)>>
}

impl ShiftingCellGroupDependencyIncrementer {
    pub fn new(cell_groups: Vec<CellGroup>, cell_group_dependencies: Vec<CellGroupDependency>) -> Self {
        ShiftingCellGroupDependencyIncrementer {
            cell_groups: cell_groups,
            cell_group_dependencies: cell_group_dependencies,
            current_cell_group_dependency_index: None,
            current_locations: Vec::new(),
            current_element_index_and_state_index_pairs: Vec::new(),
            current_elements_total: 0,
            current_states_total: 0,
            current_is_checked: BitVec::default(),
            current_is_valid: BitVec::default(),
            current_states: Vec::default()
        }
    }
}

// TODO implement Incrementer

impl Incrementer for ShiftingCellGroupDependencyIncrementer {
    type T = (u8, u8);

    fn try_increment(&mut self) -> bool {
        if self.current_cell_group_dependency_index.is_none() {
            if self.cell_group_dependencies.len() == 0 {
                return false;
            }
            debug!("starting with first dependency");
            self.current_cell_group_dependency_index = Some(0);
            // construct the bitvecs for current_is_checked and current_is_valid
            {
                let cell_group_dependency = &self.cell_group_dependencies[self.current_cell_group_dependency_index.unwrap()];
                self.current_elements_total = cell_group_dependency.encapsulated_shifter.borrow().get_length();
                self.current_states = cell_group_dependency.encapsulated_shifter.borrow().get_states();
                self.current_states_total = self.current_states.len();
                let bits_length = self.current_elements_total * self.current_elements_total * self.current_states_total * self.current_states_total;
                self.current_is_checked = BitVec::repeat(false, bits_length);
                self.current_is_valid = BitVec::repeat(false, bits_length);
            }
        }
        while self.current_cell_group_dependency_index.unwrap() != self.cell_group_dependencies.len() {
            debug!("choosing {:?}th dependency", self.current_cell_group_dependency_index);
            let cell_group_dependency = &self.cell_group_dependencies[self.current_cell_group_dependency_index.unwrap()];
            let encapsulated_shifter: &mut EncapsulatedShifter<(u8, u8)> = &mut cell_group_dependency.encapsulated_shifter.borrow_mut();
            let encapsulated_shifter_length = encapsulated_shifter.get_length();
            // loop until a valid collection of locations has been discovered
            let mut is_forward_required: bool;
            if self.current_locations.len() == encapsulated_shifter_length {
                debug!("popping last location to make room for next possible location");
                self.current_locations.pop();  // remove the last valid location
                self.current_element_index_and_state_index_pairs.pop();
                debug!("determined that forward is not required");
                is_forward_required = false;
            }
            else {
                debug!("determined that forward is required");
                is_forward_required = true;
                if self.current_locations.len() != 0 {
                    panic!("Unexpected state of current locations when next while loop should only result in 0 or max elements.");
                }
            }
            let mut is_fully_backward: bool = false;
            while self.current_locations.len() != encapsulated_shifter_length && !is_fully_backward {
                if is_forward_required {
                    debug!("moving forward to next shift index");
                    encapsulated_shifter.try_forward();
                }
                debug!("incrementing at current shift index");
                let is_increment_successful = encapsulated_shifter.try_increment();
                if !is_increment_successful {
                    debug!("increment was not successful, so popping and backing up");
                    if !encapsulated_shifter.try_backward() {
                        debug!("done with shifter, so trying next dependency");
                        // this encapsulated shifter is done, so move onto the next dependency
                        self.current_cell_group_dependency_index = Some(self.current_cell_group_dependency_index.unwrap() + 1);
                        if self.current_cell_group_dependency_index.unwrap() != self.cell_group_dependencies.len() { 
                            let cell_group_dependency = &self.cell_group_dependencies[self.current_cell_group_dependency_index.unwrap()];
                            self.current_elements_total = cell_group_dependency.encapsulated_shifter.borrow().get_length();
                            self.current_states = cell_group_dependency.encapsulated_shifter.borrow().get_states();
                            self.current_states_total = self.current_states.len();
                            let bits_length = self.current_elements_total * self.current_elements_total * self.current_states_total * self.current_states_total;
                            self.current_is_checked.clear();
                            self.current_is_checked = BitVec::repeat(false, bits_length);
                            self.current_is_valid.clear();
                            self.current_is_valid = BitVec::repeat(false, bits_length);
                        }
                        is_fully_backward = true;
                        if self.current_locations.len() != 0 {
                            panic!("Unexpected locations when the next dependency is going to be attempted.");
                        }
                    }
                    else {
                        debug!("moved backwards, so popping value to be replaced");
                        self.current_locations.pop();
                        self.current_element_index_and_state_index_pairs.pop();
                    }
                    is_forward_required = false;
                }
                else {
                    debug!("at a valid shift index, so comparing current indexed elements to cached indexed elements");
                    let mut current_element_index_and_state_index_pair = encapsulated_shifter.get_element_index_and_state_index();
                    current_element_index_and_state_index_pair.0 = cell_group_dependency.cell_group_index_mapping[current_element_index_and_state_index_pair.0];
                    let current_index_element_location = self.current_states[current_element_index_and_state_index_pair.1].clone();

                    let is_current_indexed_element_valid: bool;
                    'is_current_indexed_element_valid: {
                        for location_index in 0..self.current_locations.len() {
                            let other_element_index_and_state_index_pair = &self.current_element_index_and_state_index_pairs[location_index];

                            // check if the pair of indexed elements have already been compared
                            let bitvec_index: usize;
                            {
                                let first_element_index: usize;
                                let second_element_index: usize;
                                let first_state_index: usize;
                                let second_state_index: usize;
                                if other_element_index_and_state_index_pair.0 < current_element_index_and_state_index_pair.0 {
                                    first_element_index = other_element_index_and_state_index_pair.0;
                                    second_element_index = current_element_index_and_state_index_pair.0;
                                    first_state_index = other_element_index_and_state_index_pair.1;
                                    second_state_index = current_element_index_and_state_index_pair.1;
                                }
                                else {
                                    first_element_index = current_element_index_and_state_index_pair.0;
                                    second_element_index = other_element_index_and_state_index_pair.0;
                                    first_state_index = current_element_index_and_state_index_pair.1;
                                    second_state_index = other_element_index_and_state_index_pair.1;
                                }
                                bitvec_index = ((second_state_index * self.current_states_total + first_state_index) * self.current_elements_total + second_element_index) * self.current_elements_total + first_element_index;
                            }

                            if !self.current_is_checked[bitvec_index] {
                                let other_index_element_location = self.current_states[other_element_index_and_state_index_pair.1].clone();

                                // verify that the pair of indexed elements are valid at the same time and location
                                let mut is_current_pair_valid = true;
                                'is_current_pair_valid: {
                                    let other_cell_group = &self.cell_groups[other_element_index_and_state_index_pair.0];
                                    let current_cell_group = &self.cell_groups[current_element_index_and_state_index_pair.0];

                                    // check for overlap
                                    for other_cell in other_cell_group.cells.iter() {
                                        let calculated_other_cell: (u8, u8) = (other_cell.0 + other_index_element_location.0, other_cell.1 + other_index_element_location.1);
                                        for current_cell in current_cell_group.cells.iter() {
                                            let calculated_current_cell: (u8, u8) = (current_cell.0 + current_index_element_location.0, current_cell.1 + current_index_element_location.1);
                                            if calculated_other_cell == calculated_current_cell {
                                                debug!("found overlap at ({}, {})", calculated_current_cell.0, calculated_current_cell.1);
                                                is_current_pair_valid = false;
                                                break 'is_current_pair_valid;
                                            }
                                        }
                                    }

                                    // TODO implement detection cells and adjacency
                                }
                                
                                self.current_is_checked.set(bitvec_index, true);
                                self.current_is_valid.set(bitvec_index, is_current_pair_valid);
                            }
                            if !self.current_is_valid[bitvec_index] {
                                is_current_indexed_element_valid = false;
                                break 'is_current_indexed_element_valid;
                            }
                        }
                        debug!("cell groups are valid together");
                        is_current_indexed_element_valid = true;
                    }
                    if is_current_indexed_element_valid {
                        debug!("indexed elements are valid together, so storing location and moving forward");
                        self.current_locations.push(IndexedElement::new(current_index_element_location, current_element_index_and_state_index_pair.0));
                        self.current_element_index_and_state_index_pairs.push(current_element_index_and_state_index_pair);
                        is_forward_required = true;
                    }
                    else {
                        debug!("indexed elements are invalid together, so incrementing again");
                        is_forward_required = false;
                    }
                }
            }
            if self.current_locations.len() == encapsulated_shifter_length {
                debug!("collected a valid set of current locations");
                return true;
            }
            if self.current_locations.len() != 0 {
                panic!("Unexpected locations still cached in current locations.");
            }
        }
        // if we've gone backwards to the point that there are no longer any locations, we are done
        debug!("no remaining valid sets of locations");
        return false;
    }
    fn get(&self) -> Vec<IndexedElement<(u8, u8)>> {
        return self.current_locations.clone();
    }
    fn reset(&mut self) {
        if self.current_cell_group_dependency_index.is_some() {
            self.cell_group_dependencies[self.current_cell_group_dependency_index.unwrap()].encapsulated_shifter.borrow_mut().reset();
        }
        self.current_cell_group_dependency_index = None;
        self.current_locations.clear();
        self.current_element_index_and_state_index_pairs.clear();
    }
}

impl Iterator for ShiftingCellGroupDependencyIncrementer {
    type Item = Vec<IndexedElement<(u8, u8)>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.try_increment() {
            Some(self.get())
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod shifting_cell_group_dependency_incrementer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeSet};

    use crate::shifter::index_shifter::IndexShifter;

    use super::*;
    use bitvec::{bits, vec::BitVec};
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn one_cell_group_zero_dependencies() {
        init();

        let cell_groups: Vec<CellGroup> = vec![
            CellGroup {
                cell_group_type_index: 0,
                cells: vec![(0, 0)]
            }
        ];
        let cell_group_dependencies: Vec<CellGroupDependency> = vec![];
        let mut shifting_cell_group_dependency_incrementer = ShiftingCellGroupDependencyIncrementer::new(
            cell_groups,
            cell_group_dependencies
        );
        for _ in 0..10 {
            assert!(!shifting_cell_group_dependency_incrementer.try_increment());
        }
    }

    #[rstest]
    fn two_cell_groups_one_dependency() {
        init();

        let cell_groups: Vec<CellGroup> = vec![
            CellGroup {
                cell_group_type_index: 0,
                cells: vec![(0, 0)]
            },
            CellGroup {
                cell_group_type_index: 1,
                cells: vec![(0, 0)]
            }
        ];
        let states_per_shift_index: Vec<Vec<Rc<(u8, u8)>>> = vec![
            vec![
                Rc::new((14, 140)),
                Rc::new((15, 150))
            ],
            vec![
                Rc::new((14, 140)),
                Rc::new((15, 150))
            ]
        ];
        let shifters: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = vec![
            Rc::new(RefCell::new(IndexShifter::new(&states_per_shift_index)))
        ];
        let cell_group_dependencies: Vec<CellGroupDependency> = vec![
            CellGroupDependency {
                cell_group_index_mapping: vec![0, 1],
                encapsulated_shifter: RefCell::new(EncapsulatedShifter::new(&shifters))
            }
        ];
        let mut shifting_cell_group_dependency_incrementer = ShiftingCellGroupDependencyIncrementer::new(
            cell_groups,
            cell_group_dependencies
        );
        let mut expected_get: Vec<IndexedElement<(u8, u8)>>;
        assert!(shifting_cell_group_dependency_incrementer.try_increment());
        expected_get = vec![IndexedElement { index: 0, element: Rc::new((14, 140)) }, IndexedElement { index: 1, element: Rc::new((15, 150)) }];
        assert_eq!(expected_get, shifting_cell_group_dependency_incrementer.get());
        assert!(shifting_cell_group_dependency_incrementer.try_increment());
        expected_get = vec![IndexedElement { index: 0, element: Rc::new((15, 150)) }, IndexedElement { index: 1, element: Rc::new((14, 140)) }];
        assert_eq!(expected_get, shifting_cell_group_dependency_incrementer.get());
        assert!(!shifting_cell_group_dependency_incrementer.try_increment());
    }
    
    #[rstest]
    fn multiple_squares() {
        init();

        time_graph::enable_data_collection(true);

        let mut detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let mut is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let mut cell_group_dependencies: Vec<CellGroupDependency> = Vec::new();

        let cell_groups_total = 5;

        // Stats
        //  3
        //      2022-12-21  0.01s
        //      2022-12-22  0.00s
        //  4
        //      2022-12-21  0.56s
        //      2022-12-22  0.05s
        //      2023-01-06  0.00866s
        //  5
        //      2022-12-21  2339.69s
        //      2022-12-22    28.95s
        //      2023-01-06     3.15s on potato
        //      2023-01-08     0.392s
        //  6
        //      2023-01-08   327.12s

        let mut area_width: usize = 0;
        let mut area_height: usize = 0;

        // calculate the minimum area for holding increasing sizes of squares

        for cell_group_index in 0..cell_groups_total {
            let cell_group_size = cell_group_index + 1;
            if area_width < area_height {
                area_width += cell_group_size;
                if area_height < cell_group_size {
                    area_height = cell_group_size;
                }
            }
            else {
                area_height += cell_group_size;
                if area_width < cell_group_size {
                    area_width = cell_group_size;
                }
            }
        }

        println!("area: ({}, {})", area_width, area_height);

        // construct cell groups

        let mut cell_groups: Vec<CellGroup> = Vec::new();
        for index in 0..cell_groups_total {
            let mut cells: Vec<(u8, u8)> = Vec::new();
            for width_index in 0..=index as u8 {
                for height_index in 0..=index as u8 {
                    cells.push((width_index, height_index));
                }
            }
            cell_groups.push(CellGroup {
                cell_group_type_index: 0,
                cells: cells
            });
        }

        detection_offsets_per_cell_group_type_index_per_cell_group_type_index.push(vec![vec![]]);

        // construct adjacency bitvec
        {
            for _ in 0..cell_groups_total {
                let mut is_adjacent_cell_group_index: BitVec = BitVec::new();
                for _ in 0..cell_groups_total {
                    is_adjacent_cell_group_index.push(false);
                }
                is_adjacent_cell_group_index_per_cell_group_index.push(is_adjacent_cell_group_index);
            }
        }

        {
            // construct index incrementer for looping over locations per cell group

            let mut cell_group_index_mapping: Vec<usize> = Vec::new();
            for cell_group_index in 0..cell_groups_total {
                cell_group_index_mapping.push(cell_group_index);
            }
    
            let mut shifters: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = Vec::new();
            for cell_group_index in 0..cell_groups_total {
                let cell_group_size = cell_group_index + 1;
                let mut locations: Vec<Rc<(u8, u8)>> = Vec::new();
                for height_index in 0..(area_height - (cell_group_size - 1)) as u8 {
                    for width_index in 0..(area_width - (cell_group_size - 1)) as u8 {
                        let location = (width_index, height_index);
                        debug!("cell group {:?} can exist at location {:?}", cell_group_index, location);
                        locations.push(Rc::new(location));
                    }
                }
                shifters.push(Rc::new(RefCell::new(IndexShifter::new(&vec![locations]))));
            }
            
            cell_group_dependencies.push(CellGroupDependency {
                cell_group_index_mapping: cell_group_index_mapping,
                encapsulated_shifter: RefCell::new(EncapsulatedShifter::new(&shifters))
            });
        }

        let shifting_cell_group_dependency_incrementer = ShiftingCellGroupDependencyIncrementer::new(cell_groups.clone(), cell_group_dependencies);

        let validating_start_time = Instant::now();

        let indexed_elements_collection = shifting_cell_group_dependency_incrementer.into_iter().collect::<Vec<Vec<IndexedElement<(u8, u8)>>>>();

        println!("validation time: {:?}", validating_start_time.elapsed());
        println!("validated: {:?}", indexed_elements_collection.len());

        println!("{}", time_graph::get_full_graph().as_dot());

        // https://en.wikipedia.org/wiki/Square_pyramidal_number
        let cells_total: usize = (cell_groups_total * (cell_groups_total + 1) * (2 * cell_groups_total + 1)) / 6;
        let mut permutations: BTreeSet<Vec<Vec<String>>> = BTreeSet::new();

        for indexed_elements in indexed_elements_collection.iter() {
            let mut pixels: Vec<Vec<bool>> = Vec::new();
            let mut pixels_as_ids: Vec<Vec<String>> = Vec::new();
            for _ in 0..area_width {
                let mut pixel_column: Vec<bool> = Vec::new();
                let mut pixel_as_id_column: Vec<String> = Vec::new();
                for _ in 0..area_height {
                    pixel_column.push(false);
                    pixel_as_id_column.push(String::from(" "));
                }
                pixels.push(pixel_column);
                pixels_as_ids.push(pixel_as_id_column);
            }
            let mut valid_pixels_total: usize = 0;
            for indexed_element in indexed_elements.iter() {
                for cell in cell_groups[indexed_element.index].cells.iter() {
                    let width_index = (cell.0 + indexed_element.element.0) as usize;
                    let height_index = (cell.1 + indexed_element.element.1) as usize;
                    if !pixels[width_index][height_index] {
                        valid_pixels_total += 1;
                    }
                    pixels[width_index][height_index] = true;
                    pixels_as_ids[width_index][height_index] = indexed_element.index.to_string();
                }
            }

            let is_printed: bool;
            if false {
                is_printed = true;
            }
            else {
                is_printed = false;
            }

            for height_index in 0..area_height {
                for width_index in 0..area_width {
                    if is_printed {
                        print!("{}", pixels_as_ids[width_index][height_index]);
                    }
                }
                if is_printed {
                    println!("");
                }
            }
            if is_printed {
                println!("");
            }

            assert_eq!(valid_pixels_total, cells_total);
            permutations.insert(pixels_as_ids);
        }

        println!("permutations: {}", permutations.len());

        let expected_permutations_total: usize = match cell_groups_total {
            2 => 4,
            3 => 8,
            4 => 96,
            5 => 6400,
            _ => {
                panic!("Unexpected number of cell groups: {}", cell_groups_total);
            }
        };
        assert_eq!(expected_permutations_total, permutations.len());
    }
}