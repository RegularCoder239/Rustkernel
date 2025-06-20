use crate::{
	asm,
	allocate
};
use crate::std::{
	PerCpu
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
#[repr(C, packed)]
pub struct TSS {
	reserved_1: u32,
	ring_rsps: [u64; 3],
	reserved_2: u64,
	ist_rsps: [u64; 7],
	reserved_3: u64,
	iopb: u32
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

static GDTS: PerCpu<GDT> = PerCpu::new(GDT::DEFAULT);

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

	fn new_long_tss_low(tss: &TSS) -> (GlobalDescriptor, GlobalDescriptor) {
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
	pub const DEFAULT: GDT = GDT {
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
	};

	fn setup_tss(&mut self) {
		(self.gdt[5], self.gdt[6]) = GlobalDescriptor::new_long_tss_low(&self.tss);
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
				 "mov gs, {0:x}",
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

	fn set_ist_stack(&mut self, idx: usize, ptr: *mut u8, stack_size: u64) {
		self.tss.ist_rsps[idx] = ptr as u64 + stack_size;
	}
}

impl TSS {
	const EMPTY: TSS = TSS {
		reserved_1: 0x0,
		ring_rsps: [0x0; 3],
		reserved_2: 0x0,
		ist_rsps: [0x0; 7],
		reserved_3: 0x0,
		iopb: 0x68000000
	};
}

pub fn per_core_setup() {
	let gdt = GDTS.deref_mut();
	gdt.exception_stack = allocate!(ptr, u8, 0x5000).expect("Failed to allocate exception stack.");
	gdt.set_ist_stack(0x0, gdt.exception_stack, 0x5000);
	gdt.load();
	gdt.setup_tss();
}
