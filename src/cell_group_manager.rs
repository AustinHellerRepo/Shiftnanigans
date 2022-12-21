use std::{collections::{HashMap, HashSet}, hash::Hash, marker::PhantomData, time::Instant};

use uuid::Uuid;

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

pub struct CellGroupDependencyManager<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier, TCellGroupType> {
    cell_group_collection: CellGroupCollection<TCellGroupIdentifier, TCellGroupType>,
    cell_group_location_dependencies_per_cell_group_id: HashMap<TCellGroupIdentifier, Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>>>,
    detection_locations_per_cell_group_type_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashMap<TCellGroupType, HashSet<(i32, i32)>>>>,
    overlap_locations_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashSet<(i32, i32)>>>,
    located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashMap<(TCellGroupIdentifier, TCellGroupType, (i32, i32)), Vec<(i32, i32)>>>,
    cell_group_id_and_location_tuples_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashSet<(TCellGroupIdentifier, (i32, i32))>>
}

impl<TCellGroupLocationCollectionIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupType: Hash + Eq + std::fmt::Debug + Clone> CellGroupDependencyManager<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier, TCellGroupType> {
    pub fn new(
        cell_group_collection: CellGroupCollection<TCellGroupIdentifier, TCellGroupType>,
        cell_group_location_collections: Vec<CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>>,
        cell_group_location_dependencies: Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>>
    ) -> Self {

        // create cell group location collection lookup hashmap

        let mut cell_group_location_collection_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>> = HashMap::new();

        {
            for cell_group_location_collection in cell_group_location_collections.into_iter() {
                cell_group_location_collection_per_cell_group_location_collection_id.insert(cell_group_location_collection.id.clone(), cell_group_location_collection);
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
            for cell_group in cell_group_collection.get_cell_group_per_cell_group_id().values() {
                overlap_locations_per_location_per_cell_group_id.insert(cell_group.id.clone(), HashMap::new());

                if cell_group_location_dependencies_per_cell_group_id.contains_key(&cell_group.id) {
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
        }

        // construct located cells per cell group type per cell group location collection
        // construct lookup hashset of cell group ID and location pairs for later checking if this is a cell group location collection that should be removed

        let mut located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashMap<(TCellGroupIdentifier, TCellGroupType, (i32, i32)), Vec<(i32, i32)>>> = HashMap::new();
        let mut cell_group_id_and_location_tuples_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashSet<(TCellGroupIdentifier, (i32, i32))>> = HashMap::new();

        for (cell_group_location_collection_id, cell_group_location_collection) in cell_group_location_collection_per_cell_group_location_collection_id.iter() {
            located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.insert(cell_group_location_collection_id.clone(), HashMap::new());
            let mut cell_group_id_and_location_tuples: HashSet<(TCellGroupIdentifier, (i32, i32))> = HashSet::new();
            for (cell_group_id, location) in cell_group_location_collection.location_per_cell_group_id.iter() {
                //println!("cell group location collection {:?} with cell group {:?} is at location {:?}", cell_group_location_collection_id, cell_group_id, location);
                let cell_group = cell_group_collection.get_cell_group_per_cell_group_id().get(cell_group_id).unwrap();
                let cell_group_id_and_cell_group_type_and_location_tuple = (cell_group.id.clone(), cell_group.cell_group_type.clone(), location.clone());
                if !located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap().contains_key(&cell_group_id_and_cell_group_type_and_location_tuple) {
                    located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get_mut(cell_group_location_collection_id).unwrap().insert(cell_group_id_and_cell_group_type_and_location_tuple.clone(), Vec::new());
                }

                // append this cell group's located cells
                    
                for cell in cell_group.cells.iter() {
                    let located_cell = (location.0 + cell.0, location.1 + cell.1);
                    located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get_mut(cell_group_location_collection_id).unwrap().get_mut(&cell_group_id_and_cell_group_type_and_location_tuple).unwrap().push(located_cell);
                }

                cell_group_id_and_location_tuples.insert((cell_group_id.clone(), location.clone()));
            }
            cell_group_id_and_location_tuples_per_cell_group_location_collection_id.insert(cell_group_location_collection_id.clone(), cell_group_id_and_location_tuples);
        }

        let mut detection_locations_per_cell_group_type_per_location_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<(i32, i32), HashMap<TCellGroupType, HashSet<(i32, i32)>>>> = HashMap::new();

        {
            // iterate over every location each cell group could exist at for each cell group type it may encounter in a dependency

            for (cell_group_id, cell_group_location_dependencies) in cell_group_location_dependencies_per_cell_group_id.iter() {
                detection_locations_per_cell_group_type_per_location_per_cell_group_id.insert(cell_group_id.clone(), HashMap::new());
                for cell_group_location_dependency in cell_group_location_dependencies {
                    if !detection_locations_per_cell_group_type_per_location_per_cell_group_id.get(&cell_group_location_dependency.cell_group_id).unwrap().contains_key(&cell_group_location_dependency.location) {
                        detection_locations_per_cell_group_type_per_location_per_cell_group_id.get_mut(&cell_group_location_dependency.cell_group_id).unwrap().insert(cell_group_location_dependency.location.clone(), HashMap::new());
                    }
                    for dependent_cell_group_location_collection_id in cell_group_location_dependency.cell_group_location_collections.iter() {
                        for cell_group_id in cell_group_location_collection_per_cell_group_location_collection_id.get(dependent_cell_group_location_collection_id).unwrap().location_per_cell_group_id.keys() {
                            let dependent_cell_group = cell_group_collection.get_cell_group_per_cell_group_id().get(cell_group_id).unwrap();
                            if !detection_locations_per_cell_group_type_per_location_per_cell_group_id.get(&cell_group_location_dependency.cell_group_id).unwrap().get(&cell_group_location_dependency.location).unwrap().contains_key(&dependent_cell_group.cell_group_type) {
                                let mut detection_locations: HashSet<(i32, i32)> = HashSet::new();

                                // calculate detection locations for this location and cell group type
                                if cell_group_collection.get_detection_cells_per_cell_group_type_per_cell_group_id().contains_key(&cell_group_location_dependency.cell_group_id) &&
                                    cell_group_collection.get_detection_cells_per_cell_group_type_per_cell_group_id().get(&cell_group_location_dependency.cell_group_id).unwrap().contains_key(&dependent_cell_group.cell_group_type) {

                                    for detection_cell in cell_group_collection.get_detection_cells_per_cell_group_type_per_cell_group_id().get(&cell_group_location_dependency.cell_group_id).unwrap().get(&dependent_cell_group.cell_group_type).unwrap().iter() {
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
        }

        CellGroupDependencyManager {
            cell_group_collection: cell_group_collection,
            cell_group_location_dependencies_per_cell_group_id: cell_group_location_dependencies_per_cell_group_id,
            detection_locations_per_cell_group_type_per_location_per_cell_group_id: detection_locations_per_cell_group_type_per_location_per_cell_group_id,
            overlap_locations_per_location_per_cell_group_id: overlap_locations_per_location_per_cell_group_id,
            located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id: located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id,
            cell_group_id_and_location_tuples_per_cell_group_location_collection_id: cell_group_id_and_location_tuples_per_cell_group_location_collection_id
        }
    }
    /// This function will determine which permitted locations for this cell group are actually possible while iterating over all possible locations for the known dependent cell group
    /// Returns true if at least one cell group location dependency was removed from at least one of the known dependent cell group location dependencies
    fn try_reduce_cell_group_location_dependency_for_cell_group(&mut self, cell_group_id: &TCellGroupIdentifier) -> bool {

        // load cached overlap locations per location
        let overlap_locations_per_location = self.overlap_locations_per_location_per_cell_group_id.get(cell_group_id).unwrap();

        // load cached detection locations per cell group type per location
        let detection_locations_per_cell_group_type_per_location = self.detection_locations_per_cell_group_type_per_location_per_cell_group_id.get(cell_group_id).unwrap();

        // load expected adjacent cell group IDs
        let expected_adjacent_cell_group_ids = self.cell_group_collection.get_adjacent_cell_group_ids_per_cell_group_id().get(cell_group_id).unwrap();

        // collect invalid pairs of cell groups for when being at their respective locations never produces a valid combination
        let mut invalid_cell_group_id_and_location_tuples_per_location: HashMap<(i32, i32), Vec<(TCellGroupIdentifier, (i32, i32))>> = HashMap::new();

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

                for ((located_cell_group_id, located_cell_group_type, location), located_cells) in self.located_cells_per_cell_group_id_and_cell_group_type_and_location_tuple_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap().iter() {
                    
                    //println!("checking {:?} at {:?} against {:?}", cell_group_id, cell_group_location_dependency.location, (located_cell_group_id, located_cell_group_type, location, located_cells));
                    let is_adjacency_expected = expected_adjacent_cell_group_ids.contains(located_cell_group_id);
                    let mut is_adjacent = false;
                    let mut is_valid_cell_group = true;

                    // check to see that the located_cells do not exist in the overlap locations
                    for located_cell in located_cells.iter() {
                        if overlap_locations.contains(located_cell) ||
                            detection_locations_per_cell_group_type.get(located_cell_group_type).unwrap().contains(located_cell) {

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
                        if !invalid_cell_group_id_and_location_tuples_per_location.contains_key(&cell_group_location_dependency.location) {
                            invalid_cell_group_id_and_location_tuples_per_location.insert(cell_group_location_dependency.location, Vec::new());
                        }
                        //println!("found invalid cell group ID {:?} at location {:?}", located_cell_group_id, location);
                        invalid_cell_group_id_and_location_tuples_per_location.get_mut(&cell_group_location_dependency.location).unwrap().push((located_cell_group_id.clone(), location.clone()));

                        // not breaking here for two reasons:
                        //  there is already cached data used for each loop
                        //  this logic may find more than one pair of cell group and location issues (instead of having to iterate over the same dependency again)
                        // TODO allow for a boolean condition for either removing all invalid cell group and location pairs upon finding them versus waiting until after searching all dependencies (as is done currently)
                    }
                }

                if is_valid_cell_group_location_collection {
                    is_at_least_one_cell_group_location_collection_possible = true;
                    //println!("discovered that at least one cell group location collection is valid");
                }
            }

            if !is_at_least_one_cell_group_location_collection_possible {
                invalid_cell_group_location_dependency_indexes.push(cell_group_location_dependency_index);
                invalid_cell_group_id_and_location_tuples_per_location.remove(&cell_group_location_dependency.location);
                //println!("realized that all cell group location collections were invalid, so removing entire dependency {:?}", cell_group_location_dependency_index);
            }
        }

        let is_at_least_one_reduction_performed: bool = !invalid_cell_group_location_dependency_indexes.is_empty() || !invalid_cell_group_id_and_location_tuples_per_location.is_empty();

        // remove each invalid cell group location dependency since the current cell group at its location fails to satisfy any of the provided cell group location collections in the dependency

        for cell_group_location_dependency_index in invalid_cell_group_location_dependency_indexes.into_iter().rev() {
            //println!("removing cell group location {:?}", self.cell_group_location_dependencies_per_cell_group_id.get_mut(cell_group_id).unwrap()[cell_group_location_dependency_index]);
            self.cell_group_location_dependencies_per_cell_group_id.get_mut(cell_group_id).unwrap().remove(cell_group_location_dependency_index);
        }

        // remove any invalid_cell_group_and_location_tuples_per_location for this cell group since the combinations of the two will always lead to invalid results

        for (cell_group_location, invalid_cell_group_id_and_location_tuples) in invalid_cell_group_id_and_location_tuples_per_location.into_iter() {
            for cell_group_location_dependency in self.cell_group_location_dependencies_per_cell_group_id.get_mut(cell_group_id).unwrap().iter_mut() {
                if cell_group_location_dependency.location == cell_group_location {
                    let mut invalid_cell_group_location_collection_id_indexes: Vec<usize> = Vec::new();
                    for (cell_group_location_collection_id_index, cell_group_location_collection_id) in cell_group_location_dependency.cell_group_location_collections.iter().enumerate() {
                        let mut is_cell_group_location_valid: bool = true;
                        let cell_group_id_and_location_tuples = self.cell_group_id_and_location_tuples_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap();
                        for invalid_cell_group_id_and_location_tuple in invalid_cell_group_id_and_location_tuples.iter() {
                            if cell_group_id_and_location_tuples.contains(invalid_cell_group_id_and_location_tuple) {
                                is_cell_group_location_valid = false;
                                break;
                            }
                        }
                        if !is_cell_group_location_valid {
                            invalid_cell_group_location_collection_id_indexes.push(cell_group_location_collection_id_index);
                        }
                    }
                    for invalid_cell_group_location_collection_id_index in invalid_cell_group_location_collection_id_indexes.into_iter().rev() {
                        //println!("removing cell group location collection {:?} from dependency {:?}", cell_group_location_dependency.cell_group_location_collections[invalid_cell_group_location_collection_id_index], cell_group_location_dependency);
                        cell_group_location_dependency.cell_group_location_collections.remove(invalid_cell_group_location_collection_id_index);
                    }
                }
            }
        }

        is_at_least_one_reduction_performed
    }
    pub fn get_validated_cell_group_location_dependencies(&mut self) -> Vec<CellGroupLocationDependency<TCellGroupIdentifier, TCellGroupLocationCollectionIdentifier>> {

        // cache cell group IDs
        let cell_group_ids: Vec<TCellGroupIdentifier> = self.cell_group_collection.cell_group_per_cell_group_id.keys().cloned().collect();

        let mut is_at_least_one_cell_group_location_dependency_reduced = true;
        while is_at_least_one_cell_group_location_dependency_reduced {
            is_at_least_one_cell_group_location_dependency_reduced = false;

            // TODO consider if there is an ideal way to sort the cell group location collection IDs based on alterations
            // TODO consider if subsequent passes will ever result in any reductions

            for cell_group_id in cell_group_ids.iter() {
                if self.cell_group_location_dependencies_per_cell_group_id.contains_key(cell_group_id) {
                    let is_cell_group_location_dependency_reduced = self.try_reduce_cell_group_location_dependency_for_cell_group(cell_group_id);
                    if is_cell_group_location_dependency_reduced {
                        is_at_least_one_cell_group_location_dependency_reduced = true;
                    }
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
    pub fn filter_invalid_cell_group_location_collections(cell_group_collection: CellGroupCollection<TCellGroupIdentifier, TCellGroupType>, cell_group_location_collections: Vec<CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>>) -> Vec<CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>> {

        // construct the necessary data structures to test this cell group location collection as if each individual cell group can be located where it is defined in the cell group location collection

        let mut validated_cell_group_collection_locations: Vec<CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>> = Vec::new();

        for (cell_group_location_collection_index, cell_group_location_collection) in cell_group_location_collections.into_iter().enumerate() {
            let mut inner_cell_group_location_collections: Vec<CellGroupLocationCollection<String, TCellGroupIdentifier>> = Vec::new();
            let mut inner_cell_group_location_dependencies: Vec<CellGroupLocationDependency<TCellGroupIdentifier, String>> = Vec::new();
            let mut inner_cell_group_location_ids: HashSet<String> = HashSet::new();
            for (cell_group_index, (cell_group_id, location)) in cell_group_location_collection.location_per_cell_group_id.iter().enumerate() {

                let mut location_per_cell_group_id: HashMap<TCellGroupIdentifier, (i32, i32)> = HashMap::new();
                for (other_cell_group_index, (other_cell_group_id, other_location)) in cell_group_location_collection.location_per_cell_group_id.iter().enumerate() {
                    if other_cell_group_index != cell_group_index {
                        location_per_cell_group_id.insert(other_cell_group_id.clone(), other_location.clone());
                    }
                }

                let inner_cell_group_location_collection_id: String = format!("inner_{}_{}", cell_group_index, cell_group_location_collection_index);
                inner_cell_group_location_ids.insert(inner_cell_group_location_collection_id.clone());
                let inner_cell_group_location_collection = CellGroupLocationCollection {
                    id: inner_cell_group_location_collection_id.clone(),
                    location_per_cell_group_id: location_per_cell_group_id
                };

                inner_cell_group_location_collections.push(inner_cell_group_location_collection);

                let cell_group_location_dependency = CellGroupLocationDependency {
                    cell_group_id: cell_group_id.clone(),
                    location: location.clone(),
                    cell_group_location_collections: vec![inner_cell_group_location_collection_id]
                };

                inner_cell_group_location_dependencies.push(cell_group_location_dependency);
            }
            let mut cell_group_dependency_manager = CellGroupDependencyManager::new(cell_group_collection.clone(), inner_cell_group_location_collections, inner_cell_group_location_dependencies);
            let validated_cell_group_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();
            if validated_cell_group_dependencies.len() == cell_group_location_collection.location_per_cell_group_id.len() {
                validated_cell_group_collection_locations.push(cell_group_location_collection);
            }
        }
        
        validated_cell_group_collection_locations
    }
}

#[derive(Clone, Debug)]
pub struct CellGroupCollection<TCellGroupIdentifier, TCellGroupType> {
    cell_group_per_cell_group_id: HashMap<TCellGroupIdentifier, CellGroup<TCellGroupIdentifier, TCellGroupType>>,
    detection_cells_per_cell_group_type_per_cell_group_id: HashMap<TCellGroupIdentifier, HashMap<TCellGroupType, Vec<(i32, i32)>>>,
    adjacent_cell_group_ids_per_cell_group_id: HashMap<TCellGroupIdentifier, HashSet<TCellGroupIdentifier>>
}

impl<TCellGroupIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupType: Hash + Eq + std::fmt::Debug + Clone> CellGroupCollection<TCellGroupIdentifier, TCellGroupType> {
    pub fn new(
        cell_groups: Vec<CellGroup<TCellGroupIdentifier, TCellGroupType>>,
        detection_offsets_per_cell_group_type_pair: HashMap<(TCellGroupType, TCellGroupType), Vec<(i32, i32)>>,
        adjacent_cell_group_id_pairs: Vec<(TCellGroupIdentifier, TCellGroupIdentifier)>
    ) -> Self {

        // create cell group lookup hashmap

        let mut cell_group_per_cell_group_id: HashMap<TCellGroupIdentifier, CellGroup<TCellGroupIdentifier, TCellGroupType>> = HashMap::new();

        {
            for cell_group in cell_groups.into_iter() {
                cell_group_per_cell_group_id.insert(cell_group.id.clone(), cell_group);
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

        CellGroupCollection {
            cell_group_per_cell_group_id: cell_group_per_cell_group_id,
            detection_cells_per_cell_group_type_per_cell_group_id: detection_cells_per_cell_group_type_per_cell_group_id,
            adjacent_cell_group_ids_per_cell_group_id: adjacent_cell_group_ids_per_cell_group_id
        }
    }
    fn get_cell_group_per_cell_group_id(&self) -> &HashMap<TCellGroupIdentifier, CellGroup<TCellGroupIdentifier, TCellGroupType>> {
        &self.cell_group_per_cell_group_id
    }
    fn get_adjacent_cell_group_ids_per_cell_group_id(&self) -> &HashMap<TCellGroupIdentifier, HashSet<TCellGroupIdentifier>> {
        &self.adjacent_cell_group_ids_per_cell_group_id
    }
    fn get_detection_cells_per_cell_group_type_per_cell_group_id(&self) -> &HashMap<TCellGroupIdentifier, HashMap<TCellGroupType, Vec<(i32, i32)>>> {
        &self.detection_cells_per_cell_group_type_per_cell_group_id
    }
}

#[cfg(test)]
mod cell_group_manager_tests {
    use std::time::{Duration, Instant};

    use super::*;
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialize_cell_group_collection() {
        init();

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {}

        let cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();
        let _ = CellGroupCollection::new(cell_groups, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs);
    }

    #[rstest]
    fn one_cell_group_initialized() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Main
        }

        let mut cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();

        cell_groups.push(CellGroup {
            id: String::from("cell_group_0"),
            cell_group_type: CellGroupType::Main,
            cells: vec![(0, 0)]
        });

        let _ = CellGroupCollection::new(cell_groups, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs);
    }

    #[rstest]
    #[should_panic]
    fn one_cell_group_zero_dependencies_initialized() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Main
        }

        let mut cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();

        cell_groups.push(CellGroup {
            id: String::from("cell_group_0"),
            cell_group_type: CellGroupType::Main,
            cells: vec![(0, 0)]
        });

        let cell_group_collection = CellGroupCollection::new(cell_groups, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs);

        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        let _ = CellGroupDependencyManager::new(cell_group_collection, cell_group_location_collections, cell_group_location_dependencies);
    }

    #[rstest]
    fn one_cell_group_one_dependency_validated() {

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        enum CellGroupType {
            Main
        }

        let mut cell_groups: Vec<CellGroup<String, CellGroupType>> = Vec::new();
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();

        cell_groups.push(CellGroup {
            id: String::from("cell_group_0"),
            cell_group_type: CellGroupType::Main,
            cells: vec![(0, 0)]
        });

        let cell_group_collection = CellGroupCollection::new(cell_groups, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs);

        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        cell_group_location_dependencies.push(CellGroupLocationDependency {
            cell_group_id: String::from("cell_group_0"),
            location: (1, 2),
            cell_group_location_collections: Vec::new()
        });

        let mut cell_group_dependency_manager = CellGroupDependencyManager::new(cell_group_collection, cell_group_location_collections, cell_group_location_dependencies);

        let validated_cell_group_location_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

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
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();

        let cell_groups_total = 4;

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
        }

        let cell_group_collection = CellGroupCollection::new(cell_groups.clone(), detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs);

        let mut cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        {
            // construct index incrementer for looping over locations per cell group
    
            let mut location_totals: Vec<usize> = Vec::new();
            let mut cell_group_locations_per_cell_group_index: HashMap<usize, Vec<(i32, i32)>> = HashMap::new();
            for cell_group_index in 0..cell_groups_total {
                let cell_group_size = cell_group_index + 1;
                let mut locations: Vec<(i32, i32)> = Vec::new();
                for height_index in 0..(area_height - (cell_group_size - 1)) as i32 {
                    for width_index in 0..(area_width - (cell_group_size - 1)) as i32 {
                        let location = (width_index, height_index);
                        println!("cell group {:?} can exist at location {:?}", cell_group_index, location);
                        locations.push(location);
                    }
                }
                location_totals.push(locations.len());
                cell_group_locations_per_cell_group_index.insert(cell_group_index, locations);
            }

            println!("cell_group_locations_per_cell_group_index: {:?}", cell_group_locations_per_cell_group_index);

            let mut unfiltered_cell_group_location_collection_ids_per_cell_group_index: HashMap<usize, Vec<String>> = HashMap::new();
            let mut unfiltered_cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();

            for excluded_cell_group_index in 0..cell_groups_total {

                unfiltered_cell_group_location_collection_ids_per_cell_group_index.insert(excluded_cell_group_index, Vec::new());

                let included_location_totals = location_totals.iter().cloned().enumerate().filter(|(index, _)| index != &excluded_cell_group_index).map(|(_, location)| location).collect();

                println!("included_location_totals: {:?}", included_location_totals);

                let mut index_incrementer: IndexIncrementer = IndexIncrementer::new(included_location_totals);

                let mut is_index_incrementer_successful = true;
                while is_index_incrementer_successful {
                    let location_indexes = index_incrementer.get();
                    //println!("cell group {} location_indexes: {:?}", excluded_cell_group_index, location_indexes);
        
                    let mut location_per_cell_group_id: HashMap<String, (i32, i32)> = HashMap::new();
        
                    for (location_index_index, location_index) in location_indexes.iter().enumerate() {
                        let cell_group_index: usize;
                        if location_index_index < excluded_cell_group_index {
                            cell_group_index = location_index_index;
                        }
                        else {
                            cell_group_index = location_index_index + 1;
                        }
                        let locations = cell_group_locations_per_cell_group_index.get(&cell_group_index).unwrap();
                        let location = locations[location_index.to_owned()];

                        let cell_group_id: String = String::from(format!("cell_group_{}", cell_group_index));
                        //println!("placing {:?} at {:?}", cell_group_id, location);
                        location_per_cell_group_id.insert(cell_group_id, location);
                    }

                    let cell_group_location_collection: CellGroupLocationCollection<String, String> = CellGroupLocationCollection {
                        id: Uuid::new_v4().to_string(),
                        location_per_cell_group_id: location_per_cell_group_id
                    };

                    unfiltered_cell_group_location_collection_ids_per_cell_group_index.get_mut(&excluded_cell_group_index).unwrap().push(cell_group_location_collection.id.clone());
                    unfiltered_cell_group_location_collections.push(cell_group_location_collection);

                    is_index_incrementer_successful = index_incrementer.try_increment();
                }
            }

            // filter the cell group location collections before constructing the cell group location dependencies

            let filter_start_time = Instant::now();

            let filtered_cell_group_location_collections = CellGroupDependencyManager::filter_invalid_cell_group_location_collections(cell_group_collection.clone(), unfiltered_cell_group_location_collections);

            println!("filter time: {:?}", filter_start_time.elapsed());

            let mut filtered_cell_group_location_collection_ids_per_cell_group_index: HashMap<usize, Vec<String>> = HashMap::new();

            {
                let mut filtered_cell_group_location_collection_ids: HashSet<&String> = HashSet::new();
                for filtered_cell_group_location_collection in filtered_cell_group_location_collections.iter() {
                    filtered_cell_group_location_collection_ids.insert(&filtered_cell_group_location_collection.id);
                }

                // remove any references to filtered-out cell group location collections

                //println!("unfiltered_cell_group_location_collection_ids_per_cell_group_index: {:?}", unfiltered_cell_group_location_collection_ids_per_cell_group_index);

                for (cell_group_index, unfiltered_cell_group_location_collection_ids) in unfiltered_cell_group_location_collection_ids_per_cell_group_index.into_iter() {
                    let mut discovered_filtered_cell_group_location_collection_ids: Vec<String> = Vec::new();
                    for unfiltered_cell_group_location_collection_id in unfiltered_cell_group_location_collection_ids.into_iter() {
                        if filtered_cell_group_location_collection_ids.contains(&unfiltered_cell_group_location_collection_id) {
                            discovered_filtered_cell_group_location_collection_ids.push(unfiltered_cell_group_location_collection_id);
                        }
                    }
                    filtered_cell_group_location_collection_ids_per_cell_group_index.insert(cell_group_index, discovered_filtered_cell_group_location_collection_ids);
                }
            }

            cell_group_location_collections.extend(filtered_cell_group_location_collections);

            //println!("filtered_cell_group_location_collection_ids_per_cell_group_index: {:?}", filtered_cell_group_location_collection_ids_per_cell_group_index);

            for index in 0..cell_groups_total {
                for cell_group_location in cell_group_locations_per_cell_group_index.get(&index).unwrap().iter() {
                    cell_group_location_dependencies.push(CellGroupLocationDependency {
                        cell_group_id: String::from(format!("cell_group_{}", index)),
                        location: cell_group_location.clone(),
                        cell_group_location_collections: filtered_cell_group_location_collection_ids_per_cell_group_index.get(&index).unwrap().clone()
                    });
                }
            }
        }

        let mut cell_group_dependency_manager = CellGroupDependencyManager::new(cell_group_collection, cell_group_location_collections.clone(), cell_group_location_dependencies);

        println!("validating...");
        let validating_start_time = Instant::now();

        let validated_cell_group_location_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

        println!("validation time: {:?}", validating_start_time.elapsed());
        println!("validated: {:?}", validated_cell_group_location_dependencies);

        // all of the expected locations each cell group can exist at
        let expected_dependencies_total: usize = match cell_groups_total {
            2 => 6,
            3 => 14,
            4 => 4 + 4 + 8 + 22,
            _ => {
                panic!("Unexpected number of cell groups: {}", cell_groups_total);
            }
        };
        assert_eq!(expected_dependencies_total, validated_cell_group_location_dependencies.len());

        // https://en.wikipedia.org/wiki/Square_pyramidal_number
        let cells_total: usize = (cell_groups_total * (cell_groups_total + 1) * (2 * cell_groups_total + 1)) / 6;
        let mut permutations: HashSet<Vec<Vec<&str>>> = HashSet::new();

        for validated_cell_group_location_dependency in validated_cell_group_location_dependencies.iter() {
            let mut pixels: Vec<Vec<bool>> = Vec::new();
            let mut pixels_as_ids: Vec<Vec<&str>> = Vec::new();
            for _ in 0..area_width {
                let mut pixel_column: Vec<bool> = Vec::new();
                let mut pixel_as_id_column: Vec<&str> = Vec::new();
                for _ in 0..area_height {
                    pixel_column.push(false);
                    pixel_as_id_column.push(" ");
                }
                pixels.push(pixel_column);
                pixels_as_ids.push(pixel_as_id_column);
            }
            for cell_group in cell_groups.iter() {
                if cell_group.id == validated_cell_group_location_dependency.cell_group_id {
                    for cell in cell_group.cells.iter() {
                        let width_index = (cell.0 + validated_cell_group_location_dependency.location.0) as usize;
                        let height_index = (cell.1 + validated_cell_group_location_dependency.location.1) as usize;
                        pixels[width_index][height_index] = true;
                        pixels_as_ids[width_index][height_index] = cell_group.id.split("_").last().unwrap();
                    }
                }
            }
            for cell_group_location_collection_id in validated_cell_group_location_dependency.cell_group_location_collections.iter() {
                // iterate over all possible locations, checking if any combination of cell group and its cell group location collection(s) total to the correct number of filled cells, breaking out of the cell coordinate after finding one satisfactory condition (so as to avoid counting any possible overlap)
                let mut cloned_pixels = pixels.clone();
                let mut cloned_pixels_as_ids = pixels_as_ids.clone();
                let mut valid_pixels_total: usize = 0;

                for cell_group_location_collection in cell_group_location_collections.iter() {
                    if &cell_group_location_collection.id == cell_group_location_collection_id {
                        for (cell_group_id, location) in cell_group_location_collection.location_per_cell_group_id.iter() {
                            for cell_group in cell_groups.iter() {
                                if &cell_group.id == cell_group_id {
                                    for cell in cell_group.cells.iter() {
                                        let width_index = (cell.0 + location.0) as usize;
                                        let height_index = (cell.1 + location.1) as usize;
                                        cloned_pixels[width_index][height_index] = true;
                                        cloned_pixels_as_ids[width_index][height_index] = cell_group.id.split("_").last().unwrap();
                                    }
                                }
                            }
                        }
                    }
                }

                let is_printed: bool;
                if validated_cell_group_location_dependency.cell_group_id == String::from("cell_group_0") && false {
                    is_printed = true;
                }
                else {
                    is_printed = false;
                }

                for height_index in 0..area_height {
                    for width_index in 0..area_width {
                        if is_printed {
                            print!("{}", cloned_pixels_as_ids[width_index][height_index]);
                        }

                        // TODO check if any cell group in the current cell group location collection contains this coordinate
                        if cloned_pixels[width_index][height_index] {
                            valid_pixels_total += 1;
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
                permutations.insert(cloned_pixels_as_ids);
            }
        }

        let expected_permutations_total: usize = match cell_groups_total {
            2 => 4,
            3 => 8,
            4 => 96,
            _ => {
                panic!("Unexpected number of cell groups: {}", cell_groups_total);
            }
        };
        assert_eq!(expected_permutations_total, permutations.len());
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
        let detection_offsets_per_cell_group_type_pair: HashMap<(CellGroupType, CellGroupType), Vec<(i32, i32)>> = HashMap::new();
        let adjacent_cell_group_id_pairs: Vec<(String, String)> = Vec::new();

        let cell_group_collection = CellGroupCollection::new(cell_groups, detection_offsets_per_cell_group_type_pair, adjacent_cell_group_id_pairs);

        let cell_group_location_collections: Vec<CellGroupLocationCollection<String, String>> = Vec::new();
        let cell_group_location_dependencies: Vec<CellGroupLocationDependency<String, String>> = Vec::new();

        let mut cell_group_dependency_manager = CellGroupDependencyManager::new(cell_group_collection, cell_group_location_collections, cell_group_location_dependencies);

        let validated_cell_group_location_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

        // TODO
    }
}