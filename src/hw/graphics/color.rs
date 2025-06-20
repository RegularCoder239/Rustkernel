pub struct Color {
	r: u8,
	g: u8,
	b: u8,
	a: u8
}

impl Color {
	pub const BG: Color = Color::new(25, 25, 25);

	pub const fn new(r: u8, g: u8, b: u8) -> Color {
		Color {
			r: r,
			g: g,
			b: b,
			a: 0xff
		}
	}

	pub const fn to_u32(&self) -> u32 {
		(self.r as u32) |
		(self.g as u32) << 8 |
		(self.b as u32) << 16 |
		(self.a as u32) << 24
	}
}
