use core::fmt;

#[derive(Clone, Copy)]
pub struct Mac([u8; 6]);

#[repr(C, packed)]
pub struct Frame {
	pub destination_mac: Mac,
	pub source_mac: Mac,
	pub r#type: u16
}


impl Frame {
	pub const EMPTY: Frame = Frame {
		destination_mac: Mac([0x0; 6]),
		source_mac: Mac([0x0; 6]),
		r#type: 0x8,
	};

	pub fn new(src: Mac, dst: Mac) -> Frame {
		Frame {
			destination_mac: dst,
			source_mac: src,
			..Self::EMPTY
		}
	}
}

impl From<[u8; 6]> for Mac {
	fn from(mac: [u8; 6]) -> Self {
		Mac(mac)
	}
}

impl fmt::Display for Mac {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(fmt, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
			   self.0[0],
			   self.0[1],
			   self.0[2],
			   self.0[3],
			   self.0[4],
			   self.0[5])
	}
}
