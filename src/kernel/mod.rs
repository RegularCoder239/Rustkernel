pub mod scheduler;
mod boottask;
mod exception;
mod syscall;

pub use scheduler::{
	r#yield,
	exit_current_process,
	current_process,
	current_task_state,
	Process,
	ProcessPrivilage
};

pub use exception::{
	setup_exception_handlers
};

pub fn boot_core_setup() {
	per_core_setup();
	boottask::BootTask::add_boot_tasks();
	syscall::setup();
}

pub fn per_core_setup() {
	scheduler::init_yield_timer();
	setup_exception_handlers();
}
