use crate::hw::cpu::syscall::Function;
use crate::kernel::{
	ProcessFlags,
	graphicmanager::RGBColor
};
use crate::std::{
	Mutex
};

static GRAPHIC_MANAGER_SET: Mutex<bool> = Mutex::new(true);

const GRAPHIC_SYSCALL_METHODS: [Function; 3] = [
	Function {
		id: 0x9102f2a1f5e356fb,
		meth: |_| {
			crate::kernel::current_process()
				.expect("Attempt to set graphic manager to a early boot task.")
				.assign_flags(ProcessFlags::GraphicManager);
			*GRAPHIC_MANAGER_SET.lock() = true;
			0x0
		}
	},
	Function {
		id: 0x916b0c0ca3fee2e,
		meth: |args| {
			let layer_id = crate::kernel::graphicmanager::Layer::add(2).id;
			crate::std::log::info!("Generation: {:x}", layer_id);
			layer_id
		}
	},
	Function {
		id: 0x9f986184b9ba2dda,
		meth: |args| {
			if let Some(layer) = crate::kernel::graphicmanager::Layer::by_id(args[0]) {
				layer.lock().draw_rect(/*(args[1] as usize, args[2] as usize),
								(args[3] as usize, args[4] as usize), RGBColor::from_u32(args[5] as u32)*/
								(100, 100), (300, 300), RGBColor::from_u32(0x121212));
				0x0
			} else {
				crate::std::log::info!("RIP: {:x}", args[0]);
				0x1
			}
		}
	}
];

pub fn setup() {
	for meth in GRAPHIC_SYSCALL_METHODS {
		meth.add();
	}
}
