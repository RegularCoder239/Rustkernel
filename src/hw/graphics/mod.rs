mod display;

pub use display::{
	framebuffer,
	resolution
};

use crate::uefi_result;

/*
 * Returns true when a framebuffer is available
 */
pub fn available() -> bool {
	if let Some(result) = uefi_result!() {
		result.frame_buffer.is_some()
	} else {
		false
	}
}
