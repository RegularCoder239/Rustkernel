use crate::std::{
	Box,
	Vec
};
use core::fmt::Display;
use core::fmt;
use core::ops::{
	AddAssign,
	Add,
	Deref,
	Range
};

pub struct String {
	content: Option<Box<[u8]>>,
}

impl String {
	pub fn new() -> String {
		String {
			content: None
		}
	}
	pub fn new_filled(byte: char, length: usize) -> String {
		String {
			content: Some(Box::new_filled(byte as u8, length))
		}
	}
	pub unsafe fn new_uninit(length: usize) -> String {
		String {
			content: Some(Box::new_sized(length))
		}
	}
	pub fn as_str(&self) -> &str {
		core::str::from_utf8(
			self.bytes()
		).expect("Attempt to pass nonutf-8 string.")
	}
	pub fn bytes(&self) -> &[u8] {
		if let Some(content) = &self.content {
			content.as_slice()
		} else {
			&[]
		}
	}
	pub fn bytes_mut(&mut self) -> &mut [u8] {
		if let Some(content) = &mut self.content {
			content.as_slice_mut()
		} else {
			&mut []
		}
	}
	pub fn chars(&self) -> &[char] {
		unsafe {
			&*(self.bytes() as *const [u8] as *const [char])
		}
	}
	pub fn find_all(&self, ch: char) -> Vec<usize> {
		let mut result = Vec::<usize>::new();
		for (idx, ch2) in self.bytes().into_iter().enumerate() {
			if ch == *ch2 as char {
				result.push_back(idx);
			}
		}
		result
	}
	pub fn join<I: Iterator<Item = String>>(&self, iter: I) -> String {
		let mut result = String::new();
		for (idx, str_chunk) in iter.enumerate() {
			if idx != 0 {
				result += self.clone();
			}
			result += str_chunk;
		}
		result
	}
	pub fn len(&self) -> usize {
		if let Some(content) = &self.content {
			content.alloc_len()
		} else {
			0
		}
	}

	pub fn padded(&self, length: usize, padcharacter: char) -> String {
		if self.len() >= length {
			self.clone()
		} else {
			self.clone() + String::new_filled(padcharacter, length - self.len())
		}
	}
	pub fn split(&self, ch: char) -> Vec<String> {
		let mut result = Vec::<String>::new();
		let mut prev_idx = 0;

		let findings = self.find_all(ch);
		for idx in &findings {
			result.push_back(self.slice(prev_idx..*idx).unwrap());
			prev_idx = *idx + 1;
		}
		result.push_back(self.slice(prev_idx..self.len()).unwrap());
		result
	}
	pub fn slice(&self, range: Range<usize>) -> Option<String> {
		Some(
			String::from(self.content.as_ref()?.as_slice().get(range)?)
		)
	}
	pub fn upper(&self) -> String {
		let mut upper = unsafe {
			Self::new_uninit(self.len())
		};
		let bytesupper = upper.bytes_mut();
		for (idx, byte) in self.bytes().into_iter().enumerate() {
			let mut bytecopy = *byte;
			if bytecopy >= 97 && bytecopy <= 122 {
				bytecopy -= 32;
			}
			bytesupper[idx] = bytecopy;
		}
		upper
	}
}

impl Add<&String> for &String {
	type Output = String;
	fn add(self, toadd: &String) -> String {
		if toadd.len() == 0 {
			self.clone()
		} else {
			let mut r#box: Box<[u8]> = Box::new_sized(toadd.len() + self.len());
			let slice = r#box.as_slice_mut();
			for (idx, ch) in self.bytes().into_iter().chain(toadd.bytes().into_iter()).enumerate() {
				slice[idx] = *ch;
			}
			String {
				content: Some(r#box)
			}
		}
	}
}

impl Add<String> for String {
	type Output = String;
	fn add(self, toadd: String) -> String {
		&self + &toadd
	}
}

impl AddAssign for String {
	fn add_assign(&mut self, toadd: String) {
		if toadd.len() == 0 {
			return;
		}
		let mut r#box: Box<[u8]> = Box::new_sized(toadd.len() + self.len());
		let slice = r#box.as_slice_mut();
		for (idx, ch) in self.bytes().into_iter().chain(toadd.bytes().into_iter()).enumerate() {
			slice[idx] = *ch;
		}
		self.content = Some(r#box);
	}
}

impl Clone for String {
	fn clone(&self) -> String {
		String {
			content: if let Some(content) = &self.content {
				Some(Box::new_slice(content.as_slice()))
			} else {
				None
			}
		}
	}
}

impl Deref for String {
	type Target = str;

	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl Display for String {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		Display::fmt(self.deref(), f)
	}
}

impl From<&str> for String {
	fn from(raw: &str) -> Self {
		String::from(Box::new_slice(raw.as_bytes()))
	}
}

impl From<&[char]> for String {
	fn from(raw: &[char]) -> Self {
		String::from(Box::new_slice(
			unsafe {
				(raw as *const [char] as *const [u8]).as_ref().unwrap()
			}
		))
	}
}

impl From<&[u8]> for String {
	fn from(raw: &[u8]) -> Self {
		String::from(Box::new_slice(raw))
	}
}

impl From<Box<[u8]>> for String {
	fn from(raw: Box<[u8]>) -> String {
		String {
			content: Some(raw)
		}
	}
}

impl<const SIZE: usize> From<[char; SIZE]> for String {
	fn from(raw: [char; SIZE]) -> Self {
		String::from(raw.as_slice())
	}
}

impl<const SIZE: usize> From<[u8; SIZE]> for String {
	fn from(raw: [u8; SIZE]) -> Self {
		String::from(raw.as_slice())
	}
}

impl PartialEq for String {
	fn eq(&self, tocmp: &String) -> bool {
		self.bytes().eq(tocmp.bytes())
	}
}

impl fmt::Write for String {
	fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
		self.content = Some(Box::new_slice(string.as_bytes()));
		Ok(())
	}
}
