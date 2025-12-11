
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
lint-rust: test-cargo-doc
	cargo clippy -- --deny warnings
	cargo fmt --check

.PHONY: format-rust
format-rust:
	cargo clippy --fix --allow-no-vcs
	cargo fmt

.PHONY: publish-rust
publish-rust: publish-rust-main publish-rust-ffi

.PHONY: publish-rust-main
publish-rust-main:
	cargo publish --workspace --exclude cargo-bin

.PHONY: setup-rust
setup-rust: setup-gitignore-dirs

# Rust testing

.PHONY: test-rust
test-rust: test-rust-build-help test-rust-build-version test-rust-lib test-rust-examples test-rust-wasm test-rust-ffi

.PHONY: test-rust-build-help
test-rust-build-help: build-rust
	./target/release/twips --help

.PHONY: test-rust-build-version
test-rust-build-version: build-rust
	./target/release/twips --version

.PHONY: test-rust-lib
test-rust-lib: setup-rust test-cargo-doc
	# `twips-ffi` is covered by `make test-rust-ffi-rs`
	cargo test --workspace --exclude twips-ffi

.PHONY: test-cargo-doc
test-cargo-doc: setup-rust
	cargo doc

.PHONY: test-rust-examples
test-rust-examples: setup-rust \
	test-rust-example-kociemba_multiphase \
	test-rust-example-scramble_all_events \
	test-rust-example-2x2x2_three_phase \
	test-rust-example-readme_example

.PHONY: test-rust-example-kociemba_multiphase
test-rust-example-kociemba_multiphase: setup-rust
	cargo run --package twips --release --example kociemba_multiphase

.PHONY: test-rust-example-scramble_all_events
test-rust-example-scramble_all_events: setup-rust
	cargo run --package twips --release --example scramble_all_events

.PHONY: test-rust-example-2x2x2_three_phase
test-rust-example-2x2x2_three_phase: setup-rust
	cargo run --package twips --release --example 2x2x2_three_phase

.PHONY: test-rust-example-readme_example
test-rust-example-readme_example: setup-rust
	cargo run --package twips --release --example readme_example

.PHONY: benchmark-rust
benchmark-rust: setup-rust
	cargo run --release -- benchmark samples/benchmark/benchmark-3x3x3.def.json
