use crate::{
	hw::cpu,
	std
};
use super::{
	current_process,
	current_task_state
};

#[derive(Debug)]
enum Error {
	DivisionByZero,
	Debug,
	NMI,
	Breakpoint,
	Overflow,
	BoundRangeExceeded,
	InvalidOpcode,
	DeviceNotAvailable,
	DoubleFault,
	Unknown1,
	InvalidTSS,
	SegmentNotPresent,
	StackSegmentFault,
	GeneralProtectionFault,
	PageFault,
	Reserved,
	FloatingPointException,
	AlignmentCheck,
	MachineCheck,
	SIMDFloatingPointException,
	VirtualizationException,
	ControlProtectionException,
	Reserved2,
	HypervisorInjectionException,
	VMMCommunicationException,
	SecurityException
}

pub fn handle_exception(vector: u8, frame: cpu::InterruptFrame, error: u64) {
	log::info!("{}", 82783);
	let current_state = current_task_state();
	let process = current_process().expect("Fatal exception in boot task.");
	log::error!("Crash! PID:        {}", process.pid);
	log::error!("       RIP:        0x{:x}", frame.rip);
	log::error!("       RFLAGS:     0x{:x}", frame.rflags);
	log::error!("       CR2:        0x{:x}", std::cr2());
	log::error!("       Error       {:?}", unsafe { core::mem::transmute::<_, Error>(vector) });
	log::error!("       Error Code: 0x{:x}", error);
	log::error!("       Flags:      {:?}", process.r#type);
	std::reset_cr2();
	std::exit();
}

pub fn setup_exception_handlers() {
	for vec in 0..31 {
		cpu::connect_exception(handle_exception);
	}
}
