use core::hint;
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

	pub fn lock(&self) {
		while self.is_locked() {
			hint::spin_loop();
		}
		self.state.store(true, Ordering::Release);
	}

	pub fn unlock(&self) {
		self.state.store(false, Ordering::Release);
	}

	pub fn is_locked(&self) -> bool {
		self.state.load(Ordering::Relaxed)
	}
}
