mod testfs;
mod filestructure;
mod fat;
mod mount;

pub use filestructure::{
	FileStructure,
	FilePath,
	MountPoint
};

pub use fat::FAT32;
pub use testfs::TestFS;

use crate::std::{
	Box,
	String,
	Vec,
	VecBase,
	Mutex,
	MutexGuard
};

pub enum FSError {
	OOBRead, // Offset is greater than the length of the file while reading.
	FileNotFound,
	InvalidPath
}

static mut FILE_SYSTEMS: Vec<Mutex<Box<dyn FileStructure>>> = Vec::new();

pub fn readresult_to_str(readresult: Result<Box<[u8]>, FSError>) -> Result<String, FSError> {
	if let Ok(result) = readresult {
		Ok(String::from_bytes(result))
	} else {
		Err(readresult.err().unwrap())
	}
}

pub fn mount(mountpoint: MountPoint) -> usize {
	if let Some(fs) = mount::mount(mountpoint) {
		unsafe {
			FILE_SYSTEMS.push_back(Mutex::new(fs));
			FILE_SYSTEMS.len() - 1
		}
	} else {
		usize::MAX
	}
}

pub fn filesystem(id: usize) -> MutexGuard<'static, Box<dyn FileStructure>> {
	unsafe {
		FILE_SYSTEMS[id].lock()
	}
}
