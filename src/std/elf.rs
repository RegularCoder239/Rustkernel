use crate::kernel::{
	ProcessPrivilage,
	Process
};
use crate::std::{
	Box,
	self,
	BasicAllocator,
	PhysicalRAMAllocator,
	PageTableMapper,
	Allocator,
	Mutex
};
use crate::mm::{
	MappingInfo,
	MappingFlags
};
use elf_rs::{
	ElfFile,
	ProgramType
};

pub fn load_elf_from_file(fs_id: usize, file_path: std::FilePath) -> bool {
	if let Ok(data) = std::read_file(fs_id, file_path, usize::MAX, 0) {
		load_elf(data.as_slice())
	} else {
		false
	}
}

pub fn load_elf(data: &[u8]) -> bool {
	if let Ok(elffile) = elf_rs::Elf::from_bytes(data) {
		let mut process = Process::new(ProcessPrivilage::USER, elffile.entry_point()).unwrap();

		for entry in elffile.program_header_iter() {
			if entry.ph_type() != ProgramType::LOAD {
				continue;
			}
			// let mut content = Box::<[u8]>::new_sized(entry.memsz() as usize + 0x3000 - (entry.memsz() % 0x1000) as usize);
			// let padding = entry.vaddr() as usize & 0xfff;
			// for idx in 0.. as usize {
			// 	content[idx + padding] = data[idx + entry.offset() as usize];
			// }
			let mut flags = 0x4;
			if !entry.flags().contains(elf_rs::ProgramHeaderFlags::EXECUTE) {
				flags |= 0x8000000000000000;
			}

			process.add_unaligned_mapping(
				entry.vaddr(),
				data,
				entry.offset() as usize,
				entry.filesz() as usize,
				flags
			);
		}
		process.assign_stack(BasicAllocator::<PageTableMapper, PhysicalRAMAllocator>::new(
			PageTableMapper(
				MappingInfo {
					page_table: &process.page_table,
					address: 0,
					flags: MappingFlags::User
				}
			)
		).allocate::<u8>(0x1000).unwrap() as u64 + 0x1000);
		process.spawn();
		true
	} else {
		false
	}
}
