set debuginfod enabled on
set osabi none
add-symbol-file target/x86_64-unknown-uefi-debug/debug/Secondtry.efi -s .text 0xffff800000000000+0x1000
target remote :1234
