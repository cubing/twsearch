.PHONY: setup-js
setup-js:
	bun run "./script/check-engine-versions.ts"
	bun install --frozen-lockfile > /dev/null

.PHONY: lint-js
lint-js: setup-js
	bun x @biomejs/biome check

.PHONY: format-js
format-js: setup-js
	bun x @biomejs/biome check --write
