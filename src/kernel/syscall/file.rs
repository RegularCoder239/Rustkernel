use crate::hw::cpu::syscall::Function;
use crate::std::{
	read_file,
	CustomRAMAllocator,
	BasicAllocator,
	PageTableMapper,
	Box,
	Allocator
};
use crate::mm::MappingFlags;
use crate::virt::fs::FilePath;
use super::syscallarg_to_string;

const FILE_SYSCALL_METHODS: [Function; 1] = [
	Function {
		id: 0xd418874d2055fe49,
		meth: |args| {
			if let Some(rawstr) = syscallarg_to_string(args[1], args[2]) {
				let result = read_file(args[0] as usize, FilePath::DOS(rawstr.into()), args[3] as usize, args[4] as usize);

				if let Ok(r#box) = result {
					let userallocator: CustomRAMAllocator<PageTableMapper> = BasicAllocator::new(PageTableMapper::new(MappingFlags::User));
					core::mem::ManuallyDrop::new(
						Box::new_slice_with_alloc(r#box.as_slice(), userallocator)
					).virtual_address()
				} else if let Err(err) = result {
					err as u64
				} else {
					unreachable!()
				}
			} else {
				0xfff
			}
		}
	}
];

pub fn setup() {
	for meth in FILE_SYSCALL_METHODS {
		meth.add();
	}
}
