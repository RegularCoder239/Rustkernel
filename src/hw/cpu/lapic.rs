use crate::mapped;
use core::ops::{
	Deref,
	DerefMut
};
use core::hint;

enum Command {
	Interrupt,
	Init,
	InitDeassert,
	Sipi
}

#[repr(C, align(0x10))]
struct LAPICRegister {
	content: u32
}

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

pub fn lapic() -> Option<&'static mut LAPIC> {
	Some(&mut *(mapped!(0xfee00000, LAPIC, 0x1000)?))
}

impl LAPIC {
	// DANGER: Will reset every cpu except the current one.
	pub fn init_non_boot_cpus(&mut self, startup_addr: u32) -> &mut LAPIC {
		//self.send_command(0xc500, 0x0);
		self.send_command(0xc0600 + startup_addr / 0x1000, 0x0);
		self
	}
	fn send_command(&mut self, command: u32, target: u32) {
		*self.command_register_2 = target;
		*self.command_register_1 = command;

		/*while *self.command_register_1 & 0x1000 != 0 {
			hint::spin_loop();
		}*/
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
