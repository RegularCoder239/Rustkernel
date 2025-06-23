set -e
sh build.sh
qemu-system-x86_64 -bios /usr/share/ovmf/x64/OVMF.4m.fd -kernel target/x86_64-unknown-uefi-debug/debug/Secondtry.efi -d guest_errors,invalid_mem -m size=1G -smp cores=1 -debugcon stdio -cpu max -d int -M q35 -net nic,model=rtl8139 -no-reboot -drive file=disk.img,if=none,id=nvm -device nvme,serial=deadbeef,drive=nvm -D logfile.log
