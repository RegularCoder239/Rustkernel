use super::{
	FAT32,
	FileStructure,
	MountPoint
};
use crate::std::{
	Box
};

pub fn mount(mountpoint: MountPoint) -> Option<Box<dyn FileStructure>> {
	Some(
		Box::new(FAT32::mount(mountpoint).ok()?)
	)
}
