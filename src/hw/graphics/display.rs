use crate::{
	uefi_result,
};
use crate::std::{
	LazyMutex,
	Box
};

struct Display(Box<[u32]>);

static FRAMEBUFFER_POINTER: LazyMutex<Option<Display>> = LazyMutex::new(
	|| Display::get()
);

impl Display {
	fn get() -> Option<Self> {
		let frame_buffer_lock = uefi_result!().unwrap().frame_buffer;

		Some(
			Display(
				Box::from_raw_address_sized(
					frame_buffer_lock?.buffer as u64,
					frame_buffer_lock?.size
				)
			)
		)
	}
	fn frame_buffer(&mut self) -> (&'static mut [u32], usize) {
		(
			unsafe {
				self.0.as_static_slice_mut()
			},
			uefi_result!().unwrap().frame_buffer.unwrap().stride
		)
	}
}

unsafe impl Sync for Display {}

pub fn framebuffer() -> Option<(&'static mut [u32], usize)> {
	Some(FRAMEBUFFER_POINTER.lock().as_mut()?.frame_buffer())
}

pub fn resolution() -> Option<(usize, usize)> {
	Some(
		uefi_result!()?.frame_buffer?.resolution
	)
}
