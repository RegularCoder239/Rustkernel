use crate::std::{
	Mutex,
	Vec,
	PerCpuLazy,
	Box,
	wrmsr
};
use core::arch::{
	naked_asm,
	asm
};

pub struct Function {
	pub id: u64,
	pub meth: fn()
}

static FUNCIONALITIES: Mutex<Vec<Function>> = Mutex::new(Vec::new());
//static SYSCALL_STACKS: PerCpuLazy<[u8; 0x2000]> = PerCpuLazy::new([0; 0x2000]);

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
extern "C" fn do_syscall(function: u64) {
	let lock = FUNCIONALITIES.lock();
	if let Some(f) = lock.into_iter().find(|a| a.id == function) {
		(f.meth)();
	} else {
		crate::std::log::error!("Invalid syscall opcode: {:x}", function);
	}
}

#[unsafe(naked)]
pub unsafe extern "sysv64" fn __do_syscall() {
	naked_asm!("swapgs",
			   "xchg rsp, qword ptr gs:0x0",
			   "push rcx",
			   "push r11",
			   "mov rcx, rax",
			   "call do_syscall",
			   "pop r11",
			   "pop rcx",
			   "xchg rsp, qword ptr gs:0x0",
			   "swapgs",
			   "sysretq"
	)
}
