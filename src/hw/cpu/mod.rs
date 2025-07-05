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

pub fn setup_core() {
	gdt::per_core_setup();

	interrupt::current_idt()
		.load();
	std::cli();

	LAPIC::enable_hardware_interrupts();
	syscall::setup();
}

pub fn setup1() {
	log::info!("Setting up CPU.");

	setup_core();
	IOAPIC::activate();
}

pub fn setup2() {
	log::info!("Setting up GS.");
	gs::init();
}

pub fn awake_non_boot_cpus() {
	log::info!("Booting non-boot CPUS.");
	smp::load_smp_code();
	LAPIC::init_non_boot_cpus(0x8000);
}
