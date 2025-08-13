use crate::std::{
	Box,
	Vec,
	Mutex,
	current_core_uncached
};
use core::{
	arch::asm
};

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GSContentRaw {
	pub syscall_stack: *mut u8,
	pub core_idx: u64,
	pub signature: u64
}

pub struct GSContent {
	pub syscall_stack: Box<[u8]>,
	pub raw: GSContentRaw
}

static GS_CONTENTS: Mutex<Vec<GSContent>> = Mutex::new(Vec::new());

#[macro_export]
macro_rules! gs_read {
	($idx: ident, $type: ty) => {
		unsafe {
			let gsbase: usize;
			asm!("rdgsbase {}",
					out(reg) gsbase);
			if gsbase == 0 {
				None
			} else {
				let gs = (gsbase as *const GSContentRaw).as_ref().unwrap();
				if gs.signature == 0x8988d80d6631faec {
					Some(gs.$idx)
				} else {
					None
				}
			}
		}
	}
}

impl Default for GSContent {
	fn default() -> Self {
		let syscall_stack = Box::<[u8]>::new_sized(0x20000);
		GSContent {
			raw: GSContentRaw {
				syscall_stack: syscall_stack.as_stack(),
				core_idx: current_core_uncached() as u64,
				signature: 0x8988d80d6631faec
			},
			syscall_stack
		}
	}
}

pub fn init() {
	let base = core::ptr::addr_of!(
		GS_CONTENTS.lock().push_back(GSContent::default()).raw
	) as u64;
	unsafe {
		asm!("wrgsbase {}",
			 in(reg) base
		);
		crate::std::wrmsr(0xc0000102, base);
	}
}

pub fn current_core() -> Option<u64> {
	gs_read!(core_idx, u64)
}
