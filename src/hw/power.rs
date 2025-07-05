use uefi::runtime::{
	self,
	ResetType
};
use uefi::{
	Status
};
use core::arch::asm;
use crate::std::log;

pub fn shutdown() -> ! {
	runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None)
}

pub fn reboot() -> ! {
	log::info!("Rebooting ...");
	uefi_reboot();
	triple_fault_reboot();
}

fn uefi_reboot() {
	runtime::reset(ResetType::WARM, Status::SUCCESS, None);
	log::warn!("UEFI Reboot failed.");
}

fn triple_fault_reboot() -> ! {
	log::warn!("Rebooting with triple fault ...");
	unsafe {
		asm!("push 0x0",
			 "push 0x0",
			 "lidt [rsp]",
			 "ud2"
		);
	}
	panic!("Triple fault reboot failed.");
}
