pub type Mac = [u8; 6];

#[repr(C, packed)]
pub struct Frame {
	pub destination_mac: Mac,
	pub source_mac: Mac,
	pub r#type: u16
}


impl Frame {
	pub const EMPTY: Frame = Frame {
		destination_mac: [0; 6],
		source_mac: [0xde, 0xad, 0xbe, 0xef, 0x4d, 0xad],
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
