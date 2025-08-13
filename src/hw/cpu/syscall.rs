use crate::std::{
	Mutex,
	Vec,
	wrmsr
};
use core::arch::{
	naked_asm
};

type SyscallMeth = fn(&[u64]) -> u64;

pub struct Function {
	pub id: u64,
	pub meth: SyscallMeth
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
	let lock = FUNCIONALITIES.lock();
	if let Some(f) = lock.into_iter().find(|a| a.id == function) {
		let addr = crate::mm::kernel_offset() + f.meth as *const () as u64;
		let fmeth: SyscallMeth = unsafe {
			core::mem::transmute(addr)
		};
		(fmeth)(&[arg1, arg2, arg3, arg4, arg5, arg6])
	} else {
		crate::std::log::error!("Invalid syscall opcode: {:x}", function);
		0x1
	}
}

#[unsafe(naked)]
pub extern "sysv64" fn __do_syscall() {
	naked_asm!("swapgs",
				"xchg rsp, qword ptr gs:0x0",
				"push rbx",
				"push rcx",
				"push rbp",
				"push r10",
				"push r11",
				"push r12",
				"push r13",
				"push r15",
				"mov rcx, r14",
				"call do_syscall",
				"pop r15",
				"pop r13",
				"pop r12",
				"pop r11",
				"pop r10",
				"pop rbp",
				"pop rcx",
				"pop rbx",
				"xchg rsp, qword ptr gs:0x0",
				"swapgs",
				"sysretq"
	)
}
