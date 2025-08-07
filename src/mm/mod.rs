mod virt;
pub mod buddy;
pub use virt::*;

use uefi::mem::memory_map::{
	MemoryMapOwned,
	MemoryType,
	MemoryMap
};
use crate::std::{
	PerCpu,
	log
};

static INITALIZED: PerCpu<bool> = PerCpu::new(false);

pub const fn align_size(size: usize) -> usize {
	if size < 0x200000 {
		0x1000
	} else if size < 0x40000000 {
		0x200000
	} else {
		0x40000000
	}
}


fn get_kernel_space(memory_map: &MemoryMapOwned) -> Option<(u64, u64)> {
	let descriptors = memory_map.entries().filter(|&desc| desc.ty == MemoryType::LOADER_CODE ||
												  desc.ty == MemoryType::LOADER_DATA ||
												  desc.ty == MemoryType::RUNTIME_SERVICES_CODE ||
												  desc.ty == MemoryType::RUNTIME_SERVICES_DATA);
	let max = descriptors.clone().max_by(|&d1, &d2| d1.phys_start.cmp(&d2.phys_start))?;

	Some((
		descriptors.min_by(|&d1, &d2| d1.phys_start.cmp(&d2.phys_start))?.phys_start,
		  max.phys_start + max.page_count * 0x1000
	))
}

#[inline]
pub fn setup(memory_map: MemoryMapOwned) {
	log::debug!("Setting up memory manager.");
	buddy::add_memory_map(&memory_map);
	let kernel_space = get_kernel_space(&memory_map).expect("No kernel found (impossible!)");
	kerneltable::setup(kernel_space);

	virt::setup_global_table();
	{
		let mut page_table = current_page_table()
			.lock();
		page_table.init();
		page_table.load();
	}

	kerneltable::setup_kernel_offset();
	INITALIZED.set(true);
}

#[inline]
pub fn per_core_setup() {
	current_page_table()
		.lock()
		.load();
	INITALIZED.set(true);
}

pub fn initalized() -> bool {
	*INITALIZED.deref()
}

