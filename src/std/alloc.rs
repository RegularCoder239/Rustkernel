use crate::mm::{
	buddy,
	Mapped,
	Address,
	MappingInfo,
	MappingFlags
};
use crate::current_page_table;
use core::marker::PhantomData;
use super::{
	StackVec,
	VecBase,
	MutableRef
};

pub trait PhysicalAllocator {
	const DEFAULT: Self;

	fn allocate_phys(amount: usize) -> Option<StackVec<u64, 0x200>> where Self: Sized;
	fn free_phys(ptr: u64, amount: usize) where Self: Sized;
}

pub trait VirtualMapper: Default {
	fn map<T: ?Sized>(&self, addr: StackVec<u64, 0x200>, amount: usize) -> Option<*mut T>;
	fn unmap(&self, addr: u64, amount: usize);
}

pub trait Allocator {
	type VirtualMapper: VirtualMapper;
	type PhysicalAllocator: PhysicalAllocator;

	fn new(mapper: Self::VirtualMapper) -> Self;
	fn allocate<T: ?Sized>(&self, amount: usize) -> Option<*mut T> where Self: Sized;
	fn free(&self, ptr: *const u8, amount: usize) where Self: Sized;
}


pub struct RAMAlignedAllocator;
pub struct PhysicalRAMAllocator;
#[derive(Default)]
pub struct KernelGlobalMapper;
pub struct PageTableMapper(pub MappingInfo);
pub struct BasicAllocator<V: VirtualMapper, P: PhysicalAllocator> {
	mapper: V,
	phantom: PhantomData<P>
}

pub type RAMAllocator = BasicAllocator<KernelGlobalMapper, PhysicalRAMAllocator>;

impl PhysicalAllocator for PhysicalRAMAllocator {
	const DEFAULT: Self = Self {};
	fn allocate_phys(amount: usize) -> Option<StackVec<u64, 0x200>> {
		buddy::allocate(amount)
	}
	fn free_phys(addr: u64, amount: usize) {
		buddy::free(addr, amount);
	}
}

impl VirtualMapper for KernelGlobalMapper {
	fn map<T: ?Sized>(&self, addr: StackVec<u64, 0x200>, amount: usize) -> Option<*mut T> {
		addr.mapped_global::<T>(
			if amount % 0x1000 == 0 {
				amount
			} else {
				amount + 0x1000 - (amount % 0x1000)
			}
		)
	}
	fn unmap(&self, addr: u64, amount: usize) {
		addr.unmap(amount);
	}
}

impl Default for PageTableMapper {
	fn default() -> Self {
		PageTableMapper(
			MappingInfo {
				address: 0,
				flags: MappingFlags::None,
				page_table: MutableRef::from_ptr(current_page_table())
			}
		)
	}
}

impl VirtualMapper for PageTableMapper {
	fn map<T: ?Sized>(&self, addr: StackVec<u64, 0x200>, amount: usize) -> Option<*mut T> {
		assert!(addr.len() != 1, "Unsupported addr size.");
		MappingInfo {
			address: addr[0],
			..self.0
		}.mapped_global::<T>(
			if amount % 0x1000 == 0 {
				amount
			} else {
				amount + 0x1000 - (amount % 0x1000)
			}
		)
	}
	fn unmap(&self, addr: u64, amount: usize) {
		addr.unmap(amount);
	}
}

impl<V: VirtualMapper, P: PhysicalAllocator> Allocator for BasicAllocator<V, P> {
	type VirtualMapper = V;
	type PhysicalAllocator = P;

	fn new(mapper: Self::VirtualMapper) -> Self {
		BasicAllocator {
			mapper,
			phantom: PhantomData
		}
	}
	fn allocate<T: ?Sized>(&self, amount: usize) -> Option<*mut T> where Self: Sized {
		self.mapper.map(
			P::allocate_phys(amount)?,
			amount
		)
	}

	fn free(&self, ptr: *const u8, size: usize) where Self: Sized {
		P::free_phys(ptr.physical_address(), size);
		self.mapper.unmap(ptr.addr() as u64, size);
	}
}

impl<V: VirtualMapper + Default, P: PhysicalAllocator> Default for BasicAllocator<V, P> {
	fn default() -> Self {
		Self {
			mapper: V::default(),
			phantom: PhantomData
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
/*
#[macro_export]
macro_rules! allocate {
	(ptr_with_alloc, $allocator: ty, $r#type: ty, $size: expr) => {
		{
			use crate::std::Allocator;
			<$allocator as crate::std::Allocator>::default().allocate::<$r#type>($size)
		}
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
*/
