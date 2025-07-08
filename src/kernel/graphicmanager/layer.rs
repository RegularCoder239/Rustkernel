use crate::hw::graphics::{
	resolution,
	framebuffer
};
use super::{
	ColorComponent,
	RGBColor,
	Image
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

	framebuffer: &'static mut [u32],

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
		Layer {
			z,
			size,
			framebuffer: framebuffer().unwrap(),
			phantom: PhantomData
		}
	}

	pub fn plot_pixel(&mut self, x: usize, y: usize, color: RGBColor<T>) {
		self.framebuffer[y * self.size.0 + x] = color.into();
	}

	pub fn fill(&mut self, color: RGBColor<T>) {
		let raw_color = color.into();
		for (idx, d) in DEPTH_BUFFER.lock().as_slice().into_iter().enumerate() {
			if *d <= self.z {
				self.framebuffer[idx] = raw_color;
			}
		}
	}

	pub fn draw_rect(&mut self, pos: (usize, usize), size: (usize, usize), color: RGBColor<T>) {
		for x in pos.0..pos.0 + size.0 {
			for y in pos.1..pos.1 + size.1 {
				self.plot_pixel(x, y, color.clone());
			}
		}
	}
}
