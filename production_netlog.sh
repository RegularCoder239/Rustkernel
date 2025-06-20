set -e
sh build.sh
qemu-system-x86_64 -bios /usr/share/ovmf/x64/OVMF.4m.fd -kernel target/x86_64-unknown-uefi-debug/debug/Secondtry.efi -m size=1G -smp cores=1 -no-reboot -debugcon stdio -M q35 -d guest_errors -netdev user,id=u1 -device rtl8139,netdev=u1 -object filter-dump,id=f1,netdev=u1,file=netdump.bat
