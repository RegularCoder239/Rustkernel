rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
cargo +nightly -Zbuild-std build --target x86_64-unknown-uefi-debug.json
