pub mod r#box;
pub mod unsaferef;
pub mod string;
//pub mod mutableref;
pub mod mutablecell;

use core::mem::{
	self,
	ManuallyDrop
};
use crate::std::{
	Allocator,
	RAMAllocator
};
use core::ops::{
	Deref,
	DerefMut
};
use r#box::Box;

#[derive(Copy, Clone)]
pub struct LazyBox<T> {
	method: fn() -> T,
	content: Option<T>
}

pub struct SharedRef<T, A: Allocator = RAMAllocator> {
	content: Option<ManuallyDrop<Box<T, A>>>,
	ref_counter: usize
}

impl<T> LazyBox<T> {
	pub const fn new(meth: fn() -> T) -> LazyBox<T> {
		LazyBox {
			method: meth,
			content: None
		}
	}
	pub fn get(&mut self) -> &T {
		if self.content.is_none() {
			self.content = Some((self.meth())());
		}
		self.content.as_ref().unwrap()
	}

	pub fn get_mut(&mut self) -> &mut T {
		if self.content.is_none() {
			self.content = Some((self.meth())());
		}
		self.content.as_mut().unwrap()
	}

	pub fn set(&mut self, content: T) {
		self.content = Some(content);
	}

	pub fn meth(&self) -> fn() -> T {
		unsafe {
			core::mem::transmute::<u64, fn() -> T>(self.method as u64 + crate::mm::kernel_offset())
		}
	}

	pub fn is_initalized(&self) -> bool {
		!self.content.is_none()
	}
}

impl<T, A: Allocator + Default> SharedRef<T, A> {
	pub fn new(content: T) -> SharedRef<T, A> {
		let mut r#ref = SharedRef::<T, A> {
			content: Some(
				ManuallyDrop::new(
					Box::<T, A>::new_sized(mem::size_of::<T>())
				)
			),
			ref_counter: 1
		};
		*r#ref = content;
		r#ref
	}
	pub fn split(&mut self) -> Self {
		self.ref_counter += 1;

		SharedRef {
			content: self.split_content(),
			ref_counter: self.ref_counter
		}
	}
	fn split_content(&self) -> Option<ManuallyDrop<Box<T, A>>> {
		let addr = self.content.as_ref()?.physical_address();
		Some(
			ManuallyDrop::new(
				Box::<T, A>::from_raw_address(addr)
			)
		)
	}
}

impl<T, A: Allocator> SharedRef<T, A> {
	pub const EMPTY: SharedRef<T, A> = SharedRef::<T, A> {
		content: Option::None,
		ref_counter: 0
	};

	pub fn is_none(&self) -> bool {
		self.content.is_none()
	}

	pub fn unwrap(&self) -> Option<&T> {
		Some(
			unsafe {
				&*(self.content.as_ref()?.as_ptr())
			}
		)
	}
	pub fn unwrap_mut(&mut self) -> Option<&mut T> {
		Some(
			unsafe {
				&mut *(self.content.as_mut()?.as_ptr())
			}
		)
	}
}

impl<T, A: Allocator> Deref for SharedRef<T, A> {
	type Target = T;

	fn deref(&self) -> &T {
		self.unwrap().expect("Attempt to deref an nonreferenced SharedRef.")
	}
}

impl<T, A: Allocator> DerefMut for SharedRef<T, A> {
	fn deref_mut(&mut self) -> &mut T {
		self.unwrap_mut().expect("Attempt to deref an nonreferenced SharedRef.")
	}
}

impl<T, A: Allocator> Drop for SharedRef<T, A> {
	fn drop(&mut self) {

	}
}
