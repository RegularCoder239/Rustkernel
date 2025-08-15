/*
 * TODO: Rewrite. Processes aren´t designed for this purpose.
 */
use super::scheduler::{
	Process,
	ProcessPrivilage
};

use crate::hw;

pub struct BootTask(Process);

type BootTaskMeth = fn() -> !;

const BOOT_PROCESSES: [BootTaskMeth; 6] = [
	super::graphicmanager::setup_console_task,
	hw::acpi::setup,
	hw::pci::scan,
	hw::pci::setup,
	hw::traits::disk::setup_disks,

	super::spawn_init
];

impl BootTask {
	/*
	 * Spawn boot task process.
	 */
	pub fn spawn(meth: BootTaskMeth) {
		Process::spawn_with_stack(
			ProcessPrivilage::KERNEL,
			meth
		).expect("Failed to create critical boot task.");
	}

	/*
	 * Spawn all preconfiguted boot takss in BOOT_PROCESSES
	 */
	pub fn add_boot_tasks() {
		for taskidx in 0..BOOT_PROCESSES.len() {
			BootTask::spawn(BOOT_PROCESSES[taskidx]);
		}
	}
}
