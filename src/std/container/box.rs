use core::{
	marker::PhantomData,
	marker::Unsize,

	ops::Deref,
	ops::DerefMut,
	ops::CoerceUnsized,
	ops::Index,

	ptr::NonNull,
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
	NonNull<T>,
	PhantomData<A>,
	usize
);

impl<T, A: Allocator> Box<T, A> {
	#[inline(always)]
	#[must_use]
	pub fn new(content: T) -> Self {

		let mut r#box = Self::new_sized(mem::size_of::<T>());
		log::info!("{}", 12);
		unsafe {
			*r#box = content;
		}
		log::info!("{}", 12);
		r#box
	}
	pub fn new_uninit() -> Self {
		Self::new_sized(mem::size_of::<T>())
	}
	pub fn from_raw_address(addr: u64) -> Box<T, A> {
		Self::from_raw_address_sized(addr, mem::size_of::<T>())
	}
}

impl<T: ?Sized, A: Allocator> Box<T, A> {
	pub fn new_sized(size: usize) -> Box<T, A> {
		Box(
			NonNull::new(
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
			NonNull::new(
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
	pub fn physical_address(&self) -> u64 {
		(self.0.as_ptr().addr() as u64).physical_address()
	}
	pub fn as_ptr<T2>(&self) -> *mut T2 {
		self.0.as_ptr() as *mut T2
	}
}

impl<T: Copy, A: Allocator> Box<[T], A> {
	pub fn new_slice(data: &[T]) -> Box<[T], A> {
		let mut r#box = Self::new_sized(data.len() * mem::size_of::<T>());
		let unwrapped_content = r#box.as_slice_mut();
		for idx in 0..data.len() {
			unwrapped_content[idx] = data[idx];
		}
		r#box
	}
}

impl<T, A: Allocator> Box<[T], A> {
	pub fn as_slice(&self) -> &[T] {
		unsafe {
			core::slice::from_raw_parts(
				self.0.as_ptr() as *const T,
				self.alloc_len() / mem::size_of::<T>()
			)
		}
	}
	pub fn as_slice_mut(&self) -> &mut [T] {
		unsafe {
			core::slice::from_raw_parts_mut(
				self.0.as_ptr() as *mut T,
				self.alloc_len() / mem::size_of::<T>()
			)
		}
	}
}

impl<T: Copy, A: Allocator> Index<usize> for Box<[T], A> {
	type Output = T;
	fn index(&self, idx: usize) -> &T {
		unsafe {
			&*(self.0.as_ptr().byte_add(idx * mem::size_of::<T>()) as *const T)
		}
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

impl<T: ?Sized, A: Allocator> Drop for Box<T, A> {
	fn drop(&mut self) {
		unsafe {
			let ptr = self.0.as_ptr() as *const u8;
			if ptr as u64 == 0 {
				return;
			}
			A::free(ptr, self.alloc_len())
		}
	}
}

impl<T: Default, A: Allocator> Default for Box<T, A> {
	fn default() -> Self {
		Box::new(T::default())
	}
}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: Allocator> CoerceUnsized<Box<U, A>> for Box<T, A> {}
