use crate::hw::cpu::syscall::Function;
use crate::print;
use core::fmt::Write;

const SYSCALL_METHODS: [Function; 4] = [
	Function {
		id: 0xa4998996a6277317,
		meth: |_| crate::std::exit()
	},
	Function {
		id: 0x4a33e7eb45595ceb,
		meth: |_| crate::hw::power::shutdown()
	},
	Function {
		id: 0xba3f7ec4fdf5556b,
		meth: |_| crate::hw::power::reboot()
	},
	Function {
		id: 0x588f73f96a7de691,
		meth: |args| if let Ok(decoded_str) = str::from_utf8(
				unsafe {
					core::slice::from_raw_parts(
						args[0] as *mut u8,
						args[1] as usize
					)
				}
			) {
			let _ = print!("{}", decoded_str);
		}
	}
];

pub fn setup() {
	for meth in SYSCALL_METHODS {
		meth.add();
	}
}
