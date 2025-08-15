use crate::hw::cpu::syscall::Function;
use crate::std::{
	CustomRAMAllocator,
	BasicAllocator,
	PageTableMapper,
	Box,
	Allocator,
	LazyMutex
};
use crate::mm::MappingFlags;

const USER_ALLOCATOR: LazyMutex<CustomRAMAllocator<PageTableMapper>> =
	LazyMutex::new(|| BasicAllocator::new(PageTableMapper::new(MappingFlags::User)));

const MEM_SYSCALL_METHODS: [Function; 2] = [
	/*
	 * Allocates X bytes user memory, where X is the first argument.
	 * Returns the virtual address of the new memory
	 */
	Function {
		id: 0x5f574e0f2ba82e47,
		meth: |args| {
			USER_ALLOCATOR.lock().allocate::<u8>(args[0] as usize).unwrap() as u64
		}
	},
	/*
	 * Frees user memory. The first argument specifies the virtual
	 * address and the second one the size of the allocation.
	 */
	Function {
		id: 0x182e8b4510a7eb40,
		meth: |args| {
			USER_ALLOCATOR.lock().free(args[0] as *const u8, args[1] as usize);
			0x0
		}
	}
];

pub fn setup() {
	for meth in MEM_SYSCALL_METHODS {
		meth.add();
	}
}
