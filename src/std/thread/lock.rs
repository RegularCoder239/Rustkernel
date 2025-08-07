use core::sync::atomic::{
	AtomicBool,
	Ordering
};

pub struct Lock {
	state: AtomicBool
}

impl Lock {
	pub const fn new() -> Lock {
		Lock {
			state: AtomicBool::new(false)
		}
	}
	pub const fn new_locked() -> Lock {
		Lock {
			state: AtomicBool::new(true)
		}
	}

	pub fn lock(&self) {
		while self.state.swap(true, Ordering::Acquire) {
			crate::std::wait();
		}
		self.state.store(true, Ordering::Release);
	}

	pub fn unlock(&self) {
		self.state.store(false, Ordering::Release);
	}

	pub fn is_locked(&self) -> bool {
		self.state.load(Ordering::Relaxed)
	}

	pub fn wait(&self) {
		while self.is_locked() {
			crate::std::wait();
		}
	}
}

impl Clone for Lock {
	fn clone(&self) -> Self {
		if self.is_locked() {
			Self::new_locked()
		} else {
			Self::new()
		}
	}
}
