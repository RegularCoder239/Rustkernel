use crate::std::{
	Mutex,
	Vec,
	wrmsr
};
use core::arch::{
	naked_asm
};

pub struct Function {
	pub id: u64,
	pub meth: fn(&[u64]) -> u64
}

static FUNCIONALITIES: Mutex<Vec<Function>> = Mutex::new(Vec::new());

impl Function {
	pub fn add(self) {
		FUNCIONALITIES.lock().push_back(self);
	}
}

fn finish_functionality_list() {
	FUNCIONALITIES.lock().sort(|a, b| a.id < b.id)
}

pub fn setup() {
	wrmsr(0xc0000082, __do_syscall as u64 + crate::mm::kernel_offset()); // LSTAR
	wrmsr(0xc0000081, 0x8 << 32 | 0x1b << 48);
}

#[unsafe(no_mangle)]
pub unsafe extern "sysv64" fn do_syscall(function: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64, arg6: u64) -> u64 {
	crate::std::log::debug!("Syscall args: {:x} {:x} {:x} {:x} {:x} {:x} {:x}", function, arg1, arg2, arg3, arg4, arg5, arg6);
	let lock = FUNCIONALITIES.lock();
	if let Some(f) = lock.into_iter().find(|a| a.id == function) {
		(f.meth)(&[arg1, arg2, arg3, arg4, arg5, arg6])
	} else {
		crate::std::log::error!("Invalid syscall opcode: {:x}", function);
		0x1
	}
}

#[unsafe(naked)]
pub unsafe extern "sysv64" fn __do_syscall() {
	naked_asm!("swapgs",
				"xchg rsp, qword ptr gs:0x0",
				"push rax",
				"push rbx",
				"push rcx",
				"push rdx",
				"push rdi",
				"push rsi",
				"push rbp",
				"push r8",
				"push r9",
				"push r10",
				"push r11",
				"push r12",
				"push r13",
				"push r14",
				"push r15",
				"mov rcx, r12",
				"call do_syscall",
				"pop r15",
				"pop r14",
				"pop r13",
				"pop r12",
				"pop r11",
				"pop r10",
				"pop r9",
				"pop r8",
				"pop rbp",
				"pop rsi",
				"pop rdi",
				"pop rdx",
				"pop rcx",
				"pop rbx",
				//"pop rax",
				"xchg rsp, qword ptr gs:0x0",
				"swapgs",
				"sysretq"
	)
}
