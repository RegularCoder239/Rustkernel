use crate::asm;

pub fn outb(val: u8, port: u16) {
	unsafe {
		asm!("outb %al, %dx", in("al") val, in("dx") port, options(att_syntax));
	}
}

pub fn outw(val: u16, port: u16) {
	unsafe {
		asm!("outw %ax, %dx", in("ax") val, in("dx") port, options(att_syntax));
	}
}
pub fn outd(val: u32, port: u16) {
	unsafe {
		asm!("outd %eax, %dx", in("eax") val, in("dx") port, options(att_syntax));
	}
}
pub fn outl(val: u64, port: u16) {
	unsafe {
		asm!("outl %rax, %dx", in("rax") val, in("dx") port, options(att_syntax));
	}
}
