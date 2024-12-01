# WASM

# Depends on `cpp.Makefile`
WASMSOURCE = $(BASESOURCE) $(FFISOURCE)

WASM_CXX = emsdk/upstream/emscripten/em++
WASM_CXXFLAGS = -O3 -fno-exceptions -Wextra -Wall -pedantic -std=c++17 -Wsign-compare
WASM_COMMON_FLAGS = -DTWSEARCH_VERSION=${TWSEARCH_VERSION} -DWASM -DASLIBRARY -Isrc/cpp -Isrc/cpp/cityhash/src -sEXPORTED_FUNCTIONS=_wasm_api_set_arg,_wasm_api_set_kpuzzle_definition,_wasm_api_solve_scramble,_wasm_api_solve_position -sEXPORTED_RUNTIME_METHODS=cwrap -sALLOW_MEMORY_GROWTH
WASM_TEST_FLAGS = -DWASMTEST -sASSERTIONS
WASM_ESM_BASE64_SINGLE_FILE_FLAGS = -sEXPORT_ES6 -sSINGLE_FILE
WASM_LDFLAGS = 

emsdk: ${WASM_CXX}
${WASM_CXX}:
	make emsdk-tip-of-tree

.PHONY: emsdk-latest
emsdk-latest:
	rm -rf ./emsdk
	git clone https://github.com/emscripten-core/emsdk.git
	cd emsdk && ./emsdk install latest
	cd emsdk && ./emsdk activate latest

.PHONY: emsdk-tip-of-tree
emsdk-tip-of-tree:
	rm -rf ./emsdk
	git clone https://github.com/emscripten-core/emsdk.git
	cd emsdk && ./emsdk install tot
	cd emsdk && ./emsdk activate tot

build/emscripten/wasm-test/:
	mkdir -p build/emscripten/wasm-test/

build/emscripten/wasm-test/twsearch-test.wasm: $(WASMSOURCE) $(HSOURCE) ${WASM_CXX} | build/emscripten/wasm-test/
	$(WASM_CXX) $(WASM_CXXFLAGS) $(WASM_COMMON_FLAGS) $(WASM_TEST_FLAGS) -o $@ $(WASMSOURCE) $(WASM_LDFLAGS) -DWASMTEST

build/emscripten/wasm-single-file/:
	mkdir -p build/emscripten/wasm-single-file/

build/emscripten/wasm-single-file/twsearch.mjs: $(WASMSOURCE) $(HSOURCE) ${WASM_CXX} | build/emscripten/wasm-single-file/
	$(WASM_CXX) $(WASM_CXXFLAGS) $(WASM_COMMON_FLAGS) $(WASM_ESM_BASE64_SINGLE_FILE_FLAGS) -o $@ $(WASMSOURCE) $(WASM_LDFLAGS)

ESBUILD_COMMON_ARGS = \
		--format=esm --target=es2020 \
		--bundle --splitting \
		--external:path --external:fs --external:module \
		--external:node:* \

.PHONY: dev
dev: update-js-deps build/emscripten/wasm-single-file/twsearch.mjs
	bun x esbuild ${ESBUILD_COMMON_ARGS} \
		--sourcemap \
		--servedir=src/js/dev \
		src/js/dev/*.ts

.PHONY: build/emscripten/esm
build/emscripten/esm: update-js-deps build/emscripten/wasm-single-file/twsearch.mjs
	bun x esbuild ${ESBUILD_COMMON_ARGS} \
		--external:cubing \
		--outdir=build/emscripten/esm src/js/index.ts
	mkdir -p ./.temp
	mv build/emscripten/esm/index.js ./.temp/index.js
	echo "console.info(\"Loading twsearch ${TWSEARCH_VERSION}\");" > build/emscripten/esm/index.js
	cat "./.temp/index.js" >> build/emscripten/esm/index.js

.PHONY: build/emscripten/esm-test
build/emscripten/esm-test: update-js-deps build/emscripten/wasm-single-file/twsearch.mjs
	bun x esbuild ${ESBUILD_COMMON_ARGS} \
		--external:cubing \
		--outdir=build/emscripten/esm-test \
		src/js/dev/test.ts

.PHONY: test-cpp-wasm
test-cpp-wasm: test-cpp-wasm-wasmer test-cpp-wasm-js

.PHONY: test-cpp-wasm-wasmer
test-cpp-wasm-wasmer: build/emscripten/wasm-test/twsearch-test.wasm
	wasmer build/emscripten/wasm-test/twsearch-test.wasm || echo "This test is known to fail due to an unknown import of `_tzset_js`."

.PHONY: test-cpp-wasm-js
test-cpp-wasm-js: build/emscripten/esm-test
	bun run build/emscripten/esm-test/test.js

.PHONY: check-wasmer
check-wasmer:
	@wasmer -V > /dev/null || echo "\nEnsure \`wasmer\` is installed: https://docs.wasmer.io/install\n"
