use std::{rc::Rc, cell::RefCell, collections::{BTreeSet, BTreeMap, VecDeque}};

use crate::{CellGroup, incrementer::{round_robin_incrementer::RoundRobinIncrementer, Incrementer, shifting_cell_group_dependency_incrementer}, shifter::{Shifter, segment_permutation_shifter::{Segment, SegmentPermutationShifter}, index_shifter::IndexShifter}};

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
    wall_adjacent_index_shifters: Vec<IndexShifter<(u8, u8)>>
}

impl<TPixel: Pixel> PixelBoardRandomizer<TPixel> {
    pub fn new(pixel_board: PixelBoard<TPixel>) -> Self {
        let mut raw_cell_groups: Vec<CellGroup> = Vec::new();
        let mut adjacent_cell_group_indexes_per_cell_group_index: Vec<Vec<usize>> = Vec::new();
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
                            adjacent_pixel_board_coordinates.insert((x, 1));
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    top_right_corner_wall_cell_group_index = Some(0);
                    for y in 1..pixel_board.height {
                        if pixel_board.exists(rightmost_x, y) {
                            cells.push((rightmost_x as u8, y as u8));
                            adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_right_corner_wall_cell_group_index = Some(0);
                    for x in (0..rightmost_x).rev() {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
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
                            adjacent_pixel_board_coordinates.insert((1, y));
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    bottom_left_corner_wall_cell_group_index = Some(0);
                    for x in 1..pixel_board.width {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                        }
                        else {
                            break 'counterclockwise_collecting;
                        }
                    }
                    bottom_right_corner_wall_cell_group_index = Some(0);
                    for y in (0..bottommost_y).rev() {
                        if pixel_board.exists(rightmost_x, y) {
                            cells.push((rightmost_x as u8, y as u8));
                            adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
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
                            adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                        }
                        else {
                            break 'clockwise_collecting;
                        }
                    }
                    bottom_right_corner_wall_cell_group_index = Some(cell_group_index);
                    for x in (0..rightmost_x).rev() {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
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
                top_right_corner_wall_cell_group_index = Some(cell_group_index);
                'clockwise_collecting: {
                    for x in (0..rightmost_x).rev() {
                        if pixel_board.exists(x, bottommost_y) {
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
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
                    let mut top_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (leftmost_wall_x as u8, 0), rightmost_wall_x - leftmost_wall_x + 1, true, 1, false);
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
                    let mut bottom_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (leftmost_wall_x as u8, bottommost_y as u8), rightmost_wall_x - leftmost_wall_x + 1, true, 1, false);
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
                    let mut left_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (0, topmost_wall_y as u8), bottommost_wall_y - topmost_wall_y + 1, false, 1, false);
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
                    let mut right_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (rightmost_x as u8, topmost_wall_y as u8), bottommost_wall_y - topmost_wall_y + 1, false, 1, false);
                    right_wall_segment_permutation_shifter_option = Some(right_wall_segment_permutation_shifter);
                }
            }

            // at this point the corner walls and the wall segments have been discovered

