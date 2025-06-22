mod nvme;

pub use nvme::NVMEHeader;

use super::{
	HeaderType0,
	DeviceTrait
};
use crate::std::Box;

pub struct UnspecifiedDriveStruct(HeaderType0);

pub type UnspecifiedDrive = Box<UnspecifiedDriveStruct>;

impl DeviceTrait for UnspecifiedDrive {
	fn specific_scan(&self) {
		match self.0.header.subclass {
			8 => NVMEHeader::from_raw_address(self.physical_address()).scan(),
			_ => {}
		}
	}
}
