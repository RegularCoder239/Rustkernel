mod thread;
mod utils;
mod alloc;
mod io;
mod container;
mod vec;
mod reversebytes;

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
	unsaferef::UnsafeRef
};

pub use thread::{
	mutex::Mutex,
	mutex::OptMutexGuard,
	lock::Lock,
	lazymutex::LazyMutex,
	percpu::PerCpuLazy,
	percpu::PerCpu
};
pub use alloc::{
	Allocator,
/*	VirtualMapper,
	PhysicalAllocator,
	PhysicalRAMAllocator,
*/	Allocation,
	RAMAllocator
};
pub use utils::{
	With,
	hltloop,
	cli,
	sti
};
pub use reversebytes::ReverseBytes;
pub use io::{
	outb
};
pub use crate::kernel::r#yield;

pub fn exit() -> ! {
	crate::kernel::exit_current_process()
}
