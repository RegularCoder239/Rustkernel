use core::arch::asm;

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct InterruptDescriptor {
	offset1: u16,
	segment_selector: u16,
	reserved_1: u8,
	flags: u8,
	offset2: u16,
	offset3: u32,
	reserved_2: u32
}

#[repr(C, packed)]
pub struct IDT {
	descriptors: [InterruptDescriptor; 256],
}

#[repr(C, packed)]
struct IDTR {
	limit: u16,
	base: u64
}

type Handler = unsafe fn();

impl InterruptDescriptor {
	const fn new(addr: u64) -> InterruptDescriptor {
		InterruptDescriptor {
			offset1:			addr as u16,
			segment_selector:	0x8, // TODO: Dynamic segment selector in IDT
			reserved_1:			0x0,
			flags:				0x8e,
			offset2:			(addr >> 16) as u16,
			offset3:			(addr >> 32) as u32,
			reserved_2:			0x0
		}
	}
	fn fromCMethod(addr: unsafe extern "C" fn()) -> InterruptDescriptor {
		InterruptDescriptor::new(addr as u64)
	}
	fn fromMethod(addr: Handler) -> InterruptDescriptor {
		InterruptDescriptor::new(addr as *const i32 as u64)
	}
}

unsafe fn handleNopInterrupt() {

	asm!("pop rax; iretq");
}

unsafe fn handleErrorInterrupt() {
	panic!("Fatal exception");
}

static mut global_idt: IDT = IDT {
	descriptors: [InterruptDescriptor::new(0x100000); 0x100]
};

impl IDT {
	pub fn loadGlobalIDT() {
		unsafe {
			global_idt.setupErrorHandlers();
			global_idt.load();
		}
	}
	unsafe fn load(&self) {
		let idtr = IDTR {
			limit: 0x1000,
			base: self as *const IDT as u64
		};
		asm!("lidt [{}]", in(reg) &idtr);
	}
	fn setupErrorHandlers(&mut self) {
		for idx in 0..31 {
			self.connect(idx, handleErrorInterrupt);
		}
	}
	fn connect(&mut self, idx: usize, method: Handler) {
		self.descriptors[idx] = InterruptDescriptor::fromMethod(method);
	}
}
