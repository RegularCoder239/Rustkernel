use crate::std::{
	Vec,
	Box,
	Mutex,
	log
};
use core::ops::DerefMut;

pub const SECTOR_SIZE: usize = 512;
pub type Sector = Box<[u8; SECTOR_SIZE]>;

pub trait Disk {
	fn reset(&mut self);
	fn read_lba(&mut self, lba: usize) -> Sector;
}

pub static DISKS: Mutex<Vec<Box<dyn Disk>>> = Mutex::new(Vec::new());

pub fn add_disk(disk: Box<dyn Disk>) {
	DISKS.lock().push_back(disk)
}

pub fn read_lba(disk_idx: usize, lba: usize) -> Sector {
	let mut lock = DISKS.lock();
	lock
		.deref_mut()[disk_idx]
		.deref_mut().read_lba(lba)
}

use crate::virt::fs::{
	FAT32,
	MountPoint,
	FileStructure,
	FilePath,
	readresult_to_str
};

pub fn setup_disks() -> ! {
	log::info!("Setting up disks.");
	{
		let mut lock = DISKS.lock();
		for disk in lock.deref_mut() {
			disk.reset();
		}
	}

	crate::std::exit()
}
