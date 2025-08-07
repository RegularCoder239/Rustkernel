use super::{
	RGBColor,
	Layer
};
use crate::std::{
	LazyMutex,
	LazyMutexGuard,
	Mutex
};

pub struct ConsoleLayer {
	background_color: RGBColor,
	foreground_color: RGBColor,

	bitmap_font: &'static [u128; 256],
	char_amount: (usize, usize),

	cursor_pos: (usize, usize),

	layer: &'static Mutex<Layer>
}

static CONSOLE_LAYER: LazyMutex<ConsoleLayer> = LazyMutex::new(|| {
	ConsoleLayer::new()
});

impl ConsoleLayer {
	const GLYPH_SIZE: (usize, usize) = (8, 16);
	fn new() -> Self {
		let layer = Layer::add(0);
		ConsoleLayer {
			background_color: RGBColor::CONSOLE_BG,
			foreground_color: RGBColor::CONSOLE_FG,
			bitmap_font: &super::font::FONT,
			char_amount: (layer.size.0 / Self::GLYPH_SIZE.0,
						  layer.size.1 / Self::GLYPH_SIZE.1),
			cursor_pos: (0, 0),
			layer
		}
	}

	pub fn setup(&mut self) {
		self.layer.lock().fill_global(self.background_color);
		self.update_cursor();
	}

	fn update_cursor(&mut self) {
		self.layer.lock().draw_rect(
			(self.cursor_pos.0 * Self::GLYPH_SIZE.0, self.cursor_pos.1 * Self::GLYPH_SIZE.1 + Self::GLYPH_SIZE.1 - 4),
			(Self::GLYPH_SIZE.0, 4),
			RGBColor::CONSOLE_FG
		)
	}

	pub fn print_str(&mut self, char_list: &str) {
		for ch in char_list.chars() {
			self.print_char(ch);
		}
		self.update_cursor();
	}

	fn print_char(&mut self, ch: char) {
		if ch == '\n' {
			self.clear_char(
				(self.cursor_pos.0 * Self::GLYPH_SIZE.0, self.cursor_pos.1 * Self::GLYPH_SIZE.1)
			);
			self.cursor_pos.0 = 0;
			self.cursor_pos.1 += 1;
			if self.cursor_pos.1 == self.char_amount.1 {

				self.clear();
			}
		} else {
			self.draw_char(
				(self.cursor_pos.0 * Self::GLYPH_SIZE.0, self.cursor_pos.1 * Self::GLYPH_SIZE.1),
				ch as u8
			);
			self.cursor_pos.0 += 1;
		}
	}

	fn draw_char(&mut self, pos: (usize, usize), ch: u8) {
		let ch = self.bitmap_font[ch as usize - 0x20].reverse_bits();
		for x in 0..Self::GLYPH_SIZE.0 {
			for y in 0..Self::GLYPH_SIZE.1 {
				if (ch >> (y * Self::GLYPH_SIZE.0 + x)) & 0x1 == 0x1 {
					self.layer.lock().plot_pixel(x + pos.0, y + pos.1, RGBColor::CONSOLE_FG);
				} else {
					self.layer.lock().plot_pixel(x + pos.0, y + pos.1, RGBColor::CONSOLE_BG);
				}
			}
		}
	}
	fn clear(&mut self) {
		self.cursor_pos = (0, 0);
		for x in 0..self.char_amount.0 {
			for y in 0..self.char_amount.1 {
				self.clear_char((x * Self::GLYPH_SIZE.0, y * Self::GLYPH_SIZE.1));
			}
		}
	}
	fn clear_char(&mut self, pos: (usize, usize)) {
		for x in 0..Self::GLYPH_SIZE.0 {
			for y in 0..Self::GLYPH_SIZE.1 {
				self.layer.lock().plot_pixel(x + pos.0, y + pos.1, RGBColor::CONSOLE_BG);
			}
		}
	}
}

pub fn console() -> Option<LazyMutexGuard<'static, ConsoleLayer>> {
	if crate::hw::graphics::available() && CONSOLE_LAYER.is_initalized() {
		Some(CONSOLE_LAYER.lock())
	} else {
		None
	}
}

pub fn init_console() {
	CONSOLE_LAYER.lock().setup();
}
