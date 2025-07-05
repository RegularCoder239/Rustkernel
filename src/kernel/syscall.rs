use crate::hw::cpu::syscall::Function;

const SYSCALL_METHODS: [Function; 3] = [
	Function {
		id: 0xa4998996a6277317,
		meth: || crate::std::exit()
	},
	Function {
		id: 0x4a33e7eb45595ceb,
		meth: || crate::hw::power::shutdown()
	},
	Function {
		id: 0xba3f7ec4fdf5556b,
		meth: || crate::hw::power::reboot()
	},
];

pub fn setup() {
	for meth in SYSCALL_METHODS {
		meth.add();
	}
}
