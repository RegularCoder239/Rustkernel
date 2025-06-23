use crate::mm::{
	Mapped
};
use crate::kernel::scheduler::{
	Process,
	ProcessPrivilage
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
	unsafe {
		let init_meth_ptr = smp_init as *const u8;
		core::ptr::copy_nonoverlapping(init_meth_ptr, 0x8000_u64.mapped_global(0x1000).unwrap(), 0x3ff);
	}
}

#[unsafe(no_mangle)]
pub fn smp_core_meth() -> ! {
	//todo!("SMP Rework needed.");
	log::info!("Booting non-boot cpu");

	mm::per_core_setup();

	Process::new_with_stack(ProcessPrivilage::KERNEL,
							smp_core_setup as fn() -> !)
		.expect("Failed to create idle task")
		.jump()
}

fn smp_core_setup() -> ! {
	crate::kernel::per_core_setup();
	std::sti();

	loop {
		std::r#yield();
	}
}
