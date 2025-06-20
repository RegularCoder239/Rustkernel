#![feature(abi_x86_interrupt)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(coerce_unsized)]
#![feature(generic_const_exprs)]
#![allow(dead_code)]
#![no_main]
#![no_std]

mod errors;
mod std;
mod boot;
mod mm;
mod virt;
mod hw;
mod kernel;

use core::panic::PanicInfo;
use uefi::prelude::*;
use core::arch::asm;
use uefi::mem::memory_map::{
	MemoryMap,
	MemoryMapOwned,
	MemoryType
};
use mm::current_page_table;
use boot::UEFIResult;
use std::Mutex;
use crate::kernel::scheduler::{
	ProcessPrivilage,
	Process
};
use crate::virt::fs::{
	FilePath,
	FileStructure
};

static UEFI_RESULT: Mutex<Option<UEFIResult>> = Mutex::new(None);

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

#[entry]
fn main() -> Status {
	uefi::helpers::init().unwrap();
	log::info!("Welcome to the kernel.");

	let (uefi_result, memory_map) = boot::boot_sequence().expect("No memory map given.");
	*UEFI_RESULT.lock() = Some(uefi_result);
	mm::setup(memory_map);
	hw::cpu::setup();
		log::info!("12");
	unsafe {
		Process::new_with_stack(ProcessPrivilage::KERNEL,
								boot_core_setup as fn() -> !)
			.expect("Failed to create boot core setup task.")
			.set_pid(u64::MAX)
			.switch();
	}

	unreachable!()
}

fn boot_core_setup() -> ! {
	kernel::boot_core_setup();


	let fstest = virt::fs::TestFS {};
	log::info!("{}",
			   virt::fs::readresult_to_str(fstest.read(FilePath::new_unix("/helloworld"), 0, usize::MAX)).ok().unwrap()

	);

	log::info!("Booting non-boot CPUS.");
	hw::cpu::awake_non_boot_cpus();
	std::sti();

	loop {
		kernel::r#yield();
	}
}

#[macro_export]
macro_rules! uefi_result {
	() => {
		(*crate::UEFI_RESULT.lock()).as_ref().expect("No UEFI Result. UEFI Superseeded?")
	}
}

#[panic_handler]
fn panic(i: &PanicInfo<'_>) -> ! {
	log::error!("{}", i);

	if lapic!("lazybox").is_initalized() {
		lapic!().poweroff_other_cpus();
	}

	unsafe {
		loop {
			asm!("cli; hlt");
		}
	}
}
