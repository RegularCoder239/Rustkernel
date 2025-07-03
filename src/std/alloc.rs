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

impl PhysicalAllocator for PhysicalRAMAllocator {
	const DEFAULT: Self = Self {};
	fn allocate_phys(amount: usize) -> Option<StackVec<u64, 0x200>> {
		buddy::allocate(amount)
	}
	unsafe fn free_phys(addr: u64, amount: usize) {
		buddy::free(addr, amount);
	}
}

impl VirtualMapper for KernelGlobalMapper {
	const DEFAULT: Self = Self {};
	fn map<T: ?Sized>(addr: StackVec<u64, 0x200>, amount: usize) -> Option<*mut T> {
		addr.mapped_global::<T>(
			if amount < 0x200000 && amount > 0x10000 {
				0x200000
			} else {
				amount + 0x1000 - (amount % 0x1000)
			}
		)
	}
	unsafe fn unmap(addr: u64, amount: usize) {
		addr.unmap(amount);
	}
}

impl Allocator for RAMAllocator {
	type VirtualMapper = KernelGlobalMapper;
	type PhysicalAllocator = PhysicalRAMAllocator;

	fn allocate<T: ?Sized>(amount: usize) -> Option<*mut T> where Self: Sized {
		KernelGlobalMapper::map(
			PhysicalRAMAllocator::allocate_phys(amount)?,
			amount
		)
	}

	unsafe fn free(ptr: *const u8, size: usize) where Self: Sized {
		PhysicalRAMAllocator::free_phys(ptr.physical_address(), size);
		KernelGlobalMapper::unmap(ptr.addr() as u64, size);
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
