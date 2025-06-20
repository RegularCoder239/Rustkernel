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
		let mut checksum: u64 =
			header.dest_ip.reverse_bytes() as u64 +
			header.source_ip.reverse_bytes() as u64 +
			(data.len() as u16 + 8).reverse_bytes() as u64 +
			17_u16.reverse_bytes() as u64;
		//for b in data.windows(2) {
		//	checksum += ((b[0] as u64) << 8) | b[1] as u64;
		//}
		let mut boxed = Box::<UDPPackage>::new_sized(data.len() + core::mem::size_of::<UDPPackage>());
		*boxed = UDPPackage {
			header: header,
			srcport: src.reverse_bytes(),
			dstport: dst.reverse_bytes(),
			size: (data.len() as u16 + 8).reverse_bytes(),
			checksum: 0,//((checksum & 0xffff) as u16 + if checksum >> 16 != 0 { 2 } else { 0 }).reverse_bytes(),
			data: 0,
			crc: 0
		};
		boxed.header.len = ((data.len() + core::mem::size_of::<UDPPackage>() - 14) as u16).reverse_bytes();
		boxed.header.calculate_checksum();
		unsafe {
			(&mut boxed.data as *mut u8).copy_from(data.as_ptr(), data.len());
			(&mut boxed.data as *mut u8 as *mut u32).add(data.len()).write_unaligned(
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
