mod display;
mod color;

use display::{
	display_framebuffer
};

use color::{
	Color
};
use crate::std::{
	exit,
	log
};

fn fill_display(color: Color) {
	if let Some(display) = display_framebuffer() {
		display.fill(color.to_u32());
	}
}

pub fn setup() -> ! {
	log::info!("Setting up graphics.");
	fill_display(Color::BG);

	exit()
}
