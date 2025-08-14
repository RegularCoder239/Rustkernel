use core::ops::{
	Deref,
	DerefMut
};
use crate::std::{
	LazyMutex,
	Box,
	rdmsr
};
use crate::{
	std,
	lapic,
};

struct LAPICRegister {
	content: u32,
	padding: [u32; 3]
}

/*
 * Memory mapped LAPIC.
 */
#[repr(C, align(0x10))]
pub struct LAPIC {
	reserved: [LAPICRegister; 2],
	id: LAPICRegister,
	version: LAPICRegister,

	reserved_2: [LAPICRegister; 4],
	task_priority: LAPICRegister,
	arbitartion_priority: LAPICRegister,
	process_priority: LAPICRegister,
	eoi: LAPICRegister,
	remote_read: LAPICRegister,
	logical_destination: LAPICRegister,
	destination_format: LAPICRegister,
	spurious_interrupt_vector: LAPICRegister,
	isr_registers: [LAPICRegister; 8],
	trigger_mode_registers: [LAPICRegister; 8],
	irq_registers: [LAPICRegister; 8],
	error_status: LAPICRegister,
	reserved_3: [LAPICRegister; 7],
	command_register_1: LAPICRegister,
	command_register_2: LAPICRegister
}

/*
 * Memory mapped IOAPIC.
 */
pub struct IOAPIC {
	register_address: u32,
	reserved: [u32; 3],
	register_content: u32
}

pub static LAPICS: LazyMutex<Box<LAPIC>> = LazyMutex::new(
	|| Box::from_raw_address(rdmsr(0x1b) & !(0xfff))
);
pub static IOAPIC: LazyMutex<Box<IOAPIC>> = LazyMutex::new(
	|| Box::from_raw_address(0xfec00000)
);

impl LAPIC {
	/*
	 * Will reset every cpu except the current one.
	 * This causes a reboot if this method is called on a
	 * nonboot cpu.
	 */
	pub fn init_non_boot_cpus(startup_addr: u32) {
		let mut l = LAPICS.lock();
		l.send_command(0xc4500, 0x0);
		l.send_command(0xc0600 + (startup_addr / 0x1000), 0x0);
	}
	/*
	 * Halts every cpu core except the current one.
	 */
	pub fn poweroff_other_cpus(&mut self) {
		self.send_command(0xc8500, 0x0);
	}
	pub fn end_of_interrupt() {
		*lapic!().eoi = 0x0;
	}
	pub fn id(&self) -> u32 {
		*self.id
	}
	pub fn enable_hardware_interrupts() {
		std::outb(0xff, 0x21);
		std::outb(0xff, 0xa1);

		let mut lapiclock = LAPICS.lock();
		*lapiclock.destination_format = 0xf0000000;
		*lapiclock.logical_destination = 0xff << 24;
		*lapiclock.task_priority = 0x0;

		*lapiclock.spurious_interrupt_vector = 0x11ff;
	}
	fn send_command(&mut self, command: u32, target: u32) {
		*self.command_register_2 = target;
		*self.command_register_1 = command;
	}
}

impl IOAPIC {
	/*
	 * Rewrites the IOAPIC for recieving hardware interrupts.
	 */
	pub fn activate() {
		let mut lock = IOAPIC.lock();
		for idx in 0..0x10 {
			lock.write(idx * 2 + 0x10, (idx as u32 + 0x30) | 0x800);
			lock.write(idx * 2 + 0x11, 0xff000000);
		}
	}
	pub fn write(&mut self, addr: u8, content: u32) {
		self.register_address = addr as u32;
		self.register_content = content;
	}
	pub fn read(&mut self, addr: u8) -> u32 {
		self.register_address = addr as u32;
		self.register_content
	}
}

impl Deref for LAPICRegister {
	type Target = u32;

	fn deref(&self) -> &u32 {
		&self.content
	}
}

impl DerefMut for LAPICRegister {
	fn deref_mut(&mut self) -> &mut u32 {
		&mut self.content
	}
}

#[macro_export]
macro_rules! lapic {
	() => {
		lapic!("lazybox").lock()
	};
	// Contains the Lazymutex of the LAPIC. Used in panic handler for checking.
	("lazybox") => {
		crate::hw::cpu::lapic::LAPICS
	}
}
