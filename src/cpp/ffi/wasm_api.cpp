
#include <sstream>
#include "string.h"

#include "../puzdef.h"
#include "../prunetable.h"
#include "../solve.h"
#include "../twsearch.h"
#include "../parsemoves.h"
#include "../cmdlineops.h"
#include "ffi_api.h"

extern "C" void wasm_api_reset() {
   ffi_api_reset();
}

extern "C" void wasm_api_set_arg(const char *s) {
   ffi_api_cstr_set_arg(s);
}

extern "C" void wasm_api_set_kpuzzle_definition(const char *s) {
   ffi_api_cstr_set_kpuzzle_definition(s);
}

extern "C" const char *wasm_api_solve_scramble(const char *s) {
   return ffi_api_cstr_solve_scramble(s);
}

extern "C" const char *wasm_api_solve_position(const char *s) {
   return ffi_api_cstr_solve_position(s);
}

// Workaround for https://github.com/wasmerio/wasmer/issues/2589
extern "C" void emscripten_notify_memory_growth([[maybe_unused]] std::size_t _) {
   // Do nothing.
}
