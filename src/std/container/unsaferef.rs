pub struct UnsafeRef<T> {
	ptr: *mut T
}

impl<T> UnsafeRef<T> {
	pub fn from_ref(content: &T) -> UnsafeRef<T> {
		UnsafeRef {
			ptr: content as *const T as *mut T
		}
	}
	pub fn from_ptr(content: *const T) -> UnsafeRef<T> {
		UnsafeRef {
			ptr: content as *mut T
		}
	}

	pub fn get(&self) -> &'static mut T {
		unsafe {
			&mut *self.ptr
		}
	}
}

impl<T> Copy for UnsafeRef<T> {}

impl<T> Clone for UnsafeRef<T> {
	fn clone(&self) -> UnsafeRef<T> {
		UnsafeRef {
			ptr: self.ptr
		}
	}
}
