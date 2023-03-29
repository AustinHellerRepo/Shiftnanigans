use std::{cell::RefCell, rc::Rc};

use shiftnanigans::pixel_board::{Pixel, PixelBoard, pixel_board_randomizer::PixelBoardRandomizer};
use criterion::{Criterion, criterion_group, black_box};
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

fn small_plus_sign(width_and_height_tuple: (usize, usize)) {

    fastrand::seed(0);

    let top_wall_segment_image_id = Uuid::new_v4().to_string();
    let bottom_wall_segment_image_id = Uuid::new_v4().to_string();
    let left_wall_segment_image_id = Uuid::new_v4().to_string();
    let right_wall_segment_image_id = Uuid::new_v4().to_string();
    let floater_wall_segment_image_id = Uuid::new_v4().to_string();
    let board_width = width_and_height_tuple.0;
    let board_height = width_and_height_tuple.1;
    let board_x_mid = board_width / 2;
    let board_y_mid = board_height / 2;
    let mut pixel_board = PixelBoard::new(board_width, board_height);
    pixel_board.set(board_x_mid, 0, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
        image_id: top_wall_segment_image_id.clone()
    }))));
    pixel_board.set(board_width - 1, board_y_mid, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
        image_id: right_wall_segment_image_id.clone()
    }))));
    pixel_board.set(board_x_mid, board_height - 1, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
        image_id: bottom_wall_segment_image_id.clone()
    }))));
    pixel_board.set(0, board_y_mid, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
        image_id: left_wall_segment_image_id.clone()
    }))));
    for x in 1..(board_width - 1) {
        pixel_board.set(x, board_y_mid, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
            image_id: floater_wall_segment_image_id.clone()
        }))));
    }
    for y in 1..(board_height - 1) {
        if y != board_y_mid {
            pixel_board.set(board_x_mid, y, Rc::new(RefCell::new(ExamplePixel::Tile(Tile {
                image_id: floater_wall_segment_image_id.clone()
            }))));
        }
    }
    let pixel_board_randomizer = PixelBoardRandomizer::new(pixel_board);
    // get randomized pixel board and check for single possible location multiple times
    for _ in 0..10 {
        let random_pixel_board = pixel_board_randomizer.get_random_pixel_board();
        for y in 0..board_height {
            for x in 0..board_width {
                if x == board_x_mid || y == board_y_mid {
                    assert!(random_pixel_board.exists(x, y));
                    let wrapped_pixel = random_pixel_board.get(x, y).unwrap();
                    let borrowed_pixel: &ExamplePixel = &wrapped_pixel.borrow();
                    match (borrowed_pixel) {
                        ExamplePixel::Tile(tile) => {
                            if y == 0 {
                                assert_eq!(top_wall_segment_image_id, tile.image_id);
                            }
                            else if y == (board_height - 1) {
                                assert_eq!(bottom_wall_segment_image_id, tile.image_id);
                            }
                            else if x == 0 {
                                assert_eq!(left_wall_segment_image_id, tile.image_id);
                            }
                            else if x == (board_width - 1) {
                                assert_eq!(right_wall_segment_image_id, tile.image_id);
                            }
                            else {
                                assert_eq!(floater_wall_segment_image_id, tile.image_id);
                            }
                        },
                        ExamplePixel::Element(_) => {
                            panic!("Unexpected pixel type.");
                        }
                    }
                }
                else {
                    assert!(!random_pixel_board.exists(x, y));
                }
            }
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("small plus sign: 7x7", |b| b.iter(|| small_plus_sign(black_box((7, 7)))));
}

criterion_group!(benches, criterion_benchmark);