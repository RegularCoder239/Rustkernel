pub mod paging;
pub mod buddy; /*{
	pub enum BuddyError {
		_1gIndexFault, // Fails to find free 1G section (too much RAM?)
		_2mIndexFault, // Fails to find free 2M section
	}

	extern {
		pub fn allocate(size: u64) -> Option<u64>;
		pub fn addRegion(addr: u64, size: usize) -> Result<(), BuddyError>;
	}
}
*/
