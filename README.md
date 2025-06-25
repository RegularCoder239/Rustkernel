# Rustkernel
## Warning
This kernel isn't designed for production and hasn't been tested
for real hardware yet. It has many major issues, that weren't been fixed yet.
Please use a VM for testing.
## Requirements
* 64 bit X86_64 CPU
* 64 MB RAM
* Optional: Network RTL8139
## Implemented drivers
* UEFI-Display
* NVME (limited)
* RTL8139 (only sending packages)
## Building
Run build.sh
## Running
* Run run.sh for an emulated VM
* Run production.sh for the virtalized VM
