#include <string>

#include "../twsearch.h"

#include "rust_api.h"
#include "ffi_api.h"

#include "rust/cxx.h"

void rust_api_reset() {
  ffi_api_reset();
}

void rust_api_set_arg(rust::Str s_) {
  std::string s(s_);
  ffi_api_set_arg(s);
}

void rust_api_set_kpuzzle_definition(rust::Str s_) {
  std::string s(s_);
  ffi_api_set_kpuzzle_definition(s);
}

rust::String rust_api_solve_scramble(rust::Str s_) {
  std::string s(s_);
  return ffi_api_solve_scramble(s);
}

rust::String rust_api_solve_position(rust::Str s_) {
  std::string s(s_);
  return ffi_api_solve_position(s);
}

void rust_api_main_search(rust::Str def_file_, rust::Str scramble_file_) {
  std::string def_file_str(def_file_);
  const char* def_file = def_file_str.c_str();

  std::string scramble_file_str;
  const char* scramble_file = NULL;
  if (scramble_file_.length() > 0) {
    scramble_file_str = std::string(scramble_file_);
    scramble_file = scramble_file_str.c_str();
  }

  main_search(def_file, scramble_file);
}
