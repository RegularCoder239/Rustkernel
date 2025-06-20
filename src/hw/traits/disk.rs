use crate::std::StackVec;

pub trait Disk {
	const BLOCK_SIZE: usize;

	fn read_lba(&self, lba: usize) -> [u8; Self::BLOCK_SIZE];
	fn disks() -> StackVec<impl Disk, 64> where Self: Sized;
}
