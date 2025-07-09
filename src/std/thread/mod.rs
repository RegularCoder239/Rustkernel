pub mod lock;
pub mod mutex;
pub mod percpu;
pub mod lazymutex;

use core::arch::x86_64::__cpuid;

pub fn current_core_uncached() -> u32 {
	unsafe {
		__cpuid(0x1).ebx >> 24
	}
}

pub fn current_core() -> u64 {
	if let Some(idx) = crate::hw::cpu::gs::current_core() {
		idx
	} else {
		current_core_uncached() as u64
	}
}

pub fn count_cores() -> usize {
	crate::hw::acpi::acpi_singleton().madt_core_amount().unwrap_or(1)
}
