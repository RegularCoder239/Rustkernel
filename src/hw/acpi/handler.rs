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
		let addr = (((phys_addr as u64) & !0xfff_u64).mapped_global::<T>(size).expect("Failed to do ACPI mapping.") as usize) + (phys_addr & 0xfff);
		crate::std::log::info!("Mapping: {:x} {:x}", addr as u64, phys_addr);
		unsafe {
			PhysicalMapping::new(
				phys_addr,
				NonNull::new_unchecked(addr as *mut T),
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
