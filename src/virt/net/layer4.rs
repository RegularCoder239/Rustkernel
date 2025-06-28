use crate::std::{
	Box,
	ReverseBytes
};
use super::IPHeader;

#[repr(C, packed)]
pub struct UDPPackage {
	pub header: IPHeader,
	pub srcport: u16,
	pub dstport: u16,
	pub size: u16,
	pub checksum: u16,
	pub data: u8,
	pub crc: u32
}

const CRC_32_ETHERNET: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);

impl UDPPackage {
	pub fn new(src: u16, dst: u16, header: IPHeader, data: &[u8]) -> Box<UDPPackage> {
		let mut boxed = Box::<UDPPackage>::new_sized(data.len() + core::mem::size_of::<UDPPackage>());
		*boxed = UDPPackage {
			header: header,
			srcport: src.reverse_bytes(),
			dstport: dst.reverse_bytes(),
			size: (data.len() as u16 + 8).reverse_bytes(),
			checksum: 0,
			data: 0,
			crc: 0
		};
		boxed.header.len = ((data.len() + core::mem::size_of::<UDPPackage>() - 14) as u16).reverse_bytes();
		boxed.header.calculate_checksum();
		unsafe {
			(&mut boxed.data as *mut u8).copy_from(data.as_ptr(), data.len());
			(&mut boxed.data as *mut u8 as *mut u32).byte_add(data.len()).write_unaligned(
				CRC_32_ETHERNET.checksum(
					core::slice::from_raw_parts(
						boxed.as_ptr::<u8>(),
						data.len() + core::mem::size_of::<UDPPackage>()
					)
				)
			);
		}
		boxed
	}
}
