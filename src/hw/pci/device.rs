use crate::std::{
	Box,
	log
};
use super::{
	Header,
	Bridge,
	NetworkController,
	UnspecifiedDrive
};

pub trait DeviceTrait {
	fn scan(&self) {
		self.specific_scan();
	}
	fn specific_scan(&self);
}

pub struct UnspecifiedDeviceStruct {
	header: Header
}

pub enum DeviceEnum {
	Drive(UnspecifiedDrive),
	Network(NetworkController),
	Bridge(Bridge),
	Unknown(Header),
	Invalid
}

pub type UnspecifiedDevice = Box<UnspecifiedDeviceStruct>;

impl UnspecifiedDevice {
	fn get_specific_device(&self) -> DeviceEnum {
		match self.header.class_code {
			1 => DeviceEnum::Drive(
				UnspecifiedDrive::from_raw_address(self.physical_address())
			),
			2 => DeviceEnum::Network(
				NetworkController::from_raw_address(self.physical_address())
			),
			6 => DeviceEnum::Bridge(
				Bridge::from_raw_address(self.physical_address())
			),
			0xff => DeviceEnum::Invalid,
			_ => DeviceEnum::Unknown(self.header.clone())
		}
	}
}

impl DeviceTrait for UnspecifiedDevice {
	fn specific_scan(&self) {
		if self.header.class_code == 0xff {
			return;
		}
		log::info!("Found device: {}", self.header);
		self.get_specific_device().scan()
	}
}

impl DeviceTrait for DeviceEnum {
	fn specific_scan(&self) {
		match self {
			DeviceEnum::Drive(device) => device.scan(),
			DeviceEnum::Network(device) => device.scan(),
			DeviceEnum::Bridge(device) => device.scan(),
			DeviceEnum::Invalid => {},
			DeviceEnum::Unknown(_) => {}
		}
	}
}
