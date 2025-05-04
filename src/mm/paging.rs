use super::buddy::allocate;
use crate::std::thread::mutex::Mutex;
use core::arch::asm;

#[derive(Debug)]
pub enum PagingError {
	GeneralError,
	SnappingError,
	AttemptedNonPresentUnmap,
	OutOfBoundsIndex,
	DirectoryAssociationFailed
}

#[derive(Copy, Clone)]
struct PageTableEntry {
	content: u64
}

#[repr(C, align(0x1000))]
#[derive(Copy, Clone)]
struct PageDirectory {
	directory: [PageTableEntry; 512]
}

#[derive(Copy, Clone)]
pub struct PageTable {
	directory: PageDirectory,
	temporary_free_idx: u64,
	map_lock: i64,
	free_map_addr: u64,
	temporary_tables: [PageDirectory; 3],
	kernel_page_table_l3: PageDirectory,
	kernel_page_table_l2: PageDirectory
}

pub static global_page_table_mutex: Mutex<PageTable> = Mutex::new(PageTable::EMPTY);
const TEMPORARY_ADDRESS_SPACE: u64 = 0x8000000000;
const MAP_SPACE: u64 = 0x700000000000;
const PAGE_SIZES: [usize;3] = [
	0x1000,
	0x200000,
	0x40000000
];

#[macro_export]
macro_rules! mapped {
	($phys_addr:expr, $type:ty) => {
		mapped!($phys_addr, $type, crate::mm::paging::snap_page_sizing(core::mem::size_of::<$type>()).ok()?)
	};
	($phys_addr:expr, $type:ty, $map_size:expr) => {
		Some(unsafe {
			&mut *(crate::mm::paging::global_page_table_mutex.lock().mapped(
				$phys_addr,
				$map_size,
			) as *mut $type)
		})
	}
}

impl PageTableEntry {
	const fn new(addr: u64, flags: u64) -> PageTableEntry {
		PageTableEntry {
			content: addr | flags | 0x3
		}
	}
	fn new_dir(dir: &PageDirectory) -> PageTableEntry {
		Self::new(dir.as_addr(), 0x0)
	}
	const fn new_entry(addr: u64) -> PageTableEntry {
		Self::new(addr, 0x80)
	}
	const fn new_empty() -> PageTableEntry {
		PageTableEntry {
			content: 0x0
		}
	}
	fn clear(&mut self) {
		self.content = 0x0;
	}
	fn set_addr(&mut self, addr: u64, size: usize) {
		self.content = addr | 0x3;
		if size != 0x1000 {
			self.content |= 0x80;
		}
	}
	fn addr(&self) -> u64 {
		self.content & !0xfff
	}
	fn is_present(&self) -> bool {
		self.content & 0x1 != 0
	}
	fn associated_directory(&self) -> Option<&mut PageDirectory> {
		if !self.is_present() {
			None
		} else {
			mapped!(self.addr(), PageDirectory)
		}
	}
	fn associated_directory_or_create(&mut self) -> Option<&mut PageDirectory> {
		if !self.is_present() {
			self.set_addr(allocate(0x1000).expect("Allocation failed while mapping"), 0x1000);
		}
		mapped!(self.addr(), PageDirectory)
	}
}

impl PageDirectory {
	const EMPTY: PageDirectory = PageDirectory {
		directory: [PageTableEntry::new_empty(); 512]
	};

