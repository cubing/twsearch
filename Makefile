.PHONY: build
build: \
	build-rust \
	build-rust-wasm \
	build-rust-ffi

.PHONY: setup
setup: setup-gitignore-dirs setup-js setup-rust

.PHONY: test
test: \
	test-warning \
	lint \
	test-rust \
	test-rust-ffi \
	benchmark-rust

.PHONY: test-warning
test-warning:
	@echo "Warning: tests are slow to run right now."

.PHONY: clean
clean:
	rm -rf ./.temp ./build ./dist ./src/js/generated-wasm/twips.* ./*.dwo ./package-lock.json

.PHONY: reset
reset: clean
	rm -rf ./emsdk ./node_modules ./target ./.bin

.PHONY: lint
lint: lint-js lint-rust

.PHONY: format
format: format-js format-rust

.PHONY: publish
publish: test-rust publish-rust

.PHONY: setup-gitignore-dirs
setup-gitignore-dirs:
	bun run ./script/self-gitignore-dirs.ts ./.bin ./.temp ./build ./dist ./target

.PHONY: describe-version
describe-version:
	@ # TODO: this wastes 0.1 second running `setup-js` a second time when building both C++ and JS targets — can we avoid that?
	@ make setup-js 2>&1 > /dev/null
	@ bun x -- @lgarron-bin/repo version describe

include ./Makefiles/js.Makefile
include ./Makefiles/rust.Makefile
include ./Makefiles/rust-wasm.Makefile
include ./Makefiles/rust-ffi.Makefile
