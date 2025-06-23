pub mod r#box;
pub mod unsaferef;
pub mod string;

use core::mem;
use crate::std::{
	Allocator,
	With,
	RAMAllocator,
	Allocation
};
use core::ops::{
	Deref,
	DerefMut
};

#[derive(Copy, Clone)]
pub struct LazyBox<T> {
	method: fn() -> T,
	content: Option<T>
}

pub struct SharedRefBase<T, A: Allocator> {
	content: Option<Allocation<T, A>>,
	ref_counter: usize
}

pub type SharedRef<T> = SharedRefBase<T, RAMAllocator>;

impl<T> LazyBox<T> {
	pub const fn new(meth: fn() -> T) -> LazyBox<T> {
		LazyBox {
			method: meth,
			content: None
		}
	}
	pub fn get(&mut self) -> &T {
		if self.content.is_none() {
			self.content = Some(
				(self.method)()
			);
		}
		self.content.as_ref().unwrap()
	}

	pub fn get_mut(&mut self) -> &mut T {
		if self.content.is_none() {
			self.content = Some((self.method)());
		}
		self.content.as_mut().unwrap()
	}

	pub fn set(&mut self, content: T) {
		self.content = Some(content);
	}

	pub fn is_initalized(&self) -> bool {
		!self.content.is_none()
	}
}

impl<T, A: Allocator> SharedRefBase<T, A> {
	pub const EMPTY: SharedRefBase<T, A> = SharedRefBase::<T, A> {
		content: Option::None,
		ref_counter: 0
	};

	pub fn new(content: T) -> SharedRefBase<T, A> {
		SharedRefBase::<T, A> {
			content: Some(
				Allocation::<T, A>::new(mem::size_of::<T>())
					.expect("Failed to allocate SharedRef")
					.with(content)
			),
			ref_counter: 0
		}
	}
	pub fn as_ref(&self) -> &T {
		self.content.as_ref().unwrap().as_ref()
	}
	pub fn as_mut(&mut self) -> &mut T {
		self.content.as_mut().unwrap().as_mut()
	}
	pub fn is_none(&self) -> bool {
		self.content.is_none()
	}
	pub fn split(&mut self) -> Self {
		self.ref_counter += 1;

		SharedRefBase {
			content: self.content.clone(),
			ref_counter: self.ref_counter
		}
	}
	pub fn unwrap(&self) -> Option<&T> {
		Some(self.content.as_ref()?.as_ref())
	}
	pub fn unwrap_mut(&mut self) -> Option<&mut T> {
		Some(self.content.as_mut()?.as_mut())
	}
}

impl<T, A: Allocator> Deref for SharedRefBase<T, A> {
	type Target = T;

	fn deref(&self) -> &T {
		self.as_ref()
	}
}

impl<T, A: Allocator> DerefMut for SharedRefBase<T, A> {
	fn deref_mut(&mut self) -> &mut T {
		self.as_mut()
	}
}
