use std::{collections::{HashMap, HashSet}, hash::Hash, marker::PhantomData};

use crate::index_incrementer::{self, IndexIncrementer};

#[derive(Clone, Debug)]
pub struct CellGroup<TCellGroupIdentifier, TCellGroupType> {
    id: TCellGroupIdentifier,
    cells: Vec<(i32, i32)>,  // these should exist such that they can be added directly to location points
    cell_group_type: TCellGroupType  // each type can have relationship attributes (detection location offsets, etc.)
}

/// This struct contains a specific arrangement of cell groups, each location specified per cell group
#[derive(Clone, Debug)]
pub struct CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier> {
    id: TCellGroupLocationCollectionIdentifier,
    location_per_cell_group_id: HashMap<TCellGroupIdentifier, (i32, i32)>
}

/// This struct specifies that "this" cell group location has "these" cell group location collections as dependencies such that if being at that location makes all of them invalid, then that location must be invalid
#[derive(Clone, Debug)]
pub struct CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier> {
    cell_group_id: TCellGroupIdentifier,
    location: (i32, i32),
    cell_group_location_collections: Vec<TCellGroupLocationCollectionIdentifier>
}

#[derive(Clone, Debug)]
pub struct AnonymousCellGroupLocationCollection<TCellGroupIdentifier> {
    location_per_cell_group_id: HashSet<TCellGroupIdentifier, (i32, i32)>
}

#[derive(Clone, Debug)]
pub struct CellGroupManager<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier, TCellGroupType> {
    cell_group_per_cell_group_id: HashMap<TCellGroupIdentifier, CellGroup<TCellGroupIdentifier, TCellGroupType>>,
    cell_group_location_collection_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>>,
    cell_group_location_dependencies_per_cell_group_id: HashMap<TCellGroupIdentifier, Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>>>,
    detection_locations_per_cell_group_type_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashMap<TCellGroupType, HashSet<(i32, i32)>>>>,
    adjacent_cell_group_ids_per_cell_group_id: HashMap<TCellGroupIdentifier, HashSet<TCellGroupIdentifier>>,
    overlap_locations_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashSet<(i32, i32)>>>,
    located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashMap<(TCellGroupIdentifier, TCellGroupType, (i32, i32)), Vec<(i32, i32)>>>
}

// TODO make detection specific to a pair of cell_group_types since wall-adjacents can be within range of a wall
// TODO if all dependent cell group location collections are invalid for a specific cell group location, then that cell group location is invalid

impl<TCellGroupLocationCollectionIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupType: Hash + Eq + std::fmt::Debug + Clone> CellGroupManager<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier, TCellGroupType> {
    pub fn new(
        cell_groups: Vec<CellGroup<TCellGroupIdentifier, TCellGroupType>>,
        cell_group_location_collections: Vec<CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>>,
        detection_offsets_per_cell_group_type_pair: HashMap<(TCellGroupType, TCellGroupType), Vec<(i32, i32)>>,
        adjacent_cell_group_id_pairs: Vec<(TCellGroupIdentifier, TCellGroupIdentifier)>,
        cell_group_location_dependencies: Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>>
    ) -> Self {

        // create cell group lookup hashmap

        let mut cell_group_per_cell_group_id: HashMap<TCellGroupIdentifier, CellGroup<TCellGroupIdentifier, TCellGroupType>> = HashMap::new();

        {
            for cell_group in cell_groups.into_iter() {
                cell_group_per_cell_group_id.insert(cell_group.id.clone(), cell_group);
            }
        }

        // create cell group location collection lookup hashmap

        let mut cell_group_location_collection_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>> = HashMap::new();

        {
            for cell_group_location_collection in cell_group_location_collections.into_iter() {
                cell_group_location_collection_per_cell_group_location_collection_id.insert(cell_group_location_collection.id.clone(), cell_group_location_collection);
            }
        }

        // construct adjacent cell group cache nested hashmap

        let mut adjacent_cell_group_ids_per_cell_group_id: HashMap<TCellGroupIdentifier, HashSet<TCellGroupIdentifier>> = HashMap::new();

        {
            for adjacent_cell_group_id_pair in adjacent_cell_group_id_pairs.iter() {
                for (from_cell_group_id, to_cell_group_id) in [(adjacent_cell_group_id_pair.0.clone(), adjacent_cell_group_id_pair.1.clone()), (adjacent_cell_group_id_pair.1.clone(), adjacent_cell_group_id_pair.0.clone())] {
                    if !adjacent_cell_group_ids_per_cell_group_id.contains_key(&from_cell_group_id) {
                        adjacent_cell_group_ids_per_cell_group_id.insert(from_cell_group_id.clone(), HashSet::new());
                    }
                    adjacent_cell_group_ids_per_cell_group_id.get_mut(&from_cell_group_id).unwrap().insert(to_cell_group_id);
                }
            }

            // create an empty hashset for any cell groups that do not have an adjacency dependency
            for cell_group_id in cell_group_per_cell_group_id.keys() {
                if !adjacent_cell_group_ids_per_cell_group_id.contains_key(cell_group_id) {
                    adjacent_cell_group_ids_per_cell_group_id.insert(cell_group_id.clone(), HashSet::new());
                }
            }
        }

        // construct detection cell groups from provided cell groups

        let mut detection_locations_per_cell_group_type_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashMap<TCellGroupType, HashSet<(i32, i32)>>>> = HashMap::new();

        {
            // construct detection cell cache
            let mut detection_cells_per_cell_group_type_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<TCellGroupType, Vec<(i32, i32)>>> = HashMap::new();

            {
                // construct detection cache nested hashmap
                let mut detection_offsets_per_cell_group_type_per_cell_group_type: HashMap<TCellGroupType, HashMap<TCellGroupType, Vec<(i32, i32)>>> = HashMap::new();

                {
                    for (cell_group_type_pair, detection_offsets) in detection_offsets_per_cell_group_type_pair.iter() {
                        for (from_cell_group_type, to_cell_group_type) in [(&cell_group_type_pair.0, &cell_group_type_pair.1), (&cell_group_type_pair.1, &cell_group_type_pair.0)] {
                            if !detection_offsets_per_cell_group_type_per_cell_group_type.contains_key(from_cell_group_type) {
                                detection_offsets_per_cell_group_type_per_cell_group_type.insert(from_cell_group_type.clone(), HashMap::new());
                            }
                            if detection_offsets_per_cell_group_type_per_cell_group_type.get(from_cell_group_type).unwrap().contains_key(to_cell_group_type) {
                                panic!("Found duplicate detection offset cell group type pair ({:?}, {:?})", from_cell_group_type, to_cell_group_type);
                            }
                            detection_offsets_per_cell_group_type_per_cell_group_type.get_mut(from_cell_group_type).unwrap().insert(to_cell_group_type.clone(), detection_offsets.clone());
                        }
                    }
                }

                for cell_group in cell_group_per_cell_group_id.values() {

                    if detection_offsets_per_cell_group_type_per_cell_group_type.contains_key(&cell_group.cell_group_type) {
                        // the cell group type of the current cell group is restrictive to at least one other cell group type

                        for (cell_group_type, detection_offsets) in detection_offsets_per_cell_group_type_per_cell_group_type.get(&cell_group.cell_group_type).unwrap() {

                            // construct detection cells

                            let mut detection_cells: Vec<(i32, i32)> = Vec::new();

                            {
                                let mut traveled_cells: HashSet<(i32, i32)> = HashSet::new();
                                for cell in cell_group.cells.iter() {
                                    if !traveled_cells.contains(cell) {
                                        traveled_cells.insert(cell.to_owned());
                                        detection_cells.push(cell.to_owned());
                                    }
                                    for detection_offset in detection_offsets.iter() {
                                        let potential_detection_cell = (cell.0 + detection_offset.0, cell.1 + detection_offset.1);
                                        if !traveled_cells.contains(&potential_detection_cell) {
                                            traveled_cells.insert(potential_detection_cell.clone());
                                            detection_cells.push(potential_detection_cell);
                                        }
                                    }
                                }
                            }

                            if !detection_cells_per_cell_group_type_per_cell_group_id.contains_key(&cell_group.id) {
                                detection_cells_per_cell_group_type_per_cell_group_id.insert(cell_group.id.clone(), HashMap::new());
                            }
                            if detection_cells_per_cell_group_type_per_cell_group_id.get(&cell_group.id).unwrap().contains_key(cell_group_type) {
                                panic!("Unexpected duplicate cell group type {:?} for detection cells of cell group {:?}.", cell_group_type, cell_group.id);
                            }
                            detection_cells_per_cell_group_type_per_cell_group_id.get_mut(&cell_group.id).unwrap().insert(cell_group_type.clone(), detection_cells);
                        }
                    }
                }
            }

            // iterate over every location each cell group could exist at for each cell group type it may encounter in a dependency

            for cell_group_location_dependency in cell_group_location_dependencies.iter() {
                if !detection_locations_per_cell_group_type_per_location_per_cell_group_id.contains_key(&cell_group_location_dependency.cell_group_id) {
                    detection_locations_per_cell_group_type_per_location_per_cell_group_id.insert(cell_group_location_dependency.cell_group_id.clone(), HashMap::new());
                }
                if !detection_locations_per_cell_group_type_per_location_per_cell_group_id.get(&cell_group_location_dependency.cell_group_id).unwrap().contains_key(&cell_group_location_dependency.location) {
                    detection_locations_per_cell_group_type_per_location_per_cell_group_id.get_mut(&cell_group_location_dependency.cell_group_id).unwrap().insert(cell_group_location_dependency.location.clone(), HashMap::new());
                }
                for dependent_cell_group_location_collection_id in cell_group_location_dependency.cell_group_location_collections.iter() {
                    for cell_group_id in cell_group_location_collection_per_cell_group_location_collection_id.get(dependent_cell_group_location_collection_id).unwrap().location_per_cell_group_id.keys() {
                        let dependent_cell_group = cell_group_per_cell_group_id.get(cell_group_id).unwrap();
                        if !detection_locations_per_cell_group_type_per_location_per_cell_group_id.get(&cell_group_location_dependency.cell_group_id).unwrap().get(&cell_group_location_dependency.location).unwrap().contains_key(&dependent_cell_group.cell_group_type) {
                            let mut detection_locations: HashSet<(i32, i32)> = HashSet::new();

                            // calculate detection locations for this location and cell group type
                            if detection_cells_per_cell_group_type_per_cell_group_id.contains_key(&cell_group_location_dependency.cell_group_id) &&
                                detection_cells_per_cell_group_type_per_cell_group_id.get(&cell_group_location_dependency.cell_group_id).unwrap().contains_key(&dependent_cell_group.cell_group_type) {

                                for detection_cell in detection_cells_per_cell_group_type_per_cell_group_id.get(&cell_group_location_dependency.cell_group_id).unwrap().get(&dependent_cell_group.cell_group_type).unwrap().iter() {
                                    let detection_location = (cell_group_location_dependency.location.0 + detection_cell.0, cell_group_location_dependency.location.1 + detection_cell.1);
                                    detection_locations.insert(detection_location);
                                }
                            }

                            detection_locations_per_cell_group_type_per_location_per_cell_group_id.get_mut(&cell_group_location_dependency.cell_group_id).unwrap().get_mut(&cell_group_location_dependency.location).unwrap().insert(dependent_cell_group.cell_group_type.clone(), detection_locations);
                        }
                    }
                }
            }
        }

        // construct cell group location dependency lookup hashmap

        let mut cell_group_location_dependencies_per_cell_group_id: HashMap<TCellGroupIdentifier, Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>>> = HashMap::new();

        {
            for cell_group_location_dependency in cell_group_location_dependencies.into_iter() {
                if !cell_group_location_dependencies_per_cell_group_id.contains_key(&cell_group_location_dependency.cell_group_id) {
                    cell_group_location_dependencies_per_cell_group_id.insert(cell_group_location_dependency.cell_group_id.clone(), Vec::new());
                }
                cell_group_location_dependencies_per_cell_group_id.get_mut(&cell_group_location_dependency.cell_group_id).unwrap().push(cell_group_location_dependency);
            }
        }

        // construct overlap locations for each possible location that each cell group could exist at (based on the dependencies)

        let mut overlap_locations_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashSet<(i32, i32)>>> = HashMap::new();

        {
            for cell_group in cell_group_per_cell_group_id.values() {
                overlap_locations_per_location_per_cell_group_id.insert(cell_group.id.clone(), HashMap::new());

                let cell_group_location_dependencies: &Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>> = cell_group_location_dependencies_per_cell_group_id.get(&cell_group.id).unwrap();
                for cell_group_location_dependency in cell_group_location_dependencies.iter() {
                    if !overlap_locations_per_location_per_cell_group_id.get(&cell_group.id).unwrap().contains_key(&cell_group_location_dependency.location) {
                        // this is the first time this cell group is known to exist at this location (but there may be more instances given different dependency relationships)
                        let mut overlap_locations: HashSet<(i32, i32)> = HashSet::new();
                        for cell in cell_group.cells.iter() {
                            overlap_locations.insert((cell.0 + cell_group_location_dependency.location.0, cell.1 + cell_group_location_dependency.location.1));
                        }
                        overlap_locations_per_location_per_cell_group_id.get_mut(&cell_group.id).unwrap().insert(cell_group_location_dependency.location.clone(), overlap_locations);
                    }
                }
            }
        }

        // construct located cells per cell group type per cell group location collection

        let mut located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashMap<(TCellGroupIdentifier, TCellGroupType, (i32, i32)), Vec<(i32, i32)>>> = HashMap::new();

        for (cell_group_location_collection_id, cell_group_location_collection) in cell_group_location_collection_per_cell_group_location_collection_id.iter() {
            located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.insert(cell_group_location_collection_id.clone(), HashMap::new());
            for (cell_group_id, location) in cell_group_location_collection.location_per_cell_group_id.iter() {
                let cell_group = cell_group_per_cell_group_id.get(cell_group_id).unwrap();
                let cell_group_id_and_cell_group_type_and_location_tuple = (cell_group.id.clone(), cell_group.cell_group_type.clone(), location.clone());
                if !located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap().contains_key(&cell_group_id_and_cell_group_type_and_location_tuple) {
                    located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get_mut(cell_group_location_collection_id).unwrap().insert(cell_group_id_and_cell_group_type_and_location_tuple.clone(), Vec::new());
                }

                // append this cell group's located cells
                    
                for cell in cell_group.cells.iter() {
                    let located_cell = (location.0 + cell.0, location.1 + cell.1);
                    located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get_mut(cell_group_location_collection_id).unwrap().get_mut(&cell_group_id_and_cell_group_type_and_location_tuple).unwrap().push(located_cell);
                }
            }
        }

        // TODO remove any cell group location collections where the cell groups within the cell group location collection violate the overlap, stated adjacency, and stated detection logic



        CellGroupManager {
            cell_group_per_cell_group_id,
            cell_group_location_collection_per_cell_group_location_collection_id: cell_group_location_collection_per_cell_group_location_collection_id,
            cell_group_location_dependencies_per_cell_group_id: cell_group_location_dependencies_per_cell_group_id,
            detection_locations_per_cell_group_type_per_location_per_cell_group_id: detection_locations_per_cell_group_type_per_location_per_cell_group_id,
            adjacent_cell_group_ids_per_cell_group_id: adjacent_cell_group_ids_per_cell_group_id,
            overlap_locations_per_location_per_cell_group_id: overlap_locations_per_location_per_cell_group_id,
            located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id: located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id
        }
    }
    /// This function will determine which permitted locations for this cell group are actually possible while iterating over all possible locations for the known dependent cell group
    /// Returns true if at least one cell group location dependency was removed from at least one of the known dependent cell group location dependencies
    fn try_reduce_cell_group_location_dependency_for_cell_group(&mut self, cell_group_id: &TCellGroupIdentifier) -> bool {

        // TODO refactor to simply loop over all cell_group_location_dependencies since the need to use IndexIncrementer would have been when the cell group location collections were being constructed
        // increment over the possible cell group location collections per cell group location collection set, checking that none of the cells in any other the cell group locations overlap with a detection location

        // load cached overlap locations per location
        let overlap_locations_per_location = self.overlap_locations_per_location_per_cell_group_id.get(cell_group_id).unwrap();

        // load cached detection locations per cell group type per location
        let detection_locations_per_cell_group_type_per_location = self.detection_locations_per_cell_group_type_per_location_per_cell_group_id.get(cell_group_id).unwrap();

        // load expected adjacent cell group IDs
        let expected_adjacent_cell_group_ids = self.adjacent_cell_group_ids_per_cell_group_id.get(cell_group_id).unwrap();

        // collect invalid pairs of cell groups for when being at their respective locations never produces a valid combination
        let mut invalid_cell_group_and_location_tuple_per_location: HashMap<(i32, i32), Vec<(&TCellGroupIdentifier, &(i32, i32))>> = HashMap::new();

        // collect the cell group location dependencies that fully invalidate their cell group location collections (since that would mean there is no valid state for this dependency)
        let mut invalid_cell_group_location_dependency_indexes: Vec<usize> = Vec::new();

        for (cell_group_location_dependency_index, cell_group_location_dependency) in self.cell_group_location_dependencies_per_cell_group_id.get(cell_group_id).unwrap().iter().enumerate() {

            // load cached overlap locations
            let overlap_locations = overlap_locations_per_location.get(&cell_group_location_dependency.location).unwrap();

            // load cached detection locations per cell group type
            let detection_locations_per_cell_group_type = detection_locations_per_cell_group_type_per_location.get(&cell_group_location_dependency.location).unwrap();

            // if no cell group location collections are possible at this location, then this entire cell group location dependency is invalid (as opposed to the cell group location collections being invalid)
            let mut is_at_least_one_cell_group_location_collection_possible: bool = cell_group_location_dependency.cell_group_location_collections.is_empty();  // do not get rid of this dependency if there are no actual dependencies

            for cell_group_location_collection_id in cell_group_location_dependency.cell_group_location_collections.iter() {

                // assume that the cell group location collection is valid until at least one cell group is found to be invalid
                let mut is_valid_cell_group_location_collection: bool = true;

                for ((cell_group_id, cell_group_type, location), located_cells) in self.located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap().iter() {
                    
                    let is_adjacency_expected = expected_adjacent_cell_group_ids.contains(cell_group_id);
                    let mut is_adjacent = false;
                    let mut is_valid_cell_group = true;

                    // check to see that the located_cells do not exist in the overlap locations
                    for located_cell in located_cells.iter() {
                        if overlap_locations.contains(located_cell) ||
                            detection_locations_per_cell_group_type.get(cell_group_type).unwrap().contains(located_cell) {

                            is_valid_cell_group = false;
                            break;
                        }
                        if is_adjacency_expected {
                            if overlap_locations.contains(&(located_cell.0 - 1, located_cell.1)) ||
                                overlap_locations.contains(&(located_cell.0 + 1, located_cell.1)) ||
                                overlap_locations.contains(&(located_cell.0, located_cell.1 - 1)) ||
                                overlap_locations.contains(&(located_cell.0, located_cell.1 + 1)) {

                                is_adjacent = true;
                            }
                        }
                    }

                    if is_adjacency_expected && !is_adjacent {
                        is_valid_cell_group = false;
                    }

                    if !is_valid_cell_group {
                        is_valid_cell_group_location_collection = false;
                        
                        // store that this cell group at this location is invalid for the current cell group at its location
                        if !invalid_cell_group_and_location_tuple_per_location.contains_key(&cell_group_location_dependency.location) {
                            invalid_cell_group_and_location_tuple_per_location.insert(cell_group_location_dependency.location, Vec::new());
                        }
                        invalid_cell_group_and_location_tuple_per_location.get_mut(&cell_group_location_dependency.location).unwrap().push((cell_group_id, location));

                        // TODO consider if breaking out here is best
                    }
                }

                if is_valid_cell_group_location_collection {
                    is_at_least_one_cell_group_location_collection_possible = true;
                }
            }

            if !is_at_least_one_cell_group_location_collection_possible {
                invalid_cell_group_location_dependency_indexes.push(cell_group_location_dependency_index);
            }
        }

        // remove each invalid cell group location dependency since the current cell group at its location fails to satisfy any of the provided cell group location collections in the dependency

        // TODO

        // remove any invalid_cell_group_and_location_tuple_per_location for this cell group since the combinations of the two will always lead to invalid results

        // TODO

        !invalid_cell_group_location_dependency_indexes.is_empty() || !invalid_cell_group_and_location_tuple_per_location.is_empty()
    }
    pub fn get_validated_cell_group_location_dependencies(&mut self) -> Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>> {

        // cache cell group IDs
        let cell_group_ids: Vec<TCellGroupIdentifier> = self.cell_group_per_cell_group_id.keys().cloned().collect();

        let mut is_at_least_one_cell_group_location_dependency_reduced = true;
        while is_at_least_one_cell_group_location_dependency_reduced {
            is_at_least_one_cell_group_location_dependency_reduced = false;

            // TODO consider if there is an ideal way to sort the cell group location collection IDs based on alterations

            for cell_group_id in cell_group_ids.iter() {
                let is_cell_group_location_dependency_reduced = self.try_reduce_cell_group_location_dependency_for_cell_group(cell_group_id);
                if is_cell_group_location_dependency_reduced {
                    is_at_least_one_cell_group_location_dependency_reduced = true;
                }
            }
        }

        // at this point the existing dependent cell group location collection sets per cell group location collection are the only valid combinations

        let mut validated_cell_group_location_dependencies: Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>> = Vec::new();
        for cell_group_location_dependencies in self.cell_group_location_dependencies_per_cell_group_id.values() {
            let cloned_cell_group_location_dependencies = cell_group_location_dependencies.clone();
            validated_cell_group_location_dependencies.extend(cloned_cell_group_location_dependencies);
        }
        validated_cell_group_location_dependencies
    }
}

