#ifndef FFI_WASM_API_H

#include <cstddef>

extern "C" {
  void wasm_api_reset();
  void wasm_api_set_arg(const char *s) ;
  void wasm_api_set_kpuzzle_definition(const char *s) ;
  const char *wasm_api_solve_scramble(const char *s) ;
  const char *wasm_api_solve_position(const char *s) ;

  // Workaround for https://github.com/wasmerio/wasmer/issues/2589
  void emscripten_notify_memory_growth(std::size_t _);
}

#define FFI_WASM_API_H
#endif
