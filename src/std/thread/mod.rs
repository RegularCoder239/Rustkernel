pub mod lock;
pub mod mutex;
pub mod percpu;
pub mod lazymutex;

use core::arch::x86_64::__cpuid;

pub fn cpucore() -> u32 {
	unsafe {
		__cpuid(0x1).ebx >> 24
	}
}
