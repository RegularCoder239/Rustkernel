/*
 * Driver for a PCI bridge. It works by iterating
 * every device and scanning them.
 */
use super::{
	DeviceTrait,
	UnspecifiedDevice
};
use crate::std::Box;

pub struct BridgeStruct;

pub type Bridge = Box<BridgeStruct>;

impl DeviceTrait for Bridge {
	fn specific_scan(&self) {
		for idx in 1..32 {
			UnspecifiedDevice::from_raw_address(
				self.physical_address() + (idx << 15)
			).scan()
		}
	}
}
