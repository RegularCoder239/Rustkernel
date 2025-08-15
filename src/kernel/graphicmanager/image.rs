/*
 * Unused for now
 */

use crate::std::{
	Box
};
use super::{
	RGBColor
};

pub struct Image {
	data: Box<[u32]>,
	size: (usize, usize)
}

impl Image {
	pub fn new(size: (usize, usize)) -> Self {
		Image {
			data: Box::new_sized(size.0 * size.1 * 4),
			size,
		}
	}

	pub fn pixel_mut(&mut self, x: usize, y: usize) -> &mut u32 {
		&mut self.data[y * self.size.0 + x]
	}

	pub fn fill(&mut self, color: RGBColor) {
		self.data.as_slice_mut().fill(color.into())
	}

	pub fn raw(&self) -> &[u32] {
		self.data.as_slice()
	}
}
