use crate::{
	uefi_result,
	std::LazyMutex,
	mm::Mapped
};

static FRAMEBUFFER_POINTER: LazyMutex<Option<*mut u32>> = LazyMutex::new(
	|| {
		let frame_buffer_lock = uefi_result!().frame_buffer;
		let resolution = frame_buffer_lock?.resolution;
		(frame_buffer_lock?.buffer as u64).mapped_global::<u32>(
			resolution.0 * resolution.1 * 4
		)
	}
);

pub fn display_framebuffer() -> Option<&'static mut [u32]> {
	let frame_buffer_resolution = uefi_result!().frame_buffer?.resolution;

	unsafe {
		Some(
			core::slice::from_raw_parts_mut(
				(*FRAMEBUFFER_POINTER.lock())?,
				frame_buffer_resolution.0 * frame_buffer_resolution.1
			)
		)
	}
}
