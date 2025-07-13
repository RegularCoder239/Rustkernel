use super::{
	lock::Lock,
	super::MutableCell
};
use core::ops::{
	Deref,
	DerefMut
};

pub struct Mutex<T> {
	lock: Lock,
	rdlock: Lock,
	content: MutableCell<T>
}

pub struct MutexGuard<'a, T> {
	mutex: &'a Mutex<T>
}
pub struct OptMutexGuard<'a, T> {
	mutex: &'a Mutex<Option<T>>
}

impl<T> Mutex<T> {
	pub const fn new(value: T) -> Mutex<T> {
		Mutex {
			lock: Lock::new(),
			rdlock: Lock::new(),
			content: MutableCell::new(value)
		}
	}
	pub const fn new_rdfused(value: T) -> Mutex<T> {
		Mutex {
			lock: Lock::new(),
			rdlock: Lock::new_locked(),
			content: MutableCell::new(value)
		}
	}

	pub fn is_locked(&self) -> bool {
		self.lock.is_locked()
	}

	pub fn lock(&self) -> MutexGuard<'_, T> {
		self.lock.lock();
		MutexGuard::new(self)
	}

	pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
		if self.lock.is_locked() {
			None
		} else {
			self.lock.lock();
			Some(MutexGuard::new(self))
		}
	}

	pub fn unfuse(&self) {
		self.rdlock.unlock();
	}

	fn unlock(&self) {
		self.lock.unlock()
	}

	fn get(&self) -> &mut T {
		self.content.deref_mut()
	}
}

impl<T: Sync> Mutex<T> {
	pub fn read(&self) -> &T {
		self.rdlock.wait();
		self.get()
	}
}

impl<T: Sync> Deref for Mutex<T> {
	type Target = T;
	fn deref(&self) -> &T {
		self.read()
	}
}

impl<T> Mutex<Option<T>> {
	pub fn lock_opt(&self) -> OptMutexGuard<'_, T> {
		while self.get().is_none() {
			crate::std::wait();
		}
		OptMutexGuard::new(self)
	}
}

unsafe impl<T> Sync for Mutex<T> {}

impl<'mutex, T> MutexGuard<'mutex, T> {
	pub fn new(mutex: &'mutex Mutex<T>) -> MutexGuard<'mutex, T> {
		MutexGuard {
			mutex: mutex
		}
	}
}

impl<'mutex, T> Deref for MutexGuard<'mutex, T> {
	type Target = T;

	fn deref(&self) -> &'mutex T {
		self.mutex.get()
	}
}

impl<'mutex, T> DerefMut for MutexGuard<'mutex, T> {
	fn deref_mut(&mut self) -> &'mutex mut T {
		self.mutex.get()
	}
}

impl<T> Drop for MutexGuard<'_, T> {
	fn drop(&mut self) {
		self.mutex.unlock()
	}
}

impl<'mutex, T> OptMutexGuard<'mutex, T> {
	pub fn new(mutex: &'mutex Mutex<Option<T>>) -> OptMutexGuard<'mutex, T> {
		OptMutexGuard {
			mutex: mutex
		}
	}

	pub fn get_opt(&self) -> Option<&'mutex mut T> {
		self.mutex.get().as_mut()
	}
}

impl<'mutex, T> Deref for OptMutexGuard<'mutex, T> {
	type Target = T;

	fn deref(&self) -> &'mutex T {
		self.get_opt().unwrap()
	}
}

impl<'mutex, T> DerefMut for OptMutexGuard<'mutex, T> {
	fn deref_mut(&mut self) -> &'mutex mut T {
		self.get_opt().unwrap()
	}
}

impl<T> Drop for OptMutexGuard<'_, T> {
	fn drop(&mut self) {
		self.mutex.unlock()
	}
}
