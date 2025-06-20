use super::Frame;

#[repr(C, packed)]
pub struct IPHeader {
	pub frame: Frame,
	pub version_hlen: u8, // Higher 4 Bits: Version Lower 4 Bits: Header Length
	pub type_of_service: u8,
	pub len: u16,
	pub id: u16,
	pub fragment_offset: u16,
	pub time_to_live: u8,
	pub protocol: u8,
	pub checksum: u16,
	pub source_ip: u32,
	pub dest_ip: u32
}

#[derive(Copy, Clone)]
pub enum Protocol {
	ICMP = 1,
	TCP = 6,
	UDP = 17
}

const CRC_16_IP_ALGO: crc::Algorithm<u16> = crc::Algorithm {
	width: 16,
	poly: 0x0,
	init: 0xffff,
	refin: false,
	refout: false,
	xorout: 0x0000,
	check: 0xaee7,
	residue: 0x0000
};
const CRC_16_IP: crc::Crc<u16> = crc::Crc::<u16>::new(&CRC_16_IP_ALGO);

impl IPHeader {
	pub fn new(src: u32, dst: u32, protocol: Protocol, frame: Frame) -> IPHeader {
		IPHeader {
			frame: frame,
			version_hlen: 0x45,
			type_of_service: 0x0,
			len: 0x1_u16.reverse_bits(),
			id: 0x0,
			fragment_offset: 0x0,
			time_to_live: 0xff,
			checksum: 0,
			protocol: protocol as u8,
			source_ip: src,
			dest_ip: dst
		}
	}

	pub fn calculate_checksum(&mut self) {
		let mut checksum = 0_u64;
		for d in unsafe { core::slice::from_raw_parts(
			&self.version_hlen as *const u8 as *const u16,
			10
		) } {
			checksum += *d as u64;
		}
		checksum += (checksum >> 16) & 0xf;
		self.checksum = !(checksum & 0xffff) as u16;
	}
}
