#ifndef FFI_FFI_API_H
#include <string>

void ffi_api_set_arg(std::string s) ;
void ffi_api_set_kpuzzle_definition(std::string s) ;
std::string ffi_api_solve_scramble(std::string s) ;
std::string ffi_api_solve_position(std::string s) ;

extern "C" {
  void ffi_api_reset();

  void ffi_api_cstr_set_arg(const char *s) ;
  void ffi_api_cstr_set_kpuzzle_definition(const char *s) ;
  const char *ffi_api_cstr_solve_scramble(const char *s) ;
  const char *ffi_api_cstr_solve_position(const char *s) ;
}

#define FFI_FFI_API_H
#endif
