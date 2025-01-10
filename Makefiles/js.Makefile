.PHONY: setup-js
setup-js:
	bun install --no-save

.PHONY: lint-js
lint-js: setup-js
	bun x @biomejs/biome check

.PHONY: format-js
format-js: setup-js
	bun x @biomejs/biome check --write
