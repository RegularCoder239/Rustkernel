use crate::{
	hw::cpu,
	std,
	std::log
};
use super::{
	current_process
};

#[derive(Debug, enum_iterator::Sequence)]
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
	let process = current_process().expect("Fatal exception in boot task.").lock();
	log::error!("Crash! PID:        {}", process.pid);
	log::error!("       RIP:        0x{:x}", frame.rip);
	log::error!("       RFLAGS:     0x{:x}", frame.rflags);
	log::error!("       CR2:        0x{:x}", std::cr2());
	log::error!("       Error       {:?}",
		enum_iterator::all::<Error>().nth(vector as usize).expect("Unknown error.")
	);
	log::error!("       Error Code: 0x{:x}", error);
	log::error!("       Flags:      {:?}", process.r#type);
	std::reset_cr2();
	std::exit();
}

pub fn setup_exception_handlers() {
	cpu::connect_exception(handle_exception);
}
