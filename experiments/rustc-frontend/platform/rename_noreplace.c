/* First-party Linux no-replace publication; no overwrite-capable fallback. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE 1
#endif
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <unistd.h>

int main(int argc, char **argv) {
  if (argc != 3) {
    fputs("usage: rename_noreplace SOURCE DESTINATION\n", stderr);
    return EXIT_FAILURE;
  }
  /* The pinned C ABI is glibc 2.17, older than its renameat2 wrapper. */
  if (syscall(SYS_renameat2, AT_FDCWD, argv[1], AT_FDCWD, argv[2],
              RENAME_NOREPLACE) != 0) {
    perror("no-replace rename");
    return EXIT_FAILURE;
  }
  return EXIT_SUCCESS;
}
