use crate::std::{
	Vec,
	VecBase,
	Box,
	Mutex,
	log
};
use core::ops::DerefMut;

pub const SECTOR_SIZE: usize = 512;
pub type Sector = Box<[u8; SECTOR_SIZE]>;

pub trait PhysicalDisk {
	fn reset(&mut self);
	fn read_lba(&self, lba: usize) -> Sector;
}

pub struct VirtualDisk {
	pub physical_disk: Box<dyn PhysicalDisk>,
	id: u64
}

static DISKS: Mutex<Vec<VirtualDisk>> = Mutex::new_rdfused(Vec::new());
static IDCOUNTER: Mutex<u64> = Mutex::new(0);

unsafe impl Sync for VirtualDisk {}

pub fn add_disk(disk: Box<dyn PhysicalDisk>) {
	let mut idlock = IDCOUNTER.lock();
	*idlock += 1;
	DISKS.lock().push_back(VirtualDisk {
		physical_disk: disk,
		id: *idlock
	});
}

pub fn read_lba(disk_idx: usize, lba: usize) -> Sector {
	DISKS[disk_idx].physical_disk.read_lba(lba)
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

pub fn disk_ids() -> Vec<usize> {
	(0..DISKS.read().len()).collect()
}

pub fn setup_disks() -> ! {
	super::super::pci::wait_for_scan();
	log::info!("Setting up disks.");
	{
		let mut lock = DISKS.lock();

		for disk in lock.deref_mut() {
			disk.physical_disk.reset();
		}
		DISKS.unfuse();
		log::info!("Disks found: {}", DISKS.read().len());
	}
	crate::std::exit()
}
