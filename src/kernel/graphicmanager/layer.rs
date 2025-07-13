use crate::hw::graphics::{
	resolution,
	framebuffer
};
use super::{
	ColorComponent,
	RGBColor
};
use crate::std::{
	Box,
	Vec,
	LazyMutex,
	MutableCell
};
use core::marker::{
	PhantomData
};

pub struct Layer<T: ColorComponent> {
	pub z: u8,
	pub size: (usize, usize),

	pub framebuffer: &'static mut [u32],
	pub framebuffer_stride: usize,

	phantom: PhantomData<T>
}

static LAYERS: MutableCell<Vec<Layer<u8>>> = MutableCell::new(Vec::new());
static DEPTH_BUFFER: LazyMutex<Box<[u8]>> = LazyMutex::new(
	|| Box::new_zeroed(
		{
			let size = resolution().expect("Attempt to access depth image, when no display is available.");
			size.0 * size.1
		}
	)
);

impl Layer<u8> {
	pub fn add(z: u8) -> &'static mut Self {
		LAYERS.deref_mut().push_back(Self::new(z))
	}
}

impl<T: ColorComponent> Layer<T> {
	fn new(z: u8) -> Self {
		let size = resolution().expect("Attempt to create graphic layer without display.");
		let (framebuffer, framebuffer_stride) = framebuffer().unwrap();
		Layer {
			z,
			size,
			framebuffer,
			framebuffer_stride,
			phantom: PhantomData
		}
	}

	pub fn plot_pixel(&mut self, x: usize, y: usize, color: RGBColor) {
		self.framebuffer[y.strict_mul(self.framebuffer_stride) + x] = color.into();
	}

	pub fn fill_global(&mut self, color: RGBColor) {
		self.framebuffer.fill(color.into());
	}

	pub fn draw_rect(&mut self, pos: (usize, usize), size: (usize, usize), color: RGBColor) {
		for x in pos.0..pos.0 + size.0 {
			for y in pos.1..pos.1 + size.1 {
				self.plot_pixel(x, y, color.clone());
			}
		}
	}
}
