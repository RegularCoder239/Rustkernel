mod pageentry;
mod pagedirectory;
mod mapped;
mod pagetable;
pub mod kerneltable;

pub use pageentry::{
	PageTableEntry
};
pub use mapped::{
	Mapped,
	Address,
	MappingInfo,
	MappingFlags
};
pub use pagedirectory::{
	PageDirectory
};
pub use pagetable::{
	PageTable,
	setup_global_table
};
pub use kerneltable::{
	kernel_offset
};

use crate::std::{
	PerCpu,
	Mutex
};

#[derive(Debug)]
pub enum PagingError {
	GeneralError,
	SnappingError,
	AttemptedNonPresentUnmap,
	OutOfBoundsIndex,
	DirectoryAssociationFailed
}

pub const PAGE_SIZES: [usize; 5] = [
	0x0,
	0x1000,
	0x200000,
	0x40000000,
	0x8000000000
];

static INITIAL_PAGE_TABLE: Mutex<PageTable> = Mutex::new(PageTable::EMPTY);
static GLOBAL_PAGE_TABLE_MUTEX: PerCpu<&Mutex<PageTable>> = PerCpu::new(&INITIAL_PAGE_TABLE);

pub fn current_page_table() -> &'static Mutex<PageTable> {
	GLOBAL_PAGE_TABLE_MUTEX.deref()
}
pub fn set_current_page_table(table: &'static Mutex<PageTable>) {
	GLOBAL_PAGE_TABLE_MUTEX.set(table)
}
