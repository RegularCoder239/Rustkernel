use core::ops::{
	Deref,
	DerefMut
};

#[derive(Copy, Clone)]
pub struct MutableCell<T>(T);

impl<T> MutableCell<T> {
	pub const fn new(content: T) -> MutableCell<T> {
		MutableCell(content)
	}

	#[allow(invalid_reference_casting)]
	pub fn deref_mut(&self) -> &mut T {
		unsafe {
			&mut *(&self.0 as *const T as *mut T)
		}
	}
}

impl<T> Deref for MutableCell<T> {
	type Target = T;
	fn deref(&self) -> &T {
		&self.0
	}
}

unsafe impl<T> Sync for MutableCell<T> {}
