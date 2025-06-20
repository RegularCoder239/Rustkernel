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
};

static INITALIZED: std::Lock = std::Lock::new_locked();

pub fn scan() -> ! {
	if let Some(entries) = (*acpi_singleton()).pci_mcfg_entries() {
		for entry in &entries {
			log::info!("PCI Root Window: {:x}", entry.base_address);
			Bridge::from_raw_address(entry.base_address).scan();
		}
	}

	log::info!("Mac address: {:x?}", ethernet::DEVICES.lock()[0].mac());

	INITALIZED.unlock();
	std::exit();
}

pub fn setup() -> ! {
	INITALIZED.lock();
	log::info!("Setting up PCI devices.");

	ethernet::setup_devices();

	std::exit();

}
