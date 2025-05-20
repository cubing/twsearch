.PHONY: build
build: \
	build/bin/twsearch \
	twsearch-cpp-wrapper-cli \
	build-rust \
	build-rust-wasm \
	build-rust-ffi

.PHONY: setup
setup: setup-gitignore-dirs setup-js setup-rust

.PHONY: test
test: \
	test-warning \
	test-cpp \
	lint \
	test-rust \
	test-rust-ffi \
	benchmark-rust

.PHONY: test-cpp
test-cpp: \
	test-cpp-cli \
	test-twsearch-cpp-wrapper-cli \
	lint-cpp

.PHONY: test-warning
test-warning:
	@echo "Warning: tests are slow to run right now."

.PHONY: clean
clean:
	rm -rf ./.temp ./build ./dist ./src/js/generated-wasm/twsearch.* ./*.dwo ./package-lock.json

.PHONY: reset
reset: clean
	rm -rf ./emsdk ./node_modules ./target ./.bin

.PHONY: lint
lint: lint-cpp lint-js lint-rust

.PHONY: format
format: format-cpp format-js format-rust

.PHONY: publish
publish: test-rust publish-rust

.PHONY: print-current-commit-hash
print-current-commit-hash:
	@bun run ./script/print-current-commit-hash.ts

.PHONY: setup-gitignore-dirs
setup-gitignore-dirs:
	bun run ./script/self-gitignore-dirs.ts ./.bin ./.temp ./build ./dist ./target

include ./Makefiles/cpp.Makefile
include ./Makefiles/js.Makefile
include ./Makefiles/rust.Makefile
include ./Makefiles/rust-wasm.Makefile
include ./Makefiles/rust-ffi.Makefile
