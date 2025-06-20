use core::ops::{
	Index,
	IndexMut,
	DerefMut
};
use crate::allocate;
use core::{
	mem,
	marker::PhantomData
};
use super::{
	SharedRef,
	Allocator,
	RAMAllocator
};

pub trait VecBase: Index<usize> + IndexMut<usize> + IntoIterator {
	fn len(&self) -> usize;
	fn is_empty(&self) -> bool;
}

pub struct FixedVec<T> {
	content: [T; 512],
	size: usize
}

pub struct CustomVec<T, A> {
	begin: SharedRef<CustomVecChunk<T>>,
	last: SharedRef<CustomVecChunk<T>>,
	pub length: usize,
	chunk_item_count: usize,
	index: usize,
	allocator: A
}

pub struct Vec<T>(CustomVec<T, RAMAllocator>);

pub struct CustomVecChunk<T> {
	content: NonNull<T>,
	amount: usize,
	next: SharedRef<CustomVecChunk<T>>
}

pub struct VecIter<'vec, T, V: VecBase> {
	vector: *mut V,
	index: usize,
	phantom: PhantomData<T>
}

impl<T> Vec<T> {
	pub const fn new_empty() -> Vec<T> {
		Vec {
			0: CustomVec::new_empty()
		}
	}

	pub fn push_back(&mut self, item: T) {
		self.0.push_back(item);
	}

	pub fn len(&self) -> usize {
		self.0.length
	}
}

impl<T, A: Allocator> CustomVec<T, A> {
	pub const fn new_empty() -> CustomVec<T, A> {
		CustomVec {
			begin: SharedRef::EMPTY,
			last: SharedRef::EMPTY,
			length: 0,
			chunk_item_count: {
				let mut page_size = 0x1000;
				while page_size < mem::size_of::<T>() {
					page_size *= 0x200;
				}
				page_size / mem::size_of::<T>()
			},
			index: 0,
			allocator: Allocator::DEFAULT
		}
	}

	pub fn push_back(&mut self, item: T) {
		if self.length % self.chunk_item_count == 0 {
			self.grow();
		}
		self.last.unwrap_mut().expect("Failed to grow Vec.").push_back(item);
		self.length += 1;
	}

	fn grow(&mut self) {
		let mut new_chunk = SharedRef::<CustomVecChunk<T>>::new_alloc::<A>(
			CustomVecChunk::<T>::new::<A>(self.chunk_item_count)
		);

		if let Some(last) = self.last.unwrap_mut() {
			(*last).next = new_chunk.split();
		} else {
			self.begin = new_chunk.split();
			self.last = new_chunk;
		}
	}
}

impl<T: Sized + Clone, A: Allocator> CustomVec<T, A> {
	pub fn new_with(array: &[T]) -> CustomVec<T, A> {
		let mut vec = CustomVec::<T, A>::new_empty();
		for e in array { vec.push_back(e.clone()); }
		vec
	}
}

impl<'a, T, A: Allocator> VecBase for CustomVec<T, A> {
	fn is_empty(&self) -> bool {
		self.length == 0
	}

	fn len(&self) -> usize {
		self.length
	}
}

impl<T, A: Allocator> Index<usize> for CustomVec<T, A> {
	type Output = T;
	fn index(&self, mut index: usize) -> &T {

		let mut chunk: &CustomVecChunk<T> = self.begin.unwrap().expect("Attempt to index empty Vec.");
		assert!(index < self.length, "Attempted index at position {} in a Vec of size {}.", index, self.length);

		while index >= chunk.amount {
			index -= chunk.amount;
			chunk = chunk.next.unwrap().expect("Attempted OOB Vec access.");
		}

		unsafe {
			&*chunk.content.wrapping_add(index)
		}
	}
}

impl<T, A: Allocator> IndexMut<usize> for CustomVec<T, A> {
	fn index_mut(&mut self, mut index: usize) -> &mut T {
		let mut chunk: &CustomVecChunk<T> = self.begin.unwrap().expect("Attempt to index empty Vec.");
		assert!(index < self.length, "Attempted index at position {} in a Vec of size {}.", index, self.length);

		while index >= chunk.amount {
			index -= chunk.amount;
			chunk = chunk.next.unwrap().expect("Attempted OOB Vec access.");
		}

		unsafe {
			&mut *chunk.content.wrapping_add(index)
		}
	}
}

impl<'vec, T: 'vec, A: Allocator> IntoIterator for CustomVec<T, A> {
	type Item = &'vec T;
	type IntoIter = VecIter<T, CustomVec<T, A>>;

	fn into_iter(self) -> VecIter<'vec, T, CustomVec<T, A>> {
		VecIter {
			vector: core::mem::ManuallyDrop::new(self).as_mut_ptr(),
			index: 0,
			phantom: PhantomData
		}
	}
}

impl<'vec, T: 'vec, V: VecBase<Output = T> + 'vec> Iterator for VecIter<'vec, T, V> {
	type Item = &T;
	fn next(&mut self) -> Option<&'vec T> {
		if self.index >= *self.vector.len() {
			self.index = 0;
			None
		} else {
			let result = Some(&'vec (*self.vector)[self.index]);
			self.index += 1;
			result
		}
	}
}

impl<T> CustomVecChunk<T> {
	fn new<A: Allocator>(amount: usize) -> CustomVecChunk<T> {
		CustomVecChunk {
			content: allocate!(ptr_with_alloc, A, T, amount).expect("Failed to allocate vectorchunk."),
			amount: 0,
			next: SharedRef::EMPTY
		}
	}

	fn push_back(&mut self, content: T) {
		unsafe {
			*self.content.wrapping_add(self.amount) = content;
		}
		self.amount += 1;
	}
}

impl<T: Default + core::marker::Copy> FixedVec<T> {
	pub fn new(size: usize) -> FixedVec<T> {
		FixedVec {
			content: [T::default(); 512],
			size
		}
	}

	pub fn with(array: &[T]) -> FixedVec<T> {
		let mut vec = Self::new(array.len());
		for idx in 0..array.len() {
			vec[idx] = array[idx];
		}
		vec
	}

	pub fn from_optfn<F>(meth: F, size: usize) -> Option<FixedVec<T>>
	where F: Fn(usize) -> Option<T> {
		let mut new_vec = FixedVec::new(size);
		for idx in 0..size {
			new_vec[idx] = meth(idx)?;
		}
		Some(new_vec)
	}

	pub fn as_slice(&self) -> &[T] {
		self.content.as_slice()
	}
}

impl<T> Index<usize> for FixedVec<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		assert!(index < self.size, "Attempt to access index {} on a FixedVec with size {}", index, self.size);
		self.content.each_ref()[index]
	}
}

impl<T> IndexMut<usize> for FixedVec<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		assert!(index < self.size, "Attempt to access index {} on a FixedVec with size {}", index, self.size);
		self.content.each_mut()[index]
	}
}

#[macro_export]
macro_rules! custom_vec {
	(empty, $type: ty, $alloc: ty) => {
		CustomVec::<$type, $alloc>::new_empty()
	};
	($alloc: ty, $($elements: expr),+ $(,)?) => {
		CustomVec::<_, $alloc>::new_with([$($elements),+].as_slice())
	}
}
