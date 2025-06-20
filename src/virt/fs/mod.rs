mod testfs;
mod filestructure;

pub use filestructure::{
	FileStructure,
	FilePath
};

pub use testfs::TestFS;

use crate::std::{
	Box,
	String
};

pub enum FSError {
	OOBRead, // Offset is greater than the length of the file while reading.
}

pub fn readresult_to_str(readresult: Result<Box<[u8]>, FSError>) -> Result<String, FSError> {
	if let Ok(result) = readresult {
		Ok(String::from_bytes(result))
	} else {
		Err(readresult.err().unwrap())
	}
}
