use super::super::{
	HeaderType0,
	super::PhysicalDisk,
	super::add_disk,
	super::Sector
};
use crate::hw::pci::{
	DeviceTrait,
	drive::Box
};
use crate::std::{
	Vec,
	VecBase,
	Mutex
};
use crate::mm::Address;

pub struct NVMEHeaderStruct(HeaderType0);

#[repr(C, packed)]
struct NVMERegisters {
	controller_capabilities: u64,
	version: u32,
	interrupt_mask_set: u32,
	interrupt_mask_clear: u32,
	controller_configuration: u32,
	reversed: u32,
	controller_status: u32,
	nvm_subsystem_reset: u32,
	admin_queue_attributes: u32,
	admin_submission_queue: u64,
	admin_completion_queue: u64
}
struct Queue {
	submission_content: Box<[SubmissionEntry; 0x200]>,
	pub submission_doorbell_idx: usize,
	completion_content: Box<[CompletionEntry; 0x200]>,
	pub completion_doorbell_idx: usize,
	pub id: usize
}
pub struct NVMEDrive {
	header: NVMEHeader,
	registers: Box<NVMERegisters>,
	doorbells: Mutex<Box<[u32; 64]>>,
	cap_stride: usize,
	io_queues: Mutex<Vec<Mutex<Queue>>>,
	admin_queue: Mutex<Queue>,
	active_namespaces: Box<[u16; 1024]>
}
#[repr(C, packed)]
struct SubmissionEntry {
	command: u32,
	nsid: u32,
	reserved: u64,
	metadata: u64,
	data: [u64; 2],
	command_specific: [u32; 6],
}
#[repr(C, packed)]
struct CompletionEntry {
	command_specific: [u32; 2],
	sq_head_pointer: u16,
	sq_indentifier: u16,
	command_identifier: u16,
	status: u16
}
pub type NVMEHeader = Box<NVMEHeaderStruct>;

impl DeviceTrait for NVMEHeader {
	fn specific_scan(&self) {
		add_disk(NVMEDrive::from_raw_address(self.physical_address()))
	}
}

impl NVMEDrive {
	pub fn from_raw_address(header_addr: u64) -> Box<dyn PhysicalDisk> {
		let header = NVMEHeader::from_raw_address(header_addr);
		let base_address = ((header.0.bar_addresses[1] as u64) << 32) | (header.0.bar_addresses[0] as u64 & 0xfffffff0);

		let admin_queue = Queue::new(0);
		let mut registers: Box<NVMERegisters> = Box::from_raw_address(base_address);
		registers.admin_submission_queue = admin_queue.submission_physical_address();
		registers.admin_completion_queue = admin_queue.completion_physical_address();

		Box::new(NVMEDrive {
			registers,
			header: header,
			cap_stride: (base_address as usize >> 12) & 0xf,
			doorbells: Mutex::new(Box::from_raw_address(base_address + 0x1000)),
			io_queues: Mutex::new(Vec::new()),
			admin_queue: Mutex::new(admin_queue),
			active_namespaces: Box::new_sized(0x1000)
		})
	}

	fn doorbell(&self, queue: &mut Queue) {
		self.doorbells.lock()[queue.id * 2 * (4 << self.cap_stride) / 4] = queue.submission_doorbell_idx as u32;
		let idx = queue.completion_doorbell_idx;
		let completion = &mut queue.completion_content[idx];
		while completion.status == 0 {}

		queue.completion_doorbell_idx += 1;
		self.doorbells.lock()[queue.id * 2 * (4 << self.cap_stride) / 4 + 1] = queue.completion_doorbell_idx as u32;
	}
	fn send_admin_command(&self, command: SubmissionEntry) {
		let mut admin_queue = self.admin_queue.lock();
		admin_queue.send(command);
		self.doorbell(&mut admin_queue);
	}
	fn send_io_command(&self, command: SubmissionEntry) {
		let io_queues = self.io_queues.read();
		let mut queue = if let Some(queue) = io_queues.into_iter().find(|q| !q.is_locked()) {
			queue
		} else {
			&io_queues[self.create_io_queue()]
		}.lock();

		queue.send(command);
		self.doorbell(&mut queue);
	}
	fn create_io_queue(&self) -> usize {
		let mut io_queue_list_lock = self.io_queues.lock();
		let queue = Queue::new(io_queue_list_lock.len()+1);

		self.send_admin_command(SubmissionEntry::new_io_c_queue(&queue));
		self.send_admin_command(SubmissionEntry::new_io_s_queue(&queue));

		io_queue_list_lock.push_back(Mutex::new(queue));
		io_queue_list_lock.len() - 1
	}
}

