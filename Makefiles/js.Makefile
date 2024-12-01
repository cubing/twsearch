.PHONY: update-js-deps
update-js-deps:
	bun install --no-save

.PHONY: lint-js
lint-js: update-js-deps
	bun x @biomejs/biome check

.PHONY: format-js
format-js: update-js-deps
	bun x @biomejs/biome check --apply
