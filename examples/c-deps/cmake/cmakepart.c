#include <features.h>
#include <gnu/libc-version.h>

const char *cmake_running_glibc(void) { return gnu_get_libc_version(); }

int cmake_header_minor(void) { return __GLIBC_MINOR__; }

int cmake_has_memfd_create(void) {
#ifdef HAVE_MEMFD_CREATE
  return 1;
#else
  return 0;
#endif
}
