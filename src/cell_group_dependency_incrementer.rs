use std::{collections::{BTreeMap, BTreeSet}, hash::Hash, marker::PhantomData, time::Instant, rc::Rc};
use bitvec::vec::BitVec;
use uuid::Uuid;
use crate::{index_incrementer::{self, IndexIncrementer}, element_indexer::{ElementIndexer, ElementIndexerIncrementer}};
use crate::segment_permutation_incrementer::{SegmentPermutationIncrementer};

#[derive(Debug)]
pub struct CellGroup {
    cells: Vec<(i32, i32)>,  // these should exist such that they can be added directly to location points
    cell_group_type_index: usize  // each type can have relationship attributes (detection location offsets, etc.)
}

#[derive(Debug)]
pub struct LocatedCellGroup {
    cell_group_index: usize,
    location: Rc<(i32, i32)>
}

/// This struct specifies that "this" cell group location has "these" cell group location collections as dependencies such that if being at that location makes all of them invalid, then that location must be invalid
#[derive(Debug)]
pub struct CellGroupDependency {
    cell_group_index_mapping: Vec<usize>,
    element_indexer_incrementer: ElementIndexerIncrementer<(i32, i32)>
}

pub struct CellGroupDependencyIncrementer {
    cell_groups: Rc<Vec<CellGroup>>,
    detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>>,
    is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
    cell_group_dependencies: Vec<CellGroupDependency>,
    current_cell_group_dependency_index: usize
}

impl CellGroupDependencyIncrementer {
    pub fn new(
        cell_groups: Rc<Vec<CellGroup>>,
        detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>>,
        is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec>,
        cell_group_dependencies: Vec<CellGroupDependency>
    ) -> Self {

        CellGroupDependencyIncrementer {
            cell_groups: cell_groups,
            detection_offsets_per_cell_group_type_index_per_cell_group_type_index: detection_offsets_per_cell_group_type_index_per_cell_group_type_index,
            is_adjacent_cell_group_index_per_cell_group_index: is_adjacent_cell_group_index_per_cell_group_index,
            cell_group_dependencies: cell_group_dependencies,
            current_cell_group_dependency_index: 0
        }
    }
    #[time_graph::instrument]
    pub fn try_get_next_snapshot(&mut self) -> Option<Vec<LocatedCellGroup>> {
        if self.current_cell_group_dependency_index == self.cell_group_dependencies.len() {
            debug!("current_cell_group_dependency_index == cell_group_dependencies.len()");
            return None;
        }
        else {
            let mut cell_group_locations_option = None;
            while cell_group_locations_option.is_none() && self.current_cell_group_dependency_index < self.cell_group_dependencies.len() {
                cell_group_locations_option = self.cell_group_dependencies[self.current_cell_group_dependency_index].element_indexer_incrementer.try_get_next_elements();
                if let Some(cell_group_locations) = &cell_group_locations_option {
                    let mut located_cell_groups: Vec<LocatedCellGroup> = Vec::new();
                    let cell_group_index_mapping = &self.cell_group_dependencies[self.current_cell_group_dependency_index].cell_group_index_mapping;
                    for (cell_group_location_index, cell_group_location) in cell_group_locations.iter().enumerate() {
                        let cell_group_index = cell_group_index_mapping[cell_group_location_index];
                        let located_cell_group = LocatedCellGroup {
                            cell_group_index: cell_group_index,
                            location: cell_group_location.clone()
                        };
                        located_cell_groups.push(located_cell_group);
                    }
                    if self.is_located_cell_groups_valid(&located_cell_groups) {
                        debug!("found a location cell group");
                        return Some(located_cell_groups);
                    }
                    else {
                        debug!("found invalid location cell group");
                        cell_group_locations_option = None;
                    }
                }
                else {
                    self.current_cell_group_dependency_index += 1;
                }
            } 
            debug!("reached the end of the inner element indexers.");
            return None;
        }
    }
    pub fn reset(&mut self) {
        for cell_group_dependency_index in 0..self.cell_group_dependencies.len() {
            self.cell_group_dependencies[cell_group_dependency_index].element_indexer_incrementer.reset();
        }
        self.current_cell_group_dependency_index = 0;
    }
    fn is_located_cell_groups_valid(&self, located_cell_groups: &Vec<LocatedCellGroup>) -> bool {
        for (located_cell_group_index, located_cell_group) in located_cell_groups.iter().enumerate() {
            let cell_group = &self.cell_groups[located_cell_group.cell_group_index];
            let mut overlap_cell_group_cell_locations: BTreeSet<(i32, i32)> = BTreeSet::new();
            for cell in cell_group.cells.iter() {
                overlap_cell_group_cell_locations.insert((located_cell_group.location.0 + cell.0, located_cell_group.location.1 + cell.1));
            }
            for (other_located_cell_group_index, other_located_cell_group) in located_cell_groups.iter().enumerate() {
                if other_located_cell_group_index != located_cell_group_index {
                    let other_cell_group = &self.cell_groups[other_located_cell_group.cell_group_index];

                    let mut detection_cell_group_cell_locations: BTreeSet<(i32, i32)> = BTreeSet::new();
                    for detection_offset in self.detection_offsets_per_cell_group_type_index_per_cell_group_type_index[cell_group.cell_group_type_index][other_cell_group.cell_group_type_index].iter() {
                        detection_cell_group_cell_locations.insert((located_cell_group.location.0 + detection_offset.0, located_cell_group.location.1 + detection_offset.1));
                    }

                    let is_adjacency_expected: bool = self.is_adjacent_cell_group_index_per_cell_group_index[located_cell_group.cell_group_index][other_located_cell_group.cell_group_index];
                    let mut is_adjacent: bool = false;
                    for other_cell in other_cell_group.cells.iter() {
                        let other_cell_location = (other_located_cell_group.location.0 + other_cell.0, other_located_cell_group.location.1 + other_cell.1);
                        if detection_cell_group_cell_locations.contains(&other_cell_location) || overlap_cell_group_cell_locations.contains(&other_cell_location) {
                            // other cell exists in detection cell locations
                            return false;
                        }
                        if is_adjacency_expected && !is_adjacent {
                            for overlap_cell_group_cell_location in overlap_cell_group_cell_locations.iter() {
                                let width_distance = (overlap_cell_group_cell_location.0 - other_cell_location.0).abs();
                                let height_distance = (overlap_cell_group_cell_location.1 - other_cell_location.1).abs();
                                if width_distance == 0 && height_distance == 1 ||
                                    width_distance == 1 && height_distance == 0 {
                                    
                                    is_adjacent = true;
                                    break;
                                }
                            }
                        }
                    }

                    if is_adjacency_expected && !is_adjacent {
                        return false;
                    }
                }
            }
        }
        return true;
    }
}

