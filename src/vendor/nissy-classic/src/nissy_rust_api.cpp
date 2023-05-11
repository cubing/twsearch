#include "rust/cxx.h"
#include <string>
#include "alg.h"
#include "commands.h"
#include "nissy_rust_api.h"

rust::String rust_nissy_solve_alg_twophase(rust::Str s) {
  std::string str(s);
  Alg* alg = new_alg(str.c_str());
  twophase_exec_scramble(alg);
  return rust::String("Look for the output");
}
