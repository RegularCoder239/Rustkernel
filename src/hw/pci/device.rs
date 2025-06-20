use crate::std::Box;
use core::fmt;
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
/*		match self {
			DeviceEnum::Invalid => {},
			DeviceEnum::Unknown(header) => {
				let vid = header.vendor_id;
				let hid = header.device_id;
				log::error!("Found unknown device: Class code: {:x} Subclass code: {:x} Vendor ID: {:x} Device ID: {:x}",
							header.class_code,
							header.subclass,
							vid,
							hid)
			},

		}
*/		match self {
			DeviceEnum::Drive(device) => device.scan(),
			DeviceEnum::Network(device) => device.scan(),
			DeviceEnum::Bridge(device) => device.scan(),
			DeviceEnum::Invalid => {},
			DeviceEnum::Unknown(_) => {}
		}
	}
}
/*
impl fmt::Display for DeviceEnum {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			DeviceEnum::Drive(_) => write!(f, "Drive"),
			DeviceEnum::Bridge(_) => write!(f, "Bridge"),
			DeviceEnum::Network(n) => write!(f, "{} Controller", n),
			_ => write!(f, "Unknown")
		}
	}
}
*/
