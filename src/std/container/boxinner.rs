use core::{
	marker::PhantomData,
	marker::Unsize,

	ops::CoerceUnsized,

	ptr::Unique,
	ptr::NonNull,
	ptr
};

use core::ops::{
	Deref,
	DerefMut,
	DerefPure
};

pub struct BoxInner<T: ?Sized>(
	Unique<T>,
	usize
);

impl<T: ?Sized> BoxInner<T> {
	pub fn new(addr: u64, size: usize) -> BoxInner<T> {
		BoxInner(
			unsafe {
				Unique::new_unchecked(
					core::ptr::from_raw_parts_mut::<T>(
						addr as *mut (),
						ptr::metadata(
							core::mem::MaybeUninit::<*const T>::zeroed().assume_init()
						)
					)
				)
			},
			size
		)
	}

	pub fn len(&self) -> usize {
		self.1
	}

	pub fn u8_ptr(&self) -> *mut u8 {
		self.0.as_ptr() as *mut u8
	}

	pub fn ptr(&self) -> *mut T {
		self.0.as_ptr()
	}
}

impl<T: ?Sized> Deref for BoxInner<T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe {
			self.0.as_ref()
		}
	}
}

impl<T: ?Sized> DerefMut for BoxInner<T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			self.0.as_mut()//&mut *(self.content.expect("Attempt to deref BoxBase::NONE") as *mut T)
		}
	}
}

impl<T: ?Sized> Copy for BoxInner<T> {}

impl<T: ?Sized> Clone for BoxInner<T> {
	fn clone(&self) -> Self {
		todo!("Yup")
	}
}

//impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Option<BoxInner<U>>> for Option<BoxInner<T>> {}
