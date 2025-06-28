use crate::{
	hw::Sector,
	hw::read_lba,
	std::Box
};
use core::{
	ops::Index
};
use crate::std::ReverseBytes;
use super::{
	FileStructure,
	MountPoint,
	FilePath,
	FSError,
	readresult_to_str
};

pub struct FAT32 {
	disk_id: usize,

	boot_sector: Sector,
	boot_sector_info: Box<BootSector>,
	root_directory_sector: Sector,
	root_directory: Directory,

	// Precalculated values
	//fat_lba_begin: usize
	data_lba: usize
}

#[repr(C, packed)]
pub struct BootSector {
	jmp_instruction: [u8; 3],
	oem_name: u64,
	bytes_per_sector: u16,
	sectors_per_cluster: u8,
	reserved_sector_count: u16,
	fat_amount: u8,
	root_entry_count: u16,
	total_sector_count_16: u16,
	media: u8,
	fat_size_16: u16,
	sector_per_track: u16,
	head_amount: u16,
	hidden_sectors: u32,
	total_sector_amount: u32,
	fat_size: u32,
	ext_flags: u16,
	fs_version: u16,
	root_cluster: u32,
	fs_info_lba: u16,
	bakup_boot_sector: u16
}

pub type Directory = Box<[DirectoryEntry]>;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DirectoryEntry {
	name: [u8; 11],
	attributes: u8,
	reserved: u8,
	creation_time_tenth: u8,
	creation_time: u16,
	creation_date: u16,
	last_access_date: u16,
	first_data_cluster_high: u16,
	modification_time: u16,
	modification_date: u16,
	first_data_cluster_low: u16,
	file_size: u32
}

impl FAT32 {
	fn read_cluster(&self, cluster: usize) -> Sector {
		read_lba(self.disk_id, self.boot_sector_info.sectors_per_cluster as usize * (cluster - 2) + self.data_lba)
	}
}

impl FileStructure for FAT32 {
	fn mount(mount_point: MountPoint) -> Result<Self, FSError> {
		if let MountPoint::Disk(disk_id) = mount_point {
			let boot_sector = read_lba(disk_id, 0);
			let boot_sector_info = Box::<BootSector>::from_raw_address(boot_sector.physical_address());
			let data_lba = boot_sector_info.reserved_sector_count as usize + boot_sector_info.fat_amount as usize * boot_sector_info.fat_size as usize;
			let root_directory_sector = read_lba(disk_id, data_lba);
			Ok(
				FAT32 {
					disk_id,
					data_lba,
					root_directory: Box::from_raw_address_sized(root_directory_sector.physical_address(), 0x200),
					root_directory_sector,
					boot_sector,
					boot_sector_info
				}
			)
		} else {
			panic!("Attempt to mount FAT32 without specifing the disk id.");
		}
	}
	fn read(&self, path: FilePath, _: usize, _: usize) -> Result<Box<[u8]>, FSError> {
		if let FilePath::DOS(fat_path) = path {
			let entry = self.root_directory.as_slice().into_iter().find(
				|entry| {
					core::str::from_utf8(&entry.name).unwrap() == fat_path
				}
			).ok_or(FSError::FileNotFound)?;
			Ok(
				self.read_cluster(entry.first_data_cluster_low as usize | ((entry.first_data_cluster_high as usize) << 16))
			)
		} else {
			Err(FSError::InvalidPath)
		}
	}
}
