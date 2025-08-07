use crate::mm::{
	PageTable
};
use crate::{
	call_asm
};
use crate::std::{
	PerCpu,
	Mutex,
	SharedRef,
	Vec,
	VecBase,
	RAMAllocator,
	Allocator,
	hltloop,
	Box,
	cli,
	sti
};
use crate::hw::{
	cpu
};
use core::{
	arch::asm,
	ops::Index
};
use cpu::GDT;

#[derive(PartialEq)]
pub enum ProcessPrivilage {
	KERNEL,
	USER
}

#[derive(Clone, Copy, PartialEq)]
#[derive(Debug)]
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

pub enum ProcessFlags {
	GraphicManager = 1 << 0
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskState {
	pub registers: [u64; 16],
	pub rip: u64,
	pub rflags: u64,
	pub cs: u64,
	pub ds: u64,
	pub gs: u64,
	pub uid: u64
}

pub struct ProcessMapping {
	virt_addr: u64,
	content: Box<[u8]>,
	flags: u64
}

pub struct Process {
	pub mappings: Vec<ProcessMapping>,
	pub mapping_pages: Vec<ProcessMapping>,
	pub page_table: Box<Mutex<PageTable>>,
	task_state: TaskState,
	pub r#type: ProcessType,
	state: ProcessState,
	pub flags: u64,

	pub pid: u64
}

pub trait RipCast {
	fn ripcast(&self) -> u64;
}

static STATE_PER_CPU: PerCpu<TaskState> = PerCpu::new(TaskState::INVALID);
static PID_PER_CPU: PerCpu<u64> = PerCpu::new(u64::MAX);
static PROCESSES: Mutex<Vec<Mutex<Process>>> = Mutex::new(Vec::new());
static UID_COUNTER: Mutex<u64> = Mutex::new(0x0);
static NEXT_PROCESS: Mutex<usize> = Mutex::new(0);

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

impl TaskState {
	const INVALID: TaskState = TaskState {
		registers: [0x0; 16],
		rip: 0x0,
		rflags: 0x0,
		cs: 0x0,
		ds: 0x0,
		gs: 0x0,
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
				0x218d628eb863ee89, // R15 (Magic)
			],
			rip: rip.ripcast(),
			rflags: 0x202,
			cs: s_base as u64,
			ds: (s_base + 0x8) as u64,
			gs: 0x0,
			uid: *UID_COUNTER.lock()
		})
	}

	pub fn jump(&self) -> ! {
		*STATE_PER_CPU.deref_mut() = *self;

		call_asm!(jump_state, self);

		panic!("Jumping failed.");
	}
	pub fn load(&self) {
		let last_state = &mut STATE_PER_CPU.deref_mut();
		*STATE_PER_CPU.deref_mut() = *self;
		call_asm!(load_state, self, last_state)
	}
}

impl PartialEq for TaskState {
	fn eq(&self, to_cmp: &TaskState) -> bool {
		self.uid == to_cmp.uid
	}
}