            // collect all wall adjacents
            {
                // contains all of the pixel board index pairs
                let mut visited_pixel_board_coordinates: BTreeSet<(usize, usize)> = BTreeSet::new();

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

                    let location_references_width = rightmost_x - 2;

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
                                wall_adjacent_cell_group_indexes.push(raw_cell_groups.len());
                                raw_cell_groups.push(CellGroup {
                                    cells: cells
                                });
                                adjacent_cell_group_indexes_per_cell_group_index.push(adjacent_wall_cell_group_indexes);
                                
                                // construct index shifter
                                let mut states: Vec<Rc<(u8, u8)>> = Vec::new();
                                let cell_group_width = rightmost_cell_group_x - leftmost_cell_group_x + 1;
                                let cell_group_height = bottommost_cell_group_y - topmost_cell_group_y + 1;
                                for y in 1..=(bottommost_y - cell_group_height + 1) {
                                    for x in 1..=(rightmost_x - cell_group_width + 1) {
                                        let location_reference_index = (y - 1) * location_references_width + (x - 1);
                                        states.push(location_references[location_reference_index].clone());
                                    }
                                }
                                let mut index_shifter = IndexShifter::new(&vec![states]);
                                wall_adjacent_index_shifters.push(index_shifter);
                            }
                        }
                    }
                }
            }
        }

        PixelBoardRandomizer {
            pixel_board: pixel_board,
            cell_groups: Rc::new(raw_cell_groups),
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
            wall_adjacent_index_shifters: wall_adjacent_index_shifters
        }
    }
    pub fn get_random_pixel_board(&self) -> PixelBoard<TPixel> {
        let mut round_robin_incrementer: RoundRobinIncrementer<(u8, u8)>;

        {
            // TODO randomize the shifters
            
            // TODO contain the corner, unmoving cell group(s)
            // TODO contain the wall segment permutation shifter(s)
            let mut wall_adjacent_shifter_per_cell_group_index: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = Vec::new();
            let mut floater_shifter_per_cell_group_index: Vec<Rc<RefCell<dyn Shifter<T = (u8, u8)>>>> = Vec::new();
            // TODO construct individual shifters for each cell group
            let mut incrementers: Vec<Rc<RefCell<dyn Incrementer<T = (u8, u8)>>>> = Vec::new();
            // TODO construct each incrementer that equates to each possible combination of cell groups depending on their location in the bounds
            round_robin_incrementer = RoundRobinIncrementer::new(incrementers);
        }

        // prepare to find the cycle as the RoundRobinIncrementer is iterated over
        let mut located_cell_group_collections: Vec<BTreeSet<(usize, (u8, u8))>> = Vec::new();
        let mut located_cell_groups_per_location_per_cell_group_index: Vec<BTreeMap<(u8, u8), Vec<(usize, (u8, u8))>>> = Vec::new();
        let mut is_cycle_found: bool = false;
        let mut current_pair: ((usize, (u8, u8)), (usize, (u8, u8)));
        let mut is_incrementer_completed: bool = false;

        while !is_cycle_found {
            // TODO get the next set of locations
            is_incrementer_completed = round_robin_incrementer.try_increment();
            if is_incrementer_completed {
                panic!("Unexpected failure to find the original placement, let alone a new random one.");
            }
            let locations = round_robin_incrementer.get();
            // TODO treat each pair individually, iterating over each pair
            for (current_indexed_element_index, current_indexed_element) in locations.iter().enumerate() {
                for (other_indexed_element_index, other_indexed_element) in locations.iter().enumerate() {
                    if current_indexed_element_index < other_indexed_element_index {
                        // TODO determine if indexed location exists in previous data structures
                        let mut current_indexed_element_cell_group_collection_index: Option<usize> = None;
                        let mut other_indexed_element_cell_group_collection_index: Option<usize> = None;
                        let current_located_cell_group: (usize, (u8, u8)) = (current_indexed_element.index, (current_indexed_element.element.0, current_indexed_element.element.1));
                        let other_located_cell_group: (usize, (u8, u8)) = (other_indexed_element.index, (other_indexed_element.element.0, other_indexed_element.element.1));
                        for (located_cell_group_collection_index, located_cell_group_collection) in located_cell_group_collections.iter().enumerate() {
                            if located_cell_group_collection.contains(&current_located_cell_group) {
                                current_indexed_element_cell_group_collection_index = Some(located_cell_group_collection_index);
                                if other_indexed_element_cell_group_collection_index.is_some() {
                                    break;
                                }
                            }
                            if located_cell_group_collection.contains(&other_located_cell_group) {
                                other_indexed_element_cell_group_collection_index = Some(located_cell_group_collection_index);
                                if current_indexed_element_cell_group_collection_index.is_some() {
                                    break;
                                }
                            }
                        }

                        if current_indexed_element_cell_group_collection_index.is_none() {
                            if other_indexed_element_cell_group_collection_index.is_none() {
                                // if both are none, then this collection is isolated, create a new BTreeSet
                                let mut located_cell_group_collection: BTreeSet<(usize, (u8, u8))> = BTreeSet::new();
                                located_cell_group_collection.insert(current_located_cell_group);
                                located_cell_group_collection.insert(other_located_cell_group);
                                located_cell_group_collections.push(located_cell_group_collection);
                            }
                            else {
                                // the current located cell group extends the existing collection that contains the other located cell group
                                located_cell_group_collections[other_indexed_element_cell_group_collection_index.unwrap()].insert(current_located_cell_group);
                            }
                        }
                        else {
                            if other_indexed_element_cell_group_collection_index.is_none() {
                                // the other located cell group extends the existing collection that contains the current located cell group
                                located_cell_group_collections[current_indexed_element_cell_group_collection_index.unwrap()].insert(other_located_cell_group);
                            }
                            else {
                                // they both exist in either the same or different collection
                                if current_indexed_element_cell_group_collection_index == other_indexed_element_cell_group_collection_index {
                                    // they exist in the same located cell group collect and now form a cycle, test to see if this is the cycle that creates a full loop
                                    // TODO ensure that each located cell group of the cycle (for each distinct cell group) forms a cliche with each other, as a statement that they all permit each other's location
                                }
                            }
                        }

                    }
                }
            }
            
        }

        let mut location_per_cell_group_index: Vec<(u8, u8)> = Vec::new();
        // TODO start with the current_pair and search the located_cell_groups_per_location_per_cell_group_index until the cycle is found

        let mut random_pixel_board: PixelBoard<TPixel> = PixelBoard::new(self.pixel_board.get_width(), self.pixel_board.get_height());
        for (cell_group_index, location) in location_per_cell_group_index.iter().enumerate() {
            for cell in self.cell_groups[cell_group_index].cells.iter() {
                let pixel_board_coordinate = self.pixel_board_coordinate_per_cell_group_index[cell_group_index];
                let calculated_pixel_board_index_x: usize = (cell.0 + location.0) as usize;
                let calculated_pixel_board_index_y: usize = (cell.1 + location.1) as usize;
                random_pixel_board.set(calculated_pixel_board_index_x, calculated_pixel_board_index_y, self.pixel_board.get(pixel_board_coordinate.0, pixel_board_coordinate.1).unwrap());
            }
        }
        return random_pixel_board;
    }
}

// TODO add test where the walls are double thick and there is a single, adjacent floater