use core::{
	marker::PhantomData,

	ops::Deref,
	ops::DerefMut,

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

pub struct BoxBase<T: ?Sized, A: Allocator> {
	pub content: Option<*mut [u8]>,
	phantom: PhantomData<A>,
	phantom2: PhantomData<T>
}

pub type Box<T> = BoxBase<T, RAMAllocator>;

impl<T, A: Allocator> BoxBase<T, A> {
	pub fn new(content: T) -> BoxBase<T, A> {
		let mut r#box = Self::new_uninit();
		r#box.set(content);
		r#box
	}
	pub fn new_uninit() -> BoxBase<T, A> {
		Self::new_sized(core::mem::size_of::<T>())
	}
	pub fn from_raw_address(addr: u64) -> BoxBase<T, A> {
		Self::from_raw_virt_address(
			unsafe {
				A::VirtualMapper::map::<u8>(stack_vec!{ addr }, mem::size_of::<T>()).unwrap() as u64
			}
		)
	}
	pub fn from_raw_virt_address(addr: u64) -> BoxBase<T, A> {
		BoxBase {
			content:
				Some(core::ptr::from_raw_parts::<[u8]>(
					addr as *const (),
					mem::size_of::<T>()
				) as *mut [u8]),

			..Self::NONE
		}
	}
	pub fn set(&mut self, content: T) {
		if self.content.is_none() {
			self.content = Some(
				core::ptr::from_raw_parts::<[u8]>(
					allocate!(ptr_with_alloc, A, u8, mem::size_of::<T>()).expect("Failed to allocate box while setting."),
					mem::size_of::<T>()
				) as *mut [u8]
			);
		}
		unsafe {
			*(self.content.unwrap() as *mut T) = content;
		}
	}

	pub unsafe fn deref_static(&mut self) -> &'static mut T {
		unsafe {
			&mut *(self.content.expect("Attempt to deref BoxBase::NONE") as *mut T)
		}
	}
}

impl<T: ?Sized, A: Allocator> BoxBase<T, A> {
	pub const NONE: BoxBase<T, A> = BoxBase {
		content: None,
		phantom: PhantomData,
		phantom2: PhantomData
	};

	pub fn new_from_slice<T2>(data: &[T2]) -> BoxBase<T, A> {
		let r#box = Self::new_sized(data.len() * mem::size_of::<T2>());
		let u8data = unsafe {
			&*(data as *const [T2] as *const [u8])
		};
		let unwrapped_content = unsafe {
			&mut *r#box.content.unwrap()
		};
		for idx in 0..data.len() * mem::size_of::<T2>() {
			unwrapped_content[idx] = u8data[idx];
		}
		r#box
	}
	pub fn new_sized(size: usize) -> BoxBase<T, A> {
		BoxBase {
			content: Some(core::ptr::from_raw_parts::<[u8]>(
					allocate!(ptr_with_alloc, A, u8, size).unwrap() as *const (),
					size
				) as *mut [u8]),

			..Self::NONE
		}
	}
	pub fn size(&self) -> usize {
		self.content.unwrap().len()
	}
	pub unsafe fn new_converted<T2>(&self) -> BoxBase<T2, A> {
		BoxBase::<T2, A>::from_raw_address(self.physical_address())
	}
	pub unsafe fn new_converted_nonboxed<T2>(content: T2) -> BoxBase<T2, A> {
		BoxBase::<T2, A>::new_from_slice::<T2>(&[content])
	}


	pub fn physical_address(&self) -> u64 {
		(self.content.expect("Attempt to gather physical address of Box::NONE") as *mut u8 as u64).physical_address()
	}

	pub fn as_ptr<T2>(&self) -> *mut T2 {
		self.content.unwrap() as *mut u8 as *mut T2
	}

	pub fn is_none(&self) -> bool {
		self.content.is_none()
	}
}

impl<T, A: Allocator> Deref for BoxBase<T, A> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe {
			&*(self.content.expect("Attempt to deref BoxBase::NONE") as *const T)
		}
	}
}

impl<T, A: Allocator> DerefMut for BoxBase<T, A> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *(self.content.expect("Attempt to deref BoxBase::NONE") as *mut T)
		}
	}
}

