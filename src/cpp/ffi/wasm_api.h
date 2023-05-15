#ifndef FFI_WASM_API_H

extern "C" {
  void wasm_api_reset();
  void wasm_api_set_arg(const char *s) ;
  void wasm_api_set_kpuzzle_definition(const char *s) ;
  const char *wasm_api_solve_scramble(const char *s) ;
  const char *wasm_api_solve_position(const char *s) ;
}

#define FFI_WASM_API_H
#endif
