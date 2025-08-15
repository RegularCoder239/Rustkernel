use crate::mm::{
	Mapped
};
use crate::kernel::scheduler::{
	Process,
};
use crate::{
	std,
	mm,
	hw::cpu
};

#[link(name="smp")]
unsafe extern "sysv64" {
	fn smp_init();
}

/*
 * Loads the smp boot code to physical address 0x8000, because
 * the x86_64 core starts in real mode.
 */
pub fn load_smp_code() {
	let init_meth_ptr = smp_init as *const u8;
	unsafe {
		core::ptr::copy_nonoverlapping(init_meth_ptr, 0x8000_u64.mapped_global(0x1000).unwrap(), 0x3ff);
	}
}

/*
 * Called after 64 bit long mode jump of the nonboot core.
 * Just sets up the memory manager, core and spawns a init process.
 */
#[unsafe(no_mangle)]
pub fn smp_core_meth() -> ! {
	std::log::info!("Booting non-boot cpu");

	mm::per_core_setup();
	cpu::setup_core();

	Process::spawn_init_process(smp_core_setup as fn() -> !)
}

fn smp_core_setup() -> ! {
	crate::kernel::per_core_setup();

	loop {
		std::r#yield();
	}
}
