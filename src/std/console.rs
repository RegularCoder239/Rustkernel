use core::fmt;
use super::Mutex;

pub struct Console;

impl fmt::Write for Console {
	fn write_str(&mut self, string: &str) -> fmt::Result {
		if let Some(mut console) = crate::kernel::graphicmanager::console() {
			console.print_str(string);
		}
		Ok(())
	}
}

#[macro_export]
macro_rules! print {
	($($args: tt)+) => {
		write!(crate::std::Console {}, $($args)+)
	};
}
#[macro_export]
macro_rules! println {
	($($args: tt)+) => {
		writeln!(crate::std::Console {}, $($args)+)
	};
}
