use crate::std::{
	Box,
	String
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
	}/*
	pub fn force_dos(&self) -> Option<FilePath> {
		Some(
			FilePath::new_dos(
				match self {
					FilePath::Unix(path) => Self::unix_to_dos(path)?,
					FilePath::DOS(path) => path.clone()
				}
			)
		)
	}

	fn unix_to_dos(unix_path: &String) -> Option<String> {
		let mut pathsegments: Vec<String> = unix_path.split('/');
		pathsegments[0] = "".into();
		for p in &pathsegments {
			if p.len() > 12 {
				crate::std::log::info!("{}", pathsegments.len());
				return None;
			}
		}


		Some(
			String::from("/").join(pathsegments.into_iter())
		)
	}*/
}
