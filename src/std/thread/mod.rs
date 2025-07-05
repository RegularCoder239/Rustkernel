pub mod lock;
pub mod mutex;
pub mod percpu;
pub mod lazymutex;

use core::arch::x86_64::__cpuid;

pub fn current_core() -> u32 {
	unsafe {
		__cpuid(0x1).ebx >> 24
	}
}

pub fn cpucore() -> u32 {
	current_core()
}

pub fn count_cores() -> usize {
	crate::hw::acpi::acpi_singleton().madt_core_amount().unwrap_or(1)
}
