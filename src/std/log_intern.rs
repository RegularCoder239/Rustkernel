use core::fmt::{
	Arguments,
	Write,
	self
};
use super::outb;

struct Logger;

#[macro_export]
macro_rules! info {
	($($args: tt)+) => {
		crate::std::log::log("INFO ", format_args!($($args)+));
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
#[macro_export]
macro_rules! warn {
	($($args: tt)+) => {
		crate::std::log::log("WARNING", format_args!($($args)+));
	};
}

impl Logger {
	fn log_port(&mut self, string: &str) {
		for byte in string.bytes() {
			outb(byte, 0xe9);
		}
	}
	fn log_console(&mut self, string: &str) {
		if let Some(mut console) = crate::kernel::graphicmanager::console() {
			console.print_str(string);
		}
	}
}

impl Write for Logger {
	fn write_str(&mut self, string: &str) -> fmt::Result {
		self.log_port(string);
		self.log_console(string);
		Ok(())
	}
}

pub fn log(section: &str, args: Arguments) {
	let _ = writeln!(Logger {}, "[{}] {}", section, args);
}
