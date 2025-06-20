use uefi::mem::memory_map::*;
use uefi;
use crate::std::{
	StackVec,
	Mutex
};
use crate::mm::align_size;

#[derive(Debug)]
pub enum BuddyError {
	_1gIndexFault, // Fails to find free 1G section (too much RAM?)
	_2mIndexFault, // Fails to find free 2M section
	UnknownSize
}

//#[derive(Copy, Clone)]
pub struct Buddy {
	buddies: [u64; 8],
	flags: u64,
	buddy_mask: u8
}
pub struct Buddy2M {
	buddy_base: Buddy,
	memory_offset: u64
}
pub struct Buddy1G {
	buddy_base: Buddy,
	children: [Buddy2M; 512]
}

type BuddyFlags = u64;
const FREE: BuddyFlags = 1 << 0;
const EMPTY: BuddyFlags = 1 << 1;
struct BuddyList {
	content: [Buddy1G; 16]
}
const BUDDY_SIZES: [usize;2] = [
	0x200000,
	0x40000000
];

static BUDDIES_MUTEX: Mutex<BuddyList> = Mutex::new(BuddyList::INVALID);

impl BuddyList {
	const INVALID: BuddyList = BuddyList {
		content: [Buddy1G::INVALID; 16]
	};
	pub fn add_1g_region(&mut self, addr: u64) -> Result<(), BuddyError> {
		let idx = self.position_empty();
		self.content[idx.ok_or(BuddyError::_1gIndexFault)?] = Buddy1G {
			children: core::array::from_fn(|offset| {
				Buddy2M::new(offset as u64 * 0x200000 + addr)
			}),
			buddy_base: Buddy::FREE
		};
		Ok(())
	}
	pub fn add_2m_region(&mut self, addr: u64) -> Result<(), BuddyError> {
		let _1g_buddy: &mut Buddy1G = self.find_non_full().ok_or(BuddyError::_1gIndexFault)?;
		let _2m_buddy_idx: usize = _1g_buddy.position_2m_empty().ok_or(BuddyError::_2mIndexFault)?;
		_1g_buddy.buddy_base.flags = FREE;
		_1g_buddy.children.as_mut()[_2m_buddy_idx] = Buddy2M::new(addr);
		_1g_buddy.buddy_base.free( _2m_buddy_idx);
		Ok(())
	}
	pub fn position(&self, meth: for<'a> fn(&'a Buddy1G) -> bool) -> Option<usize> {
		self.content.iter().position(meth)
	}
	pub fn find_mut(&mut self, meth: for<'a> fn(&'a Buddy1G) -> bool) -> Option<&mut Buddy1G> {
		let idx = self.position(meth)?;
		Some(self.content.each_mut()[idx])
	}
	pub fn position_empty(&self) -> Option<usize> {
		self.position(|buddy| buddy.buddy_base.flags & EMPTY != 0)
	}
	pub fn find_empty(&mut self) -> Option<&mut Buddy1G> {
		self.find_mut(|buddy| buddy.buddy_base.flags & EMPTY != 0)
	}
	pub fn find_non_full(&mut self) -> Option<&mut Buddy1G> {
		self.find_mut(|buddy| buddy.buddy_base.is_non_full())
	}
	pub fn find_free(&mut self) -> Option<&mut Buddy1G> {
		self.find_mut(|buddy| buddy.buddy_base.is_free())
	}
}

impl Buddy {
	const INVALID: Buddy = Buddy {
		buddies: [0x0; 8],
		flags: EMPTY,
		buddy_mask: 0x0
	};
	const FREE: Buddy = Buddy {
		buddies: [u64::MAX; 8],
		flags: FREE,
		buddy_mask: u8::MAX
	};
	pub fn allocate_index(&mut self) -> Option<u64> {
		let buddyidx = self.buddy_mask.trailing_zeros() as usize;
		if buddyidx == 8 {
			return None;
		}
		let idx = self.buddies[buddyidx].trailing_zeros() as u64;
		self.buddies[buddyidx] &= !(1 << idx);
		if self.buddies[buddyidx] == 0 {
			self.buddy_mask &= !(1 << buddyidx);
			if self.buddy_mask == 0 {
				self.flags &= !(FREE | EMPTY);
			}
		}

		Some(idx + buddyidx as u64 * 64)
	}
	pub fn free(&mut self, idx: usize) {
		self.buddies[idx / 64] |= 1 << (idx % 64);
		self.buddy_mask |= 1 << (idx / 64);
	}
	#[inline]
	pub fn is_non_full(&self) -> bool {
		self.flags & (EMPTY | FREE) != 0
	}
	#[inline]
	pub fn is_free(&self) -> bool {
		self.flags & FREE != 0
	}
}

