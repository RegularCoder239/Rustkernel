fn load_elf(data: &[u8]) -> bool {
	let elffile = elf_rs::Elf::from_bytes(data);
	for elffile.program_header() {

	}
}
