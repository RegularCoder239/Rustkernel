#![feature(abi_x86_interrupt)]
#![feature(coerce_unsized)]
#![feature(ptr_metadata)]
#![feature(unsize)]

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
	Process
};
use std::log;
use core::arch::x86_64::__cpuid;

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
	unsafe {
		let cpuid_features = __cpuid(0x1);
		if cpuid_features.edx & 0x2000269 != 0x2000269 {
			panic!("Requires a CPU with the features: FPU, PSE, MSR, PAE, APIC and SSE.");
		}
		let cpuid_features_extended = __cpuid(0x7);
		if cpuid_features_extended.ebx & 0x1 != 0x1 {
			panic!("Requires a CPU with the wrgsbase instruction.");
		}

		let idtr = [0_u64, 0];
		core::arch::asm!("push 0x2",
						 "popf",
						 "cli",
						 "lidt [{0}]",
						 "mov rax, 0x10676",
						 "mov cr4, rax",
						 "wrgsbase {1:r}",
						 in(reg) idtr.as_ptr(),
						 in(reg) 0);
	}
	*UEFI_RESULT.lock() = Some(uefi_result);
	mm::setup(memory_map);
	hw::cpu::setup1();

	log::debug!("Setting up boot setup process.");
	Process::spawn_init_process(boot_core_setup as fn() -> !)
}

fn boot_core_setup() -> ! {
	log::debug!("Boot process successfully started.");
	std::cli();
	std::wrmsr(0xc0000080, 0xd01);

	kernel::boot_core_setup();
	hw::cpu::setup2();
	hw::cpu::awake_non_boot_cpus();
	std::sti();

	loop {
		kernel::r#yield();
	}
}

#[macro_export]
macro_rules! uefi_result {
	() => {
		(*crate::UEFI_RESULT.lock()).as_ref()
	}
}

#[panic_handler]
fn panic(i: &PanicInfo<'_>) -> ! {
	//crate::hw::power::shutdown();
	unsafe {
		let array: [u64; 2] = [0, 0];
		asm!("lidt [{0}]", in(reg) array.as_ptr());
	}
	log::error!("{} Unrecovable kernel panic. Please turn off the pc.", i);

	if lapic!("lazybox").is_initalized() {
		lapic!().poweroff_other_cpus();
	}

	std::cli();
	std::hltloop();
}
