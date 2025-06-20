use super::{
	PageDirectory,
	Mapped
};
use super::super::buddy;

#[derive(Copy, Clone)]
pub struct PageTableEntry {
	pub content: u64
}

impl PageTableEntry {
	const fn new(addr: u64, flags: u64) -> PageTableEntry {
		PageTableEntry {
			content: addr | flags | 0x3
		}
	}
	pub fn new_dir(dir_addr: u64) -> PageTableEntry {
		Self::new(dir_addr, 0x0)
	}
	pub const fn new_entry(addr: u64) -> PageTableEntry {
		Self::new(addr, 0x80)
	}
	pub const fn new_empty() -> PageTableEntry {
		PageTableEntry {
			content: 0x0
		}
	}
	pub fn clear(&mut self) {
		self.content = 0x0;
	}
	pub fn set_addr(&mut self, addr: u64, size: usize) {
		self.content = addr | 0x3;
		if size != 0x1000 {
			self.content |= 0x80;
		}
	}
	pub fn addr(&self) -> u64 {
		self.content & !0xfff
	}
	pub fn is_present(&self) -> bool {
		self.content & 0x1 == 1
	}
	pub fn is_dir(&self) -> bool {
		self.content & 0x80 == 0
	}
	pub fn is_present_entry(&self) -> bool {
		self.content & 0x81 == 0x81
	}
	pub fn dir(&self) -> Option<&PageDirectory> {
		if !self.is_present() {
			None
		} else {
			Some(self.addr().mapped_temporary::<PageDirectory>(0x1000))
		}
	}
	pub fn mut_dir(&mut self) -> Option<&mut PageDirectory> {
		if !self.is_present() {
			self.set_addr(buddy::allocate_aligned(0x1000)?, 0x1000);
		}
		Some(self.addr().mapped_temporary::<PageDirectory>(0x1000))
	}
}
