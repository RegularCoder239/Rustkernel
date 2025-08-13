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
	pub const EMPTY: PageTableEntry = PageTableEntry {
		content: 0x0
	};
	const fn new(addr: u64, flags: u64) -> PageTableEntry {
		PageTableEntry {
			content: addr | flags | 0x3
		}
	}
	pub const fn new_dir(dir_addr: u64) -> PageTableEntry {
		Self::new(dir_addr, 0x4)
	}
	pub const fn new_entry(addr: u64, flags: u64) -> PageTableEntry {
		Self::new(addr, 0x80 | flags as u64)
	}
	pub fn clear(&mut self) {
		self.content = 0x0;
	}
	pub fn set_addr(&mut self, addr: u64, size: usize) {
		//self.content &= 0xfff;
		self.content = addr | 0x3;
		if size != 0x1000 {
			self.content |= 0x80;
		}
	}
	pub fn set_dir_addr(&mut self, addr: u64) {
		self.content &= 0xfff;
		self.content = addr | 0x7;
	}
	pub fn set_flags(&mut self, flags: u64) {
		self.content |= flags;
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
			let addr = buddy::allocate_aligned(0x1000)?;
			self.set_dir_addr(addr);
		}
		Some(self.addr().mapped_temporary::<PageDirectory>(0x1000))
	}
}
