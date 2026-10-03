#pragma once
// ripgrep 14.1.1 in the caller's process: src/lib.rs.
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// rg with `argv` (`argc` words, as rg takes them after its name). Each file it
// matched in, in path order, goes to `found`: its path as rg prints it,
// '/'-separated; the bytes it searched; the lines it matched, 1-based and
// ascending. stdin is searched when rg's rules say so, under "<stdin>". Returns
// 0 when a file matched, 1 when none did, 2 when rg refused the arguments, with
// its reason sent to `failed`. A file it cannot read is reported on stderr, as
// rg reports it, and skipped.
int rg_search(const char* const* argv, size_t argc, void* context,
              void (*found)(void* context, const char* path, size_t path_len, const uint8_t* bytes,
                            size_t len, const uint64_t* lines, size_t count),
              void (*failed)(void* context, const char* message, size_t len));

#ifdef __cplusplus
}
#endif
