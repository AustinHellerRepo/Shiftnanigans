use std::{collections::{BTreeMap, BTreeSet}, hash::Hash, marker::PhantomData, time::Instant};
use bitvec::vec::BitVec;
use uuid::Uuid;
use crate::index_incrementer::{self, IndexIncrementer};

#[derive(Clone, Debug)]
pub struct CellGroup {
    cells: Vec<(i32, i32)>,  // these should exist such that they can be added directly to location points
    cell_group_type_index: usize  // each type can have relationship attributes (detection location offsets, etc.)
}

/// This struct contains a specific arrangement of cell groups, each location specified per cell group
#[derive(Clone, Debug)]
pub struct CellGroupLocationCollection {
    location_per_cell_group_index: Vec<Option<(i32, i32)>>
}

/// This struct specifies that "this" cell group location has "these" cell group location collections as dependencies such that if being at that location makes all of them invalid, then that location must be invalid
#[derive(Clone, Debug)]
pub struct CellGroupLocationDependency {
    cell_group_index: usize,
    location: (i32, i32),
    cell_group_location_collection_indexes: Vec<usize>
}

pub trait CellGroupDependencyManager {
    fn new(
        cell_groups: Vec<CellGroup>,
        detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>>,
        is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
        cell_group_location_collections: Vec<CellGroupLocationCollection>,
        cell_group_location_dependencies: Vec<CellGroupLocationDependency>
    ) -> Self;
    fn get_validated_cell_group_location_dependencies(&mut self) -> Vec<CellGroupLocationDependency>;
    fn filter_invalid_cell_group_location_collections(
        cell_groups: Vec<CellGroup>,
        detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>>,
        is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
        cell_group_location_collections: Vec<CellGroupLocationCollection>
    ) -> Vec<CellGroupLocationCollection>;
}

pub struct RawCellGroupDependencyManager {
    cell_groups: Vec<CellGroup>,
    detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>>,
    is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
    cell_group_location_collections: Vec<CellGroupLocationCollection>,
    cell_group_location_dependencies: Vec<CellGroupLocationDependency>
}

