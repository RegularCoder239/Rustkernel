use cpu::interrupt;

struct HardwareStruct {
	IDT idt
};

impl HardwareStruct {
	fn new() -> HardwareStruct {
		HardwareStruct {
			idt: IDT::new()
		}
	}
}
