use super::scheduler::{
	Process,
	ProcessPrivilage
};

use crate::hw;

pub struct BootTask(Process);

type BootTaskMeth = fn() -> !;

const BOOT_PROCESSES: [BootTaskMeth; 4] = [
	hw::graphics::setup,
	hw::acpi::setup,
	hw::pci::scan,
	hw::pci::setup
];

impl BootTask {
	pub fn spawn(meth: BootTaskMeth) {
		Process::spawn_with_stack(
			ProcessPrivilage::KERNEL,
			meth
		)
			.expect("Failed to create critical boot task.");
	}

	pub fn add_boot_tasks() {
		for taskidx in 0..BOOT_PROCESSES.len() {
			BootTask::spawn(BOOT_PROCESSES[taskidx]);
		}
	}
}
