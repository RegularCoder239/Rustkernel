pub mod gop;
mod config;

use uefi::mem::memory_map::{
	MemoryMapOwned,
	MemoryType
};
use uefi::boot::*;
use uefi::*;
use crate::errors::BootError;

/*
 * Contains framebuffer information.
 * Warning: The buffer variable contains the
 * physical address to the framebuffer.
 */
#[derive(Copy, Clone)]
pub struct FrameBuffer {
	pub buffer: *mut u32,
	pub resolution: (usize, usize),
	pub stride: usize,
	pub size: usize
}

/*
 * The information, that was gathered by the UEFI boot services.
 * They are valid after exiting boot services.
 */
pub struct UEFIResult {
	pub frame_buffer: Option<FrameBuffer>,
	pub config: config::UEFIConfig
}

/*
 * Uses boot services to gather a valid UEFIResult struct.
 */
fn setup_services() -> UEFIResult {
	UEFIResult {
		frame_buffer: if let Some(mut unwrapped_gop) = gop::GOP::new() {
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
	}
}

/*
 * Gathers memory map and a valid UEFIResult struct with the
 * UEFI boot services.
 * This method shouldn´t be called twice.
 */
pub unsafe fn boot_sequence() -> (UEFIResult, MemoryMapOwned) {
	(
		setup_services(),
		unsafe {
			exit_boot_services(MemoryType::LOADER_DATA)
		}
	)
}
