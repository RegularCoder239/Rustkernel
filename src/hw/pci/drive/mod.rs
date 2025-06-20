use super::{
	HeaderType0,
	DeviceTrait
};
use crate::std::Box;

pub struct UnspecifiedDriveStruct(HeaderType0);

pub type UnspecifiedDrive = Box<UnspecifiedDriveStruct>;

impl DeviceTrait for UnspecifiedDrive {
	fn specific_scan(&self) {
		//self.get_specific_device().scan()
	}
}
/*
impl UnspecifiedDrive {
	fn get_specific_device() -> impl DeviceTrait {
		match self.0.header.subclass {
			8 => NVMEDrive
		}
	}
}
*/
