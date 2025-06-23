use super::super::{
	Allocation,
	Allocator,
	SharedRef
};
use core::ops::{
	Index,
	IndexMut
};

pub struct VecChunk<T, A: Allocator> {
	memory: Allocation<T, A>,
	capacity: usize,
	next: SharedRef<VecChunk<T, A>>
}

impl<T, A: Allocator> VecChunk<T, A> {
	pub fn new(capacity: usize) -> VecChunk<T, A> {
		VecChunk {
			memory: Allocation::<T, A>::new(capacity)
				.expect("Failed to allocate memory for VecChunk."),
			capacity: capacity,
			next: SharedRef::EMPTY
		}
	}

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
		unsafe {
			*self.memory.as_ptr().wrapping_add(pos) = what;
		}
	}
}

impl<T, A: Allocator> Index<usize> for VecChunk<T, A> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		if self.capacity <= index {
			panic!("Attempt to index {} in a vecchunk with length {}.", index, self.capacity);
		}
		unsafe {
			&mut *self.memory.as_ptr().wrapping_add(index)
		}
	}
}

impl<T, A: Allocator> IndexMut<usize> for VecChunk<T, A> {
	fn index_mut(&mut self, index: usize) -> &mut T {
		if self.capacity <= index {
			panic!("Attempt to index {} in a vecchunk with length {}.", index, self.capacity);
		}
		unsafe {
			&mut *self.memory.as_ptr().wrapping_add(index)
		}
	}
}
/*
impl<T: Clone, A: Allocator> IntoIterator for Vec<T, A> {
	type Item = T;
	type IntoIter = VecIterNonRef<T, Vec<T, A>>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::new(self)
	}
}
*/
