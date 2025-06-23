use core::{
	arch::asm
};
use super::{
	Allocation,
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
impl<T, A: Allocator> With<T> for Allocation<T, A> {
	fn with(self, content: T) -> Self {
		*self.as_mut() = content;
		self
	}
}

#[inline]
pub fn hltloop() -> ! {
	loop {
		unsafe {
			asm!("sti; hlt");
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
