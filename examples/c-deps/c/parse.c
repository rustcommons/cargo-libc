/* _GNU_SOURCE makes glibc 2.38+ headers redirect strtol to __isoc23_strtol
   (GLIBC_2.38): only the selected release's headers keep this target-safe. */
#define _GNU_SOURCE
#include <features.h>
#include <stdlib.h>

long c_parse(const char *text) { return strtol(text, NULL, 0); }

int c_header_minor(void) { return __GLIBC_MINOR__; }
