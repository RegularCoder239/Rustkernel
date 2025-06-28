use crate::std::{
	Box
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

pub enum FilePath<'path> {
	Unix(&'path str),
	DOS(&'path str)
}

impl MountPoint {
	pub fn from_disk(disk_id: usize) -> MountPoint {
		MountPoint::Disk(disk_id)
	}
}
