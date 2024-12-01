
# Rust

.PHONY: dev-rust
dev-rust:
	cargo run --release -- serve

.PHONY: dev-rust-40G
dev-rust-40G:
	cargo run --release -- serve --memory-MiB 40960

.PHONY: build-rust
build-rust:
	cargo build --release

.PHONY: lint-rust
lint-rust:
	cargo clippy

.PHONY: publish-rust
publish-rust: publish-rust-main publish-rust-ffi

.PHONY: publish-rust-main
publish-rust-main:
	@echo "WARNING: will fall back to `--no-verify` due to https://github.com/rust-lang/cargo/issues/8407" # TODO
	cargo publish --package twsearch || cargo publish --package twsearch --no-verify

.PHONY: setup-rust
setup-rust:
	cargo install cargo-run-bin

# Rust testing

.PHONY: test-rust
test-rust: test-rust-lib test-rust-wasm test-rust-ffi

.PHONY: test-rust-lib
test-rust-lib:
	cargo test
	#cargo run --release --example random_scramble_for_event

.PHONY: benchmark-rust
benchmark-rust:
	cargo run --release -- benchmark samples/json/benchmark/benchmark-3x3x3.def.json
