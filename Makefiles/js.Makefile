.PHONY: setup-js
setup-js:
	bun install --frozen-lockfile > /dev/null
	bun run "./script/check-engine-versions.ts"

.PHONY: lint-js
lint-js: setup-js
	bun x @biomejs/biome check

.PHONY: format-js
format-js: setup-js
	bun x @biomejs/biome check --write
