// Using XOR-Shift

use core::sync::atomic::{
	AtomicU64,
	Ordering
};

static STATE: AtomicU64 = AtomicU64::new(0x61c5b1c3a7628ee7);

pub fn random() -> u64 {
	let mut state = STATE.load(Ordering::Relaxed);
	state ^= state << 13;
	state ^= state >> 7;
	state ^= state << 17;
	STATE.store(state, Ordering::Relaxed);
	state
}
