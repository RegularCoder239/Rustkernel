use super::lock::Lock;
use super::super::LazyBox;
use core::cell::UnsafeCell;
use core::ops::{
	Deref,
	DerefMut
};

pub struct LazyMutex<T> {
	lock: Lock,
	inner: UnsafeCell<LazyBox<T>>
}

pub struct LazyMutexGuard<'a, T> {
	mutex: &'a LazyMutex<T>
}

impl<T> LazyMutex<T> {
	pub const fn new(meth: fn() -> T) -> LazyMutex<T> {
		LazyMutex {
			lock: Lock::new(),
			inner: UnsafeCell::new(LazyBox::new(meth))
		}
	}

	pub fn lock(&self) -> LazyMutexGuard<'_, T> {
		self.lock.lock();
		LazyMutexGuard::new(self)
	}

	fn unlock(&self) {
		self.lock.unlock()
	}

	pub fn is_initalized(&self) -> bool {
		self.get().is_initalized()
	}

	fn get(&self) -> &mut LazyBox<T> {
		unsafe {
			&mut *self.inner.get()
		}
	}
}

unsafe impl<T> Sync for LazyMutex<T> {}

impl<'mutex, T> LazyMutexGuard<'mutex, LazyBox<T>> {
	pub fn new(mutex: &'mutex LazyMutex<T>) -> LazyMutexGuard<'mutex, T> {
		LazyMutexGuard {
			mutex: mutex
		}
	}
}

impl<'mutex, T: 'static> Deref for LazyMutexGuard<'mutex, T> {
	type Target = T;

	fn deref(&self) -> &T {
		self.mutex.get().get()
	}
}

impl<'mutex, T: 'static> DerefMut for LazyMutexGuard<'mutex, T> {
	fn deref_mut(&mut self) -> &'mutex mut T {
		self.mutex.get().get_mut()
	}
}

impl<T> Drop for LazyMutexGuard<'_, T> {
	fn drop(&mut self) {
		self.mutex.unlock()
	}
}
