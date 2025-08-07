mod bridge;
mod device;
mod drive;
mod header;
mod ethernet;

pub use header::{
	Header,
	HeaderType0
};
pub use device::{
	DeviceTrait,
	UnspecifiedDevice
};
pub use drive::{
	UnspecifiedDrive
};
pub use bridge::{
	Bridge
};
pub use ethernet::{
	NetworkController
};

use crate::{
	hw::acpi::acpi_singleton,
	std,
	std::log
};

static SCAN_LOCK: std::Lock = std::Lock::new_locked();

pub fn scan() -> ! {
	if let Some(entries) = (*acpi_singleton()).pci_mcfg_entries() {
		log::info!("Scanning for PCI devices.");
		for entry in &entries {
			log::info!("PCI Root Window: {:x} {:?}", entry.base_address, entry.bus_numbers);
			Bridge::from_raw_address(entry.base_address).scan();
		}
	}

	SCAN_LOCK.unlock();
	std::exit();
}

pub fn wait_for_scan() {
	SCAN_LOCK.lock();
	SCAN_LOCK.unlock();
}

pub fn setup() -> ! {
	wait_for_scan();
	log::info!("Setting up PCI devices.");

	ethernet::setup_devices();

	std::exit();

}
