use crate::{
	uefi_result,

	std::LazyMutex,
	std::Box
};

static FRAMEBUFFER_POINTER: LazyMutex<Option<Box<[u32]>>> = LazyMutex::new(
	|| {
		let frame_buffer_lock = uefi_result!().frame_buffer;
		let resolution = frame_buffer_lock?.resolution;
		Some(
			Box::from_raw_address_sized(
				frame_buffer_lock?.buffer as u64,
				resolution.0 * resolution.1 * 4
			)
		)
	}
);

pub fn display_framebuffer() -> Option<&'static mut [u32]> {
	let pointer = unsafe {
		FRAMEBUFFER_POINTER.get_static()
	};
	if pointer.is_none() {
		log::info!("No display found");
		None
	} else {
		Some(
			pointer.as_mut().unwrap().as_slice_mut()
		)
	}
}
