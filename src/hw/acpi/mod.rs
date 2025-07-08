mod handler;

use handler::AcpiMemoryHandler;
use crate::{
	uefi_result,
};
use crate::std::{
	self,
	Mutex,
	exit,
	StackVec,
	log
};
use acpi::{
	AcpiTables,
	PhysicalMapping,

	mcfg::Mcfg,
	madt::Madt,
	madt::MadtEntry
};
use core::ops::Deref;

pub struct ACPI {
	table: AcpiTables<AcpiMemoryHandler>,
	mcfg_mapping: Option<PhysicalMapping<AcpiMemoryHandler, Mcfg>>,
	madt_mapping: Option<PhysicalMapping<AcpiMemoryHandler, Madt>>,
}

#[derive(Copy, Clone, Default)]
pub struct PCIMCFGEntry {
	pub base_address: u64,
	pub bus_numbers: (usize, usize)
}

static ACPI_SINGLETON: Mutex<Option<ACPI>> = Mutex::new(None);

impl ACPI {
	pub fn pci_mcfg_entries(&self) -> Option<StackVec<PCIMCFGEntry, 0x10>> {
		let mut pci_buses = StackVec::<PCIMCFGEntry, 0x10>::new();
		for entry in self.mcfg_mapping.as_ref()?.get().entries() {
			let start = entry.bus_number_start as usize;
			let end = entry.bus_number_end as usize;
			pci_buses.push_back(PCIMCFGEntry {
				base_address: entry.base_address,
				bus_numbers: (start, end)
			});
		}
		Some(pci_buses)
	}
	pub fn madt_core_amount(&self) -> Option<usize> {
		Some(
			self.madt_mapping.as_ref()?.get().entries()
				.map(|entry| if let MadtEntry::LocalApic(e) = entry {
					1
				} else {
					0
				})
				.sum()
		)
	}
}

impl PCIMCFGEntry {
	pub fn address_range(&self) -> (u64, u64) {
		(
			self.base_address,
			self.base_address + 0x100000 * (self.bus_numbers.1 - self.bus_numbers.0) as u64
		)
	}
}

pub fn setup() -> ! {
	log::info!("Setting up ACPI.");
	let table = unsafe {
		AcpiTables::from_rsdp(
			AcpiMemoryHandler {},
			uefi_result!().unwrap().config.acpi as usize // Why is this so stupid!!
		).expect("No ACPI found.")
	};

	*ACPI_SINGLETON.lock() = Some(ACPI {
		mcfg_mapping: table.find_table::<Mcfg>().ok(),
		madt_mapping: table.find_table::<Madt>().ok(),
		table: table
	});

	exit();
}

pub fn acpi_singleton() -> std::OptMutexGuard<'static, ACPI> {
	while ACPI_SINGLETON.lock().is_none() {
		std::r#yield();
	}
	ACPI_SINGLETON.lock_opt()
}
