pub trait ReverseBytes {
	fn reverse_bytes(self) -> Self;
}

impl ReverseBytes for u16 {
	fn reverse_bytes(self) -> Self {
		self >> 8 | self << 8
	}
}

impl ReverseBytes for u32 {
	fn reverse_bytes(self) -> Self {
		self >> 24 | self << 24 | ((self >> 8) & 0xff00) | ((self << 8) & 0xff0000)
	}
}
