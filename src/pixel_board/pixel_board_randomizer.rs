use std::{rc::Rc, cell::RefCell, collections::{BTreeSet, BTreeMap, VecDeque}};

use crate::{CellGroup, incrementer::{round_robin_incrementer::RoundRobinIncrementer, Incrementer}, shifter::{Shifter, segment_permutation_shifter::{Segment, SegmentPermutationShifter}, index_shifter::IndexShifter}};

use super::{PixelBoard, Pixel};

// TODO construct an undirected graph, search the graph starting with one of the newest edges, doing a depth-first search starting with the newest edge, only permitting the next node to be a cell group not yet traveled to and a location not yet traveled to.
//          add each new edge one at a time, performing the search per new edge.


pub struct PixelBoardRandomizer<TPixel: Pixel> {
    pixel_board: PixelBoard<TPixel>,
    cell_groups: Rc<Vec<CellGroup>>,
    pixel_board_index_pairs_per_cell_group_index: Vec<Vec<(usize, usize)>>
}

impl<TPixel: Pixel> PixelBoardRandomizer<TPixel> {
    pub fn new(pixel_board: PixelBoard<TPixel>) -> Self {
        let mut raw_cell_groups: Vec<CellGroup> = Vec::new();
        // contains the pixel board coordinates that map to which cell group
        let mut pixel_board_index_pairs_per_cell_group_index: Vec<Vec<(usize, usize)>> = Vec::new();
        // TODO identify each cell group (wall, wall-adjacent, and floater)
        {
            // contains the cell group indexes for each potential corner wall
            let mut top_left_corner_wall_cell_group_index: Option<usize> = None;
            let mut top_right_corner_wall_cell_group_index: Option<usize> = None;
            let mut bottom_left_corner_wall_cell_group_index: Option<usize> = None;
            let mut bottom_right_corner_wall_cell_group_index: Option<usize> = None;
            let mut top_left_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
            let mut top_right_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
            let mut bottom_right_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
            let mut bottom_left_corner_wall_index_shifter_option: Option<IndexShifter<(u8, u8)>> = None;
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
                bottom_left_corner_wall_index_shifter_option = Some(IndexShifter::new(&vec![
                    vec![Rc::new((0, topmost_cell_y as u8))]
                ]));
            }

            // collect the wall segments per wall side
            let mut top_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
            let mut right_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
            let mut bottom_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
            let mut left_wall_segment_cell_group_indexes: Vec<usize> = Vec::new();
            let mut top_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;
            let mut right_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;
            let mut bottom_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;
            let mut left_wall_segment_permutation_shifter_option: Option<SegmentPermutationShifter> = None;

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
                    for x in leftmost_wall_x..=rightmost_wall_x {
                        if pixel_board.exists(x, 0) {
                            current_segment_length += 1;
                            cells.push((x as u8, 0));
                            adjacent_pixel_board_coordinates.insert((x, 1));
                        }
                        else if current_segment_length != 0 {
                            top_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                        }
                    }
                    if current_segment_length != 0 {
                        top_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                    }
                    let mut top_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (leftmost_wall_x as u8, 0), rightmost_wall_x - leftmost_wall_x + 1, true, 1, false);
                    top_wall_segment_permutation_shifter.randomize();
                    top_wall_segment_permutation_shifter_option = Some(top_wall_segment_permutation_shifter);
                }
            }

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
                    for x in leftmost_wall_x..=rightmost_wall_x {
                        if pixel_board.exists(x, bottommost_y) {
                            current_segment_length += 1;
                            cells.push((x as u8, bottommost_y as u8));
                            adjacent_pixel_board_coordinates.insert((x, bottommost_y - 1));
                        }
                        else if current_segment_length != 0 {
                            bottom_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                        }
                    }
                    if current_segment_length != 0 {
                        bottom_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                    }
                    let mut bottom_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (leftmost_wall_x as u8, bottommost_y as u8), rightmost_wall_x - leftmost_wall_x + 1, true, 1, false);
                    bottom_wall_segment_permutation_shifter.randomize();
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
                    for y in topmost_wall_y..=bottommost_wall_y {
                        if pixel_board.exists(0, y) {
                            current_segment_length += 1;
                            cells.push((0, y as u8));
                            adjacent_pixel_board_coordinates.insert((1, y));
                        }
                        else if current_segment_length != 0 {
                            left_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                        }
                    }
                    if current_segment_length != 0 {
                        left_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                    }
                    let mut left_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (0, topmost_wall_y as u8), bottommost_wall_y - topmost_wall_y + 1, false, 1, false);
                    left_wall_segment_permutation_shifter.randomize();
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
                    for y in topmost_wall_y..=bottommost_wall_y {
                        if pixel_board.exists(rightmost_x, y) {
                            current_segment_length += 1;
                            cells.push((rightmost_x as u8, y as u8));
                            adjacent_pixel_board_coordinates.insert((rightmost_x - 1, y));
                        }
                        else if current_segment_length != 0 {
                            right_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                            segments.push(Rc::new(Segment::new(current_segment_length)));
                            raw_cell_groups.push(CellGroup {
                                cells: cells
                            });
                            adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                            // reset for the next potential wall segment
                            current_segment_length = 0;
                            cells = Vec::new();
                            adjacent_pixel_board_coordinates = BTreeSet::new();
                        }
                    }
                    if current_segment_length != 0 {
                        right_wall_segment_cell_group_indexes.push(raw_cell_groups.len());
                        segments.push(Rc::new(Segment::new(current_segment_length)));
                        raw_cell_groups.push(CellGroup {
                            cells: cells
                        });
                        adjacent_pixel_board_coordinates_per_cell_group_index.push(adjacent_pixel_board_coordinates);
                    }
                    let mut right_wall_segment_permutation_shifter = SegmentPermutationShifter::new(segments, (rightmost_x as u8, topmost_wall_y as u8), bottommost_wall_y - topmost_wall_y + 1, false, 1, false);
                    right_wall_segment_permutation_shifter.randomize();
                    right_wall_segment_permutation_shifter_option = Some(right_wall_segment_permutation_shifter);
                }
            }

            // at this point the corner walls and the wall segments have been discovered

            let mut wall_adjacent_cell_group_indexes: Vec<usize> = Vec::new();
            
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

                    // TODO incorporate adjacent vector to determining which cell group indexes are adjacent to each wall-adjacent as they are being constructed

                    let mut possible_wall_adjacent_coordinates_set: BTreeSet<(usize, usize)> = BTreeSet::new();
                    for cell_group_index in top_wall_segment_cell_group_indexes.iter() {
                        for cell in raw_cell_groups[*cell_group_index].cells.iter() {
                            let pixel_coordinate: (usize, usize) = (cell.0 as usize, cell.1 as usize + 1);
                            if pixel_board.exists(pixel_coordinate.0, pixel_coordinate.1) && !visited_pixel_board_coordinates.contains(&pixel_coordinate) {
                                let mut coordinate_stack: Vec<(usize, usize)> = vec![pixel_coordinate];
                                while !coordinate_stack.is_empty() {
                                    let coordinate = coordinate_stack.pop().unwrap();
                                    if coordinate.0 != 1 {
                                        // check if there is a pixel to the left
                                        let other_coordinate = (coordinate.0 - 1, coordinate.1);
                                        if !visited_pixel_board_coordinates.contains(&other_coordinate) {
                                            coordinate_stack.push(other_coordinate);
                                        }
                                    }
                                    if coordinate.0 != rightmost_x - 1 {
                                        // check if there is a pixel to the right
                                        let other_coordinate = (coordinate.0 + 1, coordinate.1);
                                        if !visited_pixel_board_coordinates.contains(&other_coordinate) {
                                            coordinate_stack.push(other_coordinate);
                                        }
                                    }
                                    if coordinate.1 != 1 {
                                        // check if there is a pixel to the top
                                        let other_coordinate = (coordinate.0, coordinate.1 - 1);
                                        if !visited_pixel_board_coordinates.contains(&other_coordinate) {
                                            coordinate_stack.push(other_coordinate);
                                        }
                                    }
                                    if coordinate.1 != bottommost_y - 1 {
                                        // check if there is a pixel to the bottom
                                        let other_coordinate = (coordinate.0, coordinate.1 + 1);
                                        if !visited_pixel_board_coordinates.contains(&other_coordinate) {
                                            coordinate_stack.push(other_coordinate);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut pixel_board_index_pair_queue: VecDeque<(usize, usize)> = VecDeque::new();

        }
        let cell_groups: Rc<Vec<CellGroup>> = Rc::new(raw_cell_groups);

        PixelBoardRandomizer {
            pixel_board: pixel_board,
            cell_groups: cell_groups,
            pixel_board_index_pairs_per_cell_group_index: pixel_board_index_pairs_per_cell_group_index
        }
    }
    pub fn get_random_pixel_board(&self) -> PixelBoard<TPixel> {
        let mut round_robin_incrementer: RoundRobinIncrementer<(u8, u8)>;

        {
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
            for pixel_board_index_pair in self.pixel_board_index_pairs_per_cell_group_index[cell_group_index].iter() {
                let calculated_pixel_board_index_x: usize = pixel_board_index_pair.0 + location.0 as usize;
                let calculated_pixel_board_index_y: usize = pixel_board_index_pair.1 + location.1 as usize;
                random_pixel_board.set(calculated_pixel_board_index_x, calculated_pixel_board_index_y, self.pixel_board.get(pixel_board_index_pair.0, pixel_board_index_pair.1).unwrap());
            }
        }
        return random_pixel_board;
    }
}