# Rust WASM

.PHONY: build-rust-wasm
build-rust-wasm: setup-js
	rm -rf "./.temp/rust-wasm"
	cargo tool-run-bin wasm-pack build --release --target web --out-dir "../../.temp/rust-wasm" src/rs-wasm
	node script/node-esm-compat.js
	bun run script/node-esm-compat.js
	bun run "./script/build-wasm-package.ts"

.PHONY: test-rust-wasm
test-rust-wasm: setup-js build-rust-wasm
	bun run "script/test-dist-wasm.ts"
