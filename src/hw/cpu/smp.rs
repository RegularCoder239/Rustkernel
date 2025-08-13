use crate::mm::{
	Mapped
};
use crate::kernel::scheduler::{
	Process,
};
use crate::{
	std,
	mm
};

#[link(name="smp")]
unsafe extern "sysv64" {
	fn smp_init();
}

pub fn load_smp_code() {
	let init_meth_ptr = smp_init as *const u8;
	unsafe {
		core::ptr::copy_nonoverlapping(init_meth_ptr, 0x8000_u64.mapped_global(0x1000).unwrap(), 0x3ff);
	}
}

#[unsafe(no_mangle)]
pub fn smp_core_meth() -> ! {
	std::log::info!("Booting non-boot cpu");

	mm::per_core_setup();

	Process::spawn_init_process(smp_core_setup as fn() -> !)
}

fn smp_core_setup() -> ! {
	crate::kernel::per_core_setup();

	loop {
		std::r#yield();
	}
}
