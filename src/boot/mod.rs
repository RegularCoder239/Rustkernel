pub mod gop;

use self::gop::GOPError;
use uefi::mem::memory_map::{
	MemoryMapOwned,
	MemoryType,
	MemoryMap
};
use uefi::boot::*;
use uefi::*;

#[derive(Debug)]
pub enum BootError {
	GOPError(GOPError),
	UEFIError(uefi::Error)
}

impl From<GOPError> for BootError {
	fn from(err: GOPError) -> Self {
		Self::GOPError(err)
	}
}

impl From<uefi::Error> for BootError {
	fn from(err: uefi::Error) -> Self {
		Self::UEFIError(err)
	}
}

pub fn boot_sequence() -> Result<MemoryMapOwned, BootError> {
//	let gop = gop::GOP::new()?;
	let memory_map = memory_map(MemoryType::LOADER_DATA).unwrap();

	unsafe {
		exit_boot_services(MemoryType::LOADER_DATA);
		Ok(memory_map)
	}
}
