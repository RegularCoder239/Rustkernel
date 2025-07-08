use crate::std::{
	Vec,
	Box,
	Mutex,
	log,
	self
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
	DISKS.lock().push_back(disk);
}

pub fn read_lba(disk_idx: usize, lba: usize) -> Sector {
	let mut lock = DISKS.lock();
	lock
		.deref_mut()[disk_idx]
		.deref_mut().read_lba(lba)
}

pub fn read_lbas(disk_idx: usize, lba: usize, amount: usize) -> Box<[u8]> {
	let mut r#box = Box::<[u8]>::new_sized(amount * SECTOR_SIZE);

	for idx in 0..amount {
		let sector = read_lba(disk_idx, lba + idx);
		for idx2 in idx * SECTOR_SIZE..SECTOR_SIZE * (idx + 1) {
			r#box[idx2] = sector[idx2 % SECTOR_SIZE];
		}
	}

	r#box
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
/*
	let filecontent = FAT32::mount(MountPoint::Disk(0)).ok().unwrap().read(
		FilePath::DOS("TEST    ELF"), 0, usize::MAX
	).ok().unwrap();
	std::elf::load_elf(filecontent.as_slice());
*/
	crate::std::exit()
}
