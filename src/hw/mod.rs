pub mod cpu;
pub mod graphics;
pub mod acpi;
pub mod pci;
pub mod traits;

pub use traits::{
	disk::Disk,
	disk::add_disk,
	disk::Sector,
	disk::read_lba
};
