#![no_main]
#![no_std]

use uefi::prelude::*;
use core::arch::asm;
use uefi::boot::*;
use uefi::mem::memory_map::{
	MemoryMap,
	MemoryMapOwned,
	MemoryType
};

mod std;
mod boot;
mod mm;
mod hw;

fn get_kernel_space(memory_map: &MemoryMapOwned) -> Option<(u64, u64)> {
	let descriptors = memory_map.entries().filter(|&desc| desc.ty == MemoryType::LOADER_CODE || desc.ty == MemoryType::LOADER_DATA || desc.ty == MemoryType::BOOT_SERVICES_CODE || desc.ty == MemoryType::BOOT_SERVICES_DATA);
	let max = descriptors.clone().max_by(|&d1, &d2| d1.phys_start.cmp(&d2.phys_start))?;

	Some((
		descriptors.min_by(|&d1, &d2| d1.phys_start.cmp(&d2.phys_start))?.phys_start,
		max.phys_start + max.page_count * 0x1000
	))
}

#[entry]
#[unsafe(no_mangle)]
fn main() -> Status {
	uefi::helpers::init().unwrap();
	let memory_map = boot::boot_sequence().expect("No memory map given.");

	unsafe {
		asm!("cli; lidt [0x0]");
	}

	mm::buddy::add_memory_map(&memory_map).expect("Failed to generate memory map.");
	unsafe {
		asm!("ud2");
	}
	mm::paging::global_page_table_mutex.lock()
		.setup_initial_tables(get_kernel_space(&memory_map).expect("No kernel found (impossible!)"))
		.load();

	hw::cpu::lapic::lapic().unwrap()
		.init_non_boot_cpus(0x0);
	/*loop {
		unsafe {
			asm!("ud2");
		}
	}*/
	//
	Status::SUCCESS
}
