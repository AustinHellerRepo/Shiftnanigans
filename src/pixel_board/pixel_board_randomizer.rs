use std::{rc::Rc, cell::RefCell, collections::{BTreeSet, BTreeMap}};
use bitvec::vec::BitVec;
use itertools::Itertools;
use crate::{CellGroup, incrementer::{round_robin_incrementer::RoundRobinIncrementer, Incrementer, shifting_cell_group_dependency_incrementer::{self, CellGroupDependency, ShiftingCellGroupDependencyIncrementer}, shifter_incrementer::ShifterIncrementer}, shifter::{Shifter, segment_permutation_shifter::{Segment, SegmentPermutationShifter}, index_shifter::IndexShifter, combined_shifter::CombinedShifter, shifting_square_breadth_first_search_shifter::ShiftingSquareBreadthFirstSearchShifter, hyper_graph_cliche_shifter::{StatefulHyperGraphNode, HyperGraphClicheShifter}}};
use super::{PixelBoard, Pixel};

// TODO construct an undirected graph, search the graph starting with one of the newest edges, doing a depth-first search starting with the newest edge, only permitting the next node to be a cell group not yet traveled to and a location not yet traveled to.
//          add each new edge one at a time, performing the search per new edge.


pub struct PixelBoardRandomizer<TPixel: Pixel> {
    pixel_board: PixelBoard<TPixel>,
    cell_groups: Rc<Vec<CellGroup>>,
    pixel_board_coordinate_per_cell_group_index: Vec<(usize, usize)>,
    top_left_corner_wall_cell_group_index: Option<usize>,
    top_right_corner_wall_cell_group_index: Option<usize>,
    bottom_left_corner_wall_cell_group_index: Option<usize>,
    bottom_right_corner_wall_cell_group_index: Option<usize>,
    top_left_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>>,
    top_right_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>>,
    bottom_right_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>>,
    bottom_left_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>>,
    top_wall_segment_cell_group_indexes: Vec<usize>,
    right_wall_segment_cell_group_indexes: Vec<usize>,
    bottom_wall_segment_cell_group_indexes: Vec<usize>,
    left_wall_segment_cell_group_indexes: Vec<usize>,
    top_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter>,
    right_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter>,
    bottom_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter>,
    left_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter>,
    wall_adjacent_cell_group_indexes: Vec<usize>,
    wall_adjacent_index_shifters: Vec<IndexShifter<(u8, u8)>>,
    detection_offsets_per_cell_group_index_per_cell_group_index: Rc<Vec<Vec<Vec<(i16, i16)>>>>,
    is_adjacent_cell_group_index_per_cell_group_index: Rc<Vec<BitVec>>
}

