# Rustkernel
## Warning
This kernel is still in early development.
This kernel isn't designed for production and should only be
booted on VMs. It has many major issues, that weren't been fixed yet
and almost no security. I won´t be responsible for hardware damages,
if this OS is tested on real hardware. This is only a hobby project.
## Documentation
Can be found under the directory doc/.
Warning: Incomplete
## Requirements
* 64 bit X86_64 CPU
* 64 MB RAM
* UEFI
## Implemented drivers
* UEFI-Display
* NVME (reading sectors only)
* RTL8139 (only sending packages)
* SMP
* IOAPIC
* LAPIC
## Filesystems
* FAT32
## Features
* Layered graphics
* Text console layer
* Simple virtual memory manager
* Buddy allocator
* Interrupts
* Syscalls
* Init executions
* Simple task switcher
## Building
1. Install cargo, rustup, gcc and mingw binutils
2. Run build.sh
## Running
Install qemu before running these scripts.
* Run run.sh for an emulated VM
* Run production.sh for the virtalized VM
