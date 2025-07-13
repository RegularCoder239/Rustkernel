use crate::{
	uefi_result,
};
use crate::std::{
	LazyMutex,
	Box
};

static FRAMEBUFFER_POINTER: LazyMutex<Option<Box<[u32]>>> = LazyMutex::new(
	|| {
		let frame_buffer_lock = uefi_result!().unwrap().frame_buffer;

		Some(
			Box::from_raw_address_sized(
				frame_buffer_lock?.buffer as u64,
				frame_buffer_lock?.size
			)
		)
	}
);

pub fn framebuffer() -> Option<(&'static mut [u32], usize)> {
	Some(
		(
			FRAMEBUFFER_POINTER.get().as_mut()?.as_slice_mut(),
			uefi_result!()?.frame_buffer?.stride
		)
	)
}

pub fn resolution() -> Option<(usize, usize)> {
	Some(
		uefi_result!()?.frame_buffer?.resolution
	)
}
