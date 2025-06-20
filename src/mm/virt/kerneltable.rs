use crate::std::{
	Mutex,
	PerCpu
};
use super::{
	PageDirectory,
	PageTable,
	PAGE_SIZES
};

struct KernelTable {
	legacy_tables: [PageDirectory; 2],
	higher_half_l3_table: PageDirectory,
	higher_half_l2_table: PageDirectory,
	higher_half_l1_tables: [PageDirectory; 50],
	higher_half_l1_table_amount: usize,
	legacy_table_l4_idx: usize,
	higher_half_table_l4_idx: usize,
	kernel_offset: u64
}

const KERNEL_ADDRESS_SPACE: u64 = 0xffff800000000000;
static KERNEL_TABLE: Mutex<KernelTable> = Mutex::new(
	KernelTable {
		legacy_tables: [
			PageDirectory::EMPTY,
			PageDirectory::EMPTY
		],
		higher_half_l3_table: PageDirectory::EMPTY,
		higher_half_l2_table: PageDirectory::EMPTY,
		higher_half_l1_tables: [PageDirectory::EMPTY; 50],
		higher_half_l1_table_amount: 0x0,
		legacy_table_l4_idx: 0x0,
		higher_half_table_l4_idx: (KERNEL_ADDRESS_SPACE & 0xffffffffffff) as usize / PAGE_SIZES[4],
		kernel_offset: 0x0
	}
);
static CURRENT_KERNEL_OFFSET: PerCpu<u64> = PerCpu::new(0x0);

impl KernelTable {
	pub fn setup_mapped(&mut self, kernel_region: (u64, u64)) {
		let aligned_kernel_region = (
			kernel_region.0 - (kernel_region.0 % 0x200000),
			kernel_region.1 + (0x200000 - (kernel_region.1 % 0x200000))
		);

		self.legacy_tables[1] = PageDirectory::new_mapped(aligned_kernel_region, 0x200000);
		self.legacy_table_l4_idx = kernel_region.0 as usize / PAGE_SIZES[4];
		self.kernel_offset = KERNEL_ADDRESS_SPACE - kernel_region.0;

		self.generate_l1_tables(kernel_region);

		let legacy_l3_idx = (kernel_region.0 as usize % PAGE_SIZES[4]) / PAGE_SIZES[3];

		for idx in 0..self.higher_half_l1_table_amount {
			self.higher_half_l2_table[idx] = self.higher_half_l1_tables.each_ref()[idx].as_dir_entry();
		}
		self.higher_half_l3_table[0] = self.higher_half_l2_table.as_dir_entry();
		self.legacy_tables[0][legacy_l3_idx] = self.legacy_tables[1].as_dir_entry();
	}

	pub fn map_to_pagetable(&self, page_table: &mut PageTable) {
		page_table.directory[self.legacy_table_l4_idx] = self.legacy_tables[0].as_dir_entry();
		page_table.directory[self.higher_half_table_l4_idx] = self.higher_half_l3_table.as_dir_entry();
	}

	fn generate_l1_tables(&mut self, kernel_region: (u64, u64)) {
		let mut region_addr = kernel_region.0;

		while region_addr < kernel_region.1 {
			let region_end = (region_addr + 0x200000).min(kernel_region.1);
			self.higher_half_l1_tables[self.higher_half_l1_table_amount] = PageDirectory::new_mapped_nonaligned((region_addr, region_end), 0x1000);
			region_addr = region_end;
			self.higher_half_l1_table_amount += 1;
		}
	}
}

pub fn setup(kernel_region: (u64, u64)) {
	if kernel_region.0 % 0x1000 != 0 || kernel_region.1 % 0x1000 != 0 {
		panic!("Unaligned page region. Start: {:x} End: {:x}", kernel_region.0, kernel_region.1);
	}

	KERNEL_TABLE.lock().setup_mapped(kernel_region);
	*CURRENT_KERNEL_OFFSET.deref_mut() = 0x0;
}

pub fn setup_kernel_offset() {
	*CURRENT_KERNEL_OFFSET.deref_mut() = KERNEL_TABLE.lock().kernel_offset;
}

pub fn kernel_offset() -> u64 {
	*CURRENT_KERNEL_OFFSET.deref()
}

pub fn map_pagetable(page_table: &mut PageTable) {
	KERNEL_TABLE.lock().map_to_pagetable(page_table)
}
