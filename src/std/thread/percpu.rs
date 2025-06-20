use core::{
	cell::UnsafeCell,
	slice::Iter
};
use super::cpucore;
use super::super::LazyBox;

pub struct PerCpu<T> {
	content: UnsafeCell<[T; 32]>
}

pub struct PerCpuLazy<T>(PerCpu<LazyBox<T>>);

impl<T: Clone+Copy> PerCpu<T> {
	pub const fn new(value: T) -> PerCpu<T> {
		PerCpu {
			content: UnsafeCell::new([value; 32])
		}
	}
}

impl<T> PerCpu<T> {
	pub fn unwrap(&self) -> &mut [T; 32] {
		unsafe {
			&mut *self.content.get()
		}
	}
	pub fn deref(&self) -> &T {
		self.unwrap().each_ref()[cpucore() as usize]
	}
	pub fn deref_mut(&self) -> &mut T {
		self.unwrap().each_mut()[cpucore() as usize]
	}
	pub fn iter(&self) -> Iter<'_, T> {
		self.unwrap().iter()
	}
	pub fn set(&self, content: T) {
		self.unwrap()[cpucore() as usize] = content;
	}
}

unsafe impl<T> Sync for PerCpu<T> {}

impl<T: Copy> PerCpuLazy<T> {
	pub const fn new(meth: fn() -> T) -> PerCpuLazy<T> {
		PerCpuLazy(
			PerCpu::new(LazyBox::new(meth))
		)
	}
}

impl<T> PerCpuLazy<T> {
	pub fn deref(&self) -> &T {
		self.0.deref_mut().get()
	}
	pub fn deref_mut(&self) -> &mut T {
		self.0.deref_mut().get_mut()
	}
	pub fn iter(&self) -> Iter<'_, LazyBox<T>> {
		self.0.iter()
	}
	pub fn set(&self, content: T) {
		self.0.deref_mut().set(content);
	}
}

unsafe impl<T> Sync for PerCpuLazy<T> {}
