use super::{
	VecChunk,
	VecIter,
	VecIterNonRef,
	VecIterMut,

	super::Allocator,
	super::RAMAllocator,
	super::SharedRef
};
use core::{
	marker::PhantomData,
	ops::Index,
	ops::IndexMut,
	cmp::PartialEq,
	cmp::PartialOrd
};

pub trait VecBase<T> {
	fn len(&self) -> usize;
	fn index_ptr_mut(&mut self, index: usize) -> *mut T;
}

pub struct Vec<T, A: Allocator = RAMAllocator> {
	begin: SharedRef<VecChunk<T, A>>,
	last: SharedRef<VecChunk<T, A>>,
	length: usize,
	capacity: usize,
	phantom: PhantomData<A>
}

impl<T, A: Allocator> Vec<T, A> {
	pub const fn new() -> Vec<T, A> {
		Vec {
			begin: SharedRef::EMPTY,
			last: SharedRef::EMPTY,
			length: 0,
			capacity: 0,
			phantom: PhantomData
		}
	}

	pub fn from_optfn<F>(meth: F, amount: usize) -> Option<Vec<T, A>> where F: Fn(usize) -> Option<T> {
		let mut vec = Vec::new();
		for idx in 0..amount {
			vec.push_back((meth)(idx)?);
		}
		Some(vec)
	}

	pub fn push_back(&mut self, what: T) {

		let idx = self.length;
		self.length += 1;
		self.grow();

		*self.index_mut(idx) = what;
	}

	fn index_chunk(&self, mut index: usize) -> (&VecChunk<T, A>, usize) {
		if self.length <= index {
			panic!("Attempt to index {} in a vec with length {}.", index, self.length);
		}
		let mut chunk = &self.begin;
		while chunk.capacity() <= index {
			index -= chunk.capacity();
			chunk = chunk.next();
			if chunk.is_none() {
				panic!("Bug 1");
			}
		}
		(
			chunk,
			index
		)
	}
	fn index_chunk_mut(&mut self, mut index: usize) -> (&mut VecChunk<T, A>, usize) {
		if self.length <= index {
			panic!("Attempt to index {} in a vec with length {}.", index, self.length);
		}
		let mut chunk = &mut self.begin;
		while chunk.capacity() <= index {
			index -= chunk.capacity();
			chunk = chunk.next_mut();
			if chunk.is_none() {
				panic!("Bug 1");
			}
		}
		(
			chunk,
			index
		)
	}
	fn grow(&mut self) {
		if self.length < self.capacity {
			return;
		}
		if self.capacity == 0 {
			self.capacity = 1;
		}
		let mut new_chunk = SharedRef::<VecChunk<T, A>>::new(
			VecChunk::<T, A>::new(self.capacity.next_power_of_two() * 2)
		);
		if self.last.is_none() {
			self.last = new_chunk.split();
			self.begin = new_chunk;
		} else {
			*self.last.next_mut() = new_chunk;
		}

		self.capacity += self.capacity.next_power_of_two() * 2;
	}
	pub fn swap(&mut self, index1: usize, index2: usize) {
		if index1 != index2 {
			unsafe {
				core::ptr::swap(
					self.index_ptr_mut(index1),
					self.index_ptr_mut(index2)
				)
			}
		}
	}
	pub fn sort(&mut self, meth: fn(&T, &T) -> bool) {
		let mut repeat = true;
		while (repeat) {
			repeat = false;
			for idx in 0..self.len() - 1 {
				if meth(&self[idx], &self[idx+1]) {
					self.swap(idx, idx + 1);
					repeat = true;
				}
			}
		}
	}
	pub fn fastposition<T2: PartialEq + PartialOrd>(&self, meth: fn(&T) -> T2, what: T2) -> Option<usize> {
		if self.len() > 12 {
			let mut idx = 0;
			while meth(&self[idx]) < what || idx + 8 < self.len() {
				idx += 8;
			}
			if meth(&self[idx]) == what {
				return Some(idx);
			}
			while meth(&self[idx]) > what || idx > 4 {
				idx -= 4;
			}
			if meth(&self[idx]) == what {
				return Some(idx);
			}
			for idx2 in idx..(idx + 8).min(self.len()) {
				if meth(&self[idx2]) == what {
					return Some(idx2);
				}
			}
			return None;
		} else {
			self.into_iter().position(|it| meth(&it) == what)
		}
	}

