mod consolelayer;
mod layer;
mod font;
mod image;

pub use consolelayer::{
	console,
	ConsoleLayer
};
pub use image::Image;
pub use layer::Layer;

use core::marker::PhantomData;

pub trait ColorComponent: Copy + Clone {
	fn from_u8(value: u8) -> Self;
	fn into_u8(self) -> u8;
}

#[derive(Copy, Clone)]
pub struct RGBColor<T: ColorComponent> where T: Clone + Copy {
	r: u8,
	g: u8,
	b: u8,
	phantom: PhantomData<T>
}

impl<T: ColorComponent> RGBColor<T> {
	const CONSOLE_BG: RGBColor<T> = RGBColor::from_u8(0x25, 0x25, 0x25);
	const CONSOLE_FG: RGBColor<T> = RGBColor::from_u8(0xe0, 0xe0, 0xe0);

	const fn from_u8(r: u8, g: u8, b: u8) -> RGBColor<T> {
		RGBColor {
			r,
			g,
			b,
			phantom: PhantomData
		}
	}
}

impl<T: ColorComponent> Into<u32> for RGBColor<T> {
	fn into(self) -> u32 {
		(Into::<u8>::into(self.r) as u32) |
		(Into::<u8>::into(self.g) as u32) << 8 |
		(Into::<u8>::into(self.b) as u32) << 16
	}
}

impl ColorComponent for u8 {
	fn from_u8(value: u8) -> Self {
		value
	}
	fn into_u8(self) -> u8 {
		self
	}
}

pub fn setup_console_task() -> ! {
	consolelayer::init_console();
	crate::std::log::info!("Test123");
	crate::std::exit();
}
