mod graphic;
mod file;
mod mem;

use crate::hw::cpu::syscall::Function;
use crate::print;
use core::fmt::Write;
use crate::std::{
	String,
	elf::load_elf_from_file,
	FilePath
};

const SYSCALL_METHODS: [Function; 5] = [
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
		meth: |args| if let Some(decoded_str) = syscallarg_to_string(args[0], args[1]) {
			print!("{}", decoded_str);
			0x0
		} else {
			0x2
		}
	},
	Function {
		id: 0xed24224fa1bde4,
		meth: |args| if let Some(decoded_str) = syscallarg_to_string(args[1], args[2]) {
			load_elf_from_file(args[0] as usize, FilePath::Unix(decoded_str.into())) as u64
		} else {
			0x0
		}
	}
];

pub fn setup() {
	for meth in SYSCALL_METHODS {
		meth.add();
	}
	graphic::setup();
	file::setup();
	mem::setup();
}

pub fn syscallarg_to_string(ptr: u64, size: u64) -> Option<&'static str> {
	str::from_utf8(
		unsafe {
			core::slice::from_raw_parts(
				ptr as *mut u8,
				size as usize
			)
		}
	).ok()
}
