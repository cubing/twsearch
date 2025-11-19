# Rust WASM

.PHONY: build-rust-wasm
build-rust-wasm: setup-rust setup-js
	rm -rf "./.temp/rust-wasm"
	cargo tool-run-bin wasm-pack build --release --target web --out-dir "../../.temp/rust-wasm" src/wasm
	node script/node-esm-compat.js
	bun run script/node-esm-compat.js
	bun run "./script/build-wasm-package.ts"

.PHONY: test-rust-wasm
test-rust-wasm: setup-rust setup-js build-rust-wasm
	bun test "script/test-dist-wasm.wasm.test.ts"
