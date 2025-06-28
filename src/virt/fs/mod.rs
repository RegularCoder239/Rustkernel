mod testfs;
mod filestructure;
mod fat;

pub use filestructure::{
	FileStructure,
	FilePath,
	MountPoint
};

pub use fat::FAT32;
pub use testfs::TestFS;

use crate::std::{
	Box,
	String
};

pub enum FSError {
	OOBRead, // Offset is greater than the length of the file while reading.
	FileNotFound,
	InvalidPath
}

pub fn readresult_to_str(readresult: Result<Box<[u8]>, FSError>) -> Result<String, FSError> {
	if let Ok(result) = readresult {
		Ok(String::from_bytes(result))
	} else {
		Err(readresult.err().unwrap())
	}
}
