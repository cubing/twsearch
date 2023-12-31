#include <stdio.h>
#include "../../../target/release/libtwsearch_ffi.h"

int main() {
  puts("// 222");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_222));

  puts("// pyram");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_PYRAM));

  puts("// minx");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_MINX));

  puts("// 333");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_333));

  puts("// 555");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_555));

  puts("// 666");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_666));

  puts("// 777");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_777));
}
