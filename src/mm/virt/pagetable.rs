use super::{
	PagingError,
	PageTableEntry,
	PageDirectory,
	kerneltable,
	kernel_offset
};
use crate::std::{
	Mutex,
	Box,
	log
};
use core::{
	ops::Index,
	iter::*
};
use crate::mm::align_size;
use crate::assume_safe_asm;

pub struct PageTable {
	pub directory: PageDirectory,
	first_free_address: [u64; 512],
	temporary_directories: Mutex<[PageDirectory; 3]>,
	temporary_index: u64,
	physical_offset: u64,
	initalized: bool,
	cr3: u64
}

const TEMPORARY_ADDRESS_SPACE: u64 = 0x8000000000;

static GLOBAL_PAGE_DIRECTORIES_MUTEX: Mutex<[PageDirectory; 3]> = Mutex::new(
	[PageDirectory::EMPTY; 3]
);
static GLOBAL_INDEX_MUTEX: Mutex<u64> = Mutex::new(0x0);

impl PageTable {
	pub const EMPTY: PageTable = PageTable {
		directory: PageDirectory::EMPTY,
		first_free_address: [0x0; 512],
		temporary_directories: Mutex::new([PageDirectory::EMPTY; 0x3]),
		temporary_index: 0,
		physical_offset: 0,
		initalized: false,
		cr3: 0
	};
	const LEVELS: u64 = 4;

	pub fn new() -> PageTable {
		let mut table = PageTable::EMPTY;
		table.init();
		table
	}
	pub fn new_boxed() -> Box<Mutex<PageTable>> {
		let table = Box::<Mutex<PageTable>>::new(Mutex::new(PageTable::EMPTY));
		table.lock().init();
		table
	}
	pub fn as_cr3(&self) -> u64 {
		if self.directory.as_addr() > kernel_offset() {
			self.directory.as_addr() - kernel_offset()
		} else {
			self.directory.as_phys_addr()
		}
	}
	pub fn init(&mut self) {
		for idx in 0..self.temporary_directories.len() - 1 {
			self.temporary_directories.lock()[idx as usize][0] =
				self.temporary_directories[idx as usize + 1].as_dir_entry();
		}
		self.directory[1] = self.temporary_directories[0].as_dir_entry();
		self.directory[0x20] = GLOBAL_PAGE_DIRECTORIES_MUTEX.lock()[0].as_dir_entry();
		kerneltable::map_pagetable(self);
		self.initalized = true;
	}
	pub fn load(&self) {
		assume_safe_asm!("mov cr3, {}", in, self.as_cr3());
	}

