use crate::{
	uefi_result,
};
use crate::std::{
	LazyMutex,
	Box
};

/*
 * Contains memory mapped framebuffer.
 */
struct Display(Box<[u32]>);

static FRAMEBUFFER_POINTER: LazyMutex<Option<Display>> = LazyMutex::new(
	|| Display::get()
);

impl Display {
	/*
	 * This method shall be called as few times
	 * as possible, because it maps the framebuffer
	 * on every method call. This method return None,
	 * when no display is avaiable.
	 */
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
	/*
	 * Returns a tuple with a mutable slice pointing to the
	 * framebuffer content and the framebuffer stride.
	 * This method return None, when no display is avaiable.
	 */
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

/*
 * Returns a tuple with a mutable slice pointing to the
 * framebuffer content and the framebuffer stride.
 * This method return None, when no display is avaiable.
 */
pub fn framebuffer() -> Option<(&'static mut [u32], usize)> {
	Some(FRAMEBUFFER_POINTER.lock().as_mut()?.frame_buffer())
}

/*
 * Returns a tuple with a mutable slice pointing to the
 * framebuffer content and the framebuffer stride
 * This method return None, when no display is avaiable.
 */
pub fn resolution() -> Option<(usize, usize)> {
	Some(
		uefi_result!()?.frame_buffer?.resolution
	)
}
