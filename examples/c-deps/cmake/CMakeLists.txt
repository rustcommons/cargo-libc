cmake_minimum_required(VERSION 3.10)
# C only: the target has no C++ runtime.
project(cmakepart C)
include(CheckFunctionExists)
# Configure checks link test programs; they must see the target's glibc.
check_function_exists(memfd_create HAVE_MEMFD_CREATE)
add_library(cmakepart STATIC cmakepart.c)
if(HAVE_MEMFD_CREATE)
  target_compile_definitions(cmakepart PRIVATE HAVE_MEMFD_CREATE=1)
endif()
install(TARGETS cmakepart ARCHIVE DESTINATION lib)
