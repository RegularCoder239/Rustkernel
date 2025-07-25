mod consolelayer;
mod layer;
mod font;
mod image;

pub use consolelayer::{
	console
};
pub use image::Image;
pub use layer::Layer;

pub trait ColorComponent: Copy + Clone {
	fn from_u8(value: u8) -> Self;
	fn into_u8(self) -> u8;
}

#[derive(Copy, Clone)]
pub struct RGBColor {
	r: u8,
	g: u8,
	b: u8
}

impl RGBColor {
	const CONSOLE_BG: RGBColor = RGBColor::from_u8(0x25, 0x25, 0x25);
	const CONSOLE_FG: RGBColor = RGBColor::from_u8(0xe0, 0xe0, 0xe0);
	const WHITE: RGBColor = RGBColor::from_u8(0xff, 0xff, 0xff);

	const fn from_u8(r: u8, g: u8, b: u8) -> RGBColor {
		RGBColor {
			r,
			g,
			b
		}
	}
	fn from<T: ColorComponent>(r: T, g: T, b: T) -> RGBColor {
		RGBColor {
			r: r.into_u8(),
			g: g.into_u8(),
			b: b.into_u8()
		}
	}
}

impl Into<u32> for RGBColor {
	fn into(self) -> u32 {
		(self.r as u32) |
		(self.g as u32) << 8 |
		(self.b as u32) << 16
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
impl ColorComponent for f32 {
	fn from_u8(value: u8) -> Self {
		value as f32 / 255.0_f32
	}
	fn into_u8(self) -> u8 {
		(self * 255.0_f32) as u8
	}
}

pub fn setup_console_task() -> ! {
	consolelayer::init_console();

	crate::std::exit();
}
