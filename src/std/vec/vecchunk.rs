use super::super::{
	Allocator,
	SharedRef,
	Box
};
use core::ops::{
	Index,
	IndexMut
};

pub struct VecChunk<T, A: Allocator> {
	memory: Box<[T], A>,
	capacity: usize,
	next: SharedRef<VecChunk<T, A>>
}

impl<T, A: Allocator + Default> VecChunk<T, A> {
	pub fn new(capacity: usize) -> VecChunk<T, A> {
		VecChunk {
			memory: Box::new_sized(capacity * core::mem::size_of::<T>()),
			capacity: capacity,
			next: SharedRef::EMPTY
		}
	}
}

impl<T, A: Allocator> VecChunk<T, A> {
	pub fn capacity(&self) -> usize {
		self.capacity
	}

	pub fn next(&self) -> &SharedRef<VecChunk<T, A>> {
		&self.next
	}
	pub fn next_mut(&mut self) -> &mut SharedRef<VecChunk<T, A>> {
		&mut self.next
	}

	pub fn push(&mut self, what: T, pos: usize) {
		self.memory.as_slice_mut()[pos] = what;
	}
}

impl<T, A: Allocator> Index<usize> for VecChunk<T, A> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		if self.capacity <= index {
			panic!("Attempt to index {} in a vecchunk with length {}.", index, self.capacity);
		}

		self.memory.as_slice().index(index)
	}
}

impl<T, A: Allocator> IndexMut<usize> for VecChunk<T, A> {
	fn index_mut(&mut self, index: usize) -> &mut T {
		if self.capacity <= index {
			panic!("Attempt to index {} in a vecchunk with length {}.", index, self.capacity);
		}

		self.memory.as_slice_mut().index_mut(index)
	}
}