	const fn new() -> PageDirectory {
		PageDirectory {
			directory: [PageTableEntry::new_empty(); 512]
		}
	}
	fn new_fixed_offset(phys_addr: u64, size: u64) -> PageDirectory {
		PageDirectory {
			directory: core::array::from_fn(|idx| PageTableEntry::new_entry(phys_addr + size as u64 * idx as u64))
		}
	}
	fn new_mapped(mut start: u64, end: u64, size: u64) -> PageDirectory {
		let mut dir = Self::EMPTY;
		for idx in start/size..end/size {
			dir.directory[idx as usize] = PageTableEntry::new_entry(start);
			start += 0x200000;
		}
		dir
	}
	fn as_dir_entry(&self) -> PageTableEntry {
		PageTableEntry::new_dir(self)
	}
	fn as_addr(&self) -> u64 {
		self.directory.as_ptr() as u64
	}
	fn is_present(&self, idx: u64) -> bool {
		self.directory[idx as usize].is_present()
	}
	fn nth(&self, idx: u64) -> PageTableEntry {
		self.directory[idx as usize]
	}
	fn nth_mut(&mut self, idx: u64) -> &mut PageTableEntry {
		&mut self.directory[idx as usize]
	}
	fn nth_dir(&self, idx: usize) -> Result<&PageDirectory, PagingError> {
		Ok(&*(self.directory.each_ref()[idx as usize].associated_directory().ok_or(PagingError::DirectoryAssociationFailed)?))
	}
	fn nth_mut_dir(&mut self, idx: usize) -> Result<&mut PageDirectory, PagingError> {
		Ok(&mut *(self.directory.each_mut()[idx as usize].associated_directory_or_create().ok_or(PagingError::DirectoryAssociationFailed)?))
	}
}

impl PageTable {
	const EMPTY: PageTable = PageTable {
		directory: PageDirectory::new(),
		temporary_free_idx: 0x0,
		map_lock: 0x0,
		free_map_addr: MAP_SPACE,
		temporary_tables: [PageDirectory::EMPTY; 3],
		kernel_page_table_l3: PageDirectory::new(),
		kernel_page_table_l2: PageDirectory::new()
	};
	const LEVELS: u64 = 4;
/*
	pub fn current() -> PageTable {
		PageTable {
			directory: PageDirectory::new(),
			temporary_free_idx: 0x0,
			map_lock: 0x0,
			free_map_addr: MAP_SPACE,
			temporary_tables: [PageDirectory::EMPTY; 3],
			kernel_page_table_l3: PageDirectory::new(),
			kernel_page_table_l2: PageDirectory::new()
		}
	}
*/
	pub fn setup_initial_tables(&mut self, (mut kernel_phys_start, mut kernel_phys_end): (u64, u64)) -> &mut PageTable {
		kernel_phys_start -= kernel_phys_start % 0x200000;
		kernel_phys_end += 0x200000 - (kernel_phys_end % 0x200000);
		self.directory.directory[1] = PageTableEntry::new_dir(
			&self.temporary_tables[0]
		);
		for idx in 0..self.temporary_tables.len() - 1 {
			self.temporary_tables[idx as usize].directory[0] = PageTableEntry::new_dir(
				&self.temporary_tables[idx as usize + 1]
			);
		}

		let (l4_idx, remainder_l4) = get_page_table_index(kernel_phys_start, 4);
		let (l3_idx, _) = get_page_table_index(remainder_l4, 3);
		self.kernel_page_table_l2 = PageDirectory::new_mapped(kernel_phys_start, kernel_phys_end, 0x200000);
		self.kernel_page_table_l3.directory[l3_idx] = PageTableEntry::new_dir(&self.kernel_page_table_l2);
		self.directory.directory[l4_idx] = PageTableEntry::new_dir(&self.kernel_page_table_l3);
		self
	}
	pub fn load(&mut self) -> &mut PageTable {
		let addr = self.directory.as_addr();

		unsafe {
			asm!("mov cr3, {}", in(reg) addr);
		}
		self
	}
	fn flush(&mut self) -> &mut PageTable {
		self.load()
	}
	fn is_free(&self, virt_addr_: u64, size: usize) -> bool {
		let result: Result<bool,PagingError> = (|| {
			let mut virt_addr = virt_addr_;
			let mut directory: Result<&PageDirectory, PagingError> = Ok(&self.directory);
			for level in (size_as_page_level(size)+1..Self::LEVELS).rev() {
				let idx = virt_addr / page_level_as_size(level) as u64;
				virt_addr %= page_level_as_size(level) as u64;
				if !directory.as_ref().unwrap().is_present(idx) {
					return Err::<bool, PagingError>(PagingError::GeneralError);
				}
				directory = directory.unwrap().nth_dir(idx as usize);
			}
			Ok(directory?.is_present(virt_addr / size as u64))
		})();
		result.is_err() || !result.expect("Bug 1")
	}
	fn get_page_entry(&mut self, virt_addr_: u64, size: usize) -> Result<&mut PageTableEntry, PagingError> {
		self.map_lock += 1;

		let mut virt_addr = virt_addr_;
		let mut directory: Result<&mut PageDirectory, _> = Ok(&mut self.directory);
		for level in (size_as_page_level(size)+1..Self::LEVELS).rev() {
			let idx = virt_addr / page_level_as_size(level) as u64;
			virt_addr %= page_level_as_size(level) as u64;
			directory = directory?.nth_mut_dir(idx as usize);
		}

		let idx = virt_addr / size as u64;
		self.map_lock -= 1;
		Ok(directory?.nth_mut(idx))
	}
	fn map_page(&mut self, virt_addr: u64, phys_addr: u64, size: usize) -> Result<(), PagingError> {
		self.get_page_entry(virt_addr, size)?.set_addr(phys_addr, size);
		self.flush();
		Ok(())
	}
	fn unmap_page(&mut self, virt_addr: u64, size: usize) -> Result<(), PagingError> {
		let entry = self.get_page_entry(virt_addr, size)?;
		if !entry.is_present() {
			return Err(PagingError::AttemptedNonPresentUnmap);
		}
		entry.clear();
		self.flush();
		Ok(())
	}
	fn map_temporary(&mut self, phys_addr: u64, idx: u64, size: usize) -> u64 {
		let virt_addr = TEMPORARY_ADDRESS_SPACE + size as u64 * idx;
		self.temporary_tables[(Self::LEVELS - size_as_page_level(size) - 2) as usize].directory[idx as usize].set_addr(phys_addr, 0x1000);
		self.flush();
		virt_addr
	}
	pub fn mapped(&mut self, phys_addr: u64, size: usize) -> u64 {
		self.load();

		if self.map_lock != 0 {
			self.temporary_free_idx += 1;
			self.temporary_free_idx %= 0x200;

			self.map_temporary(phys_addr, self.temporary_free_idx, size)
		} else {
			self.free_map_addr += size as u64 - (self.free_map_addr % size as u64);
			while !self.is_free(self.free_map_addr, size) {
				self.free_map_addr += size as u64;
			}
			self.map(self.free_map_addr, phys_addr, size);

			self.free_map_addr
		}
	}

