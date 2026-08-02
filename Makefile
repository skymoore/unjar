.PHONY: prepare check test docs build update

prepare:
	cargo fmt
	cargo clippy --fix --all-targets --locked --allow-dirty -- -D warnings
	cargo check --release --locked
	cargo check --lib --no-default-features --locked

check:
	cargo fmt --check
	cargo clippy --all-targets --locked -- -D warnings
	cargo check --release --locked
	cargo check --lib --no-default-features --locked

test:
	cargo test --locked
	cargo test --lib --no-default-features --locked

docs:
	RUSTDOCFLAGS="-D warnings" cargo doc --lib --no-default-features --locked --no-deps --open

build:
	cargo build --release
	ls -lh target/release/$(shell basename $(CURDIR))

update:
	cargo upgrade -i
