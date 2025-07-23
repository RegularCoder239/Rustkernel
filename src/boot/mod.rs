pub mod gop;
mod config;

use uefi::mem::memory_map::{
	MemoryMapOwned,
	MemoryType
};
use uefi::boot::*;
use uefi::*;
use crate::errors::BootError;

#[derive(Copy, Clone)]
pub struct FrameBuffer {
	pub buffer: *mut u32,
	pub resolution: (usize, usize),
	pub stride: usize,
	pub size: usize
}

pub struct UEFIResult {
	pub frame_buffer: Option<FrameBuffer>,
	pub config: config::UEFIConfig
}

fn setup_services() -> Result<UEFIResult, BootError> {
	let gop = gop::GOP::new();
	Ok(UEFIResult {
		frame_buffer: if let Some(mut unwrapped_gop) = gop {
			Some(FrameBuffer {
				buffer: unwrapped_gop.frame_buffer(),
				resolution: unwrapped_gop.resolution(),
				stride: unwrapped_gop.stride(),
				size: unwrapped_gop.size()
			})
		} else {
			crate::std::log::warn!("No display found.");
			None
		},
		config: config::UEFIConfig::generate()
	})
}

pub fn boot_sequence() -> Result<(UEFIResult, MemoryMapOwned), BootError> {
	Ok(
		(
			setup_services()?,
			unsafe {
				exit_boot_services(MemoryType::LOADER_DATA)
			}
		)
	)
}
