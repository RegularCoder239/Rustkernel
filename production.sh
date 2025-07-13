set -e
sh build.sh
qemu-system-x86_64 -bios /usr/share/ovmf/x64/OVMF.4m.fd -kernel target/x86_64-unknown-uefi/debug/Secondtry.efi -cpu max -m size=1G -smp cores=1 -debugcon stdio -M q35 -d guest_errors -nic model=rtl8139 -enable-kvm -drive file=disk.img,if=none,id=nvm -device nvme,serial=deadbeef,drive=nvm
