.PHONY: setup-js
setup-js: setup-gitignore-dirs
	bun install --frozen-lockfile > /dev/null
	bun run "./script/check-engine-versions.ts"

.PHONY: lint-js
lint-js: lint-js-biome lint-js-tsc

.PHONY: lint-js-biome
lint-js-biome: setup-js 
	bun x @biomejs/biome check

.PHONY: lint-js-tsc
lint-js-tsc: setup-js build-rust-wasm
	bun x tsc --project .

.PHONY: format-js
format-js: setup-js
	bun x @biomejs/biome check --write
