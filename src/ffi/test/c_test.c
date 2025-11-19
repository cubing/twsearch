#include <stdio.h>
#include "../../../target/release/libtwips_ffi.h"


void generate_scramble(const char* event_id) {
  printf("// %s\n", event_id);
  puts(ffi_random_scramble_for_event(event_id));

}

int main() {
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
