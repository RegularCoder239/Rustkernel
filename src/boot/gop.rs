use uefi::boot::ScopedProtocol;

#[derive(Debug)]
pub enum GOPError {
	UefiError(uefi::Error),
	NoModeFound
}

pub struct GOP {
	protocol: ScopedProtocol<GraphicsOutput>
}

impl From<uefi::Error> for GOPError {
	fn from(err: uefi::Error) -> Self {
		Self::UefiError(err)
	}
}

use uefi::proto::console::gop::{
	GraphicsOutput,
	ModeInfo
};
use uefi::boot;

impl GOP {
	pub fn new() -> Result<GOP, GOPError> {
		let mut gop = GOP {
			protocol: boot::open_protocol_exclusive::<GraphicsOutput>(
				boot::get_handle_for_protocol::<GraphicsOutput>()?
			)?
		};
		let mode = gop.protocol.modes().max_by(|mode, mode2| {
			fn pixels(mode: &ModeInfo) -> usize {
				let (resx, resy) = mode.resolution();
				if resx as f64 / resy as f64 == 16.0 / 9.0 {
					resx * resy * 4
				} else {
					resx * resy
				}
			}
			pixels(mode.info()).cmp(
				&pixels(mode2.info())
			)
		}).ok_or(GOPError::NoModeFound)?;
		gop.protocol.set_mode(&mode);
		Ok(gop)
	}
}
