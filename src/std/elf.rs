use crate::kernel::{
	ProcessPrivilage,
	Process
};
use crate::std::Box;
use elf_rs::ElfFile;

pub fn load_elf(data: &[u8]) -> bool {
	if let Ok(elffile) = elf_rs::Elf::from_bytes(data) {
		let mut process = Process::new(ProcessPrivilage::USER, elffile.entry_point()).unwrap();

		for entry in elffile.program_header_iter() {
			let mut content = Box::<[u8]>::new_sized(entry.memsz() as usize + 0x1000 - (entry.memsz() % 0x1000) as usize);
			for idx in 0..entry.filesz() as usize {
				content[idx] = data[idx + entry.offset() as usize];
			}
			let mut flags = 0x4;
			if !entry.flags().contains(elf_rs::ProgramHeaderFlags::EXECUTE) {
				flags |= 0x8000000000000000;
			}
			process.page_table.map(
				entry.vaddr(), content.physical_address(),
				entry.memsz() as usize, flags
			);
		}
		process.spawn();
		true
	} else {
		false
	}
}