	pub fn flush() {
		assume_safe_asm!("mov rax, cr3\n
						  mov cr3, rax");
	}
	pub fn is_free(&self, virt_addr: u64, size: usize) -> bool {
		for s in virt_addr_iterator(virt_addr, size) {

			let entry = self.get_page_entry(virt_addr + s, align_size(size));

			if entry.is_some() && entry.unwrap().is_present() {
				return false;
			}
		}
		return true;
	}
	fn get_page_entry(&self, virt_addr_: u64, size: usize) -> Option<&PageTableEntry> {
		let mut virt_addr = virt_addr_;
		let mut directory: &PageDirectory = &self.directory;

		for level in (size_as_page_level(size)+1..Self::LEVELS).rev() {
			let idx = virt_addr / page_level_as_size(level) as u64;
			virt_addr %= page_level_as_size(level) as u64;
			directory = directory[idx as usize].dir()?;
		}
		Some(&directory[virt_addr as usize / size])
	}
	fn get_page_entry_mut(&mut self, mut virt_addr: u64, size: usize) -> Option<&mut PageTableEntry> {
		let mut directory: &mut PageDirectory = &mut self.directory;

		for level in (size_as_page_level(size)+1..Self::LEVELS).rev() {
			let idx = virt_addr / page_level_as_size(level) as u64;
			virt_addr %= page_level_as_size(level) as u64;
			directory = directory[idx as usize].mut_dir()?;
		}
		Some(&mut directory[virt_addr as usize / size])
	}
	fn map_page(&mut self, virt_addr: u64, phys_addr: u64, size: usize, flags: u64) -> bool {
		assert!(phys_addr & 0xfff == 0, "Attempt to map unaligned address: {:x}", phys_addr);
		if let Some(entry) = self.get_page_entry_mut(virt_addr, size) {
			entry.set_addr(phys_addr, size);
			entry.set_flags(flags);
			true
		} else {
			false
		}
	}
	fn unmap_page(&mut self, virt_addr: u64, size: usize) -> bool {
		if let Some(entry) = self.get_page_entry_mut(virt_addr, size) {
			entry.clear(); true
		} else {
			false
		}
	}
	pub fn gather_physical_address(&mut self, mut virtual_address: u64) -> Option<u64> {
		let virtual_address_cpy = virtual_address;
		virtual_address &= 0xffffffffffff;
		let mut directory: Option<&mut PageDirectory> = Some(&mut self.directory);

		for level in (0..Self::LEVELS).rev() {
			let idx = virtual_address / page_level_as_size(level) as u64;
			virtual_address %= page_level_as_size(level) as u64;
			let dir_unpacked = directory?;
			if dir_unpacked[idx as usize].is_present() && (!dir_unpacked[idx as usize].is_dir() || level == 0x0) {
				return Some(dir_unpacked[idx as usize].addr());
			} else if !dir_unpacked[idx as usize].is_present() {
				log::error!("Attempt to gather physical address of invalid virtual address: {:x}", virtual_address_cpy);
				return None;
			}
			directory = dir_unpacked[idx as usize].mut_dir();
		}
		log::error!("Attempt to gather physical address of invalid virtual address: {:x}", virtual_address_cpy);
		return None;
	}
	pub fn mapped_at<X: Index<usize, Output = u64> + ?Sized>(&mut self, addr_space: u64, phys_addresses: &X, phys_addresses_amount: usize, mut size: usize, flags: u64) -> Result<u64, PagingError> {

		assert!(self.initalized, "Attempt to map in uninitalized page table.");
		if size < 0x1000 {
			size = 0x1000;
		}

		let free_map_idx: usize = addr_space as usize / 0x8000000000;
		let mut first_free_address = self.first_free_address[free_map_idx] + addr_space;
		let aligned_size = align_size(size);

		first_free_address += aligned_size as u64 - (first_free_address % aligned_size as u64);
		while !self.is_free(first_free_address, size) {
			first_free_address += aligned_size as u64;
		}

		self.first_free_address[free_map_idx] = first_free_address + size as u64 - addr_space;

		let mut current_phys_address = phys_addresses[phys_addresses_amount - 1];
		for (idx, offset) in virt_addr_iterator(first_free_address, size).enumerate() {

			if !self.map_page(first_free_address + offset,
							  if idx < phys_addresses_amount {
								  phys_addresses[idx]
							  } else {
								  current_phys_address += aligned_size as u64;
								  current_phys_address
							  },
							  aligned_size,
							  flags) {
				panic!("Mapping failed.");
			}
		}

		PageTable::flush();
		Ok(first_free_address)
	}

	pub fn map(&mut self, virt_addr: u64, phys_addr: u64, amount: usize, flags: u64) -> bool {
		for _ in virt_addr_iterator(phys_addr, amount) {
			if !self.map_page(virt_addr + 0,
							  phys_addr + 0,
							  amount,
							  flags) {
				return false;
			}
		}
		PageTable::flush();
		true
	}
	pub fn unmap(&mut self, virt_addr: u64, amount: usize) -> bool {
		for offset in virt_addr_iterator(virt_addr, amount) {
			if !self.unmap_page(virt_addr + offset, align_size(amount)) {
				return false;
			}
		}
		PageTable::flush();
		true
	}
	pub fn mapped_temporary<T>(&self, phys_addr: u64, size: usize) -> &'static mut T {
		if size != 0x1000 {
			todo!("Temporary mapping support for other sizes than 4k.");
		}
		self.temporary_directories.lock()[2 - (size_as_page_level(size)) as usize][1].set_addr(phys_addr, size);
		PageTable::flush();
		unsafe {
			((TEMPORARY_ADDRESS_SPACE + size as u64) as *mut T).as_mut().expect("Temporary address calculation failed.")
		}
	}
}

const fn page_level_as_size(level: u64) -> usize {
	(1 << (level * 9)) * 0x1000
}
const fn size_as_page_level(size: usize) -> u64 {
	(size / 0x1000).trailing_zeros() as u64 / 9
}

// INFO: First parameter: Index Second Parameter: Remainder
fn get_page_table_index(mut addr: u64, level: u64) -> (usize, u64) {
	addr &= 0xffffffffffff;
	let size = page_level_as_size(level-1);
	(
		addr as usize / size,
  addr % size as u64
	)
}
fn virt_addr_iterator(_: u64, amount: usize) -> core::iter::StepBy<core::ops::Range<u64>> {
	(0..amount as u64).step_by(align_size(amount))
}

pub fn setup_global_table() {
	let mut global_tables = GLOBAL_PAGE_DIRECTORIES_MUTEX.lock();
	for idx in 0..global_tables.len() - 1 {
		global_tables[idx as usize][0] = PageTableEntry::new_dir(
			global_tables[idx as usize + 1].as_addr()
		);
	}
}