#[cfg(test)]
mod cell_group_manager_tests {
    use super::*;
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialize() {
        init();

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {}

        let cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();
        let cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();
        let _ = CellGroupManager::new(cell_groups, cell_group_location_collections, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs, cell_group_location_dependencies);
    }

    #[rstest]
    #[should_panic]
    fn one_cell_group_zero_dependencies_initialized() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Main
        }

        let mut cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();
        let cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        cell_groups.push(CellGroup {
            id: String::from("cell_group_0"),
            cell_group_type: CellGroupType::Main,
            cells: vec![(0, 0)]
        });

        let _ = CellGroupManager::new(cell_groups, cell_group_location_collections, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs, cell_group_location_dependencies);
    }

    #[rstest]
    fn one_cell_group_one_dependency_validated() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Main
        }

        let mut cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        cell_groups.push(CellGroup {
            id: String::from("cell_group_0"),
            cell_group_type: CellGroupType::Main,
            cells: vec![(0, 0)]
        });

        cell_group_location_dependencies.push(CellGroupLocationDependency {
            cell_group_id: String::from("cell_group_0"),
            location: (1, 2),
            cell_group_location_collections: Vec::new()
        });

        let mut cell_group_manager = CellGroupManager::new(cell_groups, cell_group_location_collections, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs, cell_group_location_dependencies);

        let validated_cell_group_location_dependencies = cell_group_manager.get_validated_cell_group_location_dependencies();

        println!("validated: {:?}", validated_cell_group_location_dependencies);

        assert_eq!(1, validated_cell_group_location_dependencies.len());
        assert_eq!(String::from("cell_group_0"), validated_cell_group_location_dependencies[0].cell_group_id);
        assert_eq!((1, 2), validated_cell_group_location_dependencies[0].location);
        assert!(validated_cell_group_location_dependencies[0].cell_group_location_collections.is_empty());
    }

    #[rstest]
    fn two_cell_groups_two_dependencies_validated() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Main
        }

        let mut cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let mut cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        let cell_groups_total = 2;

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

        // construct index incrementer for looping over locations per cell group

        let mut location_totals: Vec<usize> = Vec::new();
        let mut cell_group_locations_per_cell_group_index: HashMap<usize, Vec<(i32, i32)>> = HashMap::new();
        for cell_group_index in 0..cell_groups_total {
            let cell_group_size = cell_group_index + 1;
            let mut locations: Vec<(i32, i32)> = Vec::new();
            for height_index in 0..(area_height - (cell_group_size - 1)) as i32 {
                for width_index in 0..(area_width - (cell_group_size - 1)) as i32 {
                    locations.push((width_index, height_index));
                }
            }
            location_totals.push(locations.len());
            cell_group_locations_per_cell_group_index.insert(cell_group_index, locations);
        }

        let mut cell_group_location_collection_ids_per_cell_group_index: HashMap<usize, Vec<String>> = HashMap::new();

        for excluded_cell_group_index in 0..cell_groups_total {

            cell_group_location_collection_ids_per_cell_group_index.insert(excluded_cell_group_index, Vec::new());

            let included_location_totals = location_totals.iter().cloned().enumerate().filter(|(index, _)| index != &excluded_cell_group_index).map(|(_, location)| location).collect();

            let mut index_incrementer: IndexIncrementer = IndexIncrementer::new(included_location_totals);

            let mut is_index_incrementer_successful = true;
            while is_index_incrementer_successful {
                let location_indexes = index_incrementer.get();
                println!("cell group {} location_indexes: {:?}", excluded_cell_group_index, location_indexes);
    
                let mut location_per_cell_group_id: HashMap<String, (i32, i32)> = HashMap::new();
    
                for (location_index_index, location_index) in location_indexes.iter().enumerate() {
                    let locations = cell_group_locations_per_cell_group_index.get(&location_index_index).unwrap();
                    let location = locations[location_index.to_owned()];

                    let cell_group_id: String;
                    if location_index_index < excluded_cell_group_index {
                        cell_group_id = String::from(format!("cell_group_{}", location_index_index));
                    }
                    else {
                        cell_group_id = String::from(format!("cell_group_{}", location_index_index + 1));
                    }
                    location_per_cell_group_id.insert(cell_group_id, location);
                }

                let cell_group_location_collection: CellGroupLocationCollection<String, String> = CellGroupLocationCollection {
                    id: Uuid::new_v4().to_string(),
                    location_per_cell_group_id: location_per_cell_group_id
                };

                cell_group_location_collection_ids_per_cell_group_index.get_mut(&excluded_cell_group_index).unwrap().push(cell_group_location_collection.id.clone());
                cell_group_location_collections.push(cell_group_location_collection);

                is_index_incrementer_successful = index_incrementer.try_increment();
            }
        }

        for index in 0..cell_groups_total {
            let mut cells: Vec<(i32, i32)> = Vec::new();
            for width_index in 0..=index as i32 {
                for height_index in 0..=index as i32 {
                    cells.push((width_index, height_index));
                }
            }
            cell_groups.push(CellGroup {
                id: String::from(format!("cell_group_{}", index)),
                cell_group_type: CellGroupType::Main,
                cells: cells
            });

            for cell_group_location in cell_group_locations_per_cell_group_index.get(&index).unwrap().iter() {
                cell_group_location_dependencies.push(CellGroupLocationDependency {
                    cell_group_id: String::from(format!("cell_group_{}", index)),
                    location: cell_group_location.clone(),
                    cell_group_location_collections: cell_group_location_collection_ids_per_cell_group_index.get(&index).unwrap().clone()
                });
            }
        }

        let mut cell_group_manager = CellGroupManager::new(cell_groups, cell_group_location_collections, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs, cell_group_location_dependencies);

        println!("validating...");

        let validated_cell_group_location_dependencies = cell_group_manager.get_validated_cell_group_location_dependencies();

        println!("validated: {:?}", validated_cell_group_location_dependencies);

        assert_eq!(1, validated_cell_group_location_dependencies.len());
        assert_eq!(String::from("cell_group_0"), validated_cell_group_location_dependencies[0].cell_group_id);
        assert_eq!((1, 2), validated_cell_group_location_dependencies[0].location);
        assert!(validated_cell_group_location_dependencies[0].cell_group_location_collections.is_empty());
    }

    #[rstest]
    fn simple_level_example() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Wall,
            WallAdjacent,
            Floater
        }

        let cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();
        let cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();
        let mut cell_group_manager = CellGroupManager::new(cell_groups, cell_group_location_collections, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs, cell_group_location_dependencies);

        let validated_cell_group_location_dependencies = cell_group_manager.get_validated_cell_group_location_dependencies();

        // TODO
    }
}