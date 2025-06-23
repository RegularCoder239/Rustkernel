use crate::std::{
	Box
};
use super::FSError;

pub trait FileStructure {
	fn read(&self, path: FilePath, offset: usize, len: usize) -> Result<Box<[u8]>, FSError>;
}

pub struct FilePath<'path> {
	path: &'path str
}

impl FilePath<'_> {
	pub fn new_unix(path: &str) -> FilePath<'_> {
		FilePath {
			path: path
		}
	}
}