	pub fn fastfind<T2: PartialEq + PartialOrd>(&self, meth: fn(&T) -> T2, what: T2) -> Option<&T> {
		Some(
			self.index(self.fastposition(meth, what)?)
		)
	}
}

impl<T: Clone, A: Allocator> Vec<T, A> {
	pub fn from_slice(slice: &[T]) -> Vec<T, A> {
		let mut vec = Vec::new();
		for idx in 0..slice.len() {
			vec.push_back(slice[idx].clone());
		}
		vec
	}
}

impl<T, A: Allocator> crate::std::vec::vec::VecBase<T> for Vec<T, A> {
	fn len(&self) -> usize {
		self.length
	}
	fn index_ptr_mut(&mut self, index: usize) -> *mut T {
		let (chunk, remainder) = self.index_chunk_mut(index);
		chunk.index_mut(remainder) as *mut T
	}
}

impl<T, A: Allocator> crate::std::vec::vec::VecBase<T> for &Vec<T, A> {
	fn len(&self) -> usize {
		self.length
	}
	fn index_ptr_mut(&mut self, _: usize) -> *mut T {
		panic!("Unable to index ptr mutable from readonly vecbase.");
	}
}

impl<T, A: Allocator> crate::std::vec::vec::VecBase<T> for &mut Vec<T, A> {
	fn len(&self) -> usize {
		self.length
	}
	fn index_ptr_mut(&mut self, index: usize) -> *mut T {
		let (chunk, remainder) = self.index_chunk_mut(index);
		chunk.index_mut(remainder) as *mut T
	}
}

impl<T, A: Allocator> Index<usize> for Vec<T, A> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		let (chunk, remainder) = self.index_chunk(index);
		chunk.index(remainder)
	}
}

impl<T, A: Allocator> Index<usize> for &Vec<T, A> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		let (chunk, remainder) = self.index_chunk(index);
		chunk.index(remainder)
	}
}
impl<T, A: Allocator> Index<usize> for &mut Vec<T, A> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		let (chunk, remainder) = self.index_chunk(index);
		chunk.index(remainder)
	}
}
impl<T, A: Allocator> IndexMut<usize> for Vec<T, A> {
	fn index_mut(&mut self, index: usize) -> &mut T {
		let (chunk, remainder) = self.index_chunk_mut(index);
		chunk.index_mut(remainder)
	}
}
impl<T, A: Allocator> IndexMut<usize> for &mut Vec<T, A> {
	fn index_mut<'vec>(&'vec mut self, index: usize) -> &'vec mut T {
		let (chunk, remainder) = self.index_chunk_mut(index);
		chunk.index_mut(remainder)
	}
}

impl<T: Clone, A: Allocator> IntoIterator for Vec<T, A> {
	type Item = T;
	type IntoIter = VecIterNonRef<T, Vec<T, A>>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::new(self)
	}
}

impl<'vec, T: 'vec, A: Allocator> IntoIterator for &'vec Vec<T, A> {
	type Item = &'vec T;
	type IntoIter = VecIter<'vec, T, Vec<T, A>>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::new(self)
	}
}

impl<'vec, T: 'vec, A: Allocator> IntoIterator for &'vec mut Vec<T, A> {
	type Item = &'vec mut T;
	type IntoIter = VecIterMut<'vec, T, &'vec mut Vec<T, A>>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::new(self)
	}
}
