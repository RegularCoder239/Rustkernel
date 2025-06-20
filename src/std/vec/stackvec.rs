use core::ops::{
	Index
};
use super::{
	VecIter,
	VecBase
};

pub struct StackVec<T, const SIZE: usize> {
	array: [T; SIZE],
	length: usize
}

impl<T: Copy, const SIZE: usize> StackVec<T, SIZE> {
	pub const fn new_filled(item: T) -> StackVec<T, { SIZE }> {
		StackVec {
			array: [item; SIZE],
			length: 0
		}
	}

}

impl<T: Copy + Default, const SIZE: usize> StackVec<T, SIZE> {
	pub fn new() -> StackVec<T, { SIZE }> {
		StackVec {
			array: [T::default(); SIZE],
			length: 0
		}
	}

	pub fn from_optfn<F>(meth: F, amount: usize) -> Option<StackVec<T, { SIZE }>> where F: Fn(usize) -> Option<T> {
		let mut vec = StackVec::new();
		for idx in 0..amount {
			vec.array[idx] = (meth)(idx)?;
		}
		vec.length = amount;
		Some(vec)
	}
	pub fn from_slice(slice: &[T]) -> StackVec<T, { SIZE }> {
		let mut vec = StackVec::new();
		for idx in 0..slice.len() {
			vec.array[idx] = slice[idx];
		}
		vec.length = slice.len();
		vec
	}
	pub fn push_back(&mut self, content: T) {
		assert!(self.length < SIZE, "Capacity limit reached on StackVec.");
		self.array[self.length] = content;
		self.length += 1;
	}
}

impl<T, const SIZE: usize> VecBase<T> for StackVec<T, SIZE> {
	fn len(&self) -> usize {
		self.length
	}
	fn index_ptr(&self, index: usize) -> *const T {
		&self.array[index] as *const T
	}
	fn index_ptr_mut(&mut self, index: usize) -> *mut T {
		&mut self.array[index] as *mut T
	}
}
impl<T, const SIZE: usize> VecBase<T> for &StackVec<T, SIZE> {
	fn len(&self) -> usize {
		self.length
	}
	fn index_ptr(&self, index: usize) -> *const T {
		&self.array[index] as *const T
	}
	fn index_ptr_mut(&mut self, _: usize) -> *mut T {
		todo!("Attempt to index readonly Stackvec as mutable.")
	}
}

impl<T: Copy, const SIZE: usize> Index<usize> for StackVec<T, SIZE> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		self.array.each_ref()[index]
	}
}

impl<T, const SIZE: usize> Index<usize> for &StackVec<T, SIZE> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		self.array.each_ref()[index]
	}
}

impl<'vec, T: Copy + 'vec, const SIZE: usize> IntoIterator for &'vec StackVec<T, SIZE> {
	type Item = &'vec T;
	type IntoIter = VecIter<'vec, T, &'vec StackVec<T, SIZE>>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::new(self)
	}
}
