use crate::{
	uefi_result,
};
use crate::std::{
	LazyMutex,
	Box
};

static FRAMEBUFFER_RESOLUTION: LazyMutex<Option<(usize, usize)>> = LazyMutex::new(
	|| Some(
		uefi_result!().unwrap().frame_buffer?.resolution
	)
);
static FRAMEBUFFER_POINTER: LazyMutex<Option<Box<[u32]>>> = LazyMutex::new(
	|| {
		let frame_buffer_lock = uefi_result!().unwrap().frame_buffer;
		let resolution = frame_buffer_lock?.resolution;

		Some(
			Box::from_raw_address_sized(
				frame_buffer_lock?.buffer as u64,
				resolution.0 * resolution.1 * 4
			)
		)
	}
);

pub fn framebuffer() -> Option<&'static mut [u32]> {
	Some(
		FRAMEBUFFER_POINTER.get().as_mut()?.as_slice_mut()
	)
}

pub fn resolution() -> Option<(usize, usize)> {
	FRAMEBUFFER_RESOLUTION.lock().clone()
}
