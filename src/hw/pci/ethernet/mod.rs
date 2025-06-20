mod rtl8139;

use crate::std::{
	Box,
	Mutex,
	Vec
};
use super::{
	DeviceTrait,
	HeaderType0
};
use core::{
	fmt,
	ops::DerefMut
};
use rtl8139::RTL8139;
use crate::virt::net;

trait NetworkDeviceTrait {
	fn setup(&mut self);
	fn mac(&self) -> [u8; 6];
	fn send_package(&mut self, frame: *mut u8, len: usize);
}

pub struct NetworkControllerStruct(HeaderType0);

enum Device {
	RTL8139(Box<RTL8139>),
	Unknown
}

pub type NetworkController = Box<NetworkControllerStruct>;

pub struct NetworkDevice {
	pci_header: NetworkController,
	device: Device
}

pub static DEVICES: Mutex<Vec<NetworkDevice>> = Mutex::new(Vec::new());

impl DeviceTrait for NetworkController {
	fn specific_scan(&self) {
		DEVICES.lock().push_back(NetworkDevice {
			pci_header: unsafe {
				Box::new_converted(self)
			},
			device: match self.0.header.device_id as u32 | ((self.0.header.vendor_id as u32) << 16) {
				0x10ec8139 => Device::RTL8139(Box::<RTL8139>::from_raw_address(self.0.bar_addresses[1] as u64)),
				_ => Device::Unknown
			}
		});
	}
}

impl fmt::Display for NetworkController {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.0.header.subclass {
			0 => write!(f, "Ethernet"),
			_ => write!(f, "Unknown")
		}
	}
}

impl NetworkDevice {
	fn device(&self) -> &Box<impl NetworkDeviceTrait> {
		match &self.device {
			Device::RTL8139(d) => d,
			_ => panic!("Unknown network device.")
		}
	}
	fn device_mut(&mut self) -> &mut Box<impl NetworkDeviceTrait> {
		match &mut self.device {
			Device::RTL8139(d) => d,
			_ => panic!("Unknown network device.")
		}
	}
	pub fn setup(&mut self) {
		self.device_mut().setup();
	}
	pub fn mac(&self) -> [u8; 6] {
		self.device().mac()
	}
}

pub fn setup_devices() {
	let mut device_lock = DEVICES.lock();
	for d in device_lock.deref_mut() {
		d.setup();
	}
}
