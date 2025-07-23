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
	ops::IndexMut
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
		}
		(
			chunk,
			index
		)
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
		while repeat {
			repeat = false;
			for idx in 0..self.len() - 1 {
				if meth(&self[idx], &self[idx+1]) {
					self.swap(idx, idx + 1);
					repeat = true;
				}
			}
		}
	}
	pub fn empty(&self) -> bool {
		self.len() == 0
	}
}

impl<T, A: Allocator + Default> Vec<T, A> {
	pub fn from_optfn<F>(meth: F, amount: usize) -> Option<Vec<T, A>> where F: Fn(usize) -> Option<T> {
		let mut vec = Vec::new();
		for idx in 0..amount {
			vec.push_back((meth)(idx)?);
		}
		Some(vec)
	}

	fn grow(&mut self) {
		if self.capacity == 0 {

			let mut new_chunk = SharedRef::<VecChunk<T, A>>::new(
				VecChunk::<T, A>::new(4)
			);
			self.last = new_chunk.split();
			self.begin = new_chunk;
			self.capacity = 4;
		}
		while self.length >= self.capacity {
			let size = self.capacity * 2;
			let new_chunk = SharedRef::<VecChunk<T, A>>::new(
				VecChunk::<T, A>::new(size)
			);

			*(self.last.next_mut()) = new_chunk;

			self.capacity += size;
		}
	}

	pub fn push_back(&mut self, what: T) -> &mut T {
		let idx = self.length;
		self.length += 1;
		self.grow();

		let ptr = self.index_mut(idx);
		*ptr = what;
		ptr
	}
}

impl<T: Default, A: Allocator + Default> Vec<T, A> {
	pub fn resize(&mut self, target: usize) {
		let diff = target - self.length;
		let begin = self.length;
		self.length = target;
		self.grow();
		for idx in 0..diff {
			*self.index_mut(begin + idx) = T::default();
		}
	}
}

impl<T: Clone, A: Allocator + Default> Vec<T, A> {
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
	fn index_mut(&mut self, index: usize) -> &mut T {
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
	type IntoIter = VecIterMut<'vec, T, Vec<T, A>>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::new(self)
	}
}

impl<T: core::fmt::Display> FromIterator<T> for Vec<T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut vec = Vec::new();
		for i in iter {
			vec.push_back(i);
		}
		vec
	}
}

unsafe impl<T: Sync, A: Allocator> Sync for Vec<T, A> {}
