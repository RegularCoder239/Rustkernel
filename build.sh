export RUSTFLAGS="-Cforce-unwind-tables -Clink-arg=-Wl,eh_frame.ld"
cargo +nightly -Z build-std=core,alloc build --target ./x86_64-unknown-uefi-debug.json