impl<TPixel: Pixel> PixelBoardRandomizer<TPixel> {
    pub fn new(pixel_board: PixelBoard<TPixel>) -> Self {
        let mut raw_cell_groups: Vec<CellGroup> = Vec::new();
        // contains the pixel board coordinates that map to which cell group
        // useful for creating the random pixel board instance, copying the exact TPixel value from this instance at the same cell location + coordinate
        let mut pixel_board_coordinate_per_cell_group_index: Vec<(usize, usize)> = Vec::new();
        // TODO identify each cell group (wall, wall-adjacent, and floater)

        // contains the cell group indexes for each potential corner wall
        let mut top_left_corner_wall_cell_group_index: Option<usize> = None;
        let mut top_right_corner_wall_cell_group_index: Option<usize> = None;
        let mut bottom_left_corner_wall_cell_group_index: Option<usize> = None;
        let mut bottom_right_corner_wall_cell_group_index: Option<usize> = None;
        let mut top_left_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
        let mut top_right_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
        let mut bottom_right_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
        let mut bottom_left_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;

        let mut top_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
        let mut right_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
        let mut bottom_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
        let mut left_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
        let mut top_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;
        let mut right_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;
        let mut bottom_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;
        let mut left_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;

        let mut wall_adjacent_cell_group_indexes: Vec<usize> = Vec::new();
        let mut wall_adjacent_index_shifters: Vec<IndexShifter<(u8, u8)>> = Vec::new();

        let mut detection_offsets_per_cell_group_index_per_cell_group_index: Vec<Vec<Vec<(i16, i16)>>> = Vec::new();
        // TODO fill detection offsets based on TPixel information
        let mut is_adjacent_cell_group_index_per_cell_group_index: Vec<BitVec> = Vec::new();
        // TODO fill is_adjacent based on wall-adjacent identification

        {
            let mut adjacent_pixel_board_coordinates_per_cell_group_index: Vec<BTreeSet<(usize, usize)>> = Vec::new();

            let rightmost_x: usize = pixel_board.width - 1;
            let bottommost_y: usize = pixel_board.height - 1;

            // construct the cell group for the top left wall corner
            if pixel_board.exists(0, 0) {
                let mut cells: Vec<(u8, u8)> = vec![(0, 0)];
                let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                top_left_corner_wall_cell_group_index = Some(0);
                'clockwise_collecting: {
                    for x in 1..pixel_board.width {
                        if pixel_board.exists(x, 0) {
                            cells.push((x as u8, 0));
                            if x != rightmost_x {
                                adjacent_pixel_board_coordinates.insert((x, 1));
                            }
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    top_right_corner_wall_cell_group_index = Some(0);
                    for y in 1..pixel_board.height {
                        if pixel_board.exists(rightmost_x, y) {
                            cells.push((rightmost_x as u8, y as u8));
                            if y != bottommost_y {
                                adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                            }
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_right_corner_wall_cell_group_index = Some(0);
                    for x in (0..rightmost_x).rev() {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            if x != 0 {
                                adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                            }
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_left_corner_wall_cell_group_index = Some(0);
                    for y in (1..pixel_board.height).rev() {
                        if pixel_board.exists(0, y) {
                            cells.push((0, y as u8));
                            adjacent_pixel_board_coordinates.insert((1, y));
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    // at this point the entire perimeter is a wall
                }
                'counterclockwise_collecting: {
                    for y in 1..pixel_board.height {
                        if pixel_board.exists(0, y) {
                            cells.push((0, y as u8));
                            if y != bottommost_y {
                                adjacent_pixel_board_coordinates.insert((1, y));
                            }
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    bottom_left_corner_wall_cell_group_index = Some(0);
                    for x in 1..pixel_board.width {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            if x != rightmost_x {
                                adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                            }
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    bottom_right_corner_wall_cell_group_index = Some(0);
                    for y in (0..bottommost_y).rev() {
                        if pixel_board.exists(rightmost_x, y) {
                            cells.push((rightmost_x as u8, y as u8));
                            if y != 0 {
                                adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                            }
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    top_right_corner_wall_cell_group_index = Some(0);
                    for x in (2..rightmost_x).rev() {
                        if pixel_board.exists(x, 0) {
                            cells.push((x as u8, 0));
                            adjacent_pixel_board_coordinates.insert((x, 1));
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    // at this point the perimeter (excluding the pixel at (1, 0)) is an entire wall
                }
                raw_cell_groups.push(CellGroup {
                    cells: cells
                });
                adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                pixel_board_coordinate_per_cell_group_index.push((0, 0));
                top_left_corner_wall_index_shifter_option = Some(IndexShifter::new(&vec![
                    vec![Rc::new((0, 0))]
                ]));
            }

            // construct the cell group for the top right wall corner
            if top_right_corner_wall_cell_group_index.is_none() && pixel_board.exists(rightmost_x, 0) {
                let mut leftmost_cell_x: usize = rightmost_x;
                let mut cells: Vec<(u8, u8)> = vec![(rightmost_x as u8, 0)];
                let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                let cell_group_index = raw_cell_groups.len();
                top_right_corner_wall_cell_group_index = Some(cell_group_index);
                'clockwise_collecting: {
                    for y in 1..pixel_board.height {
                        if pixel_board.exists(rightmost_x, y) {
                            cells.push((rightmost_x as u8, y as u8));
                            if y != bottommost_y {
                                adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                            }
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_right_corner_wall_cell_group_index = Some(cell_group_index);
                    for x in (0..rightmost_x).rev() {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            if x != 0 {
                                adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                            }
                            leftmost_cell_x = x;
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_left_corner_wall_cell_group_index = Some(cell_group_index);
                    for y in (1..bottommost_y).rev() {
                        if pixel_board.exists(0, y) {
                            cells.push((0, y as u8));
                            adjacent_pixel_board_coordinates.insert((1, y));
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    // at this point a large U-shaped perimeter wall was discovered that does not include the point (0, 0)
                    // the top left cannot be absorbed by this process because the top right would have already been absorbed
                }
                'counterclockwise_collecting: {
                    for x in (1..rightmost_x).rev() {
                        if pixel_board.exists(x, 0) {
                            cells.push((x as u8, 0));
                            adjacent_pixel_board_coordinates.insert((x, 1));
                            if x < leftmost_cell_x {
                                leftmost_cell_x = x;
                            }
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    // at this point the top wall contains everything except the point at (0, 0)
                }
                raw_cell_groups.push(CellGroup {
                    cells: cells
                });
                adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_x, 0));
                top_right_corner_wall_index_shifter_option = Some(IndexShifter::new(&vec![
                    vec![Rc::new((leftmost_cell_x as u8, 0))]
                ]));
            }
            
            // construct the cell group for the bottom right wall corner
            if bottom_right_corner_wall_cell_group_index.is_none() && pixel_board.exists(rightmost_x, bottommost_y) {
                let mut leftmost_cell_x: usize = rightmost_x;
                let mut topmost_cell_y: usize = bottommost_y;
                let mut cells: Vec<(u8, u8)> = vec![(rightmost_x as u8, bottommost_y as u8)];
                let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                let cell_group_index = raw_cell_groups.len();
                bottom_right_corner_wall_cell_group_index = Some(cell_group_index);
                'clockwise_collecting: {
                    for x in (0..rightmost_x).rev() {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            if x != 0 {
                                adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                            }
                            leftmost_cell_x = x;
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_left_corner_wall_cell_group_index = Some(cell_group_index);
                    for y in (1..bottommost_y).rev() {
                        if pixel_board.exists(0, y) {
                            cells.push((0, y as u8));
                            adjacent_pixel_board_coordinates.insert((1, y));
                            topmost_cell_y = y;
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    // at this point the bottom and left wall are collected except point (0, 0)
                }
                'counterclockwise_collecting: {
                    for y in (1..bottommost_y).rev() {
                        if pixel_board.exists(rightmost_x, y) {
                            cells.push((rightmost_x as u8, y as u8));
                            adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                            if y < topmost_cell_y {
                                topmost_cell_y = y;
                            }
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    // at this point the right wall was collected except top-right point
                }
                raw_cell_groups.push(CellGroup {
                    cells: cells
                });
                adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_x, topmost_cell_y));
                bottom_right_corner_wall_index_shifter_option = Some(IndexShifter::new(&vec![
                    vec![Rc::new((leftmost_cell_x as u8, topmost_cell_y as u8))]
                ]));
            }
            
            // construct the cell group for the bottom left wall corner
            if bottom_left_corner_wall_cell_group_index.is_none() && pixel_board.exists(0, bottommost_y) {
                let mut topmost_cell_y: usize = bottommost_y;
                let mut cells: Vec<(u8, u8)> = vec![(0, bottommost_y as u8)];
                let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                let cell_group_index = raw_cell_groups.len();
                bottom_left_corner_wall_cell_group_index = Some(cell_group_index);
                'clockwise_collecting: {
                    for y in (1..bottommost_y).rev() {
                        if pixel_board.exists(0, y) {
                            cells.push((0, y as u8));
                            adjacent_pixel_board_coordinates.insert((1, y));
                            topmost_cell_y = y;
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    // at this point the left wall was collected except the point (0, 0)
                }
                'counterclockwise_collecting: {
                    for x in 1..rightmost_x {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    // at this point the bottom wall was collected except for the bottom-right point
                }
                raw_cell_groups.push(CellGroup {
                    cells: cells
                });
                adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                pixel_board_coordinate_per_cell_group_index.push((0, topmost_cell_y));
                bottom_left_corner_wall_index_shifter_option = Some(IndexShifter::new(&vec![
                    vec![Rc::new((0, topmost_cell_y as u8))]
                ]));
            }

            // collect the wall segments per wall side

            {
                // collect the top wall segments
                let mut leftmost_wall_x: usize = rightmost_x;
                let mut rightmost_wall_x: usize = 0;
                'leftmost_search: {
                    let mut is_left_gap_found = false;
                    for x in 0..pixel_board.width {
                        if !is_left_gap_found && !pixel_board.exists(x, 0) {
                            is_left_gap_found = true;
                        }
                        else if is_left_gap_found && pixel_board.exists(x, 0) {
                            leftmost_wall_x = x;
                            break 'leftmost_search;
                        }
                    }
                }
                'rightmost_search: {
                    let mut is_right_gap_found = false;
                    for x in (0..pixel_board.width).rev() {
                        if !is_right_gap_found && !pixel_board.exists(x, 0) {
                            is_right_gap_found = true;
                        }
                        else if is_right_gap_found && pixel_board.exists(x, 0) {
                            rightmost_wall_x = x;
                            break 'rightmost_search;
                        }
                    }
                }
                if leftmost_wall_x <= rightmost_wall_x {
                    let mut cells: Vec<(u8, u8)> = Vec::new();
                    let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                    let mut segments: Vec<Rc<Segment>> = Vec::new();
                    let mut current_segment_length: usize = 0;
                    let mut leftmost_cell_x: Option<usize> = None;
                    for x in leftmost_wall_x..=rightmost_wall_x {
                        if pixel_board.exists(x, 0) {
                            current_segment_length += 1;
                            cells.push((x as u8, 0));
                            adjacent_pixel_board_coordinates.insert((x, 1));
                            if leftmost_cell_x.is_none() {
                                leftmost_cell_x = Some(x);
                            }
                        }
                        else if current_segment_length != 0 {
                            top_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_x.unwrap(), 0));
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                            leftmost_cell_x = None;
                        }
                    }
                    if current_segment_length != 0 {
                        top_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                        pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_x.unwrap(), 0));
                    }
                    // find bounding length
                    let mut leftmost_bounding_x: Option<usize> = None;
                    for x in 1..=leftmost_wall_x {
                        if !pixel_board.exists(x - 1, 0) {
                            leftmost_bounding_x = Some(x);
                            break;
                        }
                    }
                    if leftmost_bounding_x.is_none() {
                        panic!("Failed to find left-most x bounding length point for bottom wall segments.");
                    }
                    let mut rightmost_bounding_x: Option<usize> = None;
                    for x in (rightmost_wall_x..rightmost_x).rev() {
                        if !pixel_board.exists(x + 1, 0) {
                            rightmost_bounding_x = Some(x);
                            break;
                        }
                    }
                    if rightmost_bounding_x.is_none() {
                        panic!("Failed to find right-most x bounding length point for bottom wall segments.");
                    }
                    let top_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (leftmost_wall_x as u8, 0), rightmost_bounding_x.unwrap() - leftmost_bounding_x.unwrap() + 1, true, 1, false);
                    top_wall_segment_permutation_shifter_option = Some(top_wall_segment_permutation_shifter);
                }
            }

            // TODO add leftmost_cell_x implementation just like what is done in the scope above

            {
                // collect the bottom wall segments
                let mut leftmost_wall_x: usize = rightmost_x;
                let mut rightmost_wall_x: usize = 0;
                'leftmost_search: {
                    let mut is_left_gap_found = false;
                    for x in 0..pixel_board.width {
                        if !is_left_gap_found && !pixel_board.exists(x, bottommost_y) {
                            is_left_gap_found = true;
                        }
                        else if is_left_gap_found && pixel_board.exists(x, bottommost_y) {
                            leftmost_wall_x = x;
                            break 'leftmost_search;
                        }
                    }
                }
                'rightmost_search: {
                    let mut is_right_gap_found = false;
                    for x in (0..pixel_board.width).rev() {
                        if !is_right_gap_found && !pixel_board.exists(x, bottommost_y) {
                            is_right_gap_found = true;
                        }
                        else if is_right_gap_found && pixel_board.exists(x, bottommost_y) {
                            rightmost_wall_x = x;
                            break 'rightmost_search;
                        }
                    }
                }
                if leftmost_wall_x <= rightmost_wall_x {
                    let mut cells: Vec<(u8, u8)> = Vec::new();
                    let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                    let mut segments: Vec<Rc<Segment>> = Vec::new();
                    let mut current_segment_length: usize = 0;
                    let mut leftmost_cell_x: Option<usize> = None;
                    for x in leftmost_wall_x..=rightmost_wall_x {
                        if pixel_board.exists(x, bottommost_y) {
                            current_segment_length += 1;
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                            if leftmost_cell_x.is_none() {
                                leftmost_cell_x = Some(x);
                            }
                        }
                        else if current_segment_length != 0 {
                            bottom_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_x.unwrap(), bottommost_y));
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                            leftmost_cell_x = None;
                        }
                    }
                    if current_segment_length != 0 {
                        bottom_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                        pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_x.unwrap(), bottommost_y));
                    }
                    // find bounding length
                    let mut leftmost_bounding_x: Option<usize> = None;
                    for x in 1..=leftmost_wall_x {
                        if !pixel_board.exists(x - 1, bottommost_y) {
                            leftmost_bounding_x = Some(x);
                            break;
                        }
                    }
                    if leftmost_bounding_x.is_none() {
                        panic!("Failed to find left-most x bounding length point for bottom wall segments.");
                    }
                    let mut rightmost_bounding_x: Option<usize> = None;
                    for x in (rightmost_wall_x..rightmost_x).rev() {
                        if !pixel_board.exists(x + 1, bottommost_y) {
                            rightmost_bounding_x = Some(x);
                            break;
                        }
                    }
                    if rightmost_bounding_x.is_none() {
                        panic!("Failed to find right-most x bounding length point for bottom wall segments.");
                    }
                    let bottom_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (leftmost_wall_x as u8, bottommost_y as u8), rightmost_bounding_x.unwrap() - leftmost_bounding_x.unwrap() + 1, true, 1, false);
                    bottom_wall_segment_permutation_shifter_option = Some(bottom_wall_segment_permutation_shifter);
                }
            }

            {
                // collect the left wall segments
                let mut topmost_wall_y: usize = bottommost_y;
                let mut bottommost_wall_y: usize = 0;
                'topmost_search: {
                    let mut is_top_gap_found = false;
                    for y in 0..pixel_board.height {
                        if !is_top_gap_found && !pixel_board.exists(0, y) {
                            is_top_gap_found = true;
                        }
                        else if is_top_gap_found && pixel_board.exists(0, y) {
                            topmost_wall_y = y;
                            break 'topmost_search;
                        }
                    }
                }
                'bottommost_search: {
                    let mut is_bottom_gap_found = false;
                    for y in (0..pixel_board.height).rev() {
                        if !is_bottom_gap_found && !pixel_board.exists(0, y) {
                            is_bottom_gap_found = true;
                        }
                        else if is_bottom_gap_found && pixel_board.exists(0, y) {
                            bottommost_wall_y = y;
                            break 'bottommost_search;
                        }
                    }
                }
                if topmost_wall_y <= bottommost_wall_y {
                    let mut cells: Vec<(u8, u8)> = Vec::new();
                    let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                    let mut segments: Vec<Rc<Segment>> = Vec::new();
                    let mut current_segment_length: usize = 0;
                    let mut topmost_cell_y: Option<usize> = None;
                    for y in topmost_wall_y..=bottommost_wall_y {
                        if pixel_board.exists(0, y) {
                            current_segment_length += 1;
                            cells.push((0, y as u8));
                            adjacent_pixel_board_coordinates.insert((1, y));
                            if topmost_cell_y.is_none() {
                                topmost_cell_y = Some(y);
                            }
                        }
                        else if current_segment_length != 0 {
                            left_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            pixel_board_coordinate_per_cell_group_index.push((0, topmost_cell_y.unwrap()));
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                            topmost_cell_y = None;
                        }
                    }
                    if current_segment_length != 0 {
                        left_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                        pixel_board_coordinate_per_cell_group_index.push((0, topmost_cell_y.unwrap()));
                    }
                    // find bounding length
                    let mut topmost_bounding_y: Option<usize> = None;
                    for y in 1..=topmost_wall_y {
                        if !pixel_board.exists(0, y - 1) {
                            topmost_bounding_y = Some(y);
                            break;
                        }
                    }
                    if topmost_bounding_y.is_none() {
                        panic!("Failed to find top-most y bounding length point for left wall segments.");
                    }
                    let mut bottommost_bounding_y: Option<usize> = None;
                    for y in (bottommost_wall_y..bottommost_y).rev() {
                        if !pixel_board.exists(0, y + 1) {
                            bottommost_bounding_y = Some(y);
                            break;
                        }
                    }
                    if bottommost_bounding_y.is_none() {
                        panic!("Failed to find bottom-most y bounding length point for left wall segments.");
                    }
                    let left_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (0, topmost_wall_y as u8), bottommost_bounding_y.unwrap() - topmost_bounding_y.unwrap() + 1, false, 1, false);
                    left_wall_segment_permutation_shifter_option = Some(left_wall_segment_permutation_shifter);
                }
            }

            {
                // collect the right wall segments
                let mut topmost_wall_y: usize = bottommost_y;
                let mut bottommost_wall_y: usize = 0;
                'topmost_search: {
                    let mut is_top_gap_found = false;
                    for y in 0..pixel_board.height {
                        if !is_top_gap_found && !pixel_board.exists(rightmost_x, y) {
                            is_top_gap_found = true;
                        }
                        else if is_top_gap_found && pixel_board.exists(rightmost_x, y) {
                            topmost_wall_y = y;
                            break 'topmost_search;
                        }
                    }
                }
                'bottommost_search: {
                    let mut is_bottom_gap_found = false;
                    for y in (0..pixel_board.height).rev() {
                        if !is_bottom_gap_found && !pixel_board.exists(rightmost_x, y) {
                            is_bottom_gap_found = true;
                        }
                        else if is_bottom_gap_found && pixel_board.exists(rightmost_x, y) {
                            bottommost_wall_y = y;
                            break 'bottommost_search;
                        }
                    }
                }
                if topmost_wall_y <= bottommost_wall_y {
                    let mut cells: Vec<(u8, u8)> = Vec::new();
                    let mut adjacent_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();
                    let mut segments: Vec<Rc<Segment>> = Vec::new();
                    let mut current_segment_length: usize = 0;
                    let mut topmost_cell_y: Option<usize> = None;
                    for y in topmost_wall_y..=bottommost_wall_y {
                        if pixel_board.exists(rightmost_x, y) {
                            current_segment_length += 1;
                            cells.push((rightmost_x as u8, y as u8));
                            adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                            if topmost_cell_y.is_none() {
                                topmost_cell_y = Some(y);
                            }
                        }
                        else if current_segment_length != 0 {
                            right_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            pixel_board_coordinate_per_cell_group_index.push((rightmost_x, topmost_cell_y.unwrap()));
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                            topmost_cell_y = None;
                        }
                    }
                    if current_segment_length != 0 {
                        right_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                        pixel_board_coordinate_per_cell_group_index.push((rightmost_x, topmost_cell_y.unwrap()));
                    }
                    // find bounding length
                    let mut topmost_bounding_y: Option<usize> = None;
                    for y in 1..=topmost_wall_y {
                        if !pixel_board.exists(rightmost_x, y - 1) {
                            topmost_bounding_y = Some(y);
                            break;
                        }
                    }
                    if topmost_bounding_y.is_none() {
                        panic!("Failed to find top-most y bounding length point for right wall segments.");
                    }
                    let mut bottommost_bounding_y: Option<usize> = None;
                    for y in (bottommost_wall_y..bottommost_y).rev() {
                        if !pixel_board.exists(rightmost_x, y + 1) {
                            bottommost_bounding_y = Some(y);
                            break;
                        }
                    }
                    if bottommost_bounding_y.is_none() {
                        panic!("Failed to find bottom-most y bounding length point for right wall segments.");
                    }
                    let right_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (rightmost_x as u8, topmost_wall_y as u8), bottommost_bounding_y.unwrap() - topmost_bounding_y.unwrap() + 1, false, 1, false);
                    right_wall_segment_permutation_shifter_option = Some(right_wall_segment_permutation_shifter);
                }
            }

            // at this point the corner walls and the wall segments have been discovered

            // collect all wall adjacents
            {
                // contains all of the pixel board index pairs
                let mut visited_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();

                // contains the cell group indexes which are adjacent per cell group index
                let mut adjacent_cell_group_indexes_per_cell_group_index: Vec<Vec<usize>> = Vec::new();
                let mut wall_adjacent_cell_group_index_offset_option: Option<usize> = None;

                // search all walls for wall-adjacents
                {
                    let mut wall_cell_group_indexes: Vec<usize> = Vec::new();
                    if top_left_corner_wall_cell_group_index.is_some() {
                        wall_cell_group_indexes.push(top_left_corner_wall_cell_group_index.unwrap());
                    }
                    if top_right_corner_wall_cell_group_index.is_some() {
                        wall_cell_group_indexes.push(top_right_corner_wall_cell_group_index.unwrap());
                    }
                    if bottom_right_corner_wall_cell_group_index.is_some() {
                        wall_cell_group_indexes.push(bottom_right_corner_wall_cell_group_index.unwrap());
                    }
                    if bottom_left_corner_wall_cell_group_index.is_some() {
                        wall_cell_group_indexes.push(bottom_left_corner_wall_cell_group_index.unwrap());
                    }
                    wall_cell_group_indexes.extend(&top_wall_segment_cell_group_indexes);
                    wall_cell_group_indexes.extend(&right_wall_segment_cell_group_indexes);
                    wall_cell_group_indexes.extend(&bottom_wall_segment_cell_group_indexes);
                    wall_cell_group_indexes.extend(&left_wall_segment_cell_group_indexes);

                    let mut location_references: Vec<Rc<(u8, u8)>> = Vec::new();
                    for y in 1..bottommost_y as u8 {
                        for x in 1..rightmost_x as u8 {
                            location_references.push(Rc::new((x, y)));
                        }
                    }

                    if rightmost_x > 1 && bottommost_y > 1 {
                        let location_references_width = rightmost_x - 1;

                        // TODO incorporate adjacent vector to determining which cell group indexes are adjacent to each wall-adjacent as they are being constructed

                        for y in 1..bottommost_y {
                            for x in 1..rightmost_x {
                                let pixel_board_coordinate: (usize, usize) = (x, y);
                                if pixel_board.exists(pixel_board_coordinate.0, pixel_board_coordinate.1) && !visited_pixel_board_coordinates.contains(&pixel_board_coordinate) {
                                    let mut cells: Vec<(u8, u8)> = Vec::new();
                                    let mut topmost_cell_group_y: usize = bottommost_y;
                                    let mut bottommost_cell_group_y: usize = 0;
                                    let mut leftmost_cell_group_x: usize = rightmost_x;
                                    let mut rightmost_cell_group_x: usize = 0;
                                    let mut adjacent_wall_cell_group_indexes: Vec<usize> = Vec::new();
                                    let mut possible_cell_at_pixel_board_coordinates: Vec<(usize, usize)> = vec![pixel_board_coordinate];
                                    while !possible_cell_at_pixel_board_coordinates.is_empty() {
                                        let cell_pixel_board_coordinate = possible_cell_at_pixel_board_coordinates.pop().unwrap();
                                        visited_pixel_board_coordinates.insert(cell_pixel_board_coordinate);
                                        // check to see if the top-left can be shifted up or left
                                        if cell_pixel_board_coordinate.0 < leftmost_cell_group_x {
                                            leftmost_cell_group_x = cell_pixel_board_coordinate.0;
                                        }
                                        if cell_pixel_board_coordinate.0 > rightmost_cell_group_x {
                                            rightmost_cell_group_x = cell_pixel_board_coordinate.0;
                                        }
                                        if cell_pixel_board_coordinate.1 < topmost_cell_group_y {
                                            topmost_cell_group_y = cell_pixel_board_coordinate.1;
                                        }
                                        if cell_pixel_board_coordinate.1 > bottommost_cell_group_y {
                                            bottommost_cell_group_y = cell_pixel_board_coordinate.1;
                                        }
                                        // check if there are any wall indexes this cell is adjacent to
                                        for wall_cell_group_index in wall_cell_group_indexes.iter() {
                                            if adjacent_pixel_board_coordinates_per_cell_group_index[*wall_cell_group_index].contains(&cell_pixel_board_coordinate) && !adjacent_wall_cell_group_indexes.contains(wall_cell_group_index) {
                                                adjacent_wall_cell_group_indexes.push(*wall_cell_group_index);
                                            }
                                        }
                                        let cell = (cell_pixel_board_coordinate.0 as u8, cell_pixel_board_coordinate.1 as u8);
                                        cells.push(cell);
                                        if cell_pixel_board_coordinate.0 > 1 {
                                            let next_pixel_board_coordinate = (cell_pixel_board_coordinate.0 - 1, cell_pixel_board_coordinate.1);
                                            if pixel_board.exists(next_pixel_board_coordinate.0, next_pixel_board_coordinate.1) && !visited_pixel_board_coordinates.contains(&next_pixel_board_coordinate) && !possible_cell_at_pixel_board_coordinates.contains(&next_pixel_board_coordinate) {
                                                possible_cell_at_pixel_board_coordinates.push(next_pixel_board_coordinate);
                                            }
                                        }
                                        if cell_pixel_board_coordinate.1 > 1 {
                                            let next_pixel_board_coordinate = (cell_pixel_board_coordinate.0, cell_pixel_board_coordinate.1 - 1);
                                            if pixel_board.exists(next_pixel_board_coordinate.0, next_pixel_board_coordinate.1) && !visited_pixel_board_coordinates.contains(&next_pixel_board_coordinate) && !possible_cell_at_pixel_board_coordinates.contains(&next_pixel_board_coordinate) {
                                                possible_cell_at_pixel_board_coordinates.push(next_pixel_board_coordinate);
                                            }
                                        }
                                        if cell_pixel_board_coordinate.0 < rightmost_x - 1 {
                                            let next_pixel_board_coordinate = (cell_pixel_board_coordinate.0 + 1, cell_pixel_board_coordinate.1);
                                            if pixel_board.exists(next_pixel_board_coordinate.0, next_pixel_board_coordinate.1) && !visited_pixel_board_coordinates.contains(&next_pixel_board_coordinate) && !possible_cell_at_pixel_board_coordinates.contains(&next_pixel_board_coordinate) {
                                                possible_cell_at_pixel_board_coordinates.push(next_pixel_board_coordinate);
                                            }
                                        }
                                        if cell_pixel_board_coordinate.1 < bottommost_y - 1 {
                                            let next_pixel_board_coordinate = (cell_pixel_board_coordinate.0, cell_pixel_board_coordinate.1 + 1);
                                            if pixel_board.exists(next_pixel_board_coordinate.0, next_pixel_board_coordinate.1) && !visited_pixel_board_coordinates.contains(&next_pixel_board_coordinate) && !possible_cell_at_pixel_board_coordinates.contains(&next_pixel_board_coordinate) {
                                                possible_cell_at_pixel_board_coordinates.push(next_pixel_board_coordinate);
                                            }
                                        }
                                    }
                                    // at this point there are one or more cells found
                                    if wall_adjacent_cell_group_index_offset_option.is_none() {
                                        wall_adjacent_cell_group_index_offset_option = Some(raw_cell_groups.len());
                                    }
                                    wall_adjacent_cell_group_indexes.push(raw_cell_groups.len());
                                    raw_cell_groups.push(CellGroup {
                                        cells: cells
                                    });
                                    adjacent_cell_group_indexes_per_cell_group_index.push(adjacent_wall_cell_group_indexes.clone());
                                    adjacent_wall_cell_group_indexes.clear();
                                    pixel_board_coordinate_per_cell_group_index.push((leftmost_cell_group_x, topmost_cell_group_y));
                                    
                                    // construct index shifter
                                    let mut states: Vec<Rc<(u8, u8)>> = Vec::new();
                                    let cell_group_width = rightmost_cell_group_x - leftmost_cell_group_x + 1;
                                    let cell_group_height = bottommost_cell_group_y - topmost_cell_group_y + 1;
                                    for y in 1..=(bottommost_y - cell_group_height) {
                                        for x in 1..=(rightmost_x - cell_group_width) {
                                            let location_reference_index = (y - 1) * location_references_width + (x - 1);
                                            states.push(location_references[location_reference_index].clone());
                                        }
                                    }
                                    let index_shifter = IndexShifter::new(&vec![states]);
                                    wall_adjacent_index_shifters.push(index_shifter);
                                }
                            }
                        }
                    }
                }

                // at this point all cell groups are known

                for cell_group_index in 0..raw_cell_groups.len() {

                    let pixel_board_coordinate = pixel_board_coordinate_per_cell_group_index[cell_group_index];

                    // construct is_adjacent booleans per cell group pair
                    let mut is_adjacent_per_cell_group_index: BitVec = BitVec::repeat(false, raw_cell_groups.len());
                    if wall_adjacent_cell_group_index_offset_option.is_some() && cell_group_index >= wall_adjacent_cell_group_index_offset_option.unwrap() {
                        for adjacent_cell_group_index in adjacent_cell_group_indexes_per_cell_group_index[cell_group_index - wall_adjacent_cell_group_index_offset_option.unwrap()].iter() {
                            is_adjacent_per_cell_group_index.set(*adjacent_cell_group_index, true);
                            // TODO determine if the mirror reference should be made
                        }
                    }
                    is_adjacent_cell_group_index_per_cell_group_index.push(is_adjacent_per_cell_group_index);

                    // TODO construct detection offsets per cell group pair
                    let mut detection_offsets_per_cell_group_index: Vec<Vec<(i16, i16)>> = Vec::new();
                    for other_cell_group_index in 0..raw_cell_groups.len() {

                        let mut raw_detection_offsets: Vec<(i16, i16)> = Vec::new();

                        // TODO fill detection offsets as needed
                        if other_cell_group_index != cell_group_index {
                            for cell_location in raw_cell_groups[cell_group_index].cells.iter() {
                                if let Some(pixel) = pixel_board.get(cell_location.0 as usize, cell_location.1 as usize) {
                                    let borrowed_pixel: &TPixel = &pixel.borrow();
                                    for other_cell_location in raw_cell_groups[other_cell_group_index].cells.iter() {
                                        // get the invalid location offsets based on the locations of the cells
                                        if let Some(other_pixel) = pixel_board.get(other_cell_location.0 as usize, other_cell_location.1 as usize) {
                                            let borrowed_other_pixel: &TPixel = &other_pixel.borrow();
                                            let invalid_location_offsets = borrowed_pixel.get_invalid_location_offsets_for_other_pixel(borrowed_other_pixel);
                                            for invalid_location_offset in invalid_location_offsets.iter() {
                                                let x = (cell_location.0 as usize - pixel_board_coordinate.0) as i16 + invalid_location_offset.0;
                                                let y = (cell_location.1 as usize - pixel_board_coordinate.1) as i16 + invalid_location_offset.1;
                                                let detection_offset = (x, y);
                                                raw_detection_offsets.push(detection_offset);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let detection_offsets = raw_detection_offsets.into_iter().unique().collect();
                        detection_offsets_per_cell_group_index.push(detection_offsets);
                    }
                    detection_offsets_per_cell_group_index_per_cell_group_index.push(detection_offsets_per_cell_group_index);
                }
            }
        }

        // move raw cell groups to top-left corner
        let mut transformed_cell_groups: Vec<CellGroup> = Vec::new();
        for raw_cell_group in raw_cell_groups {
            let mut left_most_x: Option<u8> = None;
            let mut top_most_y: Option<u8> = None;
            for cell in raw_cell_group.cells.iter() {
                if left_most_x.is_none() || left_most_x.unwrap() > cell.0 {
                    left_most_x = Some(cell.0);
                }
                if top_most_y.is_none() || top_most_y.unwrap() > cell.1 {
                    top_most_y = Some(cell.1);
                }
            }
            let mut cells = Vec::new();
            for cell in raw_cell_group.cells {
                cells.push((cell.0 - left_most_x.unwrap(), cell.1 - top_most_y.unwrap()));
            }
            transformed_cell_groups.push(CellGroup {
                cells: cells
            });
        }

        PixelBoardRandomizer {
            pixel_board: pixel_board,
            cell_groups: Rc::new(transformed_cell_groups),
            pixel_board_coordinate_per_cell_group_index: pixel_board_coordinate_per_cell_group_index,
            top_left_corner_wall_cell_group_index: top_left_corner_wall_cell_group_index,
            top_right_corner_wall_cell_group_index: top_right_corner_wall_cell_group_index,
            bottom_left_corner_wall_cell_group_index: bottom_left_corner_wall_cell_group_index,
            bottom_right_corner_wall_cell_group_index: bottom_right_corner_wall_cell_group_index,
            top_left_corner_wall_index_shifter_option: top_left_corner_wall_index_shifter_option,
            top_right_corner_wall_index_shifter_option: top_right_corner_wall_index_shifter_option,
            bottom_right_corner_wall_index_shifter_option: bottom_right_corner_wall_index_shifter_option,
            bottom_left_corner_wall_index_shifter_option: bottom_left_corner_wall_index_shifter_option,
            top_wall_segment_cell_group_indexes: top_wall_segment_cell_group_indexes,
            right_wall_segment_cell_group_indexes: right_wall_segment_cell_group_indexes,
            bottom_wall_segment_cell_group_indexes: bottom_wall_segment_cell_group_indexes,
            left_wall_segment_cell_group_indexes: left_wall_segment_cell_group_indexes,
            top_wall_segment_permutation_shifter_option: top_wall_segment_permutation_shifter_option,
            right_wall_segment_permutation_shifter_option: right_wall_segment_permutation_shifter_option,
            bottom_wall_segment_permutation_shifter_option: bottom_wall_segment_permutation_shifter_option,
            left_wall_segment_permutation_shifter_option: left_wall_segment_permutation_shifter_option,
            wall_adjacent_cell_group_indexes: wall_adjacent_cell_group_indexes,
            wall_adjacent_index_shifters: wall_adjacent_index_shifters,
            detection_offsets_per_cell_group_index_per_cell_group_index: Rc::new(detection_offsets_per_cell_group_index_per_cell_group_index),
            is_adjacent_cell_group_index_per_cell_group_index: Rc::new(is_adjacent_cell_group_index_per_cell_group_index)
        }
    }
    pub fn get_random_pixel_board(&self) -> PixelBoard<TPixel> {
        let mut round_robin_incrementer: RoundRobinIncrementer<(u8, u8)>;

        {
            // randomize the shifters
            let mut corner_wall_index_shifters: Vec<IndexShifter<(u8, u8)>> = Vec::new();
            let mut corner_wall_cell_group_index_per_shifter: Vec<usize> = Vec::new();
            for (shifter_option, cell_group_index_option) in [
                (self.top_left_corner_wall_index_shifter_option.as_ref(), self.top_left_corner_wall_cell_group_index.as_ref()),
                (self.top_right_corner_wall_index_shifter_option.as_ref(), self.top_right_corner_wall_cell_group_index.as_ref()),
                (self.bottom_right_corner_wall_index_shifter_option.as_ref(), self.bottom_right_corner_wall_cell_group_index.as_ref()),
                (self.bottom_left_corner_wall_index_shifter_option.as_ref(), self.bottom_left_corner_wall_cell_group_index.as_ref())
            ] {
                if shifter_option.is_some() {
                    let mut shifter = shifter_option.unwrap().clone();
                    shifter.randomize();
                    corner_wall_index_shifters.push(shifter);
                    corner_wall_cell_group_index_per_shifter.push(*cell_group_index_option.unwrap());
                }
            }
            let mut wall_segment_permutation_shifters: Vec<SegmentPermutationShifter> = Vec::new();
            let mut wall_segment_cell_group_indexes_per_shifter: Vec<Vec<usize>> = Vec::new();
            for (shifter_option, cell_group_indexes) in [
                (self.top_wall_segment_permutation_shifter_option.as_ref(), self.top_wall_segment_cell_group_indexes.clone()),
                (self.right_wall_segment_permutation_shifter_option.as_ref(), self.right_wall_segment_cell_group_indexes.clone()),
                (self.bottom_wall_segment_permutation_shifter_option.as_ref(), self.bottom_wall_segment_cell_group_indexes.clone()),
                (self.left_wall_segment_permutation_shifter_option.as_ref(), self.left_wall_segment_cell_group_indexes.clone())
            ] {
                if shifter_option.is_some() {
                    let mut shifter = shifter_option.unwrap().clone();
                    shifter.randomize();
                    wall_segment_permutation_shifters.push(shifter);
                    wall_segment_cell_group_indexes_per_shifter.push(cell_group_indexes);
                }
            }
            let mut wall_adjacent_index_shifters: Vec<IndexShifter<(u8, u8)>> = Vec::new();
            let mut wall_adjacent_cell_group_index_per_shifter: Vec<usize> = Vec::new();
            for (index_shifter, cell_group_index) in self.wall_adjacent_index_shifters.iter().zip(self.wall_adjacent_cell_group_indexes.iter()) {
                let mut shifter = index_shifter.clone();
                shifter.randomize();
                wall_adjacent_index_shifters.push(shifter);
                wall_adjacent_cell_group_index_per_shifter.push(*cell_group_index);
            }
            
            // TODO construct each shifting cell group dependency incrementer per pair of shifters
            let mut incrementers: Vec<Rc<RefCell<dyn Incrementer<T = (u8, u8)>>>> = Vec::new();

            // fill the incrementers that will be used by the round-robin
            {
                let mut cell_group_dependencies: Vec<CellGroupDependency> = Vec::new();

                if corner_wall_index_shifters.len() == 0 && wall_segment_permutation_shifters.len() == 0 && wall_adjacent_index_shifters.len() == 0 {
                    todo!();
                }
                else if corner_wall_index_shifters.len() + wall_segment_permutation_shifters.len() + wall_adjacent_index_shifters.len() == 1 {
                    if corner_wall_index_shifters.len() == 1 {
                        let cell_group_dependency = CellGroupDependency::new(corner_wall_cell_group_index_per_shifter, Rc::new(RefCell::new(ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(corner_wall_index_shifters[0].clone()))], true))));
                        cell_group_dependencies.push(cell_group_dependency);
                    }
                    else if wall_segment_permutation_shifters.len() == 1 {
                        let cell_group_dependency = CellGroupDependency::new(wall_segment_cell_group_indexes_per_shifter[0].clone(), Rc::new(RefCell::new(ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(wall_segment_permutation_shifters[0].clone()))], true))));
                        cell_group_dependencies.push(cell_group_dependency);
                    }
                    else if wall_adjacent_index_shifters.len() == 1 {
                        let cell_group_dependency = CellGroupDependency::new(wall_adjacent_cell_group_index_per_shifter, Rc::new(RefCell::new(ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(wall_adjacent_index_shifters[0].clone()))], true))));
                        cell_group_dependencies.push(cell_group_dependency);
                    }
                    else {
                        panic!("Unexpected difference between encapsulating if-statement and split if-statements.");
                    }
                }
                else {

                    // create a combined shifter per pair of corner wall shifters
                    if !corner_wall_index_shifters.is_empty() {
                        for shifter_index in 0..(corner_wall_index_shifters.len() - 1) {
                            for other_shifter_index in (shifter_index + 1)..corner_wall_index_shifters.len() {
                                let shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(corner_wall_index_shifters[shifter_index].clone())), Rc::new(RefCell::new(corner_wall_index_shifters[other_shifter_index].clone()))], true);
                                let combined_cell_group_indexes: Vec<usize> = vec![corner_wall_cell_group_index_per_shifter[shifter_index], corner_wall_cell_group_index_per_shifter[other_shifter_index]];
                                let cell_group_dependency = CellGroupDependency::new(combined_cell_group_indexes, Rc::new(RefCell::new(shifter)));
                                cell_group_dependencies.push(cell_group_dependency);
                            }
                        }
                    }
                    // create a combined shifter per pair of segment wall shifters
                    if !wall_segment_permutation_shifters.is_empty() {
                        for shifter_index in 0..(wall_segment_permutation_shifters.len() - 1) {
                            for other_shifter_index in (shifter_index + 1)..wall_segment_permutation_shifters.len() {
                                let shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(wall_segment_permutation_shifters[shifter_index].clone())), Rc::new(RefCell::new(wall_segment_permutation_shifters[other_shifter_index].clone()))], true);
                                let mut combined_cell_group_indexes: Vec<usize> = Vec::new();
                                for wall_segment_cell_group_index in wall_segment_cell_group_indexes_per_shifter[shifter_index].iter().chain(wall_segment_cell_group_indexes_per_shifter[other_shifter_index].iter()) {
                                    combined_cell_group_indexes.push(*wall_segment_cell_group_index);
                                }
                                let cell_group_dependency = CellGroupDependency::new(combined_cell_group_indexes, Rc::new(RefCell::new(shifter)));
                                cell_group_dependencies.push(cell_group_dependency);
                            }
                        }
                    }
                    // create a combined shifter per pair of non-wall shifters
                    if !wall_adjacent_index_shifters.is_empty() {
                        for shifter_index in 0..(wall_adjacent_index_shifters.len() - 1) {
                            for other_shifter_index in (shifter_index + 1)..wall_adjacent_index_shifters.len() {
                                let shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(wall_adjacent_index_shifters[shifter_index].clone())), Rc::new(RefCell::new(wall_adjacent_index_shifters[other_shifter_index].clone()))], true);
                                let combined_cell_group_indexes: Vec<usize> = vec![wall_adjacent_cell_group_index_per_shifter[shifter_index], wall_adjacent_cell_group_index_per_shifter[other_shifter_index]];
                                let cell_group_dependency = CellGroupDependency::new(combined_cell_group_indexes, Rc::new(RefCell::new(shifter)));
                                cell_group_dependencies.push(cell_group_dependency);
                            }
                        }
                    }
                    // create a combined shifter per corner wall shifter and segment wall shifter pair
                    for corner_wall_shifter_index in 0..corner_wall_index_shifters.len() {
                        for wall_segment_shifter_index in 0..wall_segment_permutation_shifters.len() {
                            let shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(corner_wall_index_shifters[corner_wall_shifter_index].clone())), Rc::new(RefCell::new(wall_segment_permutation_shifters[wall_segment_shifter_index].clone()))], true);
                            let mut combined_cell_group_indexes: Vec<usize> = vec![corner_wall_cell_group_index_per_shifter[corner_wall_shifter_index]];
                            for wall_segment_cell_group_index in wall_segment_cell_group_indexes_per_shifter[wall_segment_shifter_index].iter() {
                                combined_cell_group_indexes.push(*wall_segment_cell_group_index);
                            }
                            let cell_group_dependency = CellGroupDependency::new(combined_cell_group_indexes, Rc::new(RefCell::new(shifter)));
                            cell_group_dependencies.push(cell_group_dependency);
                        }
                    }
                    // create a combined shifter per corner wall shifter and non-wall shifter pair
                    for corner_wall_shifter_index in 0..corner_wall_index_shifters.len() {
                        for wall_adjacent_shifter_index in 0..wall_adjacent_index_shifters.len() {
                            let shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(corner_wall_index_shifters[corner_wall_shifter_index].clone())), Rc::new(RefCell::new(wall_adjacent_index_shifters[wall_adjacent_shifter_index].clone()))], true);
                            let combined_cell_group_indexes: Vec<usize> = vec![corner_wall_cell_group_index_per_shifter[corner_wall_shifter_index], wall_adjacent_cell_group_index_per_shifter[wall_adjacent_shifter_index]];
                            let cell_group_dependency = CellGroupDependency::new(combined_cell_group_indexes, Rc::new(RefCell::new(shifter)));
                            cell_group_dependencies.push(cell_group_dependency);
                        }
                    }
                    // create a combined shifter per segment wall shifter and non-wall shifter pair
                    for wall_segment_shifter_index in 0..wall_segment_permutation_shifters.len() {
                        for wall_adjacent_shifter_index in 0..wall_adjacent_index_shifters.len() {
                            let shifter = ShiftingSquareBreadthFirstSearchShifter::new(vec![Rc::new(RefCell::new(wall_segment_permutation_shifters[wall_segment_shifter_index].clone())), Rc::new(RefCell::new(wall_adjacent_index_shifters[wall_adjacent_shifter_index].clone()))], true);
                            let mut combined_cell_group_indexes: Vec<usize> = Vec::new();
                            for wall_segment_cell_group_index in wall_segment_cell_group_indexes_per_shifter[wall_segment_shifter_index].iter() {
                                combined_cell_group_indexes.push(*wall_segment_cell_group_index);
                            }
                            combined_cell_group_indexes.push(wall_adjacent_cell_group_index_per_shifter[wall_adjacent_shifter_index]);
                            let cell_group_dependency = CellGroupDependency::new(combined_cell_group_indexes, Rc::new(RefCell::new(shifter)));
                            cell_group_dependencies.push(cell_group_dependency);
                        }
                    }
                }

                // create the shifting cell group dependency incrementers
                for cell_group_dependency in cell_group_dependencies {
                    let shifting_cell_group_dependency_incrementer = ShiftingCellGroupDependencyIncrementer::new(self.cell_groups.clone(), vec![cell_group_dependency], self.detection_offsets_per_cell_group_index_per_cell_group_index.clone(), self.is_adjacent_cell_group_index_per_cell_group_index.clone());
                    incrementers.push(Rc::new(RefCell::new(shifting_cell_group_dependency_incrementer)));
                }
            }

            // TODO construct each incrementer that equates to each possible combination of cell groups depending on their location in the bounds
            round_robin_incrementer = RoundRobinIncrementer::new(incrementers);
        }

        // prepare to find the cycle as the RoundRobinIncrementer is iterated over
        // the idea is that we round-robin across all shifters, building up graphs of connected locations until we find that the next pair to be connected already exist in the same graph, then we check for a cycle
        
        // TODO remove these variables as they are no longer used
        // this contains how many times this cell group at this location has been found at this location within the graph at the current index of the vector
        let mut observations_total_per_located_cell_group_collection_per_isolated_graph: Vec<BTreeMap<(usize, (u8, u8)), usize>> = Vec::new();
        let mut first_fully_connected_located_cell_groups_per_isolated_graph: Vec<Vec<(usize, (u8, u8))>> = Vec::new();
        let mut is_cell_group_index_fully_connected_per_isolated_graph: Vec<BitVec> = Vec::new();
        let fully_connected_length = self.cell_groups.len() - 1;

        // contains all of the states discovered thus far
        let mut stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = Vec::new();
        for _ in 0..self.cell_groups.len() {
            stateful_hyper_graph_nodes_per_hyper_graph_node_index.push(Vec::new());
        }

        let mut is_incrementer_completed: bool = false;
        while !is_incrementer_completed {
            // TODO get the next set of locations
            is_incrementer_completed = !round_robin_incrementer.try_increment();
            if !is_incrementer_completed {
                let locations = round_robin_incrementer.get();
                if locations.len() == 1 {
                    if self.cell_groups.len() != 1 {
                        panic!("Unexpected location count of 1 while having more than one cell group.");
                    }
                    let cell_group_index: usize = 0;
                    let location = locations[0].element.as_ref();
                    let mut random_pixel_board: PixelBoard<TPixel> = PixelBoard::new(self.pixel_board.get_width(), self.pixel_board.get_height());
                    for cell in self.cell_groups[cell_group_index].cells.iter() {
                        let calculated_pixel_board_index_x: usize = (location.0 + cell.0) as usize;
                        let calculated_pixel_board_index_y: usize = (location.1 + cell.1) as usize;
                        let pixel_board_coordinate = self.pixel_board_coordinate_per_cell_group_index[cell_group_index];
                        let original_pixel_board_index_x: usize = (cell.0 as usize + pixel_board_coordinate.0);
                        let original_pixel_board_index_y: usize = (cell.1 as usize + pixel_board_coordinate.1);
                        random_pixel_board.set(calculated_pixel_board_index_x, calculated_pixel_board_index_y, self.pixel_board.get(original_pixel_board_index_x, original_pixel_board_index_y).unwrap());
                    }
                    return random_pixel_board;
                }
                else {
                    // TODO treat each pair individually, iterating over each pair
                    for (current_indexed_element_index, current_indexed_element) in locations.iter().enumerate() {
                        for (other_indexed_element_index, other_indexed_element) in locations.iter().enumerate() {
                            if current_indexed_element_index < other_indexed_element_index {
                                let mut current_stateful_hyper_graph_node_option: Option<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>> = None;
                                for stateful_hyper_graph_node in stateful_hyper_graph_nodes_per_hyper_graph_node_index[current_indexed_element.index].iter() {
                                    if stateful_hyper_graph_node.borrow().state == current_indexed_element.element {
                                        current_stateful_hyper_graph_node_option = Some(stateful_hyper_graph_node.clone());
                                        break;
                                    }
                                }
                                if current_stateful_hyper_graph_node_option.is_none() {
                                    let current_stateful_hyper_graph_node = Rc::new(RefCell::new(StatefulHyperGraphNode::new(current_indexed_element.element.clone())));
                                    stateful_hyper_graph_nodes_per_hyper_graph_node_index[current_indexed_element.index].push(current_stateful_hyper_graph_node.clone());
                                    current_stateful_hyper_graph_node_option = Some(current_stateful_hyper_graph_node);
                                }
                                let mut other_stateful_hyper_graph_node_option: Option<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>> = None;
                                for stateful_hyper_graph_node in stateful_hyper_graph_nodes_per_hyper_graph_node_index[other_indexed_element.index].iter() {
                                    if stateful_hyper_graph_node.borrow().state == other_indexed_element.element {
                                        other_stateful_hyper_graph_node_option = Some(stateful_hyper_graph_node.clone());
                                        break;
                                    }
                                }
                                if other_stateful_hyper_graph_node_option.is_none() {
                                    let other_stateful_hyper_graph_node = Rc::new(RefCell::new(StatefulHyperGraphNode::new(other_indexed_element.element.clone())));
                                    stateful_hyper_graph_nodes_per_hyper_graph_node_index[other_indexed_element.index].push(other_stateful_hyper_graph_node.clone());
                                    other_stateful_hyper_graph_node_option = Some(other_stateful_hyper_graph_node);
                                }
                                
                                // set each as neighbors to each other
                                let current_stateful_hyper_graph_node = current_stateful_hyper_graph_node_option.unwrap();
                                let other_stateful_hyper_graph_node = other_stateful_hyper_graph_node_option.unwrap();
                                current_stateful_hyper_graph_node.borrow_mut().add_neighbor(other_indexed_element.index, other_stateful_hyper_graph_node.clone());
                                other_stateful_hyper_graph_node.borrow_mut().add_neighbor(current_indexed_element.index, current_stateful_hyper_graph_node);
                            }

                            if false {  // TODO remove since this logic does not work and was replaced with the HyperGraphClicheShifter
                                // TODO determine if indexed location exists in previous data structures
                                let mut current_indexed_element_isolated_graph_index: Option<usize> = None;
                                let mut other_indexed_element_isolated_graph_index: Option<usize> = None;
                                let current_located_cell_group: (usize, (u8, u8)) = (current_indexed_element.index, (current_indexed_element.element.0, current_indexed_element.element.1));
                                let other_located_cell_group: (usize, (u8, u8)) = (other_indexed_element.index, (other_indexed_element.element.0, other_indexed_element.element.1));
                                for (isolated_graph_index, observations_total_per_located_cell_group_collection) in observations_total_per_located_cell_group_collection_per_isolated_graph.iter().enumerate() {
                                    if observations_total_per_located_cell_group_collection.contains_key(&current_located_cell_group) {
                                        current_indexed_element_isolated_graph_index = Some(isolated_graph_index);
                                        if other_indexed_element_isolated_graph_index.is_some() {
                                            break;
                                        }
                                    }
                                    if observations_total_per_located_cell_group_collection.contains_key(&other_located_cell_group) {
                                        other_indexed_element_isolated_graph_index = Some(isolated_graph_index);
                                        if current_indexed_element_isolated_graph_index.is_some() {
                                            break;
                                        }
                                    }
                                }

                                let mut check_for_cycle_at_isolated_graph_index: Option<usize> = None;

                                if current_indexed_element_isolated_graph_index.is_none() {
                                    if other_indexed_element_isolated_graph_index.is_none() {
                                        // if both are none, then this collection is isolated, create a new BTreeSet
                                        let mut located_cell_group_collection: BTreeMap<(usize, (u8, u8)), usize> = BTreeMap::new();
                                        located_cell_group_collection.insert(current_located_cell_group, 1);
                                        located_cell_group_collection.insert(other_located_cell_group, 1);
                                        observations_total_per_located_cell_group_collection_per_isolated_graph.push(located_cell_group_collection);
                                        first_fully_connected_located_cell_groups_per_isolated_graph.push(Vec::new());
                                        is_cell_group_index_fully_connected_per_isolated_graph.push(BitVec::repeat(false, self.cell_groups.len()));

                                        // add the located cell groups if they are now fully connected
                                        if fully_connected_length == 1 {
                                            let isolated_graph_index = observations_total_per_located_cell_group_collection_per_isolated_graph.len() - 1;
                                            if !is_cell_group_index_fully_connected_per_isolated_graph[isolated_graph_index][current_located_cell_group.0] {
                                                is_cell_group_index_fully_connected_per_isolated_graph[isolated_graph_index].set(current_located_cell_group.0, true);
                                                first_fully_connected_located_cell_groups_per_isolated_graph[isolated_graph_index].push(current_located_cell_group);
                                                check_for_cycle_at_isolated_graph_index = Some(isolated_graph_index);
                                            }
                                            if !is_cell_group_index_fully_connected_per_isolated_graph[isolated_graph_index][other_located_cell_group.0] {
                                                is_cell_group_index_fully_connected_per_isolated_graph[isolated_graph_index].set(other_located_cell_group.0, true);
                                                first_fully_connected_located_cell_groups_per_isolated_graph[isolated_graph_index].push(other_located_cell_group);
                                                check_for_cycle_at_isolated_graph_index = Some(isolated_graph_index);
                                            }
                                        }
                                    }
                                    else {
                                        // the current located cell group extends the existing collection that contains the other located cell group
                                        observations_total_per_located_cell_group_collection_per_isolated_graph[other_indexed_element_isolated_graph_index.unwrap()].insert(current_located_cell_group, 1);

                                        // add the current located cell group if it is now fully connected
                                        if fully_connected_length == 1 {
                                            if !is_cell_group_index_fully_connected_per_isolated_graph[other_indexed_element_isolated_graph_index.unwrap()][current_located_cell_group.0] {
                                                is_cell_group_index_fully_connected_per_isolated_graph[other_indexed_element_isolated_graph_index.unwrap()].set(current_located_cell_group.0, true);
                                                first_fully_connected_located_cell_groups_per_isolated_graph[other_indexed_element_isolated_graph_index.unwrap()].push(current_located_cell_group);
                                                check_for_cycle_at_isolated_graph_index = Some(other_indexed_element_isolated_graph_index.unwrap());
                                            }
                                        }
                                        else {
                                            {
                                                let previous_observations_total = observations_total_per_located_cell_group_collection_per_isolated_graph[other_indexed_element_isolated_graph_index.unwrap()][&other_located_cell_group];
                                                let next_observations_total = previous_observations_total + 1;
                                                if next_observations_total == fully_connected_length {
                                                    if !is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][other_located_cell_group.0] {
                                                        is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].set(other_located_cell_group.0, true);
                                                        first_fully_connected_located_cell_groups_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].push(other_located_cell_group);
                                                        check_for_cycle_at_isolated_graph_index = Some(current_indexed_element_isolated_graph_index.unwrap());
                                                    }
                                                }
                                                observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].insert(other_located_cell_group, next_observations_total);
                                            }
                                        }
                                    }
                                }
                                else {
                                    if other_indexed_element_isolated_graph_index.is_none() {
                                        // the other located cell group extends the existing collection that contains the current located cell group
                                        observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].insert(other_located_cell_group, 1);

                                        // TODO add the other located cell group if it is now fully connected
                                        if fully_connected_length == 1 {
                                            if !is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][other_located_cell_group.0] {
                                                is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].set(other_located_cell_group.0, true);
                                                first_fully_connected_located_cell_groups_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].push(other_located_cell_group);
                                                check_for_cycle_at_isolated_graph_index = Some(current_indexed_element_isolated_graph_index.unwrap());
                                            }
                                        }
                                        else {
                                            // TODO if any new fully connected located cell groups are appended, check for cycle
                                            {
                                                let previous_observations_total = observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][&current_located_cell_group];
                                                let next_observations_total = previous_observations_total + 1;
                                                if next_observations_total == fully_connected_length {
                                                    if !is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][current_located_cell_group.0] {
                                                        is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].set(current_located_cell_group.0, true);
                                                        first_fully_connected_located_cell_groups_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].push(current_located_cell_group);
                                                        check_for_cycle_at_isolated_graph_index = Some(current_indexed_element_isolated_graph_index.unwrap());
                                                    }
                                                }
                                                observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].insert(current_located_cell_group, next_observations_total);
                                            }
                                        }
                                    }
                                    else {

                                        // they both exist in either the same or different collection
                                        if current_indexed_element_isolated_graph_index != other_indexed_element_isolated_graph_index {
                                            // they exist in different collections, so they must now be combined
                                            let first_cell_group_collection_index: usize;
                                            let second_cell_group_collection_index: usize;
                                            if current_indexed_element_isolated_graph_index < other_indexed_element_isolated_graph_index {
                                                first_cell_group_collection_index = current_indexed_element_isolated_graph_index.unwrap();
                                                second_cell_group_collection_index = other_indexed_element_isolated_graph_index.unwrap();
                                            }
                                            else {
                                                first_cell_group_collection_index = other_indexed_element_isolated_graph_index.unwrap();
                                                second_cell_group_collection_index = current_indexed_element_isolated_graph_index.unwrap();
                                            }
                                            let second_observations_total_per_cell_group_collection = observations_total_per_located_cell_group_collection_per_isolated_graph.remove(second_cell_group_collection_index);
                                            for (located_cell_group_collection, observations_total) in second_observations_total_per_cell_group_collection.into_iter() {
                                                let next_observations_total;
                                                if observations_total_per_located_cell_group_collection_per_isolated_graph[first_cell_group_collection_index].contains_key(&located_cell_group_collection) {
                                                    let previous_observations_total = observations_total_per_located_cell_group_collection_per_isolated_graph[first_cell_group_collection_index][&located_cell_group_collection];
                                                    next_observations_total = previous_observations_total + observations_total;
                                                    if next_observations_total == fully_connected_length {
                                                        if !is_cell_group_index_fully_connected_per_isolated_graph[first_cell_group_collection_index][located_cell_group_collection.0] {
                                                            is_cell_group_index_fully_connected_per_isolated_graph[first_cell_group_collection_index].set(located_cell_group_collection.0, true);
                                                            first_fully_connected_located_cell_groups_per_isolated_graph[first_cell_group_collection_index].push(located_cell_group_collection);
                                                            check_for_cycle_at_isolated_graph_index = Some(first_cell_group_collection_index);
                                                        }
                                                    }
                                                }
                                                else {
                                                    next_observations_total = observations_total;
                                                }
                                                observations_total_per_located_cell_group_collection_per_isolated_graph[first_cell_group_collection_index].insert(located_cell_group_collection, next_observations_total);
                                            }

                                            // merge the located cell groups if they were previously fully connected
                                            // if any new fully connected located cell groups are appended, check for cycle
                                            let is_cell_group_index_fully_connected = is_cell_group_index_fully_connected_per_isolated_graph.remove(second_cell_group_collection_index);
                                            let first_fully_connected_located_cell_groups = first_fully_connected_located_cell_groups_per_isolated_graph.remove(second_cell_group_collection_index);
                                            for located_cell_group in first_fully_connected_located_cell_groups {
                                                if !is_cell_group_index_fully_connected_per_isolated_graph[first_cell_group_collection_index][located_cell_group.0] {
                                                    is_cell_group_index_fully_connected_per_isolated_graph[first_cell_group_collection_index].set(located_cell_group.0, true);
                                                    first_fully_connected_located_cell_groups_per_isolated_graph[first_cell_group_collection_index].push(located_cell_group);
                                                    check_for_cycle_at_isolated_graph_index = Some(first_cell_group_collection_index);
                                                }
                                            }
                                        }
                                        else {
                                            // they exist in the same located cell group collect and now form a cycle, test to see if this is the cycle that creates a full loop
                                            // TODO ensure that each located cell group of the cycle (for each distinct cell group) forms a cliche with each other, as a statement that they all permit each other's location
                                            {
                                                let previous_observations_total = observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][&current_located_cell_group];
                                                let next_observations_total = previous_observations_total + 1;
                                                if next_observations_total == fully_connected_length {
                                                    if !is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][current_located_cell_group.0] {
                                                        is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].set(current_located_cell_group.0, true);
                                                        first_fully_connected_located_cell_groups_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].push(current_located_cell_group);
                                                        check_for_cycle_at_isolated_graph_index = Some(current_indexed_element_isolated_graph_index.unwrap());
                                                    }
                                                }
                                                observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].insert(current_located_cell_group, next_observations_total);
                                            }

                                            {
                                                let previous_observations_total = observations_total_per_located_cell_group_collection_per_isolated_graph[other_indexed_element_isolated_graph_index.unwrap()][&other_located_cell_group];
                                                let next_observations_total = previous_observations_total + 1;
                                                if next_observations_total == fully_connected_length {
                                                    if !is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()][other_located_cell_group.0] {
                                                        is_cell_group_index_fully_connected_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].set(other_located_cell_group.0, true);
                                                        first_fully_connected_located_cell_groups_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].push(other_located_cell_group);
                                                        check_for_cycle_at_isolated_graph_index = Some(current_indexed_element_isolated_graph_index.unwrap());
                                                    }
                                                }
                                                observations_total_per_located_cell_group_collection_per_isolated_graph[current_indexed_element_isolated_graph_index.unwrap()].insert(other_located_cell_group, next_observations_total);
                                            }
                                        }
                                    }
                                }

                                if check_for_cycle_at_isolated_graph_index.is_some() {
                                    if first_fully_connected_located_cell_groups_per_isolated_graph[check_for_cycle_at_isolated_graph_index.unwrap()].len() == self.cell_groups.len() {
                                        // the number of connected located cell groups contains all of the valid located cell groups
                                        // construct and return the randomized PixelBoard instance

                                        let mut random_pixel_board: PixelBoard<TPixel> = PixelBoard::new(self.pixel_board.get_width(), self.pixel_board.get_height());
                                        for (cell_group_index, location) in first_fully_connected_located_cell_groups_per_isolated_graph.remove(check_for_cycle_at_isolated_graph_index.unwrap()).into_iter() {
                                            for cell in self.cell_groups[cell_group_index].cells.iter() {
                                                let calculated_pixel_board_index_x: usize = (location.0 + cell.0) as usize;
                                                let calculated_pixel_board_index_y: usize = (location.1 + cell.1) as usize;
                                                let pixel_board_coordinate = self.pixel_board_coordinate_per_cell_group_index[cell_group_index];
                                                let original_pixel_board_index_x: usize = (cell.0 as usize + pixel_board_coordinate.0);
                                                let original_pixel_board_index_y: usize = (cell.1 as usize + pixel_board_coordinate.1);
                                                random_pixel_board.set(calculated_pixel_board_index_x, calculated_pixel_board_index_y, self.pixel_board.get(original_pixel_board_index_x, original_pixel_board_index_y).unwrap());
                                            }
                                        }
                                        return random_pixel_board;
                                    }
                                }
                            }
                        }
                    }

                    // look for cliches given the stateful hyper graph nodes of the latest set of provided location pairs
                    let hyper_graph_cliche_shifter = Rc::new(RefCell::new(HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index.clone())));
                    let mut shifter_incrementer = ShifterIncrementer::new(hyper_graph_cliche_shifter);
                    if shifter_incrementer.try_increment() {
                        // found cliche
                        let cliche = shifter_incrementer.get();
                        let mut random_pixel_board: PixelBoard<TPixel> = PixelBoard::new(self.pixel_board.get_width(), self.pixel_board.get_height());
                        for indexed_element in cliche {
                            let location = *indexed_element.element.as_ref();
                            for cell in self.cell_groups[indexed_element.index].cells.iter() {
                                let calculated_pixel_board_index_x: usize = (location.0 + cell.0) as usize;
                                let calculated_pixel_board_index_y: usize = (location.1 + cell.1) as usize;
                                let pixel_board_coordinate = self.pixel_board_coordinate_per_cell_group_index[indexed_element.index];
                                let original_pixel_board_index_x: usize = (cell.0 as usize + pixel_board_coordinate.0);
                                let original_pixel_board_index_y: usize = (cell.1 as usize + pixel_board_coordinate.1);
                                random_pixel_board.set(calculated_pixel_board_index_x, calculated_pixel_board_index_y, self.pixel_board.get(original_pixel_board_index_x, original_pixel_board_index_y).unwrap());
                            }
                        }
                        return random_pixel_board;
                    }
                }
            }
        }

        panic!("Unexpected failure to find the original placement, let alone a new random one.");
    }
}

