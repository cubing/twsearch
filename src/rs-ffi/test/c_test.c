#include <stdio.h>

extern char* ffi_random_scramble_for_event(char *event_id);

int main() {
  puts("// 222");
  puts(ffi_random_scramble_for_event("222"));

  puts("// pyram");
  puts(ffi_random_scramble_for_event("pyram"));

  puts("// minx");
  puts(ffi_random_scramble_for_event("minx"));

  puts("// 333");
  puts(ffi_random_scramble_for_event("333"));

  puts("// 555");
  puts(ffi_random_scramble_for_event("555"));

  puts("// 666");
  puts(ffi_random_scramble_for_event("666"));

  puts("// 777");
  puts(ffi_random_scramble_for_event("777"));
}
