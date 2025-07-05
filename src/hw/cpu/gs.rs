use crate::std::{
	Box,
	Vec,
	VecBase,
	Mutex,
	count_cores
};
use core::{
	arch::asm,
	ops::Index
};

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GSContentRaw {
	pub syscall_stack: *mut u8
}

pub struct GSContent {
	pub syscall_stack: Box<[u8]>,
	pub raw: GSContentRaw
}

static GS_CONTENTS: Mutex<Vec<GSContent>> = Mutex::new(Vec::new());

impl Default for GSContent {
	fn default() -> Self {
		let syscall_stack = Box::<[u8]>::new_sized(0x2000);
		GSContent {
			raw: GSContentRaw {
				syscall_stack: unsafe {
					syscall_stack.as_ptr::<u8>().add(0x2000)
				}
			},
			syscall_stack
		}
	}
}

pub fn init() {
	let core = crate::std::current_core();
	let mut gs_contents = GS_CONTENTS.lock();
	gs_contents.resize(32);

	unsafe {
		let base = core::ptr::addr_of!(gs_contents[0].raw) as u64;
		asm!("wrgsbase {}",
			 in(reg) base
		);
		crate::std::wrmsr(0xc0000102, base);
	}
}
