#include <stdio.h>
#include "../../../target/release/libtwips_ffi.h"


void generate_scramble(const char* event_id) {
  printf("// %s\n", event_id);
  puts(ffi_random_scramble_for_event(event_id));

}

int main() {
  // Here we just make sure we can make the call without crashing.
  // `js_test.ts` tests the actual output.
  puts("Derived scramble: ");
  puts(ffi_derive_scramble_for_event("67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67", "EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF/scrambles/333/r1/g1/a1/333/sub1", CUBING_EVENT_333));

  printf("Freed %u scramble finder(s).\n", ffi_free_memory_for_all_scramble_finders()); 

  generate_scramble(CUBING_EVENT_222);
  generate_scramble(CUBING_EVENT_PYRAM);
  generate_scramble(CUBING_EVENT_MINX);
  generate_scramble(CUBING_EVENT_333);
  generate_scramble(CUBING_EVENT_333BF);
  generate_scramble(CUBING_EVENT_333FM);
  generate_scramble(CUBING_EVENT_555);
  generate_scramble(CUBING_EVENT_666);
  generate_scramble(CUBING_EVENT_777);
  generate_scramble(CUBING_EVENT_SKEWB);
  generate_scramble(CUBING_EVENT_SQ1);

  printf("Freed %u scramble finder(s).\n", ffi_free_memory_for_all_scramble_finders()); 
}
