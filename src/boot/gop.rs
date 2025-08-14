use uefi::boot::ScopedProtocol;
use uefi::proto::console::gop::{
	GraphicsOutput,
	ModeInfo,
	Mode
};
use uefi::boot;

/*
 * Wrapper for the GOP of UEFI.
 * Used to gather framebuffer information.
 */
pub struct GOP {
	protocol: ScopedProtocol<GraphicsOutput>,
	mode: Mode
}

impl GOP {
	pub fn new() -> Option<GOP> {
		let protocol = boot::open_protocol_exclusive::<GraphicsOutput>(
			boot::get_handle_for_protocol::<GraphicsOutput>().ok()?
		).ok()?;
		let mut gop = GOP {
			mode: protocol.modes().max_by(|mode, mode2| {
				pixel_amount(mode.info()).cmp(&pixel_amount(mode2.info()))
			})?,
			protocol,
		};
		gop.protocol.set_mode(&gop.mode).ok()?;
		Some(gop)
	}
	pub fn frame_buffer(&mut self) -> *mut u32 {
		self.protocol.frame_buffer().as_mut_ptr() as *mut u32
	}
	pub fn stride(&self) -> usize {
		self.mode.info().stride()
	}
	pub fn size(&mut self) -> usize {
		self.protocol.frame_buffer().size()
	}
	pub fn resolution(&self) -> (usize, usize) {
		self.protocol.current_mode_info().resolution()
	}
}

/*
 * Get amount of total pixels of GOPmode.
 * It´s calculated by multiplying the x- and y-resolution.
 */
fn pixel_amount(mode: &ModeInfo) -> usize {
	let (resx, resy) = mode.resolution();
	if resx as f64 / resy as f64 == 16.0 / 9.0 {
		resx * resy * 4
	} else {
		resx * resy
	}
}
