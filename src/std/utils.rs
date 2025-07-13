use core::arch::asm;
use crate::kernel::is_initalized;

#[macro_export]
macro_rules! assume_safe_asm {
	($code: expr) => {
		unsafe {
			core::arch::asm!($code);
		}
	};
	($code: expr, in, $($inputs:expr), +) => {
		unsafe {
			core::arch::asm!($code, $(in(reg) $inputs)+);
		}
	}
}

#[inline]
pub fn hltloop() -> ! {
	loop {
		assume_safe_asm!("hlt");
	}
}

#[inline]
pub fn wait() {
	if is_initalized() {
		super::r#yield();
	} else {
		assert!(super::count_cores() > 1, "Fatal deadlock.");
		assume_safe_asm!("pause");
	}
}

#[inline]
pub fn cli() {
	assume_safe_asm!("cli");
}

#[inline]
pub fn sti() {
	assume_safe_asm!("sti");
}

#[inline]
pub fn cr2() -> u64 {
	let cr2;
	unsafe {
		asm!("mov {0}, cr2",
			 out(reg) cr2);
	}
	cr2
}

#[inline]
pub fn reset_cr2() {
	unsafe {
		asm!("mov cr2, {0:r}",
			 in(reg) 0x0);
	}
}

#[inline]
pub fn wrmsr(msr: u32, value: u64) {
	unsafe {
		asm!("wrmsr",
			 in("eax") value as u64,
			 in("ecx") msr,
			 in("edx") (value >> 32) as u32
		);
	}
}
#[inline]
pub fn rdmsr(msr: u32) -> u64 {
	let (high, low): (u32, u32);
	unsafe {
		asm!("rdmsr", out("eax") low, out("edx") high, in("ecx") msr);
	}
	((high as u64) << 32) | (low as u64)
}

#[macro_export]
macro_rules! call_asm {
	($meth: expr, $arg1: expr) => {
		unsafe {
			asm!("mov r8, {0}",
				"add r8, {1}",
				"call r8",
				in(reg) $meth,
				in(reg) crate::mm::kerneltable::kernel_offset(),
				in("rdi") $arg1)
		}

	};
	($meth: expr, $arg1: expr, $arg2: expr) => {
		unsafe {
			asm!("mov r8, {0}",
				 "add r8, {1}",
				 "call r8",
				 in(reg) $meth,
				 in(reg) crate::mm::kerneltable::kernel_offset(),
				 in("rdi") $arg1,
				 in("rsi") $arg2)
		}
	}
}

#[macro_export]
macro_rules! add_meth {
	($meth: expr, $meth_type: ty, $offset: expr) => {{
		let mut pointer: u64 = $meth as *mut u64 as u64;
		pointer += $offset;
		core::mem::transmute::<*mut u64, $meth_type>(pointer as *mut u64)
	}}
}
