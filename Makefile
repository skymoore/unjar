.PHONY: prepare check test docs build update

prepare:
	cargo +nightly fmt
	cargo clippy --fix --all-targets --locked --allow-dirty -- -D warnings
	cargo check --release --locked
	cargo check --lib --no-default-features --locked

check:
	cargo +nightly fmt --check
	cargo clippy --all-targets --locked -- -D warnings
	cargo check --release --locked
	cargo check --lib --no-default-features --locked

test:
	cargo test --locked
	cargo test --lib --no-default-features --locked

docs:
	RUSTDOCFLAGS="-D warnings" cargo doc --lib --no-default-features --locked --no-deps --open

build:
	cargo build --release --locked
	ls -lh target/release/unjar

update:
	cargo upgrade -i
