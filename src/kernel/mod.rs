pub mod boottask;
pub mod scheduler;

pub use scheduler::{
	r#yield,
	exit_current_process
};

pub fn boot_core_setup() {
	per_core_setup();
	boottask::BootTask::add_boot_tasks();
}

pub fn per_core_setup() {
	scheduler::init_yield_timer();
}
