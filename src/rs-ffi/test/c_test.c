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

  puts("// 333bf");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_333BF));

  puts("// 333fm");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_333FM));

  puts("// 555");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_555));

  puts("// 666");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_666));

  puts("// 777");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_777));

  puts("// skewb");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_SKEWB));

  puts("// sq1");
  puts(ffi_random_scramble_for_event(CUBING_EVENT_SQ1));

  printf("Freed %u scramble finder(s).\n", ffi_free_memory_for_all_scramble_finders()); 
}
