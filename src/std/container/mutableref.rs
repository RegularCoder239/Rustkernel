use core::ops::{
	Deref
};
use crate::std::{
	Box,
	MutableCell
};

#[derive(Copy, Clone)]
pub struct MutableRef<T>(*const T);

impl<T> MutableRef<T> {
	pub const fn from_ref(content: &T) -> MutableRef<T> {
		MutableRef(content as *const T)
	}
	pub const fn from_ptr(content: *const T) -> MutableRef<T> {
		MutableRef(content)
	}

	#[allow(invalid_reference_casting)]
	pub fn deref_mut<'life>(&'life self) -> &'life mut T {
		unsafe {
			&mut *(self.0 as *mut T)
		}
	}
}

impl<T> Deref for MutableRef<T> {
	type Target = T;
	fn deref(&self) -> &T {
		self.deref_mut()
	}
}

impl<T> From<&MutableCell<Box<T>>> for MutableRef<T> {
	fn from(cell: &MutableCell<Box<T>>) -> Self {
		MutableRef::from_ptr(cell.as_ptr())
	}
}
