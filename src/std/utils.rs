use core::{
	arch::asm
};
use super::{
	Allocator
};

pub trait With<T> {
	fn with(self, content: T) -> Self;
}

impl<T> With<T> for *mut T {
	fn with(self, content: T) -> Self {
		unsafe { *self = content; }
		self
	}
}

#[inline]
pub fn hltloop() -> ! {
	loop {
		unsafe {
			asm!("hlt");
		}
	}
}

#[inline]
pub fn cli() {
	unsafe {
		asm!("cli");
	}
}

#[inline]
pub fn sti() {
	unsafe {
		asm!("cli");
	}
}

#[inline]
pub fn cr2() -> u64 {
	unsafe {
		let cr2;
		asm!("mov {0}, cr2",
			 out(reg) cr2);
		cr2
	}
}

#[inline]
pub fn reset_cr2() {
	unsafe {
		asm!("mov cr2, {0}",
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
			core::arch::asm!("mov r8, {0}",
							"add r8, {1}",
							"call r8",
							in(reg) $meth,
							in(reg) crate::mm::kerneltable::kernel_offset(),
							in("rdi") $arg1)
		}
	};
	($meth: expr, $arg1: expr, $arg2: expr) => {
		unsafe {
			core::arch::asm!("mov r8, {0}",
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
