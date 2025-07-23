export RUSTFLAGS="-Cforce-unwind-tables"
cargo +nightly -Zbuild-std build --target x86_64-unknown-uefi-debug.json
