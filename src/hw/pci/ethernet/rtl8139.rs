use super::{
	NetworkDeviceTrait
};
use crate::mm::Address;

#[repr(C, packed)]
pub struct RTL8139Registers {
	mac: [u8; 6],
	unused: u16,
	mar: u64,
	tx_status: [u32; 4],
	tx_buffer: [u32; 4],
	rx_buffer: u32,
	unused_3: [u8; 3],
	command: u8,
	unused_4: [u8; 4],
	imr: u16,
	isr: u16,
	unused_5: u32,
	config_rx: u64,
	enable: u8
}

pub struct RTL8139(RTL8139Registers);

static RX_BUFFER: [u8; 0x2000] = [0; 0x2000];

impl NetworkDeviceTrait for RTL8139 {
	fn mac(&self) -> [u8; 6] {
		self.0.mac
	}
	fn setup(&mut self) {
		self.0.enable = 0xff;
		self.0.command = 0x10;
		self.0.rx_buffer = RX_BUFFER.physical_address() as u32;
		self.0.config_rx = 0x8f;
		self.0.command = 0xc;
	}
	fn send_package(&mut self, package: *mut u8, len: usize) {
		self.0.tx_buffer[0] = package.physical_address() as u32;
		self.0.tx_status[0] = len as u32;
	}
}
