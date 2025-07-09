use crate::{
	asm
};
use crate::std::{
	PerCpu,
	RAMAllocator,
	Allocator
};

#[derive(Clone, Copy)]
struct GlobalDescriptor {
	limit_low: u16,
	base_low: u16,
	base_mid: u8,
	access_byte: u8,
	flags: u8,
	base_high: u8
}

#[derive(Clone, Copy)]
#[repr(C, packed(4))]
pub struct TSS {
	reserved_1: u32,
	pub ring_rsps: [u64; 3],
	reserved_2: u64,
	ist_rsps: [u64; 7],
	reserved_3: u64,
	reserved_4: u16,
	iopb: u16
}

#[derive(Copy, Clone)]
pub struct GDT {
	gdt: [GlobalDescriptor; 0x10],
	tss: TSS,
	exception_stack: *mut u8
}

#[repr(C, packed)]
struct GDTR {
	limit: u16,
	base: u64
}

static GDTS: PerCpu<GDT> = PerCpu::new(GDT::new());

impl GlobalDescriptor {
	const EMPTY: GlobalDescriptor = GlobalDescriptor {
		limit_low: 0x0,
		base_low: 0x0,
		base_mid: 0x0,
		access_byte: 0x0,
		flags: 0x0,
		base_high: 0x0
	};
	const KERNEL_CODE_SEG: GlobalDescriptor = GlobalDescriptor {
		access_byte: 0x98,
		flags: 0x20,
		..Self::EMPTY
	};
	const KERNEL_DATA_SEG: GlobalDescriptor = GlobalDescriptor {
		access_byte: 0x92,
		flags: 0x20,
		..Self::EMPTY
	};
	const USER_CODE_SEG: GlobalDescriptor = GlobalDescriptor {
		access_byte: 0xf8,
		flags: 0x20,
		..Self::EMPTY
	};
	const USER_DATA_SEG: GlobalDescriptor = GlobalDescriptor {
		access_byte: 0xf2,
		flags: 0x20,
		..Self::EMPTY
	};

	fn new_long_tss(tss: &TSS) -> (GlobalDescriptor, GlobalDescriptor) {
		let raw_addr = tss as *const TSS as u64 + crate::mm::kernel_offset();
		(
			GlobalDescriptor {
				limit_low: 0x68,
				base_low: (raw_addr & 0xffff) as u16,
				base_mid: ((raw_addr >> 16) & 0xff) as u8,
				access_byte: 0x89,
				flags: 0x0,
				base_high: ((raw_addr >> 24) & 0xff) as u8
			},
			GlobalDescriptor {
				limit_low: ((raw_addr >> 32) & 0xffff) as u16,
				base_low: ((raw_addr >> 48) & 0xffff) as u16,
				base_mid: 0x0,
				access_byte: 0x0,
				flags: 0x0,
				base_high: 0x0
			}
		)
	}
}

impl GDT {
	pub const EMPTY_SEG: u16 = 0x0;
	pub const CODE_SEG: u16 = 0x8;
	pub const DATA_SEG: u16 = 0x10;
	pub const fn new() -> GDT {
		GDT {
			gdt: [
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::KERNEL_CODE_SEG,
				GlobalDescriptor::KERNEL_DATA_SEG,
				GlobalDescriptor::USER_CODE_SEG,
				GlobalDescriptor::USER_DATA_SEG,
				GlobalDescriptor::EMPTY, // TSS Low
				GlobalDescriptor::EMPTY, // TSS High
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
				GlobalDescriptor::EMPTY,
			],
			exception_stack: core::ptr::null_mut(),
			tss: TSS::EMPTY
		}
	}

	fn setup_tss(&mut self) {
		(self.gdt[5], self.gdt[6]) = GlobalDescriptor::new_long_tss(&self.tss);
		self.exception_stack = RAMAllocator::DEFAULT.allocate(0x5000).unwrap();
		let value = self.exception_stack as u64 + 0x5000;
		self.tss.ist_rsps[0] = value;
		self.tss.ring_rsps[0] = value;
		unsafe {
			asm!("ltr ax", in("ax") 5 * 0x8);
		}
	}

	fn load(&self) {
		let gdtr = GDTR {
			limit: 0x80,
			base: self.gdt.as_ptr() as u64 + crate::mm::kernel_offset()
		};
		unsafe {
			asm!("lgdt [{1}]",
				 "mov ds, {0:x}",
				 "mov ss, {3:x}",
				 "mov es, {0:x}",
				 "mov fs, {0:x}",
				 "mov gs, {3:x}",
				 "push 0x8",
				 "lea rax, {2}",
				 "push rax",
				 "retfq",
				 in(reg) Self::EMPTY_SEG,
				 in(reg) &gdtr,
				 label {},
				 in(reg) Self::DATA_SEG,
			);
		}
	}
}

impl TSS {
	const EMPTY: TSS = TSS {
		reserved_1: 0x0,
		ring_rsps: [0x0; 3],
		reserved_2: 0x0,
		ist_rsps: [0x0; 7],
		reserved_3: 0x0,
		reserved_4: 0x0,
		iopb: core::mem::size_of::<TSS>() as u16
	};
}

pub fn per_core_setup() {
	let gdt = GDTS.deref_mut();
	gdt.load();
	gdt.setup_tss();
}
