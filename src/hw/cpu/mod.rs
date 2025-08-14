pub mod lapic;
pub mod smp;
pub mod interrupt;
pub mod gdt;
pub mod syscall;
pub mod gs;

pub use lapic::{
	LAPIC,
	IOAPIC
};
pub use gdt::GDT;
pub use interrupt::{
	connect_signal,
	connect_exception,
	InterruptFrame,
	TIMER
};

use crate::std::{
	log,
	self
};

/*
 * Sets up current cores GDT, IDT, LAPIC and
 * a MSR for syscalls activation.
 */
pub fn setup_core() {
	gdt::per_core_setup();

	interrupt::current_idt()
		.load();
	std::cli();

	LAPIC::enable_hardware_interrupts();
	syscall::setup();
}

/*
 * Sets up current core and
 * core-independent CPU features like
 * the IOAPIC.
 */
pub fn setup1() {
	log::info!("Setting up CPU.");

	setup_core();
	IOAPIC::activate();
}

/*
 * Boots up nonboot cores.
 * Never call this method twice or
 * from a nonboot core.
 */
pub fn awake_non_boot_cpus() {
	log::info!("Booting non-boot CPUS.");
	smp::load_smp_code();
	LAPIC::init_non_boot_cpus(0x8000);
}
