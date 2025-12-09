.PHONY: build
build: \
	build-rust \
	build-rust-wasm \
	build-rust-ffi

.PHONY: check
check: lint test build

.PHONY: setup
setup: setup-js setup-gitignore-dirs setup-rust setup-ruby

.PHONY: test
test: \
	test-warning \
	test-rust \
	test-rust-ffi \
	benchmark-rust \
	test-ruby

.PHONY: test-warning
test-warning:
	@echo "Warning: tests are slow to run right now."

.PHONY: clean
clean:
	rm -rf ./.temp ./build ./dist ./src/js/generated-wasm/twips.* ./package-lock.json ./src/ruby-gem/tmp ./src/ruby-gem/lib/twips/twips_rb.bundle 

.PHONY: reset
reset: clean
	rm -rf ./emsdk ./node_modules ./target ./.bin ./src/ruby-gem/target

.PHONY: lint
lint: lint-js lint-rust lint-ruby

.PHONY: format
format: format-js format-rust

.PHONY: publish
publish: test-rust publish-rust

.PHONY: setup-gitignore-dirs
setup-gitignore-dirs: setup-js-deps
	bun run ./script/self-gitignore-dirs.ts ./.bin ./.temp ./dist ./target ./src/ruby-gem/target

.PHONY: check-engine-versions
check-engine-versions:
	bun run "./script/check-engine-versions.ts"

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
	cargo publish --workspace --exclude cargo-bin --exclude twips-rb --exclude twips-ffi

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
	cargo test --workspace --exclude twips-ffi --exclude twips-rb

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
	cargo run --release -- benchmark samples/json/benchmark/benchmark-3x3x3.def.json

# Rust WASM

.PHONY: build-rust-wasm
build-rust-wasm: check-engine-versions setup-rust setup-js
	rm -rf "./.temp/rust-wasm"
	cargo tool-run-bin wasm-pack build --release --target web --out-dir "../../.temp/rust-wasm" src/wasm
	node script/node-esm-compat.js
	bun run script/node-esm-compat.js
	bun run "./script/build-wasm-package.ts"

.PHONY: test-rust-wasm
test-rust-wasm: setup-rust setup-js build-rust-wasm
	bun test "script/test-dist-wasm.wasm.test.ts"

# Rust FFI

.PHONY: build-rust-ffi
build-rust-ffi: setup-rust
	cargo build --release --package twips-ffi
	mkdir -p "./.temp"
	cargo tool-run-bin cbindgen --crate twips-ffi --lang c --cpp-compat --output "./.temp/twips-ffi.h"
	cat "./.temp/twips-ffi.h" | sed "s#\[..\];#;#g" | sed "s#\[.\];#;#g" | sed "s#const uint8_t#const char#g" > "./target/release/libtwips_ffi.h"

.PHONY: test-rust-ffi # TODO: non-PHONY?
test-rust-ffi: setup-rust test-rust-ffi-rs test-rust-ffi-js test-rust-ffi-c

.PHONY: test-rust-ffi-rs
test-rust-ffi-rs: setup-rust build-rust-ffi
	cargo test --package twips-ffi

.PHONY: test-rust-ffi-js
test-rust-ffi-js: setup-js build-rust-ffi
	bun run "src/ffi/test/js_test.ts"

.PHONY: test-rust-ffi-c
test-rust-ffi-c: setup-rust build-rust-ffi
	gcc -o src/ffi/test/c_test.bin -L./target/release src/ffi/test/c_test.c -ltwips_ffi
	env LD_LIBRARY_PATH=./target/release src/ffi/test/c_test.bin

.PHONY: publish-rust-ffi
publish-rust-ffi: setup-rust
	@echo "WARNING: will fall back to \`--no-verify\` due to https://github.com/rust-lang/cargo/issues/8407" # TODO
	cargo publish --package twips-ffi || cargo publish --package twips-ffi --no-verify

# JS

.PHONY: setup-js
setup-js: setup-js-deps setup-gitignore-dirs

.PHONY: setup-js-deps
setup-js-deps: check-engine-versions
	bun install --frozen-lockfile > /dev/null

.PHONY: lint-js
lint-js: lint-js-biome lint-js-tsc

.PHONY: lint-js-biome
lint-js-biome: setup-js
	bun x @biomejs/biome check

.PHONY: lint-js-tsc
lint-js-tsc: setup-js build-rust-wasm
	bun x tsc --noEmit --project .

.PHONY: format-js
format-js: setup-js
	bun x @biomejs/biome check --write

RUBY_GEM_DIR = ./src/ruby-gem/
RUBY_VERSION = $(shell cat ./src/ruby-gem/.ruby-version)
RUBY = rv ruby run ${RUBY_VERSION} -- -C ./src/ruby-gem/

.PHONY: test-ruby
test-ruby: build-ruby
	${RUBY} ./test/test-api.rb
	cargo test --package twips-rb

.PHONY: lint-ruby
lint-ruby:
	bun run -- script/ruby-version/check.ts

.PHONY: build-ruby
build-ruby: setup-ruby
	${RUBY} -S rake compile

.PHONY: setup-ruby
setup-ruby:
	${RUBY} -e "" || rv ruby install ${RUBY_VERSION} # TODO: remove this once https://github.com/spinel-coop/rv/issues/72 is available.
	${RUBY} -S bundle install

.PHONY: ruby-update-lockfile
ruby-update-lockfile:
	${RUBY} -S bundle lock
