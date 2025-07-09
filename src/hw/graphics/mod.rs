mod display;

pub use display::{
	framebuffer,
	resolution
};

use crate::uefi_result;

pub fn available() -> bool {
	if let Some(result) = uefi_result!() {
		result.frame_buffer.is_some()
	} else {
		false
	}
}
