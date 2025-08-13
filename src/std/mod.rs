mod thread;
mod utils;
mod alloc;
mod io;
mod container;
mod vec;
mod reversebytes;
mod log_intern;
mod console;
mod file;
mod random;

pub mod elf;
pub mod log {
	pub use crate::{
		info,
		error,
		debug,
		warn
	};
	pub use super::log_intern::{
		log
	};
}
pub use console::Console;
pub use vec::{
	Vec,
	StackVec,
	VecBase
};
pub use container::{
	LazyBox,
	SharedRef,
	r#box::Box,
	string::String
};

pub use thread::{
	mutex::Mutex,
	mutex::OptMutexGuard,
	mutex::MutexGuard,
	lock::Lock,
	lazymutex::LazyMutex,
	lazymutex::LazyMutexGuard,
	percpu::PerCpuLazy,
	percpu::PerCpu,
	count_cores,
	current_core,
	current_core_uncached
};
pub use file::{
	mount,
	read_file,
	FilePath
};
pub use alloc::{
	Allocator,
	RAMAllocator,
	BasicAllocator,
	PhysicalRAMAllocator,
	PageTableMapper,
	CustomRAMAllocator
};
pub use utils::{
	hltloop,
	cli,
	sti,
	cr2,
	reset_cr2,
	wrmsr,
	rdmsr,
	wait
};
pub use reversebytes::ReverseBytes;
pub use io::{
	outb
};
pub use crate::kernel::r#yield;
pub use random::random;

pub fn exit() -> ! {
	crate::kernel::exit_current_process()
}