impl CellGroupDependencyManager for RawCellGroupDependencyManager {
    fn new(
        cell_groups: Vec<CellGroup>,
        detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>>,
        is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
        cell_group_location_collections: Vec<CellGroupLocationCollection>,
        cell_group_location_dependencies: Vec<CellGroupLocationDependency>
    ) -> Self {

        RawCellGroupDependencyManager {
            cell_groups: cell_groups,
            detection_offsets_per_cell_group_type_index_per_cell_group_index: detection_offsets_per_cell_group_type_index_per_cell_group_index,
            is_adjacent_cell_group_index_per_cell_group_index: is_adjacent_cell_group_index_per_cell_group_index,
            cell_group_location_collections: cell_group_location_collections,
            cell_group_location_dependencies: cell_group_location_dependencies,
        }
    }
    #[time_graph::instrument]
    fn get_validated_cell_group_location_dependencies(&mut self) -> Vec<CellGroupLocationDependency> {

        

        todo!();
    }
    #[time_graph::instrument]
    fn filter_invalid_cell_group_location_collections(
        cell_groups: Vec<CellGroup>,
        detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>>,
        is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
        cell_group_location_collections: Vec<CellGroupLocationCollection>
    ) -> Vec<CellGroupLocationCollection> {

        // construct the necessary data structures to test this cell group location collection as if each individual cell group can be located where it is defined in the cell group location collection

        let mut validated_cell_group_collection_locations: Vec<CellGroupLocationCollection> = Vec::new();

        for (cell_group_location_collection_index, cell_group_location_collection) in cell_group_location_collections.into_iter().enumerate() {
            let mut inner_cell_group_location_collections: Vec<CellGroupLocationCollection> = Vec::new();
            let mut inner_cell_group_location_dependencies: Vec<CellGroupLocationDependency> = Vec::new();
            for (cell_group_index, location_option) in cell_group_location_collection.location_per_cell_group_index.iter().enumerate() {
                if let Some(location) = location_option {
                    let mut location_per_cell_group_index: Vec<Option<(i32, i32)>> = Vec::new();
                    for (other_cell_group_index, other_location_option) in cell_group_location_collection.location_per_cell_group_index.iter().enumerate() {
                        if other_location_option.is_none() || other_cell_group_index == cell_group_index {
                            location_per_cell_group_index.push(None);
                        }
                        else {
                            location_per_cell_group_index.push(Some(other_location_option.unwrap()));
                        }
                    }

                    let inner_cell_group_location_collection_index: usize = inner_cell_group_location_collections.len();
                    let inner_cell_group_location_collection = CellGroupLocationCollection {
                        location_per_cell_group_index: location_per_cell_group_index
                    };

                    inner_cell_group_location_collections.push(inner_cell_group_location_collection);

                    let cell_group_location_dependency = CellGroupLocationDependency {
                        cell_group_index: cell_group_index,
                        location: location.clone(),
                        cell_group_location_collection_indexes: vec![inner_cell_group_location_collection_index]
                    };

                    inner_cell_group_location_dependencies.push(cell_group_location_dependency);
                }
            }

            let mut cell_group_dependency_manager = RawCellGroupDependencyManager::new(
                cell_groups.clone(),
                detection_offsets_per_cell_group_type_index_per_cell_group_index.clone(),
                is_adjacent_cell_group_index_per_cell_group_index.clone(),
                inner_cell_group_location_collections,
                inner_cell_group_location_dependencies
            );
            let validated_cell_group_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

            let mut valid_location_per_cell_group_index_total: usize = 0;
            for location_option in cell_group_location_collection.location_per_cell_group_index.iter() {
                if location_option.is_some() {
                    valid_location_per_cell_group_index_total += 1;
                }
            }
            if validated_cell_group_dependencies.len() == valid_location_per_cell_group_index_total {
                validated_cell_group_collection_locations.push(cell_group_location_collection);
            }
        }
        
        validated_cell_group_collection_locations
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
    #[should_panic]
    fn one_cell_group_zero_dependencies_initialized() {

        let mut cell_groups: Vec<CellGroup> = Vec::new();

        cell_groups.push(CellGroup {
            cell_group_type_index: 0,
            cells: vec![(0, 0)]
        });

        let detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let cell_group_location_collections: Vec<CellGroupLocationCollection> = Vec::new();
        let cell_group_location_dependencies: Vec<CellGroupLocationDependency> = Vec::new();

        let _ = RawCellGroupDependencyManager::new(cell_groups, detection_offsets_per_cell_group_type_index_per_cell_group_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_location_collections, cell_group_location_dependencies);
    }

    #[rstest]
    fn one_cell_group_one_dependency_validated() {

        let mut cell_groups: Vec<CellGroup> = Vec::new();

        cell_groups.push(CellGroup {
            cell_group_type_index: 0,
            cells: vec![(0, 0)]
        });

        let detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let cell_group_location_collections: Vec<CellGroupLocationCollection> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency> = Vec::new();

        cell_group_location_dependencies.push(CellGroupLocationDependency {
            cell_group_index: 0,
            location: (1, 2),
            cell_group_location_collection_indexes: Vec::new()
        });

        let mut cell_group_dependency_manager = RawCellGroupDependencyManager::new(cell_groups, detection_offsets_per_cell_group_type_index_per_cell_group_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_location_collections, cell_group_location_dependencies);

        let validated_cell_group_location_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

        println!("validated: {:?}", validated_cell_group_location_dependencies);

        assert_eq!(1, validated_cell_group_location_dependencies.len());
        assert_eq!(0, validated_cell_group_location_dependencies[0].cell_group_index);
        assert_eq!((1, 2), validated_cell_group_location_dependencies[0].location);
        assert!(validated_cell_group_location_dependencies[0].cell_group_location_collection_indexes.is_empty());
    }

    #[rstest]
    fn two_cell_groups_two_dependencies_validated() {

        time_graph::enable_data_collection(true);

        let mut cell_groups: Vec<CellGroup> = Vec::new();
        let detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let mut cell_group_location_collections: Vec<CellGroupLocationCollection> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency> = Vec::new();

        let cell_groups_total = 2;

        // Stats
        //  3
        //      2022-12-21  0.01s
        //  4
        //      2022-12-21  0.56s
        //  5
        //      2022-12-21  2339.69s

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
                cell_group_type_index: 0,
                cells: cells
            });
        }

        {
            // construct index incrementer for looping over locations per cell group
    
            let mut location_totals: Vec<usize> = Vec::new();
            let mut cell_group_locations_per_cell_group_index: BTreeMap<usize, Vec<(i32, i32)>> = BTreeMap::new();
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

            for excluded_cell_group_index in 0..cell_groups_total {

                let mut unfiltered_cell_group_location_collections: Vec<CellGroupLocationCollection> = Vec::new();

                let included_location_totals = location_totals.iter().cloned().enumerate().filter(|(index, _)| index != &excluded_cell_group_index).map(|(_, location)| location).collect();

                println!("included_location_totals: {:?}", included_location_totals);

                let mut index_incrementer: IndexIncrementer = IndexIncrementer::new(included_location_totals);

                let mut is_index_incrementer_successful = true;
                while is_index_incrementer_successful {
                    let location_indexes = index_incrementer.get();
                    //println!("cell group {} location_indexes: {:?}", excluded_cell_group_index, location_indexes);
        
                    let mut location_per_cell_group_index: Vec<Option<(i32, i32)>> = Vec::new();
        
                    for (location_index_index, location_index) in location_indexes.iter().enumerate() {
                        let cell_group_index: usize;
                        if location_index_index < excluded_cell_group_index {
                            cell_group_index = location_index_index;
                        }
                        else {
                            if location_index_index == excluded_cell_group_index {
                                location_per_cell_group_index.push(None);
                            }
                            cell_group_index = location_index_index + 1;
                        }
                        let locations = cell_group_locations_per_cell_group_index.get(&cell_group_index).unwrap();
                        let location = locations[location_index.to_owned()];

                        location_per_cell_group_index.push(Some(location));
                    }

                    let cell_group_location_collection: CellGroupLocationCollection = CellGroupLocationCollection {
                        location_per_cell_group_index: location_per_cell_group_index
                    };

                    unfiltered_cell_group_location_collections.push(cell_group_location_collection);

                    is_index_incrementer_successful = index_incrementer.try_increment();
                }

                // filter the cell group location collections before constructing the cell group location dependencies

                let filtered_cell_group_location_collections = RawCellGroupDependencyManager::filter_invalid_cell_group_location_collections(cell_groups.clone(), detection_offsets_per_cell_group_type_index_per_cell_group_index.clone(), is_adjacent_cell_group_index_per_cell_group_index.clone(), unfiltered_cell_group_location_collections);

                let mut cell_group_location_collection_indexes: Vec<usize> = Vec::new();
                for filtered_cell_group_location_collection in filtered_cell_group_location_collections.into_iter() {
                    cell_group_location_collection_indexes.push(cell_group_location_collections.len());
                    cell_group_location_collections.push(filtered_cell_group_location_collection);
                }

                for cell_group_location in cell_group_locations_per_cell_group_index.get(&excluded_cell_group_index).unwrap().iter() {
                    cell_group_location_dependencies.push(CellGroupLocationDependency {
                        cell_group_index: excluded_cell_group_index.clone(),
                        location: cell_group_location.clone(),
                        cell_group_location_collection_indexes: cell_group_location_collection_indexes.clone()
                    });
                }
            }
        }

        println!("validating {} dependencies...", cell_group_location_dependencies.len());

        let mut cell_group_dependency_manager = RawCellGroupDependencyManager::new(cell_groups.clone(), detection_offsets_per_cell_group_type_index_per_cell_group_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_location_collections.clone(), cell_group_location_dependencies);

        let validating_start_time = Instant::now();

        let validated_cell_group_location_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

        println!("validation time: {:?}", validating_start_time.elapsed());
        println!("validated: {:?}", validated_cell_group_location_dependencies);

        println!("{}", time_graph::get_full_graph().as_dot());

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
        let mut permutations: BTreeSet<Vec<Vec<String>>> = BTreeSet::new();

        for validated_cell_group_location_dependency in validated_cell_group_location_dependencies.iter() {
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
            for (cell_group_index, cell_group) in cell_groups.iter().enumerate() {
                if cell_group_index == validated_cell_group_location_dependency.cell_group_index {
                    for cell in cell_group.cells.iter() {
                        let width_index = (cell.0 + validated_cell_group_location_dependency.location.0) as usize;
                        let height_index = (cell.1 + validated_cell_group_location_dependency.location.1) as usize;
                        pixels[width_index][height_index] = true;
                        pixels_as_ids[width_index][height_index] = cell_group_index.to_string();
                    }
                }
            }
            for cell_group_location_collection_index in validated_cell_group_location_dependency.cell_group_location_collection_indexes.iter() {
                // iterate over all possible locations, checking if any combination of cell group and its cell group location collection(s) total to the correct number of filled cells, breaking out of the cell coordinate after finding one satisfactory condition (so as to avoid counting any possible overlap)
                let mut cloned_pixels = pixels.clone();
                let mut cloned_pixels_as_ids = pixels_as_ids.clone();
                let mut valid_pixels_total: usize = 0;

                let cell_group_location_collection = &cell_group_location_collections[*cell_group_location_collection_index];
                for (cell_group_index, location_option) in cell_group_location_collection.location_per_cell_group_index.iter().enumerate() {
                    if let Some(location) = location_option {
                        let cell_group = &cell_groups[cell_group_index];
                        for cell in cell_group.cells.iter() {
                            let width_index = (cell.0 + location.0) as usize;
                            let height_index = (cell.1 + location.1) as usize;
                            cloned_pixels[width_index][height_index] = true;
                            cloned_pixels_as_ids[width_index][height_index] = cell_group_index.to_string();
                        }
                    }
                }

                let is_printed: bool;
                if validated_cell_group_location_dependency.cell_group_index == 0 && false {
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

        enum CellGroupType {
            Wall,
            WallAdjacent,
            Floater
        }

        let mut cell_groups: Vec<CellGroup> = Vec::new();
        let detection_offsets_per_cell_group_type_index_per_cell_group_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let mut cell_group_location_collections: Vec<CellGroupLocationCollection> = Vec::new();
        let mut cell_group_location_dependencies: Vec<CellGroupLocationDependency> = Vec::new();

        let mut cell_group_dependency_manager = RawCellGroupDependencyManager::new(cell_groups, detection_offsets_per_cell_group_type_index_per_cell_group_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_location_collections.clone(), cell_group_location_dependencies);

        let validated_cell_group_location_dependencies = cell_group_dependency_manager.get_validated_cell_group_location_dependencies();

        // TODO
    }
}