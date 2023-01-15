use std::{rc::Rc, cell::RefCell};
pub mod pixel_board_randomizer;

pub trait Pixel {
    fn get_invalid_location_offsets_for_other_pixel(other_pixel: Self) -> Vec<(i16, i16)>;
}

pub struct PixelBoard<T: Pixel> {
    width: usize,
    height: usize,
    pixels: Vec<Option<Rc<RefCell<T>>>>
}

impl<T: Pixel> PixelBoard<T> {
    pub fn new(width: usize, height: usize) -> Self {
        let mut pixels = Vec::new();
        for _ in 0..(width * height) {
            pixels.push(None);
        }
        PixelBoard {
            width: width,
            height: height,
            pixels: pixels
        }
    }
    pub fn set(&mut self, x: usize, y: usize, pixel: Rc<RefCell<T>>) {
        let index = y * self.width + x;
        let _ = self.pixels[index].insert(pixel);
    }
    pub fn exists(&self, x: usize, y: usize) -> bool {
        let index = y * self.width + x;
        self.pixels[index].is_some()
    }
    pub fn get(&self, x: usize, y: usize) -> Option<Rc<RefCell<T>>> {
        let index = y * self.width + x;
        self.pixels[index].clone()
    }
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
}
