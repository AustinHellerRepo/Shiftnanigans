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

pub struct ShiftingCellGroupDependencyIncrementer {
    encapsulated_shifter: EncapsulatedShifter<(i32, i32)>,
    current_locations: Vec<IndexedElement<(i32, i32)>>
}

impl ShiftingCellGroupDependencyIncrementer {
    pub fn new(encapsulated_shifter: EncapsulatedShifter<(i32, i32)>) -> Self {
        ShiftingCellGroupDependencyIncrementer {
            encapsulated_shifter: encapsulated_shifter,
            current_locations: Vec::new()
        }
    }
    fn is_cell_group_pair_valid(&self, from_cell_group_indexed_element: &IndexedElement<(i32, i32)>, to_cell_group_indexed_element: &IndexedElement<(i32, i32)>) -> bool {
        todo!("implement code that compares the two cell groups");
    }
}

// TODO implement Incrementer

impl Incrementer for ShiftingCellGroupDependencyIncrementer {
    type T = (i32, i32);

    fn try_increment(&mut self) -> bool {
        if self.current_locations.len() == 0 {
            // the main algorithm requires that there exist at least one element in the current locations to compare to
            let is_at_least_one_element = self.encapsulated_shifter.try_forward();
            // if the shifter does not contain any elements, then it cannot increment
            if !is_at_least_one_element {
                return false;
            }
            // if the shifter could move forward but not increment, this indicates a bug
            if !self.encapsulated_shifter.try_increment() {
                panic!("Unexpected forward but not increment.");
            }
            let indexed_element = self.encapsulated_shifter.get();
            self.current_locations.push(indexed_element);
        }
        // loop until a valid collection of locations has been discovered
        let mut is_forward_required: bool = true;
        while self.current_locations.len() != self.encapsulated_shifter.length() && self.current_locations.len() != 0 {
            if is_forward_required {
                self.encapsulated_shifter.try_forward();
            }
            let is_increment_successful = self.encapsulated_shifter.try_increment();
            if !is_increment_successful {
                self.encapsulated_shifter.try_backward();
                self.current_locations.pop();
                is_forward_required = false;
            }
            else {
                let current_indexed_element = self.encapsulated_shifter.get();

                let is_current_indexed_element_valid: bool;
                'is_current_indexed_element_valid: {
                    for location_index in 0..self.current_locations.len() {
                        let from_cell_group_indexed_element = &self.current_locations[location_index];
                        if !self.is_cell_group_pair_valid(from_cell_group_indexed_element, &current_indexed_element) {
                            is_current_indexed_element_valid = false;
                            break 'is_current_indexed_element_valid;
                        }
                    }
                    is_current_indexed_element_valid = true;
                }
                if is_current_indexed_element_valid {
                    self.current_locations.push(current_indexed_element);
                    is_forward_required = true;
                }
                else {
                    is_forward_required = false;
                }
            }
        }
        // if we've gone backwards to the point that there are no longer any locations, we are done
        return self.current_locations.len() != 0;
    }
    fn get(&self) -> Vec<IndexedElement<(i32, i32)>> {
        return self.current_locations.clone();
    }
    fn reset(&mut self) {
        self.encapsulated_shifter.reset();
        self.current_locations.clear();
    }
}