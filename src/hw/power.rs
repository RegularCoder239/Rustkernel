use uefi::runtime::{
	self,
	ResetType
};
use uefi::{
	Status
};
use crate::std::log;

pub fn shutdown() -> ! {
	runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None)
}

pub fn reboot() -> ! {
	log::info!("Rebooting ...");
	uefi_reboot();
}

fn uefi_reboot() -> ! {
	runtime::reset(ResetType::WARM, Status::SUCCESS, None)
}
