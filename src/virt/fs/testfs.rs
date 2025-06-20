use super::{
	FileStructure,
	FilePath,
	FSError
};
use crate::std::{
	Allocation,
	Box
};

pub struct TestFS;

impl TestFS {
	const TESTFS_CONTENT: &str = "Message from TestFS.";
}

impl FileStructure for TestFS {
	fn read(&self, _: FilePath, offset: usize, len: usize) -> Result<Box<[u8]>, FSError> {
		if offset >= Self::TESTFS_CONTENT.len() {
			return Result::Err(FSError::OOBRead);
		} else {
			Ok(
				Box::new_from_slice(
					(&Self::TESTFS_CONTENT[offset..offset+len.min(Self::TESTFS_CONTENT.len() - offset)]).as_bytes()
				)
			)
		}
	}
}
