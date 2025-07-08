use uefi::boot::ScopedProtocol;
use uefi::proto::console::gop::{
	GraphicsOutput,
	ModeInfo
};
use uefi::boot;

pub struct GOP {
	protocol: ScopedProtocol<GraphicsOutput>
}

impl GOP {
	pub fn new() -> Option<GOP> {
		let mut gop = GOP {
			protocol: boot::open_protocol_exclusive::<GraphicsOutput>(
				boot::get_handle_for_protocol::<GraphicsOutput>().ok()?
			).ok()?
		};
		let mode = gop.protocol.modes().max_by(|mode, mode2| {
			pixel_amount(mode.info()).cmp(&pixel_amount(mode2.info()))
		})?;
		gop.protocol.set_mode(&mode).ok()?;
		Some(gop)
	}
	pub fn frame_buffer(&mut self) -> *mut u32 {
		self.protocol.frame_buffer().as_mut_ptr() as *mut u32
	}
	pub fn resolution(&self) -> (usize, usize) {
		self.protocol.current_mode_info().resolution()
	}
}

fn pixel_amount(mode: &ModeInfo) -> usize {
	let (resx, resy) = mode.resolution();
	if resx as f64 / resy as f64 == 16.0 / 9.0 {
		resx * resy * 4
	} else {
		resx * resy
	}
}
