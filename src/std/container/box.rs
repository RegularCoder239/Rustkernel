use core::{
	marker::PhantomData,
	marker::Unsize,

	ops::Deref,
	ops::DerefMut,
	ops::CoerceUnsized,

	ptr::Unique,
	ptr,

	mem
};
use crate::{
	allocate,
	stack_vec,

	mm::Address
};
use crate::std::{
	Allocator,
	RAMAllocator,
	alloc::VirtualMapper
};

pub struct Box<T: ?Sized, A: Allocator = RAMAllocator>(
	Unique<T>,
	PhantomData<A>,
	usize
);

impl<T, A: Allocator> Box<T, A> {
	#[inline(always)]
	#[must_use]
	#[cfg_attr(miri, track_caller)]
	pub fn new(content: T) -> Self {
		let mut r#box = Self::new_sized(mem::size_of::<T>());
		r#box.set(content);
		r#box
	}
	pub fn new_uninit() -> Self {
		Self::new_sized(mem::size_of::<T>())
	}
	pub fn from_raw_address(addr: u64) -> Box<T, A> {
		Self::from_raw_address_sized(addr, mem::size_of::<T>())
	}
	pub fn set(&mut self, content: T) {
		unsafe {
			*(self.0.as_mut()) = content;
		}
	}
}

impl<T: ?Sized, A: Allocator> Box<T, A> {
	pub fn new_from_slice<T2>(data: &[T2]) -> Box<T, A> {
		let r#box = Self::new_sized(data.len() * mem::size_of::<T2>());
		let u8data = unsafe {
			&*(data as *const [T2] as *const [u8])
		};
		let unwrapped_content = unsafe {
			&mut *core::ptr::slice_from_raw_parts_mut(
				r#box.as_ptr::<u8>(),
				r#box.alloc_len()
			 )
		};
		for idx in 0..data.len() * mem::size_of::<T2>() {
			unwrapped_content[idx] = u8data[idx];
		}
		r#box
	}
	pub fn new_sized(size: usize) -> Box<T, A> {
		Box(
			Unique::new(
				allocate!(ptr_with_alloc, A, T, size).unwrap()
			).unwrap(),
			PhantomData,
			size
		)
	}
	pub fn from_raw_address_sized(addr: u64, size: usize) -> Box<T, A> {
		Self::from_raw_virt_address_sized(
			A::VirtualMapper::map::<u8>(stack_vec!{ addr }, size).unwrap() as u64,
			size
		)
	}
	pub fn from_raw_virt_address_sized(addr: u64, size: usize) -> Box<T, A> {
		Box(
			Unique::new(
				ptr::from_raw_parts_mut::<T>(
					addr as *mut (),
					ptr::metadata(
						unsafe {
							core::mem::MaybeUninit::<*const T>::zeroed().assume_init()
						}
					)
				)
			).unwrap(),
			PhantomData,
			size
		)
	}
	pub fn alloc_len(&self) -> usize {
		self.2
	}
	pub unsafe fn new_converted<T2>(&self) -> Box<T2, A> {
		Box::<T2, A>::from_raw_address(self.physical_address())
	}
	pub fn physical_address(&self) -> u64 {
		(self.0.as_ptr().addr() as u64).physical_address()
	}

	pub fn as_ptr<T2>(&self) -> *mut T2 {
		self.0.as_ptr() as *mut T2
	}
}

impl<T: ?Sized, A: Allocator> Deref for Box<T, A> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe {
			self.0.as_ref()
		}
	}
}

impl<T: ?Sized, A: Allocator> DerefMut for Box<T, A> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			self.0.as_mut()
		}
	}
}

impl<T: ?Sized, A: Allocator> Unpin for Box<T, A> {}

impl<T: Default, A: Allocator> Default for Box<T, A> {
	fn default() -> Self {
		Box::new(T::default())
	}
}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: Allocator> CoerceUnsized<Box<U, A>> for Box<T, A> {}
