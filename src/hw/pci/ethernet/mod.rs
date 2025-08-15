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
use crate::virt::net::Mac;

/*
 * This trait implements commands to a network controller.
 */
trait NetworkDeviceTrait {
	/*
	 * Early setup method. This method shall be called before
	 * any other method in this trait.
	 */
	fn setup(&mut self);

	/*
	 * Returns the on the network card stored mac address.
	 */
	fn mac(&self) -> Mac;

	/*
	 * Send layer 2 ethernet frame to the network card.
	 */
	fn send_package(&mut self, frame: *mut u8, len: usize);
}

/*
 * The enum for specifing the type of network card.
 * Currently only the RTL8139 is implemented
 */
enum Device {
	RTL8139(Box<RTL8139>),
	Unknown
}

pub struct NetworkController(Box<HeaderType0>);

pub struct NetworkDevice {
	pci_header: NetworkController,
	device: Device
}

pub static DEVICES: Mutex<Vec<NetworkDevice>> = Mutex::new(Vec::new());

impl NetworkController {
	pub fn from_raw_address(addr: u64) -> NetworkController {
		NetworkController(
			Box::from_raw_address(addr)
		)
	}
}

impl DeviceTrait for NetworkController {
	fn specific_scan(&self) {
		DEVICES.lock().push_back(NetworkDevice {
			device: match self.0.header.device_id as u32 | ((self.0.header.vendor_id as u32) << 16) {
				0x10ec8139 => Device::RTL8139(Box::<RTL8139>::from_raw_address(self.0.bar_addresses[1] as u64)),
				_ => Device::Unknown
			},
			pci_header: NetworkController::from_raw_address(self.0.physical_address())
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
	/*
	 * Returns the stored network device.
	 */
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
	/*
	 * Sets up the underlying network device.
	 */
	pub fn setup(&mut self) {
		self.device_mut().setup();
	}

	/*
	 * Returns the on the network card stored mac address.
	 */
	pub fn mac(&self) -> Mac {
		self.device().mac()
	}
}

/*
 * Sets up all added network devices for sending packages.
 */
pub fn setup_devices() {
	let mut device_lock = DEVICES.lock();
	for d in device_lock.deref_mut() {
		d.setup();
	}
}
