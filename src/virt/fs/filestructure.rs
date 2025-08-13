use crate::std::{
	Box,
	String,
	Vec
};
use super::FSError;

pub enum MountPoint {
	Disk(usize),
	Standalone
}

pub trait FileStructure {
	fn mount(mount_point: MountPoint) -> Result<Self, FSError> where Self: Sized;
	fn read(&self, path: FilePath, offset: usize, len: usize) -> Result<Box<[u8]>, FSError>;
}

pub enum FilePath {
	Unix(String),
	DOS(String)
}

impl MountPoint {
	pub fn from_disk(disk_id: usize) -> MountPoint {
		MountPoint::Disk(disk_id)
	}
}

impl FilePath {
	pub const fn new_unix(path: String) -> FilePath {
		FilePath::Unix(path)
	}
	pub const fn new_dos(path: String) -> FilePath {
		FilePath::DOS(path)
	}
	pub fn segments(&self) -> Vec<String> {
		match self {
			FilePath::DOS(s) => (String::from("/") + s.clone()).split('/'),
			FilePath::Unix(s) => s.split('/')
		}
	}
}
