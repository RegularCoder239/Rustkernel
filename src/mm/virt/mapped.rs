use crate::{
	current_page_table,
	stack_vec
};
use crate::std::{
	StackVec,
	VecBase,
	MutableRef
};
use crate::mm::{
	PageTable
};
use core::ops::{
	Index
};

const GLOBAL_ADDRESS_SPACE: u64 = 0x100000000000;
const MAP_SPACE: u64 = 0x7f8000000000;

pub trait Mapped {
	fn mapped_global<T: ?Sized>(&self, amount: usize) -> Option<*mut T> {
		self.mapped_at(GLOBAL_ADDRESS_SPACE, amount)
	}
	fn mapped<T: ?Sized>(&self, amount: usize) -> Option<*mut T> {
		self.mapped_at(MAP_SPACE, amount)
	}
	fn mapped_to_page_table<T: ?Sized, X: Index<usize, Output = u64>>(addr_space: u64, amount: usize, physical_addresses: X, amount_physical_addresses: usize, page_table: &mut PageTable, flags: u64) -> Option<*mut T> {
		Some(
			core::ptr::from_raw_parts_mut::<T>(
				page_table.mapped_at(
					addr_space,
					physical_addresses,
					amount_physical_addresses,
					amount,
					flags
				).ok()? as *mut (),
				core::ptr::metadata(
					unsafe {
						core::mem::MaybeUninit::<*const T>::zeroed().assume_init()
					}
				)
			)
		)
	}

	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T;
	fn mapped_at<T: ?Sized>(&self, addr_space: u64, amount: usize) -> Option<*mut T>;
	fn unmap(&self, amount: usize) -> bool;
}

pub trait Address {
	fn physical_address(&self) -> u64;
}

#[derive(Clone, Copy)]
pub enum MappingFlags {
	None = 0,
	ReadOnly = 1 << 1,
	User = 1 << 2,
	Executable = 1 << 63
}

pub struct MappingInfo<'mapping_info> {
	pub page_table: MutableRef<'mapping_info, PageTable>,
	pub address: u64,
	pub flags: MappingFlags
}

impl Mapped for StackVec<u64, 0x200> {
	fn mapped_temporary<T>(&self, _: usize) -> &'static mut T {
		todo!("Temporary mapping only avaiable with one single aligned physical address.")
	}

	fn mapped_at<T: ?Sized>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		Self::mapped_to_page_table(addr_space, amount, self, self.len(), current_page_table(), 0)
	}

	fn unmap(&self, _: usize) -> bool {
		todo!("Unmapping only makes sense for a single address.")
	}
}

impl Mapped for u64 {
	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T {
		unsafe {
			&mut *(current_page_table().mapped_temporary(*self, amount) as *mut T)
		}
	}

	fn mapped_at<T: ?Sized>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		Self::mapped_to_page_table(addr_space, amount, [*self], 1, current_page_table(), 0)
	}

	fn unmap(&self, amount: usize) -> bool {
		current_page_table().unmap(*self, amount)
	}
}

impl<T2> Mapped for *const T2 {
	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T {
		(*self as u64).mapped_temporary(amount)
	}

	fn mapped_at<T: ?Sized>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		(*self as u64).mapped_at(addr_space, amount)
	}
	fn unmap(&self, amount: usize) -> bool {
		(*self as u64).unmap(amount)
	}
}

impl Mapped for MappingInfo<'_> {
	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T {
		todo!()
	}

	fn mapped_at<T: ?Sized>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		Self::mapped_to_page_table(addr_space, amount, [self.address], 1, self.page_table.deref_mut(), self.flags as u64)
	}
	fn unmap(&self, amount: usize) -> bool {
		self.page_table.deref_mut().unmap(self.address, amount)
	}
}

impl<T> Address for *const T {
	fn physical_address(&self) -> u64 {
		(*self as u64).physical_address()
	}
}
impl<T> Address for *mut T {
	fn physical_address(&self) -> u64 {
		(*self as u64).physical_address()
	}
}

impl<T> Address for &T {
	fn physical_address(&self) -> u64 {
		(*self as *const T).physical_address()
	}
}

impl Address for u64 {
	fn physical_address(&self) -> u64 {
		if !crate::mm::initalized() {
			*self
		} else {
			current_page_table().physical_address(*self).expect("Attempt to gather physical address of invalid virtual address.")
		}
	}
}

impl<T, const SIZE: usize> Address for [T; SIZE] {
	fn physical_address(&self) -> u64 {
		self.as_slice().as_ptr().physical_address()
	}
}