	pub fn map(&mut self, virt_addr: u64, phys_addr: u64, amount: usize) -> Result<(), PagingError> {
		for offset in virt_addr_iterator(amount)? {
			self.map_page(virt_addr + offset,
						  phys_addr + offset,
						  snap_page_sizing(amount)?)?;
		}
		self.flush();
		Ok(())
	}
	pub fn unmap(&mut self, virt_addr: u64, amount: usize) -> Result<(), PagingError> {
		for offset in virt_addr_iterator(amount)? {
			self.unmap_page(virt_addr + offset, snap_page_sizing(amount)?)?;
		}
		self.flush();
		Ok(())
	}
}

const fn page_level_as_size(level: u64) -> usize {
	(1 << (level * 9)) * 0x1000
}
const fn size_as_page_level(size: usize) -> u64 {
	(size / 0x1000).trailing_zeros() as u64 / 9
}
//pub fn current_page_table() -> Mutex<PageTable> {
//	Mutex::new()
//}
// INFO: First parameter: Index Second Parameter: Remainder
fn get_page_table_index(addr: u64, level: u64) -> (usize, u64) {
	let size = page_level_as_size(level-1);
	(
		addr as usize / size,
		addr % size as u64
	)
}
pub const fn snap_page_sizing(size: usize) -> Result<usize,PagingError> {
	if size < 0x1000 {
		Ok(0x1000)
	} else if size < 0x200000 {
		Ok(0x1000)
	} else if size < 0x40000000 {
		Ok(0x200000)
	} else {
		Ok(0x40000000)
	}
}
fn virt_addr_iterator(amount: usize) -> Result<core::iter::StepBy<core::ops::Range<u64>>,PagingError> {
	Ok((0..amount as u64).step_by(snap_page_sizing(amount)?))
}
