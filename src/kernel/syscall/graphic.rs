use crate::hw::cpu::syscall::Function;
use crate::kernel::{
	ProcessFlags,
	graphicmanager::RGBColor,
	current_process
};
use crate::std::{
	Mutex
};

static GRAPHIC_MANAGER_SET: Mutex<bool> = Mutex::new(true);

const GRAPHIC_SYSCALL_METHODS: [Function; 3] = [
	/*
	 * Claims the graphicmanager flag for the current process.
	 * This method will fail, when the flag is already given
	 * away.
	 */
	Function {
		id: 0x9102f2a1f5e356fb,
		meth: |_| {
			crate::kernel::current_process()
				.expect("Attempt to set graphic manager to a early init task.")
				.assign_flags(ProcessFlags::GraphicManager);
			*GRAPHIC_MANAGER_SET.lock() = true;
			0x0
		}
	},
	/*
	 * Creates new layer with z coordinate from the first argument.
	 * Returns the layer id, which is used in other graphic syscalls.
	 * This method will fail, when the process hasn´t claimed the graphicmanager
	 * flag.
	 */
	Function {
		id: 0x916b0c0ca3fee2e,
		meth: |args| {
			if current_process().expect("Attempt to do syscalls in early init task.").has_flag(ProcessFlags::GraphicManager) {
				crate::kernel::graphicmanager::Layer::add(args[0] as u8).id
			} else {
				u64::MAX
			}
		}
	},
	/*
	 * Draw a rectangle with the position from argument two and three,
	 * the size from argument four and five, the color from argument
	 * six in layer with the id from the first argument.
	 */
	Function {
		id: 0x9f986184b9ba2dda,
		meth: |args| {
			if let Some(layer) = crate::kernel::graphicmanager::Layer::by_id(args[0]) {
				layer.lock().draw_rect(
					(args[1] as usize, args[2] as usize),
					(args[3] as usize, args[4] as usize),
					RGBColor::from_u32(args[5] as u32)
				);
				0x0
			} else {
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
