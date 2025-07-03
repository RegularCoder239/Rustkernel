use core::fmt::{
	Arguments,
	Write,
	self
};
use super::outb;

struct PortLogger;

#[macro_export]
macro_rules! info {
	($($args: tt)+) => {
		crate::std::log::log("INFO", format_args!($($args)+));
	};
}
#[macro_export]
macro_rules! error {
	($($args: tt)+) => {
		crate::std::log::log("ERROR", format_args!($($args)+));
	};
}
#[macro_export]
macro_rules! debug {
	($($args: tt)+) => {
		crate::std::log::log("DEBUG", format_args!($($args)+));
	};
}

impl PortLogger {
	fn new() -> PortLogger {
		PortLogger {}
	}
}

impl Write for PortLogger {
	fn write_str(&mut self, string: &str) -> fmt::Result {
		for byte in string.bytes() {
			outb(byte, 0xe9);
		}
		Ok(())
	}
}

pub fn log(section: &str, args: Arguments) {
	writeln!(PortLogger::new(), "[{}] {}", section, args);
}
