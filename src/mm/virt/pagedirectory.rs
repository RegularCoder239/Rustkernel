use super::{
	PageTableEntry,
	Address
};
use core::ops::{
	Index,
	IndexMut
};

#[repr(C, align(0x1000))]
#[derive(Clone, Copy)]
pub struct PageDirectory {
	directory: [PageTableEntry; 512]
}

impl PageDirectory {
	pub const EMPTY: PageDirectory = PageDirectory {
		directory: [PageTableEntry::new_empty(); 512]
	};

	pub const fn new() -> PageDirectory {
		PageDirectory {
			directory: [PageTableEntry::new_empty(); 512]
		}
	}
	pub fn new_fixed_offset(phys_addr: u64, size: u64) -> PageDirectory {
		PageDirectory {
			directory: core::array::from_fn(|idx| PageTableEntry::new_entry(phys_addr + size as u64 * idx as u64))
		}
	}
	pub fn new_mapped((mut start, end): (u64, u64), size: u64) -> PageDirectory {
		let mut dir = Self::EMPTY;
		let mut endidx = end % (size*0x200);
		if endidx == 0 {
			endidx = 0x200;
		} else {
			endidx /= size;
		}
		for idx in (start & (size*0x200-1))/size..endidx {
			dir.directory[idx as usize] = PageTableEntry::new_entry(start);
			start += size;
		}
		dir
	}
	pub fn new_mapped_nonaligned((mut start, end): (u64, u64), size: u64) -> PageDirectory {
		let mut dir = Self::EMPTY;
		for idx in 0..(end - start) / size {
			dir.directory[idx as usize] = PageTableEntry::new_entry(start);
			start += size;
		}
		dir
	}
	pub fn as_dir_entry(&self) -> PageTableEntry {
		PageTableEntry::new_dir(self.as_phys_addr())
	}
	pub fn as_addr(&self) -> u64 {
		self.directory.as_ptr() as u64
	}
	pub fn as_phys_addr(&self) -> u64 {
		self.as_addr().physical_address()
	}
	pub fn is_present(&self, idx: usize) -> bool {
		self.directory[idx].is_present()
	}
	pub fn idx_present(&self, idx: usize) -> Option<&PageTableEntry> {
		if self.is_present(idx) {
			Some(self.index(idx))
		} else {
			None
		}
	}
	pub fn idx_present_mut(&mut self, idx: usize) -> Option<&mut PageTableEntry> {
		if self.is_present(idx) {
			Some(self.index_mut(idx))
		} else {
			None
		}
	}
}

impl Index<usize> for PageDirectory {
	type Output = PageTableEntry;
	fn index(&self, index: usize) -> &PageTableEntry {
		&self.directory[index]
	}
}

impl IndexMut<usize> for PageDirectory {
	fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
		&mut self.directory[index]
	}
}