impl Buddy2M {
	const INVALID: Buddy2M = Buddy2M {
		buddy_base: Buddy::INVALID,
		memory_offset: 0x1111111111111111
	};
	pub fn new(addr: u64) -> Buddy2M {
		Buddy2M {
			buddy_base: Buddy::FREE,
			memory_offset: addr
		}
	}

	pub fn allocate_4k(&mut self) -> Option<u64> {
		let idx = self.buddy_base.allocate_index();
		Some(self.memory_offset + idx? * 0x1000 /* addr */)
	}

	pub fn reserve(&mut self) {
		self.buddy_base.buddy_mask = 0x0;
		self.buddy_base.flags ^= FREE | EMPTY;
	}
}

impl Buddy1G {
	const INVALID: Buddy1G = Buddy1G {
		children: [Buddy2M::INVALID; 512],
		buddy_base: Buddy::INVALID
	};
	pub fn allocate_2m(&mut self) -> Option<u64> {
		let idx = self.buddy_base.allocate_index()? as usize;
		self.children[idx].reserve();
		Some(self.children[idx].memory_offset /* addr */)
	}
	pub fn allocate_4k(&mut self) -> Option<u64> {
		let idx = self.position_2m_non_full();
		self.children.each_mut()[idx?].allocate_4k()
	}
	pub fn position_2m_empty(&self) -> Option<usize> {
		self.children.iter().position(|buddy| buddy.buddy_base.flags & EMPTY != 0)
	}
	pub fn position_2m_non_full(&self) -> Option<usize> {
		self.children.iter().position(|buddy| buddy.buddy_base.is_non_full())
	}
}

pub fn allocate_aligned(size: usize) -> Option<u64> {
	let mut buddies = BUDDIES_MUTEX.lock();
	let _1g_buddy: &mut Buddy1G = buddies.find_non_full()?;

	match size {
		0x1000 => _1g_buddy.allocate_4k(),
		0x200000 => _1g_buddy.allocate_2m(),
		_ => None // Unknown allocation size.
	}
}

pub fn allocate(size: usize) -> Option<StackVec<u64, 0x200>> {
	let alloc_size = align_size(size);

	let vec = StackVec::from_optfn(
		|_| allocate_aligned(alloc_size),
		(size / alloc_size) + 1
	);
	vec
}

pub fn add_region(addr: u64, size: usize) -> Result<(), BuddyError> {
	let mut buddies = BUDDIES_MUTEX.lock();
	match size {
		0x40000000 => buddies.add_1g_region(addr),
		0x200000 => buddies.add_2m_region(addr),
		_ => Err(BuddyError::UnknownSize)
	}
}

pub fn add_regions(mut addr: u64, mut size: usize) -> Result<(), BuddyError> {
	for s in BUDDY_SIZES {
		addr += s as u64 - (addr as usize % s) as u64;
		while size > s {
			add_region(addr, s)?;
			size -= s;
			addr += s as u64;
		}
	}
	Ok(())
}

pub fn add_memory_map(memory_map: &MemoryMapOwned) -> Result<(), BuddyError> {
	for entry in memory_map.entries().filter(|&d| d.ty == MemoryType::CONVENTIONAL || d.ty == MemoryType::BOOT_SERVICES_CODE) {
		add_regions(entry.phys_start, entry.page_count as usize * uefi::boot::PAGE_SIZE)?;
	}
	Ok(())
}
