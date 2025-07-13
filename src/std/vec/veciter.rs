use core::{
	marker::PhantomData,

	ops::Index,
	ops::IndexMut
};
use super::{
	vec::VecBase
};

pub struct VecIter<'vec, T, V> {
	vec: &'vec V,
	phantom: PhantomData<&'vec T>,
	idx: usize
}
pub struct VecIterMut<'iter, T, V> {
	vec: &'iter mut V,
	phantom: PhantomData<&'iter mut T>,
	idx: usize
}
pub struct VecIterNonRef<T, V> {
	vec: V,
	phantom: PhantomData<T>,
	idx: usize
}

impl<'vec, T, V: Index<usize>> VecIter<'vec, T, V> {
	pub fn new(vec: &'vec V) -> VecIter<'vec, T, V> {
		VecIter {
			vec: vec,
			phantom: PhantomData,
			idx: 0
		}
	}
}

impl<'vec, T, V: VecBase<T> + Index<usize, Output = T> + 'vec> Iterator for VecIter<'vec, T, V> {
	type Item = &'vec T;

	fn next(&mut self) -> Option<&'vec T> {
		if self.vec.len() <= self.idx {
			self.idx = 0;
			None
		} else {
			self.idx += 1;
			Some(self.vec.index(self.idx - 1))
		}
	}
}

impl<'vec, T, V: VecBase<T> + Index<usize, Output = T> + 'vec> core::iter::ExactSizeIterator for VecIter<'vec, T, V> {
	fn len(&self) -> usize {
		self.vec.len()
	}
}

impl<'vec, T, V: Index<usize>> VecIterMut<'vec, T, V> {
	pub fn new(vec: &'vec mut V) -> VecIterMut<'vec, T, V> {
		VecIterMut {
			vec: vec,
			phantom: PhantomData,
			idx: 0
		}
	}
}

impl<'vec, T, V: VecBase<T> + Index<usize, Output = T> + IndexMut<usize> + 'vec> Iterator for VecIterMut<'vec, T, V> {
	type Item = &'vec mut T;

	fn next(&mut self) -> Option<&'vec mut T> {
		if self.vec.len() <= self.idx {
			self.idx = 0;
			None
		} else {
			self.idx += 1;
			Some(unsafe {
				self.vec.index_ptr_mut(self.idx - 1).as_mut().unwrap()
			})
		}
	}
}

impl<T, V: Index<usize>> VecIterNonRef<T, V> {
	pub fn new(vec: V) -> VecIterNonRef<T, V> {
		VecIterNonRef {
			vec: vec,
			phantom: PhantomData,
			idx: 0
		}
	}
}

impl<T: Clone, V: VecBase<T> + Index<usize, Output = T>> Iterator for VecIterNonRef<T, V> {
	type Item = T;

	fn next(&mut self) -> Option<T> {
		if self.vec.len() <= self.idx {
			self.idx = 0;
			None
		} else {
			Some(self.vec.index(self.idx).clone())
		}
	}
}
