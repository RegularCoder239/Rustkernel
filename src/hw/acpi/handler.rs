use acpi::{
	PhysicalMapping,

	handler::AcpiHandler
};
use core::ptr::NonNull;
use crate::mm::Mapped;

/*
 * Just a struct, that implements the AcpiHandler trait,
 * which maps and unmaps physical regions.
 */
#[derive(Copy, Clone)]
pub struct AcpiMemoryHandler {}

impl AcpiHandler for AcpiMemoryHandler {
	unsafe fn map_physical_region<T>(&self, phys_addr: usize, size: usize) -> PhysicalMapping<Self, T> {
		let addr = (((phys_addr as u64) & !0xfff_u64).mapped_global::<T>(size).expect("Failed to do ACPI mapping.") as usize) + (phys_addr & 0xfff);
		unsafe {
			PhysicalMapping::new(
				phys_addr,
				NonNull::new(addr as *mut T).expect("Attempt to map physical address 0x0 as an acpi region."),
				size,
				size + 0x2000,
				AcpiMemoryHandler {}
			)
		}
	}
	fn unmap_physical_region<T>(_: &PhysicalMapping<Self, T>) {
		// TODO: Unmap ACPI physical region.
	}
}
