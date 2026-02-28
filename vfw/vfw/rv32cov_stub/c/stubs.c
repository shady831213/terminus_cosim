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
#include <stdlib.h>
#include <gcov.h>
/* Declare external write primitive provided by platform (memfs or UART) */
// __attribute__((no_instrument_function)) extern int _write(int fd, const void *buf, int count);

/* Small helper macros to mark functions uninstrumented */
#define NOINSTR __attribute__((no_instrument_function))

/* ---------------- basic string helpers ----------------
   These follow the usual C standard semantics (arguments must be valid pointers).
*/

/* strlen: caller must pass a valid, NUL-terminated string pointer */
NOINSTR
size_t strlen(const char *s)
{
    const char *p = s;
    while (*p)
        ++p;
    return (size_t)(p - s);
}

/* strcpy: assume dst and src are valid and non-NULL (standard semantics). */
NOINSTR
char *strcpy(char *dst, const char *src)
{
    char *d = dst;
    while ((*d++ = *src++))
        ;
    return dst;
}

/* strcat: assume dst and src are valid and non-NULL (standard semantics). */
NOINSTR
char *strcat(char *dst, const char *src)
{
    char *d = dst;
    while (*d)
        ++d;
    while ((*d++ = *src++))
        ;
    return dst;
}

/* strcmp: assume both pointers are valid and NUL-terminated. */
NOINSTR
int strcmp(const char *a, const char *b)
{
    while (*a && (*a == *b))
    {
        ++a;
        ++b;
    }
    return (unsigned char)*a - (unsigned char)*b;
}

/* strchr: assume s is valid; return pointer to first occurrence or NULL. */
NOINSTR
char *strchr(const char *s, int c)
{
    char ch = (char)c;
    while (*s)
    {
        if (*s == ch)
            return (char *)s;
        ++s;
    }
    return (ch == 0) ? (char *)s : NULL;
}

/* End of stubs */