/*
 Minimal libc stubs to satisfy libgcov / basic C runtime needs on bare-metal.
 - These are intentionally small, not fully standards-compliant, and meant for test/coverage use.
 - They MUST be compiled/linked WITHOUT function instrumentation (no -finstrument-functions
   and do not instrument these translation units).
 - Do NOT depend on malloc/free here. If output exceeds internal buffer, it will be truncated.
 - The platform must provide a binary write primitive with signature:
       int _write(int fd, const void *buf, int count);
   typically implemented by your memfs/UART/semihost layer.

 Implemented symbols:
   strcpy, strcat, strchr, strlen, strcmp, sprintf, vsnprintf, vfprintf, fprintf,
   fputs, atoi, getenv (returns NULL), getpid (returns 1), _impure_ptr

 Notes:
 - These implementations assume callers follow the C standard semantics (pointers passed
   to standard library functions like strcpy/strcat/strlen/strcmp/fputs are non-NULL).
 - Avoids comparisons against NULL for parameters that compilers mark as nonnull to
   silence -Wnonnull-compare. If you need defensive behavior for NULL pointers,
   compile this TU with -fno-builtin or add explicit pragmas.
 - The sprintf implementation below formats into a temporary bounded buffer (SPRINTF_TMP_SZ)
   to avoid using (size_t)-1 which triggers -Wformat-truncation.
*/

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include "vfw_mb.h"
#include <stdlib.h>
#include <string.h>
#include <gcov.h>

/* Declare external write primitive provided by platform (memfs or UART) */
// __attribute__((no_instrument_function)) extern int _write(int fd, const void *buf, int count);

/* Small helper macros to mark functions uninstrumented */
#define NOINSTR __attribute__((no_instrument_function))

struct gcda_ctx
{
    FD fd; /* memfs file descriptor (>=3) */
    /* You can add more fields (filename copy, counters, etc.) */
};

NOINSTR static void
dump(const void *d, unsigned n, void *arg)
{
    struct gcda_ctx *ctx = (struct gcda_ctx *)arg;
    /* write may be truncated if memfs full; ignore errors for now */
    mailbox_fwrite(ctx->fd, d, n);
}

/* The filename is serialized to a gcfn data stream by the
   __gcov_filename_to_gcfn() function.  The gcfn data is used by the
   "merge-stream" subcommand of the "gcov-tool" to figure out the filename
   associated with the gcov information. */

NOINSTR
const char *path_basename(const char *path)
{
    if (!path)
        return NULL;
    /* find end */
    const char *end = path + strlen(path);
    if (end == path)
        return path; /* empty string */

    /* skip trailing separators */
    const char *p = end;
    while (p > path && (*(p - 1) == '/' || *(p - 1) == '\\'))
        --p;

    if (p == path)
        return path; /* only separators or empty */

    /* find start of last component */
    const char *comp = p;
    while (comp > path)
    {
        if (*(comp - 1) == '/' || *(comp - 1) == '\\')
            break;
        --comp;
    }
    return comp;
}

NOINSTR static void
filename(const char *f, void *arg)
{
    struct gcda_ctx *ctx = (struct gcda_ctx *)arg;
    /* Flags: create/truncate/write-only (platform _open returns fd >= 3) */
    FD fd = mailbox_fopen(path_basename(f), /* flags */ FILE_WRITE /* O_CREAT|O_WRONLY|O_TRUNC? platform-specific */);
    /* If open fails, set fd to -1; dump callback should handle it */
    ctx->fd = fd;
}

/* The __gcov_info_to_gcda() function may have to allocate memory under
   certain conditions.  Simply try it out if it is needed for your application
   or not.  */

NOINSTR static void *
allocate(unsigned length, void *arg)
{
    (void)arg;
    return malloc(length);
}

/* Dump the gcov information of all translation units.  */

extern const struct gcov_info *const __gcov_info_start[];
extern const struct gcov_info *const __gcov_info_end[];

NOINSTR void
dump_gcov_info()
{
    const struct gcov_info *const *info = __gcov_info_start;
    const struct gcov_info *const *end = __gcov_info_end;

    /* Obfuscate variable to prevent compiler optimizations.  */
    __asm__("" : "+r"(info));

    while (info != end)
    {
        struct gcda_ctx ctx = {.fd = 0};
        __gcov_info_to_gcda(*info, filename, dump, allocate, &ctx);
        mailbox_fclose(ctx.fd);
        ++info;
    }
}

/* End of stubs */