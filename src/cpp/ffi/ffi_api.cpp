/*
 *   For WASM use, as an initial test, we use global state to store
 *   our puzdef and pruning table, so our API is just strings everywhere.
 */
#include "string.h"
#include <sstream>

#include "../cmdlineops.h"
#include "../parsemoves.h"
#include "../prunetable.h"
#include "../puzdef.h"
#include "../solve.h"
#include "../twsearch.h"
#include "ffi_api.h"

static puzdef emptypd;
struct wasmdata {
  puzdef pd;
  prunetable *pt;
  int havepd, havept;
  void clear() {
    pd = emptypd;
    delete pt;
    havepd = 0;
    havept = 0;
    pt = 0;
  }
} wasmdata;

static int wasm_inited = 0;
extern "C" void ffi_api_cstr_set_arg(const char *s_) {
  string s(s_);
  ffi_api_set_arg(s);
}

void ffi_api_set_arg(string s) {
  if (wasm_inited == 0) {
    reseteverything();
    wasm_inited = 1;
  }
  const char *argva[4];
  const char **argv = argva;
  int argc = 0;
  argv[0] = "w_arg";
  int space = 0;
  string scopy;
  while (space < (int)s.size() && s[space] != ' ')
    space++;
  if (space >= (int)s.size()) {
    argv[1] = strdup(s.c_str());
    argv[2] = 0;
    argc = 2;
  } else {
    scopy = s;
    scopy[space] = 0;
    argv[1] = strdup(scopy.c_str());
    argv[2] = strdup(scopy.c_str() + space + 1);
    argv[3] = 0;
    argc = 3;
  }
  processargs(argc, argv);
  if (argc != 1)
    error("! error processing argument");
}

void checkprunetable() {
  if (!wasmdata.havepd)
    error(
        "! you must set the ksolve definition before building a pruning table");
  if (!wasmdata.havept) {
    wasmdata.pt = new prunetable(wasmdata.pd, maxmem);
    wasmdata.havept = 1;
  }
}

extern "C" void ffi_api_cstr_set_kpuzzle_definition(const char *s_) {
  string s(s_);
  ffi_api_set_kpuzzle_definition(s);
}

void ffi_api_set_kpuzzle_definition(string s) {
  if (wasm_inited == 0) {
    reseteverything();
    wasm_inited = 1;
  }
  wasmdata.pd = makepuzdef(s);
  wasmdata.havepd = 1;
}

string ffi_api_solve_scramble(string s_) {
  string output(ffi_api_cstr_solve_scramble(s_.c_str()));
  return output;
}

extern "C" const char *ffi_api_cstr_solve_scramble(const char *s) {
  lastsolution = "--no solution--";
  checkprunetable();
  puzdef &pd = wasmdata.pd;
  stacksetval p1(pd);
  vector<allocsetval> movelist = parsemovelist_generously(pd, s);
  for (auto &m : movelist)
    domove(pd, p1, m);
  string noname("NoScrambleName");
  solveit(pd, *wasmdata.pt, noname, p1);
  return lastsolution.c_str();
}

string ffi_api_solve_position(string s_) {
  string output(ffi_api_cstr_solve_position(s_.c_str()));
  return output;
}

extern "C" const char *ffi_api_cstr_solve_position(const char *s) {
  lastsolution = "--no solution--";
  checkprunetable();
  stringstream is(s);
  processscrambles(&is, wasmdata.pd, *wasmdata.pt, 0);
  return lastsolution.c_str();
}

extern "C" void ffi_api_reset() {
  reseteverything();
  wasmdata.clear();
}

#ifdef WASMTEST
const char *twsfile = "Name 2x2x2\n"
                      "\n"
                      "Set CORNERS 8 3\n"
                      "\n"
                      "StartState\n"
                      "CORNERS\n"
                      "0 1 2 3 4 5 6 7\n"
                      "0 0 0 0 0 0 0 0\n"
                      "End\n"
                      "\n"
                      "MoveTransformation U\n"
                      "CORNERS\n"
                      "1 2 3 0 4 5 6 7\n"
                      "0 0 0 0 0 0 0 0\n"
                      "End\n"
                      "\n"
                      "MoveTransformation F\n"
                      "CORNERS\n"
                      "3 1 2 5 0 4 6 7\n"
                      "1 0 0 2 2 1 0 0\n"
                      "End\n"
                      "\n"
                      "MoveTransformation R\n"
                      "CORNERS\n"
                      "4 0 2 3 7 5 6 1\n"
                      "2 1 0 0 1 0 0 2\n"
                      "End\n";
const char *scrfile = "ScrambleState wheee\n"
                      "CORNERS\n"
                      "0 3 5 4 1 7 6 2\n"
                      "0 0 0 0 0 0 0 0\n"
                      "End\n";

int main() {
  ffi_api_set_arg("--nowrite"); // unnecessary if compiled with -DWASM
  ffi_api_set_kpuzzle_definition(twsfile);
  cout << ffi_api_solve_scramble("U F R U F R U F R U F R") << endl;
  cout << ffi_api_solve_position(scrfile) << endl;
  // cout << ffi_api_solve_scramble("U F R U F R U F R U F R") << endl ;
  // cout << ffi_api_solve_position(scrfile) << endl ;
}

#endif
