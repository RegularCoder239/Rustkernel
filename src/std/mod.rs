mod thread;
mod utils;
mod alloc;
mod io;
mod container;
mod vec;
mod reversebytes;
mod log_intern;

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

pub use vec::{
	Vec,
	StackVec,
	VecBase
};
pub use container::{
	LazyBox,
	SharedRef,
	r#box::Box,
	string::String,
	unsaferef::UnsafeRef,
	mutableref::MutableRef,
	mutablecell::MutableCell
};

pub use thread::{
	mutex::Mutex,
	mutex::MutexGuard,
	mutex::OptMutexGuard,
	lock::Lock,
	lazymutex::LazyMutex,
	lazymutex::LazyMutexGuard,
	percpu::PerCpuLazy,
	percpu::PerCpu,
	count_cores,
	current_core
};
pub use alloc::{
	Allocator,
	RAMAllocator
};
pub use utils::{
	With,
	hltloop,
	cli,
	sti,
	cr2,
	reset_cr2,
	wrmsr,
	rdmsr
};
pub use reversebytes::ReverseBytes;
pub use io::{
	outb
};
pub use crate::kernel::r#yield;

pub fn exit() -> ! {
	crate::kernel::exit_current_process()
}
