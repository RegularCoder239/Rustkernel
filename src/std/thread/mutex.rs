use super::lock::Lock;
use core::cell::UnsafeCell;
use core::ops::{
	Deref,
	DerefMut
};

pub struct Mutex<T> {
	lock: Lock,
	content: UnsafeCell<T>
}

pub struct MutexGuard<'a, T> {
	mutex: &'a Mutex<T>
}

impl<T> Mutex<T> {
	pub const fn new(value: T) -> Mutex<T> {
		Mutex {
			lock: Lock::new(),
			content: UnsafeCell::new(value)
		}
	}

	pub fn lock(&self) -> MutexGuard<T> {
		self.lock.lock();
		MutexGuard::new(self)
	}

	pub fn unlock(&self) {
		self.lock.unlock()
	}
}

unsafe impl<T> Sync for Mutex<T> {

}

impl<'mutex, T> MutexGuard<'mutex, T> {
	pub fn new(mutex: &'mutex Mutex<T>) -> MutexGuard<'mutex, T> {
		MutexGuard {
			mutex: mutex
		}
	}
}

impl<T> Deref for MutexGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe {
			&*self.mutex.content.get()
		}
	}
}

impl<T> DerefMut for MutexGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *self.mutex.content.get()
		}
	}
}

impl<T> Drop for MutexGuard<'_, T> {
	fn drop(&mut self) {
		self.mutex.unlock()
	}
}
