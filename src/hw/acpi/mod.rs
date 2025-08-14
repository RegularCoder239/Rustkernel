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
	log,
	Lock
};
use acpi::{
	AcpiTables,
	PhysicalMapping,

	mcfg::Mcfg,
	madt::Madt,
	madt::MadtEntry
};

/*
 * Contains ACPI mappings and tables.
 * It´s also responsible for the processing with the table.
 */
pub struct ACPI {
	table: AcpiTables<AcpiMemoryHandler>,
	mcfg_mapping: Option<PhysicalMapping<AcpiMemoryHandler, Mcfg>>,
	madt_mapping: Option<PhysicalMapping<AcpiMemoryHandler, Madt>>,
}

/*
 * Contains the physical address of a PCI-Express root window
 * and also the bus ranges.
 */
#[derive(Copy, Clone, Default)]
pub struct PCIMCFGEntry {
	pub base_address: u64,
	pub bus_numbers: (usize, usize)
}

static ACPI_SINGLETON: Mutex<Option<ACPI>> = Mutex::new(None);
static ACPI_SETUP: Lock = Lock::new_locked();

impl ACPI {
	/*
	 * Get PCIMCFGEntries using the MCFG table.
	 * This method returns None, if no MCFG table is avaiable.
	 */
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
	/*
	 * Get the CPU core amount using the MADT ACPI table.
	 */
	pub fn madt_core_amount(&self) -> Option<usize> {
		Some(
			self.madt_mapping.as_ref()?.get().entries()
				.map(|entry| if let MadtEntry::LocalApic(_) = entry {
					1
				} else {
					0
				})
				.sum()
		)
	}
}

impl PCIMCFGEntry {
	/*
	 * Reads the physical address range the root window occupies.
	 */
	pub fn address_range(&self) -> (u64, u64) {
		(
			self.base_address,
			self.base_address + 0x100000 * (self.bus_numbers.1 - self.bus_numbers.0) as u64
		)
	}
}

/*
 * Boot task, that reads some of the ACPI tables.
 * If no ACPI is found, a kernel panic will be produced
 * due to design, but it can be fixed later.
 */
pub fn setup() -> ! {
	log::info!("Setting up ACPI.");
	let table = unsafe {
		AcpiTables::from_rsdp(
			AcpiMemoryHandler {},
			uefi_result!().unwrap().config.acpi as usize
		).expect("No ACPI found.")
	};
	*ACPI_SINGLETON.lock() = Some(ACPI {
		mcfg_mapping: table.find_table::<Mcfg>().ok(),
		madt_mapping: table.find_table::<Madt>().ok(),
		table: table
	});

	ACPI_SETUP.unlock();
	exit();
}

/*
 * Reads the ACPI information singleton. The guard will
 * lock the process when accessing the ACPI information, while
 * the ACPI table isn´t initalized.
 */
pub fn acpi_singleton() -> std::OptMutexGuard<'static, ACPI> {
	ACPI_SETUP.lock();
	ACPI_SINGLETON.lock_opt()
}
