#ifndef FFI_RUST_API_H
#include "rust/cxx.h"

void rust_api_reset();
void rust_api_set_arg(rust::Str s);
void rust_api_set_kpuzzle_definition(rust::Str s);
rust::String rust_api_solve_scramble(rust::Str s);
rust::String rust_api_solve_position(rust::Str s);
// TODO: We can't use `optional` because
// https://github.com/dtolnay/cxx/issues/87 is unresolved. Use the empty string
// to indicate an empty value for `scramble_file`.
void rust_api_main_search(rust::Str def_file, rust::Str scramble_file);

#define FFI_RUST_API_H
#endif
