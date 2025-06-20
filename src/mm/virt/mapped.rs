use crate::current_page_table;
use crate::std::{
	StackVec,
	VecBase,
};

const GLOBAL_ADDRESS_SPACE: u64 = 0x100000000000;
const MAP_SPACE: u64 = 0x7f8000000000;

pub trait Mapped {
	fn mapped_global<T>(&self, amount: usize) -> Option<*mut T> {
		self.mapped_at(GLOBAL_ADDRESS_SPACE, amount)
	}
	fn mapped<T>(&self, amount: usize) -> Option<*mut T> {
		self.mapped_at(MAP_SPACE, amount)
	}

	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T;
	fn mapped_at<T>(&self, addr_space: u64, amount: usize) -> Option<*mut T>;
	fn unmap(&self, amount: usize);
}

pub trait Address {
	fn physical_address(&self) -> u64;
}

impl Mapped for StackVec<u64, 0x200> {
	fn mapped_temporary<T>(&self, _: usize) -> &'static mut T {
		todo!("Temporary mapping only avaiable with one single aligned physical address.")
	}

	fn mapped_at<T>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		Some(current_page_table().mapped_at(
			addr_space,
			self,
			self.len(),
			amount
		).ok()? as *mut T)
	}

	fn unmap(&self, _: usize) {
		todo!("Unmapping only makes sense for a single address.")
	}
}

impl Mapped for u64 {
	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T {
		unsafe {
			&mut *(current_page_table().mapped_temporary(*self, amount) as *mut T)
		}
	}

	fn mapped_at<T>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		Some(
			current_page_table().mapped_at(
				addr_space,
					[*self],
				1,
				amount
			).ok()? as *mut T,
		)
	}

	fn unmap(&self, amount: usize) {
		current_page_table().unmap(*self, amount);
	}
}

impl<T2> Mapped for *const T2 {
	fn mapped_temporary<T>(&self, amount: usize) -> &'static mut T {
		(*self as u64).mapped_temporary(amount)
	}

	fn mapped_at<T>(&self, addr_space: u64, amount: usize) -> Option<*mut T> {
		(*self as u64).mapped_at(addr_space, amount)
	}
	fn unmap(&self, unused: usize) {
		(*self as u64).unmap(unused)
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

impl<T, const Size: usize> Address for [T; Size] {
	fn physical_address(&self) -> u64 {
		self.as_slice().as_ptr().physical_address()
	}
}
