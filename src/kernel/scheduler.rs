use crate::mm::{
	PageTable
};
use crate::{
	allocate,
	vec,
	call_asm
};
use crate::std::{
	PerCpu,
	Mutex,
	Vec,
	RAMAllocator,
	hltloop,
	Box,
	cli
};
use crate::hw::{
	cpu
};
use core::ops::IndexMut;
use cpu::GDT;

#[derive(PartialEq)]
pub enum ProcessPrivilage {
	KERNEL,
	USER
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProcessType {
	INIT,
	BOOT,
	NORMAL
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ProcessState {
	RUNNING,
	IDLE,
	KILLED
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskState {
	registers: [u64; 16],
	rip: u64,
	rflags: u64,
	cs: u64,
	ds: u64,
	uid: u64
}

pub struct Process {
	page_table: Box<PageTable>,
	task_state: TaskState,
	r#type: ProcessType,
	state: ProcessState,

	pid: u64
}

pub trait RipCast {
	fn ripcast(&self) -> u64;
}

static STATE_PER_CPU: PerCpu<TaskState> = PerCpu::new(TaskState::INVALID);
static PID_PER_CPU: PerCpu<u64> = PerCpu::new(u64::MAX);
static PROCESSES: Mutex<Vec<Process>> = Mutex::new(
	vec!()
);
static UID_COUNTER: Mutex<u64> = Mutex::new(0x0);

#[link(name="switcher")]
unsafe extern "sysv64" {
	fn jump_state(state: &TaskState) -> !;
	fn load_state(state: &TaskState, to_safe: &TaskState) -> !;
}

impl RipCast for u64 {
	fn ripcast(&self) -> u64 {
		*self
	}
}

impl RipCast for fn() -> ! {
	fn ripcast(&self) -> u64 {
		*self as *const u8 as u64 + crate::mm::kernel_offset()
	}
}

impl RipCast for fn() {
	fn ripcast(&self) -> u64 {
		*self as *const u8 as u64 + crate::mm::kernel_offset()
	}
}

impl TaskState {
	const INVALID: TaskState = TaskState {
		registers: [0x0; 16],
		rip: 0x0,
		rflags: 0x0,
		cs: 0x0,
		ds: 0x0,
		uid: 0xffffffffffffffff
	};
	pub fn new<R: RipCast>(s_base: u16, rsp: u64, rip: R) -> Option<TaskState> {
		*UID_COUNTER.lock() += 1;
		Some(TaskState {
			registers: [
				0x0, // RAX
				0x0, // RBX
				0x0, // RCX
				0x0, // RDX
				0x0, // RSI
				0x0, // RDI
				0x0, // RBP
				rsp, // RSP
				0x0, // R8
				0x0, // R9
				0x0, // R10
				0x0, // R11
				0x0, // R12
				0x0, // R13
				0x0, // R14
				0x0, // R15
			],
			rip: rip.ripcast(),
			rflags: 0x202,
			cs: s_base as u64,
			ds: (s_base + 0x8) as u64,
			uid: *UID_COUNTER.lock()
		})
	}

	pub fn jump(&self) -> ! {
		*STATE_PER_CPU.deref_mut() = *self;
		unsafe {
			call_asm!(jump_state, self);
		}
		panic!("Jumping failed.");
	}
	pub fn load(&self) {
		let last_state = &mut STATE_PER_CPU.deref_mut();
		*STATE_PER_CPU.deref_mut() = *self;
		unsafe {
			call_asm!(load_state, self, last_state)
		}
	}
}

impl PartialEq for TaskState {
	fn eq(&self, to_cmp: &TaskState) -> bool {
		self.uid == to_cmp.uid
	}
}

impl Process {
	const INIT_PROCESS: Process = Process {
		task_state: TaskState::INVALID,
		page_table: Box::NONE,
		r#type: ProcessType::INIT,
		state: ProcessState::RUNNING,
		pid: 0x0
	};
	pub fn new<EntryAddr: RipCast>(privilage: ProcessPrivilage, entry_addr: EntryAddr) -> Option<Process> {
		*UID_COUNTER.lock() += 1;
		let mut process = Process {
			page_table: PageTable::new_boxed(),
			task_state: TaskState::INVALID,
			r#type: ProcessType::NORMAL,
			state: ProcessState::IDLE,
			pid: *UID_COUNTER.lock()
		};
		process.page_table.init();
		process.task_state = TaskState::new(
			if privilage == ProcessPrivilage::KERNEL { GDT::CODE_SEG } else { 0x1b },
			0x0,
			entry_addr
		)?;
		Some(process)
	}
	pub fn new_with_stack<EntryAddr: RipCast>(privilage: ProcessPrivilage, entry_addr: EntryAddr) -> Option<Process> {
		let mut process = Process::new(privilage, entry_addr)?;
		process.assign_stack(allocate!(ptr_with_alloc, RAMAllocator, u8, 0x30000)? as u64 + 0x30000);
		Some(process)
	}
	pub fn spawn_with_stack<EntryAddr: RipCast>(privilage: ProcessPrivilage, entry_addr: EntryAddr) -> Option<u64> {
		let process = Self::new_with_stack(privilage, entry_addr)?;
		let pid = process.pid;
		PROCESSES.lock().push_back(process);
		Some(pid)
	}
	pub fn from_pid(pid: u64) -> Option<&'static mut Process> {
		let processes = PROCESSES.lock();
		Some(
			unsafe {
				PROCESSES.get().index_mut((*processes).into_iter().position(|p| p.pid == pid)?)
			}
		)
	}

	pub fn set_custom_offset(&mut self, offset: u64) -> &mut Process {
		self.task_state.rip += offset;
		self
	}

	// WARNING: Only call, when task is crated, otherwise the task will go to hell.
	pub fn assign_stack(&mut self, stack: u64) {
		self.task_state.registers[7] = stack;
	}
	pub fn disable_interrupts(&mut self) -> &mut Self {
		self.task_state.rflags &= !0x200;
		self
	}

	pub fn switch(&mut self) {
		if *PID_PER_CPU.deref_mut() == u64::MAX {
			self.jump();
		}
		let current_process = Self::from_pid(*PID_PER_CPU.deref_mut()).expect("Bug 28734");
		if current_process.state == ProcessState::RUNNING {
			current_process.state = ProcessState::IDLE;
		};
		self.state = ProcessState::RUNNING;
		PID_PER_CPU.set(self.pid);
		self.page_table.load();
		unsafe {
			crate::mm::set_current_page_table(self.page_table.deref_static());
		}
		cpu::lapic::LAPIC::end_of_interrupt();
		self.task_state.load()
	}

	pub fn jump(&mut self) -> ! {
		self.state = ProcessState::RUNNING;
		PID_PER_CPU.set(self.pid);
		self.task_state.jump()
	}

	pub fn kill(&mut self) -> ! {
		self.state = ProcessState::KILLED;
		loop {
			r#yield();
		}
	}

	pub unsafe fn set_pid(&mut self, pid: u64) -> &mut Self {
		self.pid = pid;
		self
	}
}

pub fn init_yield_timer() {
//	cpu::connect_signal(cpu::TIMER, r#yield);
}

pub fn r#yield() {
	if let Some(mut lock) = PROCESSES.try_lock() {
		cli();
		let process_idx = lock.into_iter().position(|process| process.state == ProcessState::IDLE && process.task_state != *STATE_PER_CPU.deref_mut());
		unsafe {
			PROCESSES.unlock();
		}

		if let Some(unwarped_process_idx) = process_idx {
			lock[unwarped_process_idx].switch();
		} else {
			hltloop();
		}
	}
}

pub fn exit_current_process() -> ! {
	Process::from_pid(*PID_PER_CPU.deref_mut()).expect("Bug 28734")
		.kill()
}
