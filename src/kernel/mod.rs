pub mod scheduler;
pub mod graphicmanager;
mod boottask;
mod exception;
mod syscall;

use crate::std::{
	Lock,
	self,
	log,
	VecBase
};

static INITALIZATION_LOCK: Lock = Lock::new_locked();

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
	INITALIZATION_LOCK.unlock();
}

pub fn per_core_setup() {
	scheduler::init_yield_timer();
	setup_exception_handlers();
}

pub fn is_initalized() -> bool {
	!INITALIZATION_LOCK.is_locked()
}

pub fn spawn_init() -> ! {
	crate::std::log::info!("Spawning init process ...");
	let mut success = false;
	let disk_ids = crate::hw::disk_ids();
	assert!(!disk_ids.empty(), "No filesystems found to search init executable.");
	for disk_id in &disk_ids {
		let mounted = std::mount(*disk_id);
		crate::std::log::info!("Mounted disk successfully: {}", disk_id);
		if std::elf::load_elf_from_file(mounted, std::FilePath::new_unix("/init")) {
			crate::std::log::info!("Found init executable at disk: {}", disk_id);
			success = true;
		}
	}
	if !success {
		panic!("No init executable found.");
	} else {
		log::info!("Init processes started successfully.");
	}

	loop {}
}
