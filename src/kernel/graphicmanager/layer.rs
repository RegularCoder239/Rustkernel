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
	VecBase,
	LazyMutex,
	Mutex,
	random
};
use core::ops::Index;

pub struct Layer {
	pub z: u8,
	pub size: (usize, usize),
	pub id: u64,

	pub framebuffer: &'static mut [u32],
	pub framebuffer_stride: usize
}

static LAYERS: Mutex<Vec<Mutex<Layer>>> = Mutex::new(Vec::new());
static DEPTH_BUFFER: LazyMutex<Box<[u8]>> = LazyMutex::new(
	|| Box::new_zeroed(
		{
			let size = resolution().expect("Attempt to access depth image, when no display is available.");
			let (_, stride) = framebuffer().unwrap();
			stride * size.1
		}
	)
);

impl Layer {
	pub fn add(z: u8) -> &'static Mutex<Self> {
		LAYERS.lock().push_back(Mutex::new(Self::new(z)));
		LAYERS.read().index(LAYERS.len() - 1)
	}
	pub fn by_id(id: u64) -> Option<&'static Mutex<Layer>> {
		LAYERS.read().into_iter().find(|x| x.id == id)
	}
	fn new(z: u8) -> Self {
		let size = resolution().expect("Attempt to create graphic layer without display.");
		let (framebuffer, framebuffer_stride) = framebuffer().unwrap();
		Layer {
			z,
			size,
			framebuffer,
			framebuffer_stride,
			id: random()
		}
	}

	pub fn plot_raw_pixel(&mut self, x: usize, y: usize, raw_color: u32) {
		let idx = y * self.framebuffer_stride + x;
		if DEPTH_BUFFER.lock()[idx] <= self.z {
			DEPTH_BUFFER.lock()[idx] = self.z;
			self.framebuffer[idx] = raw_color;
		}
	}
	pub fn plot_pixel(&mut self, x: usize, y: usize, color: RGBColor) {
		self.plot_raw_pixel(x, y, color.into());
	}
	pub fn fill_global(&mut self, color: RGBColor) {
		self.framebuffer.fill(color.into());
	}
	pub fn draw_rect(&mut self, pos: (usize, usize), size: (usize, usize), color: RGBColor) {
		let raw_color: u32 = color.into();
		for x in pos.0..pos.0 + size.0 {
			for y in pos.1..pos.1 + size.1 {
				self.plot_raw_pixel(x, y, raw_color);
			}
		}
	}
}

unsafe impl Sync for Layer {}
