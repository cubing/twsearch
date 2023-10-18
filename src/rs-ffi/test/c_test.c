#include <stdio.h>

extern char* ffi_random_scramble_for_event(char *event_id);

int main() {
  puts("// 222");
  puts(ffi_random_scramble_for_event("222"));

  puts("// pyram");
  puts(ffi_random_scramble_for_event("pyram"));

  puts("// 333");
  puts(ffi_random_scramble_for_event("333"));
}
