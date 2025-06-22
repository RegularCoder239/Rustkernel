use crate::std::Box;
use core::fmt::Display;
use core::fmt;
use core::ops::Deref;

pub struct String {
	content: Box<[u8]>,
}

impl String {
	pub fn from_bytes(bytes: Box<[u8]>) -> String {
		String {
			content: bytes
		}
	}
}

impl Deref for String {
	type Target = str;

	fn deref(&self) -> &str {
		core::str::from_utf8(
			unsafe {
				core::slice::from_raw_parts(
					self.content.as_ptr::<u8>(),
					self.content.alloc_len()
				)
			}
		).expect("Attempt to pass nonutf-8 string.")
	}
}

impl Display for String {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		Display::fmt(self.deref(), f)
	}
}
