use core::ops::{
	Deref,
	DerefMut
};

#[derive(Copy, Clone)]
pub struct MutableRef<'life, T>(pub &'life T);

impl<T> MutableRef<'_, T> {
	pub fn from_ref<'life>(content: &'life T) -> MutableRef<'life, T> {
		MutableRef(content)
	}
	pub unsafe fn from_ptr<'life>(content: *const T) -> MutableRef<'life, T> {
		MutableRef(
			unsafe {
				&*content
			}
		)
	}

	#[allow(invalid_reference_casting)]
	pub fn deref_mut<'life>(&'life self) -> &'life mut T {
		unsafe {
			&mut *(self.0 as *const T as *mut T)
		}
	}
}

impl<'life, T> Deref for MutableRef<'life, T> {
	type Target = T;
	fn deref(&self) -> &'life T {
		self.0
	}
}
