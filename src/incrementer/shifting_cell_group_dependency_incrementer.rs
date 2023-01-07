// Purpose:
//      This struct will shift forward, always checking the new element state (the cell group location) against the existing element states from previous shift indexes (previous cell groups)
//      The idea is that once a bad pair of located cell groups are found, the current shifter can be pushed forward, skipping over any need to iterate over later shift indexes
//          A bonus is to store the earlier shift index, the earlier state (location), the current shift index, and the current state (location) so that when this pair is found again the current shift index can be incremented without having to do any real calculation
//              Implementation detail: maintain a Vec<Option<BTreeSet<TElement>>> where it is initialized to None for each shift index and the BTreeSet contains the ever-growing temp collection of bad states for that specific shift index
//                  It can be set back to None as each index is incremented across (from shift index 0 to n as each shift index state is found to be valid) since there's no need to look back
//                  It is filled from a master collection per shift index and state key of vectors of BTreeSets, filled as new bad pairs are discovered.

use std::{collections::VecDeque, rc::Rc};

use crate::{shifter::{encapsulated_shifter::EncapsulatedShifter, Shifter}, IndexedElement};
use super::Incrementer;

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
    encapsulated_shifter: EncapsulatedShifter<(u8, u8)>
}

impl CellGroupDependency {
    fn new(cell_group_index_mapping: Vec<usize>, encapsulated_shifter: EncapsulatedShifter<(u8, u8)>) -> Self {
        CellGroupDependency {
            cell_group_index_mapping: cell_group_index_mapping,
            encapsulated_shifter: encapsulated_shifter
        }
    }
}

pub struct ShiftingCellGroupDependencyIncrementer {
    cell_groups: Vec<CellGroup>,
    cell_group_dependencies: Vec<CellGroupDependency>,
    current_cell_group_dependency_index: Option<usize>,
    current_locations: Vec<IndexedElement<(u8, u8)>>
}

impl ShiftingCellGroupDependencyIncrementer {
    pub fn new(cell_groups: Vec<CellGroup>, cell_group_dependencies: Vec<CellGroupDependency>) -> Self {
        ShiftingCellGroupDependencyIncrementer {
            cell_groups: cell_groups,
            cell_group_dependencies: cell_group_dependencies,
            current_cell_group_dependency_index: None,
            current_locations: Vec::new()
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
        }
        while self.current_cell_group_dependency_index.unwrap() != self.cell_group_dependencies.len() {
            debug!("choosing {:?}th dependency", self.current_cell_group_dependency_index);
            let cell_group_dependency = &mut self.cell_group_dependencies[self.current_cell_group_dependency_index.unwrap()];
            let encapsulated_shifter: &mut EncapsulatedShifter<(u8, u8)> = &mut cell_group_dependency.encapsulated_shifter;
            let encapsulated_shifter_length = encapsulated_shifter.length();
            if self.current_locations.len() == 0 {
                debug!("starting off algorithm with first location from shifter");
                // the main algorithm requires that there exist at least one element in the current locations to compare to
                let is_at_least_one_element = encapsulated_shifter.try_forward();
                // if the shifter does not contain any elements, then it cannot increment
                if !is_at_least_one_element {
                    return false;
                }
                // if the shifter could move forward but not increment, this indicates a bug
                if !encapsulated_shifter.try_increment() {
                    panic!("Unexpected forward but not increment.");
                }
                let indexed_element = encapsulated_shifter.get();
                self.current_locations.push(indexed_element);
            }
            // loop until a valid collection of locations has been discovered
            let mut is_forward_required: bool;
            if self.current_locations.len() == encapsulated_shifter_length {
                debug!("determined that forward is not required");
                if encapsulated_shifter_length != 1 {
                    debug!("popping last location to make room for next possible location");
                    self.current_locations.pop();  // remove the last valid location
                }
                is_forward_required = false;
            }
            else {
                debug!("determined that forward is required");
                is_forward_required = true;
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
                        is_fully_backward = true;
                        if self.current_locations.len() != 0 {
                            panic!("Unexpected locations when the next dependency is going to be attempted.");
                        }
                    }
                    else {
                        debug!("moved backwards, so popping value to be replaced");
                        self.current_locations.pop();
                    }
                    is_forward_required = false;
                }
                else {
                    debug!("at a valid shift index, so comparing current indexed elements to cached indexed elements");
                    let mut current_indexed_element = encapsulated_shifter.get();
                    current_indexed_element.index = cell_group_dependency.cell_group_index_mapping[current_indexed_element.index];

                    let is_current_indexed_element_valid: bool;
                    'is_current_indexed_element_valid: {
                        for location_index in 0..self.current_locations.len() {
                            let other_indexed_element = &self.current_locations[location_index];
                            // verify that the pair of indexed elements are valid at the same time and location
                            {
                                let other_cell_group = &self.cell_groups[other_indexed_element.index];
                                let current_cell_group = &self.cell_groups[current_indexed_element.index];

                                // check for overlap
                                for other_cell in other_cell_group.cells.iter() {
                                    let calculated_other_cell: (u8, u8) = (other_cell.0 + other_indexed_element.element.0, other_cell.1 + other_indexed_element.element.1);
                                    for current_cell in current_cell_group.cells.iter() {
                                        let calculated_current_cell: (u8, u8) = (current_cell.0 + current_indexed_element.element.0, current_cell.1 + current_indexed_element.element.1);
                                        if calculated_other_cell == calculated_current_cell {
                                            debug!("found overlap at ({}, {})", calculated_current_cell.0, calculated_current_cell.1);
                                            is_current_indexed_element_valid = false;
                                            break 'is_current_indexed_element_valid;
                                        }
                                    }
                                }

                                // TODO implement detection cells and adjacency
                            }
                        }
                        debug!("cell groups are valid together");
                        is_current_indexed_element_valid = true;
                    }
                    if is_current_indexed_element_valid {
                        debug!("indexed elements are valid together, so storing location and moving forward");
                        self.current_locations.push(current_indexed_element);
                        is_forward_required = true;
                    }
                    else {
                        debug!("indexed elements are invalid together, so incrementing again");
                        is_forward_required = false;
                    }
                }
            }
            if self.current_locations.len() != 0 {
                debug!("collected a valid set of current locations");
                return true;
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
            self.cell_group_dependencies[self.current_cell_group_dependency_index.unwrap()].encapsulated_shifter.reset();
        }
        self.current_cell_group_dependency_index = None;
        self.current_locations.clear();
    }
}

#[cfg(test)]
mod shifting_cell_group_dependency_incrementer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use crate::shifter::index_shifter::IndexShifter;

    use super::*;
    use bitvec::bits;
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::try_init();
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
                encapsulated_shifter: EncapsulatedShifter::new(&shifters)
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
        assert!(shifting_cell_group_dependency_incrementer.try_increment());
        expected_get = vec![IndexedElement { index: 1, element: Rc::new((14, 140)) }, IndexedElement { index: 0, element: Rc::new((15, 150)) }];
        assert_eq!(expected_get, shifting_cell_group_dependency_incrementer.get());
        assert!(shifting_cell_group_dependency_incrementer.try_increment());
        expected_get = vec![IndexedElement { index: 1, element: Rc::new((15, 150)) }, IndexedElement { index: 0, element: Rc::new((14, 140)) }];
        assert_eq!(expected_get, shifting_cell_group_dependency_incrementer.get());
        assert!(!shifting_cell_group_dependency_incrementer.try_increment());
    }
}