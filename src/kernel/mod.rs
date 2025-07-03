pub mod boottask;
pub mod scheduler;
mod exception;

pub use scheduler::{
	r#yield,
	exit_current_process,
	current_process,
	current_task_state
};

pub use exception::{
	setup_exception_handlers
};

pub fn boot_core_setup() {
	per_core_setup();
	boottask::BootTask::add_boot_tasks();
}

pub fn per_core_setup() {

	scheduler::init_yield_timer();
	setup_exception_handlers();
}
