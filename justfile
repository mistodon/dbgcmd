winittests:
    #cargo check --features winit_0_21 (doesn't work on arm macs)
    cargo check --features winit_0_24
    cargo check --features winit_0_27
    cargo check --features winit_0_29
    cargo check --features winit_0_30

alltests: winittests
    cargo test
    cargo test --features force-enabled
    cargo test --release
    cargo test --release --features force-enabled
    cargo fmt -- --check
    cargo clippy -- -D clippy::all

fmt:
    cargo fmt
