use acpi::{
	PhysicalMapping,

	handler::AcpiHandler
};
use core::ptr::NonNull;
use crate::mm::Mapped;

#[derive(Clone)]
pub struct AcpiMemoryHandler {
}

impl AcpiHandler for AcpiMemoryHandler {
	unsafe fn map_physical_region<T>(&self, phys_addr: usize, size: usize) -> PhysicalMapping<Self, T> {
		unsafe {
			PhysicalMapping::new(
				phys_addr,
				NonNull::new_unchecked((phys_addr as u64).mapped_global(size).expect("Failed to map ACPI thing.")),
				size,
				size,
				AcpiMemoryHandler {}
			)
		}
	}
	fn unmap_physical_region<T>(_: &PhysicalMapping<Self, T>) {
		// TODO: Unmap ACPI physical region.
	}
}
