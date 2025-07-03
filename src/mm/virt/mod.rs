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
	PerCpuLazy,
	PerCpu,
	UnsafeRef
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
static INITIAL_PAGE_TABLES: PerCpu<PageTable> = PerCpu::new(PageTable::EMPTY);
static GLOBAL_PAGE_TABLE_MUTEX: PerCpuLazy<UnsafeRef<PageTable>> = PerCpuLazy::new(
	|| {
		let table = UnsafeRef::from_ref(INITIAL_PAGE_TABLES.deref_mut());
		table.get().init();
		table
	}
);

pub fn current_page_table() -> &'static mut PageTable {
	GLOBAL_PAGE_TABLE_MUTEX.deref_mut().get()
}
pub fn set_current_page_table(table: &'static mut PageTable) {
	GLOBAL_PAGE_TABLE_MUTEX.set(UnsafeRef::from_ref(table))
}
