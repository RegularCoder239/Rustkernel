pub use crate::virt::fs::{
	self,
	FilePath,
	FSError,
	MountPoint
};
use crate::std::Box;

pub fn read_file(fs_id: usize, path: FilePath, amount: usize, offset: usize) -> Result<Box<[u8]>, FSError> {
	fs::filesystem(fs_id).read(path, amount, offset)
}

pub fn mount(disk_id: usize) -> usize {
	fs::mount(MountPoint::from_disk(disk_id))
}