impl Process {
	pub fn new<EntryAddr: RipCast>(privilage: ProcessPrivilage, entry_addr: EntryAddr) -> Option<Process> {
		*UID_COUNTER.lock() += 1;
		let mut process = Process {
			mappings: Vec::new(),
			mapping_pages: Vec::new(),
			page_table: PageTable::new_boxed(),
			task_state: TaskState::INVALID,
			r#type: ProcessType::NORMAL,
			state: ProcessState::IDLE,
			flags: 0,
			pid: *UID_COUNTER.lock()
		};
		process.page_table.lock().init();
		process.task_state = TaskState::new(
			if privilage == ProcessPrivilage::KERNEL { GDT::CODE_SEG } else { 0x1b },
			0x0,
			entry_addr
		)?;
		Some(process)
	}
	pub fn new_with_stack<EntryAddr: RipCast>(privilage: ProcessPrivilage, entry_addr: EntryAddr, stack_size: usize) -> Option<Process> {
		let mut process = Process::new(privilage, entry_addr)?;
		process.assign_stack(RAMAllocator::default().allocate::<u8>(stack_size)? as u64 + stack_size as u64);
		Some(process)
	}
	pub fn spawn_init_process<EntryAddr: RipCast>(entry_addr: EntryAddr) -> ! {
		let mut process = Process::new_with_stack(ProcessPrivilage::KERNEL, entry_addr, 0x25000).expect("Failed to crate boot setup process.");
		process.set_pid(u64::MAX);
		process.task_state.rflags = 0x2;

		process.switch_init()
	}
	pub fn spawn_with_stack<EntryAddr: RipCast>(privilage: ProcessPrivilage, entry_addr: EntryAddr) -> Option<u64> {
		let process = Self::new_with_stack(privilage, entry_addr, 0x30000)?;
		let pid = process.pid;
		PROCESSES.lock().push_back(Mutex::new(process));
		Some(pid)
	}
	pub fn from_pid(pid: u64) -> Option<&'static Mutex<Process>> {
		let processes = PROCESSES.read();
		Some(
			PROCESSES.read().index(
				processes.into_iter().position(|p| p.lock().pid == pid)?
			)
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

	pub fn switch(&'static mut self) {
		if let Some(current_process) = current_process() {
			let mut current_process_lock = current_process.lock();
			if current_process_lock.state == ProcessState::RUNNING {
				current_process_lock.state = ProcessState::IDLE;
			}
		}
		self.state = ProcessState::RUNNING;
		PID_PER_CPU.set(self.pid);
		self.page_table.load();

		crate::mm::set_current_page_table(&self.page_table);
		cpu::lapic::LAPIC::end_of_interrupt();

		self.task_state.load()
	}

	fn switch_init(&mut self) -> ! {
		self.state = ProcessState::RUNNING;
		PID_PER_CPU.set(self.pid);
		self.task_state.jump()
	}

	fn set_pid(&mut self, pid: u64) -> &mut Self {
		self.pid = pid;
		self
	}

	pub fn spawn(self) {
		PROCESSES.lock().push_back(Mutex::new(self));
	}

	pub fn add_mapping(&mut self, virt_addr: u64, content: Box<[u8]>, flags: u64) {
		assert!(virt_addr & 0xfff == 0, "Attempt to add unaligned mapping.");
		assert!(content.alloc_len() & 0xfff == 0, "Mapped content has an unaligned size.");

		let mapping = self.mappings.push_back(ProcessMapping {
			virt_addr,
			content,
			flags
		});
		self.page_table.lock().map(virt_addr, mapping.content.physical_address(), mapping.content.alloc_len(), flags);
	}

	pub fn add_unaligned_mapping(&mut self, mut virt_addr: u64, content: &[u8], mut content_offset: usize, mut content_limit: usize, mut flags: u64) {
	//	while content_limit > 0 {
			let mut mapping_page = if let Some(mut mapping_page) = (&mut self.mapping_pages).into_iter().find(|page| page.virt_addr == virt_addr & !0xfff) {
				mapping_page
			} else {
				let mut page = self.mapping_pages.push_back(ProcessMapping {
					virt_addr: virt_addr & !0xfff,
					content: Box::new_sized(0x1000),
					flags: flags
				});
				self.page_table.lock().map(virt_addr, page.content.physical_address(), 0x1000, flags);
				page
			};
			if mapping_page.flags != flags {
				crate::std::log::warn!("Cannot assign unaligned mapping flags. Using existing flags.");
				flags = mapping_page.flags;
			}
			let tocopy = content_limit.min(0x1000 - (content_offset & 0xfff));
			mapping_page.content.copy_from_slice_with_offset(content_offset, content, content_limit, (virt_addr & 0xfff) as usize);
			content_limit -= tocopy;
			content_offset += tocopy;
			virt_addr += tocopy as u64;
//		}
	}
}

impl Mutex<Process> {
	pub fn mutex_switch(&'static self) {
		if let Some(current_process) = current_process() {
			let mut current_process_lock = current_process.lock();
			if current_process_lock.state == ProcessState::RUNNING {
				current_process_lock.state = ProcessState::IDLE;
			}
		}
		{
			let mut locked = self.lock();
			locked.state = ProcessState::RUNNING;
		}

		PID_PER_CPU.set(self.pid);

		self.page_table.load();

		crate::mm::set_current_page_table(&self.page_table);
		cpu::lapic::LAPIC::end_of_interrupt();

		self.task_state.load()
	}
	pub fn kill(&self) -> ! {
		self.lock().state = ProcessState::KILLED;
		loop {
			r#yield();
		}
	}
	pub fn assign_flags(&self, flags: ProcessFlags) {
		self.lock().flags |= flags as u64;
	}
}

unsafe impl Sync for Process {}

pub fn init_yield_timer() {
	cpu::connect_signal(cpu::TIMER, |_| r#yield());
}

pub fn r#yield() {
	if PROCESSES.is_locked() {
		return;
	}
	cli();
	let idx = {
		let mut next_process = NEXT_PROCESS.lock();
		let org_next_process = *next_process;
		let processes = PROCESSES.read();
		let mut processlock = &processes[*next_process];

		while !((processlock.state == ProcessState::IDLE) && processlock.task_state != *STATE_PER_CPU.deref_mut()) {
			*next_process = (*next_process + 1) % processes.len();
			if *next_process == org_next_process {
				// No idle process found.
				sti();
				hltloop();
			}
			processlock = &processes[*next_process];
		}
		*next_process
	};

	let process = &PROCESSES.read()[idx];
	Process::from_pid(process.pid).as_ref().unwrap().mutex_switch();
}

pub fn current_process() -> Option<&'static Mutex<Process>> {
	Process::from_pid(*PID_PER_CPU.deref_mut())
}

pub fn current_task_state() -> &'static TaskState {
	STATE_PER_CPU.deref()
}

pub fn exit_current_process() -> ! {
	current_process()
		.expect("Attempt to kill boot setup task.")
		.kill()
}
