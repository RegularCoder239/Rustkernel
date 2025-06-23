use crate::std::Box;
use core::fmt;

#[derive(Clone)]
#[repr(C, packed)]
pub struct Header {
	pub vendor_id: u16,
	pub device_id: u16,
	command_register: u16,
	status_register: u16,
	revision_id: u8,
	prog_if: u8,
	pub subclass: u8,
	pub class_code: u8,
	cache_line_size: u8,
	latency_timer: u8,
	header_type: u8,
	bist: u8
}

#[repr(C, packed)]
pub struct HeaderType0 {
	pub header: Header,
	pub bar_addresses: [u32; 6],
	cardbus_cis_pointer: u32,
	subsystem_vendor_id: u16,
	subsystem_id: u16,
	erom_base_address: u32,
	capabilities: u32,
	reserved: u32,
	interrupt_line: u8,
	interrupt_pin: u8
}

impl Header {
	fn from_raw_address(addr: u64) -> Box<Header> {
		Box::<Header>::from_raw_address(addr)
	}
}
impl Box<Header> {
	fn header_type_0(&self) -> Option<Box<HeaderType0>> {
		if self.header_type == 0 {
			Some(
				Box::from_raw_address(self.physical_address())
			)
		} else {
			None
		}
	}
}


impl fmt::Display for Header {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match self.class_code {
			1 => "Drive (",
			2 => "Network Controller (",
			6 => "Bridge (",
			_ => "Unknown ("
		})?;
		write!(f, "{})", match (self.class_code as u16) << 8 | self.subclass as u16 {
			0x108 => "NVME",
			0x200 => "Ethernet",
			0x600 => "Host",
			_ => "Unknown",
		})
	}
}
