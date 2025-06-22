use crate::mm::{
	buddy,

	Mapped,
	Address
};
use core::ops::{
	Deref,
	DerefMut
};
use core::marker::PhantomData;
use super::StackVec;

pub trait PhysicalAllocator {
	const DEFAULT: Self;

	fn allocate_phys(amount: usize) -> Option<StackVec<u64, 0x200>> where Self: Sized;
	unsafe fn free_phys(ptr: u64, amount: usize) where Self: Sized;
}

pub trait VirtualMapper {
	const DEFAULT: Self;

	fn map<T: ?Sized>(addr: StackVec<u64, 0x200>, amount: usize) -> Option<*mut T>;
	unsafe fn unmap(addr: u64, amount: usize);
}

pub trait Allocator {
	type VirtualMapper: VirtualMapper;
	type PhysicalAllocator: PhysicalAllocator;

	fn allocate<T: ?Sized>(amount: usize) -> Option<*mut T> where Self: Sized;
	unsafe fn free(ptr: *const u8, amount: usize) where Self: Sized;
}


pub struct RAMAlignedAllocator;
pub struct PhysicalRAMAllocator;
pub struct KernelGlobalMapper;
pub struct BasicAllocator<V: VirtualMapper, P: PhysicalAllocator> {
	phantom: PhantomData<V>,
	phantom2: PhantomData<P>
}

pub type RAMAllocator = BasicAllocator<KernelGlobalMapper, PhysicalRAMAllocator>;

pub struct Allocation<T, A: Allocator = RAMAllocator> {
	content: *mut T,
	size: usize,
	phantom: PhantomData<A>
}

impl PhysicalAllocator for PhysicalRAMAllocator {
	const DEFAULT: Self = Self {};
	fn allocate_phys(amount: usize) -> Option<StackVec<u64, 0x200>> {
		buddy::allocate(amount)
	}
	unsafe fn free_phys(_: u64, _: usize) {
		log::warn!("Free implementation missing.");
	}
}

impl VirtualMapper for KernelGlobalMapper {
	const DEFAULT: Self = Self {};
	fn map<T: ?Sized>(addr: StackVec<u64, 0x200>, amount: usize) -> Option<*mut T> {
		addr.mapped_global::<T>(amount)
	}
	unsafe fn unmap(addr: u64, amount: usize) {
		log::info!("Unmapped");
	//	addr.unmap(amount)
	}
}

impl Allocator for RAMAllocator {
	type VirtualMapper = KernelGlobalMapper;
	type PhysicalAllocator = PhysicalRAMAllocator;

	fn allocate<T: ?Sized>(amount: usize) -> Option<*mut T> where Self: Sized {
		unsafe {
			KernelGlobalMapper::map(
				PhysicalRAMAllocator::allocate_phys(amount)?,
				amount
			)
		}
	}

	unsafe fn free(ptr: *const u8, amount: usize) where Self: Sized {
		let ptr_u64 = ptr as u64;
		unsafe {
			// TODO: Broken free implementation fix
			/*hysicalRAMAllocator::free_phys(ptr_u64.physical_address(), amount);
			KernelGlobalMapper::unmap(ptr_u64, amount);
		*/}
	}
}

impl<T, A: Allocator> Allocation<T, A> {
	pub fn new(amount: usize) -> Option<Allocation<T, A>> {
		Some(
			Allocation {
				content: unsafe {
					A::allocate::<T>(amount * core::mem::size_of::<T>())?
				},
				size: amount,
				phantom: PhantomData
			}
		)
	}

	pub fn as_ptr(&self) -> *mut T {
		self.content
	}
	pub fn as_mut(&self) -> &mut T {
		unsafe {
			self.content.as_mut().unwrap()
		}
	}
	pub fn as_ref(&self) -> &T {
		unsafe {
			&*self.content
		}
	}
}

impl<T, A: Allocator> Deref for Allocation<T, A> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe {
			&*self.as_ptr()
		}
	}
}

impl<T, A: Allocator> DerefMut for Allocation<T, A> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *self.as_ptr()
		}
	}
}

impl<T, A: Allocator> Clone for Allocation<T, A> {
	fn clone(&self) -> Self {
		Allocation {
			content: self.content,
			size: self.size,
			phantom: PhantomData
		}
	}
}

impl<T, A: Allocator> Drop for Allocation<T, A> {
	fn drop(&mut self) {
		unsafe {
			A::free(self.content as *const T as *const u8, self.size)
		}
	}
}

fn is_page_aligned(mut size: usize) -> bool {
	if size % 0x1000 != 0 {
		return false;
	}

	while size > 0x1 {
		if size % 0x200 != 0 {
			return false;
		}
		size /= 0x200;
	}
	return true;
}

#[macro_export]
macro_rules! allocate {
	(ptr_with_alloc, $allocator: ty, $r#type: ty, $size: expr) => {
		<$allocator as crate::std::Allocator>::allocate::<$r#type>($size)
	};
	(ptr_with_alloc, $allocator: ty, $r#type: ty) => {
		allocate!(ptr_with_alloc, $allocator, $r#type, core::mem::size_of::<$r#type>())
	};
	(ptr, $r#type: ty, $size: expr) => {
		allocate!(ptr_with_alloc, crate::std::RAMAllocator, $r#type, $size)
	};
	(ptr, $r#type: ty) => {
		allocate!(ptr, $r#type, core::mem::size_of::<$r#type>())
	};
}
