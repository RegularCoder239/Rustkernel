use crate::hw::cpu::syscall::Function;
use crate::kernel::{
	ProcessFlags
};
use crate::std::{
	Mutex
};

static GRAPHIC_MANAGER_SET: Mutex<bool> = Mutex::new(true);

const GRAPHIC_SYSCALL_METHODS: [Function; 1] = [
	Function {
		id: 0x9102f2a1f5e356fb,
		meth: |_| {
			crate::kernel::current_process()
				.expect("Attempt to set graphic manager to a early boot task.")
				.assign_flags(ProcessFlags::GraphicManager);
			*GRAPHIC_MANAGER_SET.lock() = true;
		}
	}
];

pub fn setup() {
	for meth in GRAPHIC_SYSCALL_METHODS {
		meth.add();
	}
}