impl PhysicalDisk for NVMEDrive {
	fn reset(&mut self) {
		self.registers.nvm_subsystem_reset = 0x4e564d65;
		self.registers.controller_configuration = 0;
		{
			let queue = self.admin_queue.lock();

			self.registers.admin_submission_queue = queue.submission_physical_address();
			self.registers.admin_completion_queue = queue.completion_physical_address();
		}
		self.registers.admin_queue_attributes = 0x200 << 16 | 0x200;
		self.registers.controller_configuration = 0x460061;

		self.send_admin_command(SubmissionEntry::new_get_active_ns(&self.active_namespaces));
		self.create_io_queue();
	}
	fn read_lba(&self, lba: usize) -> Sector where Self: Sized {
		let buffer: Sector = Box::new_uninit();
		self.send_io_command(SubmissionEntry::new_io_read(
			self.active_namespaces[0] as u32,
			lba,
			&buffer
		));
		buffer
	}
}

impl Queue {
	fn new(id: usize) -> Queue {
		Queue {
			submission_content: Box::new_sized(0x1000),
			submission_doorbell_idx: 0,
			completion_content: Box::new_sized(0x1000),
			completion_doorbell_idx: 0,
			id: id
		}
	}

	fn send(&mut self, entry: SubmissionEntry) {
		self.submission_content[self.submission_doorbell_idx] = entry;
		self.submission_doorbell_idx += 1;
	}

	fn completion_physical_address(&self) -> u64 {
		self.completion_content.physical_address()
	}
	fn submission_physical_address(&self) -> u64 {
		self.submission_content.physical_address()
	}
}

impl SubmissionEntry {
	fn new_io_c_queue(queue: &Queue) -> SubmissionEntry {
		SubmissionEntry {
			command: 0x5,
			nsid: 0,
			reserved: 0,
			metadata: 0,
			data: [
				queue.completion_physical_address(),
				0
			],
			command_specific: [
				(0x200 - 1) << 16 | queue.id as u32,
				0x1,
				0,
				0,
				0,
				0
			]
		}
	}
	fn new_io_s_queue(queue: &Queue) -> SubmissionEntry {
		SubmissionEntry {
			command: 0x1,
			nsid: 0,
			reserved: 0,
			metadata: 0,
			data: [
				queue.submission_physical_address(),
				0
			],
			command_specific: [
				(0x200 - 1) << 16 | queue.id as u32,
				(queue.id as u32) << 16 | 0x1,
				0,
				0,
				0,
				0
			]
		}
	}
	fn new_get_active_ns(nvme_info: &[u16; 1024]) -> SubmissionEntry {
		SubmissionEntry {
			command: 0x6,
			nsid: 0,
			reserved: 0,
			metadata: 0,
			data: [
				nvme_info.physical_address(),
				0
			],
			command_specific: [
				2,
				0,
				0,
				0,
				0,
				0
			]
		}
	}
	fn new_io_read(namespace: u32, lba: usize, buffer: &Sector) -> SubmissionEntry {
		SubmissionEntry {
			command: 0x2,
			nsid: namespace,
			reserved: 0,
			metadata: 0,
			data: [
				buffer.physical_address(),
				0
			],
			command_specific: [
				(lba & 0xffffffff) as u32,
				(lba >> 32) as u32,
				1,
				0,
				0,
				0
			]
		}
	}
}
