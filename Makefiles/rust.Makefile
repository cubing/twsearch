
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
	cargo clippy -- --deny warnings

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
test-rust: test-rust-lib test-rust-examples test-rust-wasm test-rust-ffi

.PHONY: test-rust-lib
test-rust-lib:
	cargo test
	cargo doc

.PHONY: test-rust-examples
test-rust-examples: \
	test-rust-example-kociemba_multiphase \
	test-rust-example-scramble_all_events \
	test-rust-example-2x2x2_three_phase \
	test-rust-example-readme_example

.PHONY: test-rust-example-kociemba_multiphase
test-rust-example-kociemba_multiphase:

	cargo run --release --example kociemba_multiphase
.PHONY: test-rust-example-scramble_all_events
test-rust-example-scramble_all_events:

	cargo run --release --example scramble_all_events
.PHONY: test-rust-example-2x2x2_three_phase
test-rust-example-2x2x2_three_phase:

	cargo run --release --example 2x2x2_three_phase
.PHONY: test-rust-example-readme_example
test-rust-example-readme_example:

	cargo run --release --example readme_example

.PHONY: benchmark-rust
benchmark-rust:
	cargo run --release -- benchmark samples/json/benchmark/benchmark-3x3x3.def.json