// TODO add test where the walls are double thick and there is a single, adjacent floater
#[cfg(test)]
mod pixel_board_randomizer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use super::*;
    use rstest::rstest;
    use uuid::Uuid;

    struct Tile {
        image_id: String
    }

    struct Element {
        element_id: String,
        padding: u8
    }

    enum ExamplePixel {
        Tile(Tile),
        Element(Element)
    }

    impl Pixel for ExamplePixel {
        fn get_invalid_location_offsets_for_other_pixel(&self, other_pixel: &ExamplePixel) -> Vec<(i16, i16)> {
            match self {
                ExamplePixel::Tile(ref tile) => {
                    return Vec::new();
                },
                ExamplePixel::Element(ref element) => {
                    let mut invalid_location_offsets: Vec<(i16, i16)> = Vec::new();
                    if element.padding != 0 {
                        let max: i16 = element.padding as i16;
                        let min: i16 = -max;
                        for y in min..=max {
                            for x in min..=max {
                                if x != 0 && y != 0 {
                                    invalid_location_offsets.push((x, y));
                                }
                            }
                        }
                    }
                    return invalid_location_offsets;
                }
            }
        }
    }

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn full_wall_surrounding_one_empty_spot() {
        init();
    
        let image_id_a = Uuid::new_v4().to_string();
        let wall_pixel_a: Rc<RefCell<ExamplePixel>> = Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: image_id_a.clone()
        })));
        let image_id_b = Uuid::new_v4().to_string();
        let wall_pixel_b: Rc<RefCell<ExamplePixel>> = Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: image_id_b.clone()
        })));
        let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(3, 3);
        pixel_board.set(0, 0, wall_pixel_a.clone());
        pixel_board.set(1, 0, wall_pixel_a.clone());
        pixel_board.set(2, 0, wall_pixel_a.clone());
        pixel_board.set(0, 1, wall_pixel_a.clone());
        pixel_board.set(2, 1, wall_pixel_a.clone());
        pixel_board.set(0, 2, wall_pixel_b.clone());
        pixel_board.set(1, 2, wall_pixel_b.clone());
        pixel_board.set(2, 2, wall_pixel_b.clone());
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
        for (x, y, image_id) in [
            (0, 0, &image_id_a),
            (1, 0, &image_id_a),
            (2, 0, &image_id_a),
            (0, 1, &image_id_a),
            (2, 1, &image_id_a),
            (0, 2, &image_id_b),
            (1, 2, &image_id_b),
            (2, 2, &image_id_b)
        ] {
            let borrowed_pixel = random_pixel_board.get(x, y);
            let pixel: &ExamplePixel = &borrowed_pixel.as_ref().unwrap().borrow();
            if let ExamplePixel::Tile(tile) = pixel {
                //println!("location ({}, {}) is looking for {}.", x, y, image_id);
                assert_eq!(image_id, &tile.image_id);
            }
            else {
                panic!("Unexpected ExamplePixel type");
            }
        }
    }

    #[rstest]
    fn single_dot_in_center_of_three_by_three() {
        let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(3, 3);
        pixel_board.set(1, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: Uuid::new_v4().to_string()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        for _ in 0..10 {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            for y in 0..=2 {
                for x in 0..=2 {
                    let pixel_option = random_pixel_board.get(x, y);
                    if x == 1 && y == 1 {
                        assert!(pixel_option.is_some());
                    }
                    else {
                        assert!(pixel_option.is_none());
                    }
                }
            }
        }
    }

    #[rstest]
    fn two_pixels_as_wall_segments_alone_and_vertical() {
        let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(3, 6);
        pixel_board.set(0, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: Uuid::new_v4().to_string()
        }))));
        pixel_board.set(0, 4, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: Uuid::new_v4().to_string()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        for _ in 0..10 {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            assert!(random_pixel_board.get(0, 0).is_none());
            assert!(random_pixel_board.get(0, 5).is_none());
            for y in 0..6 {
                for x in 1..3 {
                    let pixel_option = random_pixel_board.get(x, y);
                    assert!(pixel_option.is_none());
                }
            }
            let mut pixels_total = 0;
            let mut is_previous_index_a_pixel = false;
            for index in 1..=4 {
                if random_pixel_board.get(0, index).is_some() {
                    pixels_total += 1;
                    assert!(!is_previous_index_a_pixel);
                    is_previous_index_a_pixel = true;
                }
                else {
                    is_previous_index_a_pixel = false;
                }
            }
            assert_eq!(2, pixels_total);
        }
    }

    #[rstest]
    fn two_pixels_as_wall_segments_alone_and_horizontal() {
        let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(6, 3);
        pixel_board.set(1, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: Uuid::new_v4().to_string()
        }))));
        pixel_board.set(4, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: Uuid::new_v4().to_string()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        for _ in 0..10 {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            assert!(random_pixel_board.get(0, 0).is_none());
            assert!(random_pixel_board.get(5, 0).is_none());
            for y in 1..3 {
                for x in 0..6 {
                    let pixel_option = random_pixel_board.get(x, y);
                    assert!(pixel_option.is_none());
                }
            }
            let mut pixels_total = 0;
            let mut is_previous_index_a_pixel = false;
            for index in 1..=4 {
                if random_pixel_board.get(index, 0).is_some() {
                    pixels_total += 1;
                    assert!(!is_previous_index_a_pixel);
                    is_previous_index_a_pixel = true;
                }
                else {
                    is_previous_index_a_pixel = false;
                }
            }
            assert_eq!(2, pixels_total);
        }
    }

    #[rstest]
    fn top_left_corner_wall_with_wall_adjacent_one_each() {
        for board_width in 4..=5 {
            for wall_height in 3..=8 {
                let mut wall_image_ids: Vec<String> = Vec::new();
                let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(board_width, wall_height);
                for height_index in 0..(wall_height - 1) {
                    let image_id = Uuid::new_v4().to_string();
                    pixel_board.set(0, height_index, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                        image_id: image_id.clone()
                    }))));
                    wall_image_ids.push(image_id);
                }
                let wall_adjacent_image_id = Uuid::new_v4().to_string();
                pixel_board.set(1, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: wall_adjacent_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut appearances_totals: Vec<u32> = Vec::new();
                for _ in 0..(wall_height - 2) {
                    appearances_totals.push(0);
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    assert!(!random_pixel_board.exists(1, 0));
                    assert!(!random_pixel_board.exists(1, wall_height - 1));
                    let mut wall_adjacents_total = 0;
                    for height_index in 0..wall_height {
                        if height_index != (wall_height - 1) {
                            assert!(random_pixel_board.exists(0, height_index));
                            {
                                let wrapped_random_wall_pixel = random_pixel_board.get(0, height_index).unwrap();
                                let borrowed_random_wall_pixel: &ExamplePixel = &wrapped_random_wall_pixel.borrow();
                                if let ExamplePixel::Tile(random_wall_pixel) = borrowed_random_wall_pixel {
                                    let wall_image_id = &wall_image_ids[height_index];
                                    assert_eq!(wall_image_id, &random_wall_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        else {
                            assert!(!random_pixel_board.exists(0, height_index));
                        }
                        {
                            let wrapped_wall_adjacent_pixel_option = random_pixel_board.get(1, height_index);
                            if wrapped_wall_adjacent_pixel_option.is_some() {
                                wall_adjacents_total += 1;
                                appearances_totals[height_index - 1] += 1;
                                let wrapped_wall_adjacent_pixel = wrapped_wall_adjacent_pixel_option.unwrap();
                                let borrowed_wall_adjacent_pixel: &ExamplePixel = &wrapped_wall_adjacent_pixel.borrow();
                                if let ExamplePixel::Tile(wall_adjacent_pixel) = borrowed_wall_adjacent_pixel {
                                    assert_eq!(&wall_adjacent_image_id, &wall_adjacent_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        
                        for board_width_index in 0..(board_width - 2) {
                            assert!(!random_pixel_board.exists(board_width_index + 2, height_index));
                        }
                    }
                    assert_eq!(1, wall_adjacents_total);
                }
                println!("appearances_totals: {:?}", appearances_totals);
                for appearances_total in appearances_totals.iter() {
                    let expected_value = &(iterations_total / appearances_totals.len() as u32 - (iterations_total / 5) / appearances_totals.len() as u32);
                    assert!(appearances_total > expected_value);
                }
            }
        }
    }

    #[rstest]
    fn left_corner_wall_with_wall_adjacent_one_each() {
        for board_width in 4..=5 {
            for wall_height in 3..=8 {
                let mut wall_image_ids: Vec<String> = Vec::new();
                let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(board_width, wall_height);
                for height_index in 0..wall_height {
                    let image_id = Uuid::new_v4().to_string();
                    pixel_board.set(0, height_index, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                        image_id: image_id.clone()
                    }))));
                    wall_image_ids.push(image_id);
                }
                let wall_adjacent_image_id = Uuid::new_v4().to_string();
                pixel_board.set(1, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: wall_adjacent_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut appearances_totals: Vec<u32> = Vec::new();
                for _ in 0..(wall_height - 2) {
                    appearances_totals.push(0);
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    assert!(!random_pixel_board.exists(1, 0));
                    assert!(!random_pixel_board.exists(1, wall_height - 1));
                    let mut wall_adjacents_total = 0;
                    for height_index in 0..wall_height {
                        assert!(random_pixel_board.exists(0, height_index));
                        {
                            let wrapped_random_wall_pixel = random_pixel_board.get(0, height_index).unwrap();
                            let borrowed_random_wall_pixel: &ExamplePixel = &wrapped_random_wall_pixel.borrow();
                            if let ExamplePixel::Tile(random_wall_pixel) = borrowed_random_wall_pixel {
                                let wall_image_id = &wall_image_ids[height_index];
                                assert_eq!(wall_image_id, &random_wall_pixel.image_id);
                            }
                            else {
                                panic!("Unexpected ExamplePixel type");
                            }
                        }
                        {
                            let wrapped_wall_adjacent_pixel_option = random_pixel_board.get(1, height_index);
                            if wrapped_wall_adjacent_pixel_option.is_some() {
                                wall_adjacents_total += 1;
                                appearances_totals[height_index - 1] += 1;
                                let wrapped_wall_adjacent_pixel = wrapped_wall_adjacent_pixel_option.unwrap();
                                let borrowed_wall_adjacent_pixel: &ExamplePixel = &wrapped_wall_adjacent_pixel.borrow();
                                if let ExamplePixel::Tile(wall_adjacent_pixel) = borrowed_wall_adjacent_pixel {
                                    assert_eq!(&wall_adjacent_image_id, &wall_adjacent_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        
                        for board_width_index in 0..(board_width - 2) {
                            assert!(!random_pixel_board.exists(board_width_index + 2, height_index));
                        }
                    }
                    assert_eq!(1, wall_adjacents_total);
                }
                println!("appearances_totals: {:?}", appearances_totals);
                for appearances_total in appearances_totals.iter() {
                    let expected_value = &(iterations_total / appearances_totals.len() as u32 - (iterations_total / 5) / appearances_totals.len() as u32);
                    assert!(appearances_total > expected_value);
                }
            }
        }
    }

    #[rstest]
    fn bottom_left_corner_wall_with_wall_adjacent_one_each() {
        for board_width in 4..=5 {
            for wall_height in 3..=8 {
                let mut wall_image_ids: Vec<String> = Vec::new();
                let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(board_width, wall_height);
                for height_index in 0..(wall_height - 1) {
                    let image_id = Uuid::new_v4().to_string();
                    pixel_board.set(0, height_index + 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                        image_id: image_id.clone()
                    }))));
                    wall_image_ids.push(image_id);
                }
                let wall_adjacent_image_id = Uuid::new_v4().to_string();
                pixel_board.set(1, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: wall_adjacent_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut appearances_totals: Vec<u32> = Vec::new();
                for _ in 0..(wall_height - 2) {
                    appearances_totals.push(0);
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    assert!(!random_pixel_board.exists(1, 0));
                    assert!(!random_pixel_board.exists(1, wall_height - 1));
                    let mut wall_adjacents_total = 0;
                    for height_index in 0..wall_height {
                        if height_index != 0 {
                            assert!(random_pixel_board.exists(0, height_index));
                            {
                                let wrapped_random_wall_pixel = random_pixel_board.get(0, height_index).unwrap();
                                let borrowed_random_wall_pixel: &ExamplePixel = &wrapped_random_wall_pixel.borrow();
                                if let ExamplePixel::Tile(random_wall_pixel) = borrowed_random_wall_pixel {
                                    let wall_image_id = &wall_image_ids[height_index - 1];
                                    assert_eq!(wall_image_id, &random_wall_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        else {
                            assert!(!random_pixel_board.exists(0, height_index));
                        }
                        {
                            let wrapped_wall_adjacent_pixel_option = random_pixel_board.get(1, height_index);
                            if wrapped_wall_adjacent_pixel_option.is_some() {
                                wall_adjacents_total += 1;
                                appearances_totals[height_index - 1] += 1;
                                let wrapped_wall_adjacent_pixel = wrapped_wall_adjacent_pixel_option.unwrap();
                                let borrowed_wall_adjacent_pixel: &ExamplePixel = &wrapped_wall_adjacent_pixel.borrow();
                                if let ExamplePixel::Tile(wall_adjacent_pixel) = borrowed_wall_adjacent_pixel {
                                    assert_eq!(&wall_adjacent_image_id, &wall_adjacent_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        
                        for board_width_index in 0..(board_width - 2) {
                            assert!(!random_pixel_board.exists(board_width_index + 2, height_index));
                        }
                    }
                    assert_eq!(1, wall_adjacents_total);
                }
                println!("appearances_totals: {:?}", appearances_totals);
                for appearances_total in appearances_totals.iter() {
                    let expected_value = &(iterations_total / appearances_totals.len() as u32 - (iterations_total / 5) / appearances_totals.len() as u32);
                    assert!(appearances_total > expected_value);
                }
            }
        }
    }

    #[rstest]
    fn top_right_corner_wall_with_wall_adjacent_one_each() {
        for board_width in 4..=5 {
            for wall_height in 3..=8 {
                let mut wall_image_ids: Vec<String> = Vec::new();
                let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(board_width, wall_height);
                for height_index in 0..(wall_height - 1) {
                    let image_id = Uuid::new_v4().to_string();
                    pixel_board.set(board_width - 1, height_index, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                        image_id: image_id.clone()
                    }))));
                    wall_image_ids.push(image_id);
                }
                let wall_adjacent_image_id = Uuid::new_v4().to_string();
                pixel_board.set(board_width - 2, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: wall_adjacent_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut appearances_totals: Vec<u32> = Vec::new();
                for _ in 0..(wall_height - 2) {
                    appearances_totals.push(0);
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    assert!(!random_pixel_board.exists(board_width - 2, 0));
                    assert!(!random_pixel_board.exists(board_width - 2, wall_height - 1));
                    let mut wall_adjacents_total = 0;
                    for height_index in 0..wall_height {
                        if height_index != (wall_height - 1) {
                            assert!(random_pixel_board.exists(board_width - 1, height_index));
                            {
                                let wrapped_random_wall_pixel = random_pixel_board.get(board_width - 1, height_index).unwrap();
                                let borrowed_random_wall_pixel: &ExamplePixel = &wrapped_random_wall_pixel.borrow();
                                if let ExamplePixel::Tile(random_wall_pixel) = borrowed_random_wall_pixel {
                                    let wall_image_id = &wall_image_ids[height_index];
                                    assert_eq!(wall_image_id, &random_wall_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        else {
                            assert!(!random_pixel_board.exists(board_width - 1, height_index));
                        }
                        {
                            let wrapped_wall_adjacent_pixel_option = random_pixel_board.get(board_width - 2, height_index);
                            if wrapped_wall_adjacent_pixel_option.is_some() {
                                wall_adjacents_total += 1;
                                appearances_totals[height_index - 1] += 1;
                                let wrapped_wall_adjacent_pixel = wrapped_wall_adjacent_pixel_option.unwrap();
                                let borrowed_wall_adjacent_pixel: &ExamplePixel = &wrapped_wall_adjacent_pixel.borrow();
                                if let ExamplePixel::Tile(wall_adjacent_pixel) = borrowed_wall_adjacent_pixel {
                                    assert_eq!(&wall_adjacent_image_id, &wall_adjacent_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        
                        for board_width_index in 0..(board_width - 2) {
                            assert!(!random_pixel_board.exists(board_width_index, height_index));
                        }
                    }
                    assert_eq!(1, wall_adjacents_total);
                }
                println!("appearances_totals: {:?}", appearances_totals);
                for appearances_total in appearances_totals.iter() {
                    let expected_value = &(iterations_total / appearances_totals.len() as u32 - (iterations_total / 5) / appearances_totals.len() as u32);
                    assert!(appearances_total > expected_value);
                }
            }
        }
    }

    #[rstest]
    fn right_corner_wall_with_wall_adjacent_one_each() {
        for board_width in 4..=5 {
            for wall_height in 3..=8 {
                let mut wall_image_ids: Vec<String> = Vec::new();
                let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(board_width, wall_height);
                for height_index in 0..wall_height {
                    let image_id = Uuid::new_v4().to_string();
                    pixel_board.set(board_width - 1, height_index, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                        image_id: image_id.clone()
                    }))));
                    wall_image_ids.push(image_id);
                }
                let wall_adjacent_image_id = Uuid::new_v4().to_string();
                pixel_board.set(board_width - 2, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: wall_adjacent_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut appearances_totals: Vec<u32> = Vec::new();
                for _ in 0..(wall_height - 2) {
                    appearances_totals.push(0);
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    assert!(!random_pixel_board.exists(board_width - 2, 0));
                    assert!(!random_pixel_board.exists(board_width - 2, wall_height - 1));
                    let mut wall_adjacents_total = 0;
                    for height_index in 0..wall_height {
                        assert!(random_pixel_board.exists(board_width - 1, height_index));
                        {
                            let wrapped_random_wall_pixel = random_pixel_board.get(board_width - 1, height_index).unwrap();
                            let borrowed_random_wall_pixel: &ExamplePixel = &wrapped_random_wall_pixel.borrow();
                            if let ExamplePixel::Tile(random_wall_pixel) = borrowed_random_wall_pixel {
                                let wall_image_id = &wall_image_ids[height_index];
                                assert_eq!(wall_image_id, &random_wall_pixel.image_id);
                            }
                            else {
                                panic!("Unexpected ExamplePixel type");
                            }
                        }
                        {
                            let wrapped_wall_adjacent_pixel_option = random_pixel_board.get(board_width - 2, height_index);
                            if wrapped_wall_adjacent_pixel_option.is_some() {
                                wall_adjacents_total += 1;
                                appearances_totals[height_index - 1] += 1;
                                let wrapped_wall_adjacent_pixel = wrapped_wall_adjacent_pixel_option.unwrap();
                                let borrowed_wall_adjacent_pixel: &ExamplePixel = &wrapped_wall_adjacent_pixel.borrow();
                                if let ExamplePixel::Tile(wall_adjacent_pixel) = borrowed_wall_adjacent_pixel {
                                    assert_eq!(&wall_adjacent_image_id, &wall_adjacent_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        
                        for board_width_index in 0..(board_width - 2) {
                            assert!(!random_pixel_board.exists(board_width_index, height_index));
                        }
                    }
                    assert_eq!(1, wall_adjacents_total);
                }
                println!("appearances_totals: {:?}", appearances_totals);
                for appearances_total in appearances_totals.iter() {
                    let expected_value = &(iterations_total / appearances_totals.len() as u32 - (iterations_total / 5) / appearances_totals.len() as u32);
                    assert!(appearances_total > expected_value);
                }
            }
        }
    }

    #[rstest]
    fn bottom_right_corner_wall_with_wall_adjacent_one_each() {
        for board_width in 4..=5 {
            for wall_height in 3..=8 {
                let mut wall_image_ids: Vec<String> = Vec::new();
                let mut pixel_board: PixelBoard<ExamplePixel> = PixelBoard::new(board_width, wall_height);
                for height_index in 0..(wall_height - 1) {
                    let image_id = Uuid::new_v4().to_string();
                    pixel_board.set(board_width - 1, height_index + 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                        image_id: image_id.clone()
                    }))));
                    wall_image_ids.push(image_id);
                }
                let wall_adjacent_image_id = Uuid::new_v4().to_string();
                pixel_board.set(board_width - 2, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: wall_adjacent_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut appearances_totals: Vec<u32> = Vec::new();
                for _ in 0..(wall_height - 2) {
                    appearances_totals.push(0);
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    assert!(!random_pixel_board.exists(board_width - 2, 0));
                    assert!(!random_pixel_board.exists(board_width - 2, wall_height - 1));
                    let mut wall_adjacents_total = 0;
                    for height_index in 0..wall_height {
                        if height_index != 0 {
                            assert!(random_pixel_board.exists(board_width - 1, height_index));
                            {
                                let wrapped_random_wall_pixel = random_pixel_board.get(board_width - 1, height_index).unwrap();
                                let borrowed_random_wall_pixel: &ExamplePixel = &wrapped_random_wall_pixel.borrow();
                                if let ExamplePixel::Tile(random_wall_pixel) = borrowed_random_wall_pixel {
                                    let wall_image_id = &wall_image_ids[height_index - 1];
                                    assert_eq!(wall_image_id, &random_wall_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        else {
                            assert!(!random_pixel_board.exists(board_width - 1, height_index));
                        }
                        {
                            let wrapped_wall_adjacent_pixel_option = random_pixel_board.get(board_width - 2, height_index);
                            if wrapped_wall_adjacent_pixel_option.is_some() {
                                wall_adjacents_total += 1;
                                appearances_totals[height_index - 1] += 1;
                                let wrapped_wall_adjacent_pixel = wrapped_wall_adjacent_pixel_option.unwrap();
                                let borrowed_wall_adjacent_pixel: &ExamplePixel = &wrapped_wall_adjacent_pixel.borrow();
                                if let ExamplePixel::Tile(wall_adjacent_pixel) = borrowed_wall_adjacent_pixel {
                                    assert_eq!(&wall_adjacent_image_id, &wall_adjacent_pixel.image_id);
                                }
                                else {
                                    panic!("Unexpected ExamplePixel type");
                                }
                            }
                        }
                        
                        for board_width_index in 0..(board_width - 2) {
                            assert!(!random_pixel_board.exists(board_width_index, height_index));
                        }
                    }
                    assert_eq!(1, wall_adjacents_total);
                }
                println!("appearances_totals: {:?}", appearances_totals);
                for appearances_total in appearances_totals.iter() {
                    let expected_value = &(iterations_total / appearances_totals.len() as u32 - (iterations_total / 5) / appearances_totals.len() as u32);
                    assert!(appearances_total > expected_value);
                }
            }
        }
    }

    #[rstest]
    fn top_left_corner_and_floater() {
        init();

        let corner_image_id = Uuid::new_v4().to_string();
        let floater_image_id = Uuid::new_v4().to_string();
        let corner_location = (0, 0);
        let floater_location = (1, 1);
        for board_width in 4..=10 {
            for board_height in 4..=10 {
                let open_area = (board_width - 2) * (board_height - 2);
                let mut pixel_board = PixelBoard::new(board_width, board_height);
                pixel_board.set(corner_location.0, corner_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: corner_image_id.clone()
                }))));
                pixel_board.set(floater_location.0, floater_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: floater_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        count_per_location.insert((x + 1, y + 1), 0);
                    }
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    let mut location_total = 0;
                    let mut location_option: Option<(usize, usize)> = None;
                    for x in 0..(board_width - 2) {
                        for y in 0..(board_height - 2) {
                            if random_pixel_board.exists(x + 1, y + 1) {
                                location_option = Some((x + 1, y + 1));
                                location_total += 1;
                            }
                        }
                    }
                    assert_eq!(1, location_total);
                    if let Some(location) = location_option {
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        let unwrapped_pixel = random_pixel_board.get(location.0, location.1).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(floater_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("unexpected element.");
                            }
                        }
                    }
                    for x in 0..board_width {
                        for y in [0, board_height - 1] {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                    for x in [0, board_width - 1] {
                        for y in 0..board_height {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                }
                //println!("count_per_location: {:?}", count_per_location);
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        let location = (x + 1, y + 1);
                        let count = count_per_location[&location] as f32;
                        let expected = iterations_total as f32 / open_area as f32;
                        assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
                    }
                }
            }
        }
    }

    #[rstest]
    fn top_right_corner_and_floater() {
        init();

        let corner_image_id = Uuid::new_v4().to_string();
        let floater_image_id = Uuid::new_v4().to_string();
        let floater_location = (1, 1);
        for board_width in 4..=10 {
            for board_height in 4..=10 {
                let corner_location = (board_width - 1, 0);
                let open_area = (board_width - 2) * (board_height - 2);
                let mut pixel_board = PixelBoard::new(board_width, board_height);
                pixel_board.set(corner_location.0, corner_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: corner_image_id.clone()
                }))));
                pixel_board.set(floater_location.0, floater_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: floater_image_id.clone()
                }))));
                let mut pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        count_per_location.insert((x + 1, y + 1), 0);
                    }
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    let mut location_total = 0;
                    let mut location_option: Option<(usize, usize)> = None;
                    for x in 0..(board_width - 2) {
                        for y in 0..(board_height - 2) {
                            if random_pixel_board.exists(x + 1, y + 1) {
                                location_option = Some((x + 1, y + 1));
                                location_total += 1;
                            }
                        }
                    }
                    assert_eq!(1, location_total);
                    if let Some(location) = location_option {
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        let unwrapped_pixel = random_pixel_board.get(location.0, location.1).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(floater_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("unexpected element.");
                            }
                        }
                    }
                    for x in 0..board_width {
                        for y in [0, board_height - 1] {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                    for x in [0, board_width - 1] {
                        for y in 0..board_height {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                }
                //println!("count_per_location: {:?}", count_per_location);
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        let location = (x + 1, y + 1);
                        let count = count_per_location[&location] as f32;
                        let expected = iterations_total as f32 / open_area as f32;
                        assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
                    }
                }
            }
        }
    }

    #[rstest]
    fn bottom_right_corner_and_floater() {
        init();

        let corner_image_id = Uuid::new_v4().to_string();
        let floater_image_id = Uuid::new_v4().to_string();
        let floater_location = (1, 1);
        for board_width in 4..=10 {
            for board_height in 4..=10 {
                let corner_location = (board_width - 1, board_height - 1);
                let open_area = (board_width - 2) * (board_height - 2);
                let mut pixel_board = PixelBoard::new(board_width, board_height);
                pixel_board.set(corner_location.0, corner_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: corner_image_id.clone()
                }))));
                pixel_board.set(floater_location.0, floater_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: floater_image_id.clone()
                }))));
                let mut pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        count_per_location.insert((x + 1, y + 1), 0);
                    }
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    let mut location_total = 0;
                    let mut location_option: Option<(usize, usize)> = None;
                    for x in 0..(board_width - 2) {
                        for y in 0..(board_height - 2) {
                            if random_pixel_board.exists(x + 1, y + 1) {
                                location_option = Some((x + 1, y + 1));
                                location_total += 1;
                            }
                        }
                    }
                    assert_eq!(1, location_total);
                    if let Some(location) = location_option {
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        let unwrapped_pixel = random_pixel_board.get(location.0, location.1).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(floater_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("unexpected element.");
                            }
                        }
                    }
                    for x in 0..board_width {
                        for y in [0, board_height - 1] {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                    for x in [0, board_width - 1] {
                        for y in 0..board_height {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                }
                //println!("count_per_location: {:?}", count_per_location);
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        let location = (x + 1, y + 1);
                        let count = count_per_location[&location] as f32;
                        let expected = iterations_total as f32 / open_area as f32;
                        assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
                    }
                }
            }
        }
    }

    #[rstest]
    fn bottom_left_corner_and_floater() {
        init();

        let corner_image_id = Uuid::new_v4().to_string();
        let floater_image_id = Uuid::new_v4().to_string();
        let floater_location = (1, 1);
        for board_width in 4..=10 {
            for board_height in 4..=10 {
                let corner_location = (0, board_height - 1);
                let open_area = (board_width - 2) * (board_height - 2);
                let mut pixel_board = PixelBoard::new(board_width, board_height);
                pixel_board.set(corner_location.0, corner_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: corner_image_id.clone()
                }))));
                pixel_board.set(floater_location.0, floater_location.1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                    image_id: floater_image_id.clone()
                }))));
                let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
                let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        count_per_location.insert((x + 1, y + 1), 0);
                    }
                }
                let iterations_total = 1000;
                for _ in 0..iterations_total {
                    let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
                    let mut location_total = 0;
                    let mut location_option: Option<(usize, usize)> = None;
                    for x in 0..(board_width - 2) {
                        for y in 0..(board_height - 2) {
                            if random_pixel_board.exists(x + 1, y + 1) {
                                location_option = Some((x + 1, y + 1));
                                location_total += 1;
                            }
                        }
                    }
                    assert_eq!(1, location_total);
                    if let Some(location) = location_option {
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        let unwrapped_pixel = random_pixel_board.get(location.0, location.1).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(floater_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("unexpected element.");
                            }
                        }
                    }
                    for x in 0..board_width {
                        for y in [0, board_height - 1] {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                    for x in [0, board_width - 1] {
                        for y in 0..board_height {
                            let check_location = (x, y);
                            if check_location == corner_location {
                                assert!(random_pixel_board.exists(x, y));
                                let unwrapped_pixel = random_pixel_board.get(x, y).unwrap();
                                let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                                match borrowed_pixel {
                                    ExamplePixel::Tile(tile) => {
                                        assert_eq!(corner_image_id, tile.image_id);
                                    },
                                    ExamplePixel::Element(_) => {
                                        panic!("unexpected element.");
                                    }
                                }
                            }
                            else {
                                assert!(!random_pixel_board.exists(x, y));
                            }
                        }
                    }
                }
                //println!("count_per_location: {:?}", count_per_location);
                for x in 0..(board_width - 2) {
                    for y in 0..(board_height - 2) {
                        let location = (x + 1, y + 1);
                        let count = count_per_location[&location] as f32;
                        let expected = iterations_total as f32 / open_area as f32;
                        assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
                    }
                }
            }
        }
    }

    #[rstest]
    fn left_wall_segments_one_alone() {
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 3;
        let board_height = 4;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(0, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for y in 0..(board_height - 2) {
            count_per_location.insert((0, y + 1), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for y in 0..board_height {
                if y == 0 || y == (board_height - 1) {
                    assert!(!random_pixel_board.exists(0, y));
                }
                else {
                    if random_pixel_board.exists(0, y) {
                        let location = (0 as usize, y);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                    }
                }
            }
            assert!(is_segment_found);
            for x in 1..board_width {
                for y in 0..board_height {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for y in 0..(board_height - 2) {
            let location = (0, y + 1);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_height - 2) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn right_wall_segments_one_alone() {
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 3;
        let board_height = 4;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(board_width - 1, 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for y in 0..(board_height - 2) {
            count_per_location.insert((board_width - 1, y + 1), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for y in 0..board_height {
                if y == 0 || y == (board_height - 1) {
                    assert!(!random_pixel_board.exists(board_width - 1, y));
                }
                else {
                    if random_pixel_board.exists(board_width - 1, y) {
                        let location = ((board_width - 1) as usize, y);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                    }
                }
            }
            assert!(is_segment_found);
            for x in 0..(board_width - 1) {
                for y in 0..board_height {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for y in 0..(board_height - 2) {
            let location = (board_width - 1, y + 1);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_height - 2) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn top_wall_segments_one_alone() {
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 4;
        let board_height = 3;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(1, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for x in 0..(board_width - 2) {
            count_per_location.insert((x + 1, 0), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for x in 0..board_width {
                if x == 0 || x == (board_width - 1) {
                    assert!(!random_pixel_board.exists(x, 0));
                }
                else {
                    if random_pixel_board.exists(x, 0) {
                        let location = (x, 0 as usize);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                    }
                }
            }
            assert!(is_segment_found);
            for x in 0..board_width {
                for y in 1..board_height {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for x in 0..(board_width - 2) {
            let location = (x + 1, 0);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_width - 2) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn bottom_wall_segments_one_alone() {
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 4;
        let board_height = 3;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(1, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for x in 0..(board_width - 2) {
            count_per_location.insert((x + 1, board_height - 1), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for x in 0..board_width {
                if x == 0 || x == (board_width - 1) {
                    assert!(!random_pixel_board.exists(x, board_height - 1));
                }
                else {
                    if random_pixel_board.exists(x, board_height - 1) {
                        let location = (x, (board_height - 1) as usize);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                    }
                }
            }
            assert!(is_segment_found);
            for x in 0..board_width {
                for y in 0..(board_height - 1) {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for x in 0..(board_width - 2) {
            let location = (x + 1, board_height - 1);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_width - 2) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn left_wall_segments_one_top_left_and_bottom_left_corner_walls() {
        let top_corner_wall_image_id = Uuid::new_v4().to_string();
        let bottom_corner_wall_image_id = Uuid::new_v4().to_string();
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 3;
        let board_height = 6;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(0, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: top_corner_wall_image_id.clone()
        }))));
        pixel_board.set(0, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: bottom_corner_wall_image_id.clone()
        }))));
        pixel_board.set(0, 2, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id.clone()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for y in 0..(board_height - 4) {
            count_per_location.insert((0, y + 2), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for y in 0..board_height {
                if y == 0 {
                    assert!(random_pixel_board.exists(0, y));
                    let unwrapped_pixel = random_pixel_board.get(0, y).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(top_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else if y == 1 || y == (board_height - 2) {
                    assert!(!random_pixel_board.exists(0, y));
                }
                else if y == (board_height - 1) {
                    assert!(random_pixel_board.exists(0, y));
                    let unwrapped_pixel = random_pixel_board.get(0, y).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(bottom_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else {
                    if random_pixel_board.exists(0, y) {
                        let location = (0 as usize, y);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                        let unwrapped_pixel = random_pixel_board.get(0, y).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(segment_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("Unexpected pixel type.");
                            }
                        }
                    }
                }
            }
            assert!(is_segment_found);
            for x in 1..board_width {
                for y in 0..board_height {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for y in 0..(board_height - 4) {
            let location = (0, y + 2);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_height - 4) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn right_wall_segments_one_top_right_and_bottom_right_corner_walls() {
        let top_corner_wall_image_id = Uuid::new_v4().to_string();
        let bottom_corner_wall_image_id = Uuid::new_v4().to_string();
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 3;
        let board_height = 6;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(board_width - 1, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: top_corner_wall_image_id.clone()
        }))));
        pixel_board.set(board_width - 1, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: bottom_corner_wall_image_id.clone()
        }))));
        pixel_board.set(board_width - 1, 2, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id.clone()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for y in 0..(board_height - 4) {
            count_per_location.insert((board_width - 1, y + 2), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for y in 0..board_height {
                if y == 0 {
                    assert!(random_pixel_board.exists(board_width - 1, y));
                    let unwrapped_pixel = random_pixel_board.get(board_width - 1, y).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(top_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else if y == 1 || y == (board_height - 2) {
                    assert!(!random_pixel_board.exists(board_width - 1, y));
                }
                else if y == (board_height - 1) {
                    assert!(random_pixel_board.exists(board_width - 1, y));
                    let unwrapped_pixel = random_pixel_board.get(board_width - 1, y).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(bottom_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else {
                    if random_pixel_board.exists(board_width - 1, y) {
                        let location = ((board_width - 1) as usize, y);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                        let unwrapped_pixel = random_pixel_board.get(board_width - 1, y).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(segment_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("Unexpected pixel type.");
                            }
                        }
                    }
                }
            }
            assert!(is_segment_found);
            for x in 0..(board_width - 1) {
                for y in 0..board_height {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for y in 0..(board_height - 4) {
            let location = (board_width - 1, y + 2);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_height - 4) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn top_wall_segments_one_top_left_and_top_right_corner_walls() {
        let top_corner_wall_image_id = Uuid::new_v4().to_string();
        let right_corner_wall_image_id = Uuid::new_v4().to_string();
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 6;
        let board_height = 3;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(0, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: top_corner_wall_image_id.clone()
        }))));
        pixel_board.set(board_width - 1, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: right_corner_wall_image_id.clone()
        }))));
        pixel_board.set(2, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id.clone()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for x in 0..(board_width - 4) {
            count_per_location.insert((x + 2, 0), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for x in 0..board_width {
                if x == 0 {
                    assert!(random_pixel_board.exists(x, 0));
                    let unwrapped_pixel = random_pixel_board.get(x, 0).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(top_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else if x == 1 || x == (board_width - 2) {
                    assert!(!random_pixel_board.exists(x, 0));
                }
                else if x == (board_width - 1) {
                    assert!(random_pixel_board.exists(x, 0));
                    let unwrapped_pixel = random_pixel_board.get(x, 0).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(right_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else {
                    if random_pixel_board.exists(x, 0) {
                        let location = (x, 0 as usize);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                        let unwrapped_pixel = random_pixel_board.get(x, 0).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(segment_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("Unexpected pixel type.");
                            }
                        }
                    }
                }
            }
            assert!(is_segment_found);
            for y in 1..board_height {
                for x in 0..board_width {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for x in 0..(board_width - 4) {
            let location = (x + 2, 0);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_width - 4) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }

    #[rstest]
    fn bottom_wall_segments_one_bottom_left_and_bottom_right_corner_walls() {
        let top_corner_wall_image_id = Uuid::new_v4().to_string();
        let right_corner_wall_image_id = Uuid::new_v4().to_string();
        let segment_image_id = Uuid::new_v4().to_string();
        let board_width = 6;
        let board_height = 3;
        let mut pixel_board = PixelBoard::new(board_width, board_height);
        pixel_board.set(0, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: top_corner_wall_image_id.clone()
        }))));
        pixel_board.set(board_width - 1, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: right_corner_wall_image_id.clone()
        }))));
        pixel_board.set(2, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: segment_image_id.clone()
        }))));
        let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
        let mut count_per_location: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for x in 0..(board_width - 4) {
            count_per_location.insert((x + 2, board_height - 1), 0);
        }
        let iterations_total = 1000;
        for _ in 0..iterations_total {
            let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
            let mut is_segment_found = false;
            for x in 0..board_width {
                if x == 0 {
                    assert!(random_pixel_board.exists(x, board_height - 1));
                    let unwrapped_pixel = random_pixel_board.get(x, board_height - 1).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(top_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else if x == 1 || x == (board_width - 2) {
                    assert!(!random_pixel_board.exists(x, board_height - 1));
                }
                else if x == (board_width - 1) {
                    assert!(random_pixel_board.exists(x, board_height - 1));
                    let unwrapped_pixel = random_pixel_board.get(x, board_height - 1).unwrap();
                    let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                    match borrowed_pixel {
                        ExamplePixel::Tile(tile) => {
                            assert_eq!(right_corner_wall_image_id, tile.image_id);
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else {
                    if random_pixel_board.exists(x, board_height - 1) {
                        let location = (x, (board_height - 1) as usize);
                        count_per_location.insert(location, count_per_location[&location] + 1);
                        assert!(!is_segment_found);
                        is_segment_found = true;
                        let unwrapped_pixel = random_pixel_board.get(x, board_height - 1).unwrap();
                        let borrowed_pixel: &ExamplePixel = &unwrapped_pixel.borrow();
                        match borrowed_pixel {
                            ExamplePixel::Tile(tile) => {
                                assert_eq!(segment_image_id, tile.image_id);
                            },
                            ExamplePixel::Element(_) => {
                                panic!("Unexpected pixel type.");
                            }
                        }
                    }
                }
            }
            assert!(is_segment_found);
            for y in 0..(board_height - 1) {
                for x in 0..board_width {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
        println!("count_per_location: {:?}", count_per_location);
        for x in 0..(board_width - 4) {
            let location = (x + 2, board_height - 1);
            let count = count_per_location[&location] as f32;
            let expected = iterations_total as f32 / (board_width - 4) as f32;
            println!("{} < {}", (expected - count).abs(), (iterations_total as f32 / 10.0));
            assert!((expected - count).abs() < (iterations_total as f32 / 10.0));
        }
    }
}