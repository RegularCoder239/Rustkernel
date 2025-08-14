use uefi::{
	Guid,

	table::system_table_raw,
	table::cfg::ACPI2_GUID,
	table::cfg::ConfigTableEntry
};

/*
 * Wrapper of UEFI Configuration table.
 * Use to get physical address of the RSDP.
 */
pub struct UEFIConfig {
	pub acpi: u64
}

impl UEFIConfig {
	pub fn generate() -> UEFIConfig {
		unsafe {
			let system_table = system_table_raw().expect("No system table found.").as_ref();
			let elements: &[ConfigTableEntry] = core::slice::from_raw_parts(
				system_table.configuration_table as *const ConfigTableEntry,
				system_table.number_of_configuration_table_entries
			);

			UEFIConfig {
				acpi: get_entry(elements, ACPI2_GUID)
			}
		}
	}
}

/*
 * Get entry of the configuration table.
 * Warning: Panics if searching fails.
 */
fn get_entry(entries: &[ConfigTableEntry], what: Guid) -> u64 {
	entries
		.into_iter()
		.find(|e| e.guid == what)
		.expect("Failed to find UEFI-Configuration table entry.")
		.address as u64
}
