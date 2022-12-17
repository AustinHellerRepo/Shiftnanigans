use std::{collections::{HashMap, HashSet}, hash::Hash, marker::PhantomData};

use crate::index_incrementer::{self, IndexIncrementer};

pub struct CellGroup<TCellGroupIdentifier> {
    id: TCellGroupIdentifier,
    cells: Vec<(i32, i32)>,
    width: usize,
    height: usize
    // TODO add a CellGroupType field so that each type can have relationship attributes (detection location offsets, etc.)
}

/// This struct contains metadata useful for processing cell groups
struct CellGroupMetaData<TCellGroupIdentifier> {
    detection_cells: Vec<(i32, i32)>,
    contacted_cell_group_ids: HashSet<TCellGroupIdentifier>
}

impl<TCellGroupIdentifier> CellGroupMetaData<TCellGroupIdentifier> {
    fn new(detection_cells: Vec<(i32, i32)>, contacted_cell_group_ids: HashSet<TCellGroupIdentifier>) -> Self {
        CellGroupMetaData {
            detection_cells: detection_cells,
            contacted_cell_group_ids: contacted_cell_group_ids
        }
    }
}

/// This struct contains a specific arrangement of cell groups, each location specified per cell group
pub struct CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier> {
    id: TCellGroupLocationCollectionIdentifier,
    location_per_cell_group_id: HashMap<TCellGroupIdentifier, (i32, i32)>
}

// TODO create a struct for specifying that "this" cell group location has "these" cell group location collections as dependencies such that if being at that location makes all of them invalid, then that location must be invalid

/// This struct contains metadata useful for processing cell group location collections
struct CellGroupLocationCollectionMetaData<TCellGroupLocationCollectionSetIdentifier> {
    dependent_cell_group_location_collection_sets: Vec<TCellGroupLocationCollectionSetIdentifier>
}

impl<TCellGroupLocationCollectionSetIdentifier> CellGroupLocationCollectionMetaData<TCellGroupLocationCollectionSetIdentifier> {
    fn new(dependent_cell_group_location_collection_sets: Vec<TCellGroupLocationCollectionSetIdentifier>) -> Self {
        CellGroupLocationCollectionMetaData {
            dependent_cell_group_location_collection_sets: dependent_cell_group_location_collection_sets
        }
    }
}

pub struct AnonymousCellGroupLocationCollection<TCellGroupIdentifier> {
    location_per_cell_group_id: HashSet<TCellGroupIdentifier, (i32, i32)>
}

pub struct CellGroupManager<TCellGroupLocationCollectionSetIdentifier, TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier> {
    cell_group_location_collection_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>>,
    // TODO refactor line below to hold cell_group_location_dependencies, which will ultimately become cell_group_location_dependency_per_cell_group_location_id in this struct
    cell_group_location_collection_set_per_cell_group_location_collection_set_id: HashMap<TCellGroupLocationCollectionSetIdentifier, CellGroupLocationCollectionSet<TCellGroupLocationCollectionSetIdentifier, TCellGroupLocationCollectionIdentifier>>,
    cell_group_meta_data_per_cell_group_id: HashMap<TCellGroupIdentifier, CellGroupMetaData<TCellGroupIdentifier>>,
    cell_group_location_collection_meta_data_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollectionMetaData<TCellGroupLocationCollectionSetIdentifier>>,
    cached_detection_locations_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashSet<(i32, i32)>>
    // TODO create cached_overlap_locations_per_cell_group_location_id as a HashSet of the underlying calculated cell locations
}

// TODO make detection specific to a pair of cell_group_ids since wall-adjacents can be within range of a wall
// TODO if all dependent cell group location collections are invalid for a specific cell group location, then that cell group location is invalid

impl<TCellGroupLocationCollectionSetIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupLocationCollectionIdentifier: Hash + Eq + std::fmt::Debug + Clone, TCellGroupIdentifier: Hash + Eq + std::fmt::Debug + Clone> CellGroupManager<TCellGroupLocationCollectionSetIdentifier, TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier> {
    fn new(
        cell_groups: Vec<CellGroup<TCellGroupIdentifier>>,
        cell_group_location_collections: Vec<CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>>,
        detection_offsets: Vec<(i32, i32)>,
        cell_group_contacts: Vec<(TCellGroupIdentifier, TCellGroupIdentifier)>,
        cell_group_location_collection_sets: Vec<CellGroupLocationCollectionSet<TCellGroupLocationCollectionSetIdentifier, TCellGroupLocationCollectionIdentifier>>,
        dependent_cell_group_location_collection_sets_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, HashSet<TCellGroupLocationCollectionSetIdentifier>>
    ) -> Self {

        // cache cell group contact into useful structure
        let mut contacted_cell_groups_per_cell_group: HashMap<TCellGroupIdentifier, HashSet<TCellGroupIdentifier>> = HashMap::new();
        for cell_group_contact in cell_group_contacts.iter() {
            for (from_cell_group_identifier, to_cell_group_identifier) in [(cell_group_contact.0.clone(), cell_group_contact.1.clone()), (cell_group_contact.1.clone(), cell_group_contact.0.clone())] {
                if !contacted_cell_groups_per_cell_group.contains_key(&from_cell_group_identifier) {
                    contacted_cell_groups_per_cell_group.insert(from_cell_group_identifier.clone(), HashSet::new());
                }
                contacted_cell_groups_per_cell_group.get_mut(&from_cell_group_identifier).unwrap().insert(to_cell_group_identifier);
            }
        }

        // construct detection cell groups from provided cell groups
        // TODO optimize by placing this logic into one of the previous for-loops
        let mut cell_group_meta_data_per_cell_group_id: HashMap<TCellGroupIdentifier, CellGroupMetaData<TCellGroupIdentifier>> = HashMap::new();
        for cell_group in cell_groups.iter() {

            // construct detection cells
            let mut detection_cells: Vec<(i32, i32)> = Vec::new();
            let mut traveled_cells: HashSet<(i32, i32)> = HashSet::new();
            for cell in cell_group.cells.iter() {
                if !traveled_cells.contains(cell) {
                    detection_cells.push(cell.to_owned());
                }
                for detection_offset in detection_offsets.iter() {
                    let potential_detection_cell = (cell.0 + detection_offset.0, cell.1 + detection_offset.1);
                    if !traveled_cells.contains(&potential_detection_cell) {
                        detection_cells.push(potential_detection_cell);
                    }
                }
            }

            let contacted_cell_group_ids: HashSet<TCellGroupIdentifier> = contacted_cell_groups_per_cell_group.remove(&cell_group.id).unwrap();
            let detection_cell_group = CellGroupMetaData::new(detection_cells, contacted_cell_group_ids);
            cell_group_meta_data_per_cell_group_id.insert(cell_group.id.clone(), detection_cell_group);
        }

        // construct cell group location collection metadata
        let mut cell_group_location_collection_meta_data_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollectionMetaData<TCellGroupLocationCollectionSetIdentifier>> = HashMap::new();
        for cell_group_location_collection in cell_group_location_collections.iter() {

            // collect specific dependent cell group location IDs that need to be iterated over in sequences
            let dependent_cell_group_location_collection_sets: Vec<TCellGroupLocationCollectionSetIdentifier> = dependent_cell_group_location_collection_sets_per_cell_group_location_collection_id.get(&cell_group_location_collection.id).unwrap().iter().cloned().collect();
            cell_group_location_collection_meta_data_per_cell_group_location_collection_id.insert(cell_group_location_collection.id.clone(), CellGroupLocationCollectionMetaData::new(dependent_cell_group_location_collection_sets));
        }

        // create object per id instances

        let mut cell_group_location_collection_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, CellGroupLocationCollection<TCellGroupLocationCollectionIdentifier, TCellGroupIdentifier>> = HashMap::new();
        for cell_group_location_collection in cell_group_location_collections.into_iter() {
            cell_group_location_collection_per_cell_group_location_collection_id.insert(cell_group_location_collection.id.clone(), cell_group_location_collection);
        }
        
        let mut cell_group_location_collection_set_per_cell_group_location_collection_set_id: HashMap<TCellGroupLocationCollectionSetIdentifier, CellGroupLocationCollectionSet<TCellGroupLocationCollectionSetIdentifier, TCellGroupLocationCollectionIdentifier>> = HashMap::new();
        for cell_group_location_collection_set in cell_group_location_collection_sets.into_iter() {
            cell_group_location_collection_set_per_cell_group_location_collection_set_id.insert(cell_group_location_collection_set.id.clone(), cell_group_location_collection_set);
        }

        CellGroupManager {
            cell_group_location_collection_per_cell_group_location_collection_id: cell_group_location_collection_per_cell_group_location_collection_id,
            cell_group_location_collection_set_per_cell_group_location_collection_set_id: cell_group_location_collection_set_per_cell_group_location_collection_set_id,
            cell_group_meta_data_per_cell_group_id: cell_group_meta_data_per_cell_group_id,
            cell_group_location_collection_meta_data_per_cell_group_location_collection_id: cell_group_location_collection_meta_data_per_cell_group_location_collection_id,
            cached_detection_locations_per_cell_group_location_collection_id: HashMap::new()
        }
    }
    /// This function will determine which permitted locations are actually possible while iterating over all possible locations for the known dependent cell group location collection sets
    /// Returns true if at least one cell group location collection was removed from at least one of the known dependent cell group location collection sets
    fn try_reduce_cell_group_location_collection(&mut self, cell_group_location_collection_id: &TCellGroupLocationCollectionIdentifier) -> bool {

        // iterate over the different cell group location collections in each set sequentially, marking any failing cell group location collections that result in failure to be removed as a dependency

        let meta_data = self.cell_group_location_collection_meta_data_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap();
        let mut cell_group_location_collection_ids_per_cell_group_location_collection_set_id: HashMap<TCellGroupLocationCollectionSetIdentifier, Vec<TCellGroupLocationCollectionIdentifier>> = HashMap::new();
        let mut maximum_exclusive_indexes: Vec<usize> = Vec::new();

        {
            for cell_group_location_collection_set_id in meta_data.dependent_cell_group_location_collection_sets.iter() {
                let cell_group_location_collection_set = self.cell_group_location_collection_set_per_cell_group_location_collection_set_id.get(cell_group_location_collection_set_id).unwrap();
                maximum_exclusive_indexes.push(cell_group_location_collection_set.set.len());

                // fill cell group location collect ids per cell group location collection set id
                cell_group_location_collection_ids_per_cell_group_location_collection_set_id.insert(cell_group_location_collection_set_id.clone(), Vec::new());
                for cell_group_location_collection_id in cell_group_location_collection_set.set.iter() {
                    cell_group_location_collection_ids_per_cell_group_location_collection_set_id.get_mut(cell_group_location_collection_set_id).unwrap().push(cell_group_location_collection_id.clone());
                }
            }
        }

        // cache the detection locations that pertain to this cell group location collection
        let detection_cells: &HashSet<(i32, i32)>;

        {
            if !self.cached_detection_locations_per_cell_group_location_collection_id.contains_key(cell_group_location_collection_id) {
                let mut cached_detection_cells: HashSet<(i32, i32)> = HashSet::new();
                let cell_group_location_collection = self.cell_group_location_collection_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap();
                for cell_group_id in cell_group_location_collection.location_per_cell_group_id.keys() {
                    let cell_group_meta_data = self.cell_group_meta_data_per_cell_group_id.get(cell_group_id).unwrap();
                    cached_detection_cells.extend(cell_group_meta_data.detection_cells.clone());
                }
                self.cached_detection_locations_per_cell_group_location_collection_id.insert(cell_group_location_collection_id.clone(), cached_detection_cells);
            }
            detection_cells = self.cached_detection_locations_per_cell_group_location_collection_id.get(cell_group_location_collection_id).unwrap();
        }

        // TODO refactor to simply loop over all cell_group_location_dependencies since the need to use IndexIncrementer would have been when the cell group location collections were being constructed
        // increment over the possible cell group location collections per cell group location collection set, checking that none of the cells in any other the cell group locations overlap with a detection location

        let incompatible_cell_group_location_collection_ids: Vec<TCellGroupLocationCollectionIdentifier> = Vec::new();
        let index_incrementer: IndexIncrementer = IndexIncrementer::new(maximum_exclusive_indexes);

        {
            let is_index_incremented_successfully: bool = true;
            while is_index_incremented_successfully {
                let indexes = index_incrementer.get();
                for index in indexes.iter() {

                }
            }
        }

        // TODO apply this logic when iterating over cell group locations
        //      for each cell group ID that is a contact of the provided cell group but which is not in the current cell group location collection
        //          add the cell group ID to a set for later analysis (as if it was present)
        //      if the current cell is in the overlap cache for the provided cell group location's overlap set
        //          this cell group location collection is invalid, mark the provided cell group location and the current cell group location for mass removal from all data structures
        //      else if the current cell is in the provided cell group's applicable (based on type) detection location
        //          bad too
        //      else if the current cell is in the provided cell group's contact set and the current cell is actually in contact with the provided cell group
        //          add the current cell's cell group ID to a set for later analysis


        // remove any cell group location collections by ID from all cell group location collection sets in cell_group_location_collection_set_per_cell_group_location_collection_set_id

        {

        }

        !incompatible_cell_group_location_collection_ids.is_empty()
    }
    pub fn get_validated_cell_group_location_collection_sets_per_cell_group_location_collection(&mut self) -> HashMap<TCellGroupLocationCollectionIdentifier, Vec<TCellGroupLocationCollectionSetIdentifier>> {

        // cache cell group location collection IDs
        let cell_group_location_collection_ids: Vec<TCellGroupLocationCollectionIdentifier> = self.cell_group_location_collection_per_cell_group_location_collection_id.keys().cloned().collect();

        let mut is_at_least_one_cell_group_location_collection_reduced = true;
        while is_at_least_one_cell_group_location_collection_reduced {
            is_at_least_one_cell_group_location_collection_reduced = false;

            // TODO consider if there is an ideal way to sort the cell group location collection IDs based on alterations

            for cell_group_location_collection_id in cell_group_location_collection_ids.iter() {
                let is_cell_group_location_collection_reduced = self.try_reduce_cell_group_location_collection(cell_group_location_collection_id);
                if is_cell_group_location_collection_reduced {
                    is_at_least_one_cell_group_location_collection_reduced = true;
                }
            }
        }

        // at this point the existing dependent cell group location collection sets per cell group location collection are the only valid combinations

        let mut dependent_cell_group_location_collection_sets_per_cell_group_location_collection_id: HashMap<TCellGroupLocationCollectionIdentifier, Vec<TCellGroupLocationCollectionSetIdentifier>> = HashMap::new();
        for (cell_group_location_collection_id, cell_group_location_collection) in self.cell_group_location_collection_per_cell_group_location_collection_id.iter() {
            dependent_cell_group_location_collection_sets_per_cell_group_location_collection_id.insert(cell_group_location_collection_id.clone(), self.cell_group_location_collection_meta_data_per_cell_group_location_collection_id.get(&cell_group_location_collection.id).unwrap().dependent_cell_group_location_collection_sets.clone());
        }
        dependent_cell_group_location_collection_sets_per_cell_group_location_collection_id
    }
}