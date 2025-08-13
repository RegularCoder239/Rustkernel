use crate::mm::{
	Mapped,
	Address,
	MappingInfo,
	MappingFlags
};
use crate::mm::buddy::{
	self,
	BuddyAllocation
};
use crate::std::Box;
use crate::current_page_table;
use core::marker::PhantomData;
use super::{
	VecBase
};

pub trait PhysicalAllocator {
	const DEFAULT: Self;

	fn allocate_phys(amount: usize) -> Option<BuddyAllocation> where Self: Sized;
	fn free_phys(ptr: u64, amount: usize) where Self: Sized;
}

pub trait VirtualMapper: Default {
	fn map<T: ?Sized>(&self, addr: BuddyAllocation, amount: usize) -> Option<*mut T>;
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
pub struct PageTableMapper<'mapper>(pub MappingInfo<'mapper>);
pub struct BasicAllocator<V: VirtualMapper, P: PhysicalAllocator> {
	mapper: V,
	phantom: PhantomData<P>
}

pub type RAMAllocator = BasicAllocator<KernelGlobalMapper, PhysicalRAMAllocator>;
pub type CustomRAMAllocator<T> = BasicAllocator<T, PhysicalRAMAllocator>;

impl PhysicalAllocator for PhysicalRAMAllocator {
	const DEFAULT: Self = Self {};
	fn allocate_phys(amount: usize) -> Option<BuddyAllocation> {
		buddy::allocate(amount)
	}
	fn free_phys(addr: u64, amount: usize) {
		buddy::free(addr, amount);
	}
}

impl VirtualMapper for KernelGlobalMapper {
	fn map<T: ?Sized>(&self, mut addr: BuddyAllocation, amount: usize) -> Option<*mut T> {
		addr.mapped_global::<T>(
			if amount % 0x1000 == 0 {
				amount
			} else {
				amount + 0x1000 - (amount % 0x1000)
			}
		)
	}
	fn unmap(&self, mut addr: u64, amount: usize) {
		addr.unmap(amount);
	}
}

impl PageTableMapper<'_> {
	pub fn new(flags: MappingFlags) -> Self {
		PageTableMapper(
			MappingInfo {
				addresses: &[],
				address_amount: 0,
				flags,
				page_table: current_page_table()
			}
		)
	}
}

impl Default for PageTableMapper<'_> {
	fn default() -> Self {
		PageTableMapper(
			MappingInfo {
				addresses: &[],
				address_amount: 0,
				flags: MappingFlags::None,
				page_table: current_page_table()
			}
		)
	}
}

impl VirtualMapper for PageTableMapper<'_> {
	fn map<T: ?Sized>(&self, addr: BuddyAllocation, amount: usize) -> Option<*mut T> {
		let mut info = MappingInfo {
			addresses: &addr,
			address_amount: addr.len(),
			page_table: self.0.page_table,
			..self.0
		};
		info.mapped_global::<T>(
			if amount % 0x1000 == 0 {
				amount
			} else {
				amount + 0x1000 - (amount % 0x1000)
			}
		)
	}
	fn unmap(&self, mut addr: u64, amount: usize) {
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
		let addr = P::allocate_phys(amount)?;
		self.mapper.map(
			addr,
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