impl Iterator for CellGroupDependencyIncrementer {
    type Item = Vec<LocatedCellGroup>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_get_next_snapshot()
    }
}

#[cfg(test)]
mod cell_group_manager_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use crate::element_indexer::IndexIncrementerElementIndexer;

    use super::*;
    use bitvec::bits;
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn one_cell_group_zero_dependencies_initialized() {

        let mut cell_groups: Vec<CellGroup> = Vec::new();

        cell_groups.push(CellGroup {
            cell_group_type_index: 0,
            cells: vec![(0, 0)]
        });

        let detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let cell_group_dependencies: Vec<CellGroupDependency> = Vec::new();

        let _ = CellGroupDependencyIncrementer::new(Rc::new(cell_groups), detection_offsets_per_cell_group_type_index_per_cell_group_type_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_dependencies);
    }

    #[rstest]
    fn one_cell_group_one_dependency_validated() {

        let mut cell_groups: Vec<CellGroup> = Vec::new();

        cell_groups.push(CellGroup {
            cell_group_type_index: 0,
            cells: vec![(0, 0)]
        });

        let mut detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let mut is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let mut cell_group_dependencies: Vec<CellGroupDependency> = Vec::new();
        let mut index_incrementer_element_indexer = IndexIncrementerElementIndexer::new(vec![vec![(1, 2)]]);
        cell_group_dependencies.push(CellGroupDependency {
            cell_group_index_mapping: vec![0],
            element_indexer_incrementer: ElementIndexerIncrementer::new(vec![
                Rc::new(RefCell::new(index_incrementer_element_indexer))
            ])
        });

        detection_offsets_per_cell_group_type_index_per_cell_group_type_index.push(vec![vec![]]);
        is_adjacent_cell_group_index_per_cell_group_index.push(BitVec::from_bitslice(bits![0; 1]));

        let mut cell_group_dependency_incrementer = CellGroupDependencyIncrementer::new(Rc::new(cell_groups), detection_offsets_per_cell_group_type_index_per_cell_group_type_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_dependencies);

        let located_cell_groups_option = cell_group_dependency_incrementer.try_get_next_snapshot();
        assert!(located_cell_groups_option.is_some());
        let located_cell_groups = located_cell_groups_option.unwrap();
        assert_eq!(1, located_cell_groups.len());
        assert_eq!(0, located_cell_groups[0].cell_group_index);
        assert_eq!(&(1, 2), located_cell_groups[0].location.as_ref());
    }

    #[rstest]
    fn two_cell_groups_two_dependencies_validated() {

        time_graph::enable_data_collection(true);

        let mut detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let mut is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let mut cell_group_dependencies: Vec<CellGroupDependency> = Vec::new();

        let cell_groups_total = 4;

        // Stats
        //  3
        //      2022-12-21  0.01s
        //      2022-12-22  0.00s
        //  4
        //      2022-12-21  0.56s
        //      2022-12-22  0.05s
        //  5
        //      2022-12-21  2339.69s
        //      2022-12-22    28.95s

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

        let mut raw_cell_groups: Vec<CellGroup> = Vec::new();
        for index in 0..cell_groups_total {
            let mut cells: Vec<(i32, i32)> = Vec::new();
            for width_index in 0..=index as i32 {
                for height_index in 0..=index as i32 {
                    cells.push((width_index, height_index));
                }
            }
            raw_cell_groups.push(CellGroup {
                cell_group_type_index: 0,
                cells: cells
            });
        }
        let cell_groups: Rc<Vec<CellGroup>> = Rc::new(raw_cell_groups);

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
    
            let mut element_indexers: Vec<Rc<RefCell<dyn ElementIndexer<T = (i32, i32)>>>> = Vec::new();
            for cell_group_index in 0..cell_groups_total {
                let cell_group_size = cell_group_index + 1;
                let mut locations: Vec<(i32, i32)> = Vec::new();
                for height_index in 0..(area_height - (cell_group_size - 1)) as i32 {
                    for width_index in 0..(area_width - (cell_group_size - 1)) as i32 {
                        let location = (width_index, height_index);
                        debug!("cell group {:?} can exist at location {:?}", cell_group_index, location);
                        locations.push(location);
                    }
                }
                element_indexers.push(Rc::new(RefCell::new(IndexIncrementerElementIndexer::new(vec![locations]))));
            }
            
            cell_group_dependencies.push(CellGroupDependency {
                cell_group_index_mapping: cell_group_index_mapping,
                element_indexer_incrementer: ElementIndexerIncrementer::new(element_indexers)
            });
        }

        let cell_group_dependency_incrementer = CellGroupDependencyIncrementer::new(cell_groups.clone(), detection_offsets_per_cell_group_type_index_per_cell_group_type_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_dependencies);

        let validating_start_time = Instant::now();

        let located_cell_groups_collection = cell_group_dependency_incrementer.into_iter().collect::<Vec<Vec<LocatedCellGroup>>>();

        println!("validation time: {:?}", validating_start_time.elapsed());
        println!("validated: {:?}", located_cell_groups_collection.len());

        println!("{}", time_graph::get_full_graph().as_dot());

        // https://en.wikipedia.org/wiki/Square_pyramidal_number
        let cells_total: usize = (cell_groups_total * (cell_groups_total + 1) * (2 * cell_groups_total + 1)) / 6;
        let mut permutations: BTreeSet<Vec<Vec<String>>> = BTreeSet::new();

        for located_cell_groups in located_cell_groups_collection.iter() {
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
            for located_cell_group in located_cell_groups.iter() {
                for cell in cell_groups[located_cell_group.cell_group_index].cells.iter() {
                    let width_index = (cell.0 + located_cell_group.location.0) as usize;
                    let height_index = (cell.1 + located_cell_group.location.1) as usize;
                    if !pixels[width_index][height_index] {
                        valid_pixels_total += 1;
                    }
                    pixels[width_index][height_index] = true;
                    pixels_as_ids[width_index][height_index] = located_cell_group.cell_group_index.to_string();
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

    #[rstest]
    fn simple_level_example() {

        enum CellGroupType {
            Wall,
            WallAdjacent,
            Floater
        }

        let mut cell_groups: Vec<CellGroup> = Vec::new();
        let detection_offsets_per_cell_group_type_index_per_cell_group_type_index: Vec<Vec<Vec<(i32, i32)>>> = Vec::new();
        let is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        let mut cell_group_dependencies: Vec<CellGroupDependency> = Vec::new();

        let cell_group_dependency_incrementer = CellGroupDependencyIncrementer::new(Rc::new(cell_groups), detection_offsets_per_cell_group_type_index_per_cell_group_type_index, is_adjacent_cell_group_index_per_cell_group_index, cell_group_dependencies);

        let located_cell_groups_collection = cell_group_dependency_incrementer.into_iter().collect::<Vec<Vec<LocatedCellGroup>>>();

        // TODO
    }
}