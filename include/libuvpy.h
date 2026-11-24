#ifndef LIBUVPY_H
#define LIBUVPY_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Canonical Awesome-OS B8G ABI v3 magic ("B8G3"). */
#define B8G_ABI_MAGIC_V3 0x33564742u

/** Returns B8G_ABI_MAGIC_V3 (0x33564742). */
uint32_t b8g_uvpy_abi_magic(void);

/** Returns static NUL-terminated Awesome-OS uv (libuvpy) version string. */
const char* b8g_uvpy_version(void);

/**
 * Resolves <B8G_GHOST_CACHE_DIR>/uv into out_buf (NUL-terminated UTF-8).
 *
 * Returns:
 *   >= 0 : byte length excluding NUL terminator (or required length if out_buf == NULL || out_cap == 0)
 *   -1   : B8G_GHOST_CACHE_DIR is unset or empty
 *   -2   : out_cap <= required length
 */
int32_t b8g_uvpy_resolve_cas_root(char* out_buf, size_t out_cap);

/**
 * Resolves <B8G_GHOST_CACHE_DIR>/uv/python into out_buf (NUL-terminated UTF-8).
 *
 * Returns:
 *   >= 0 : byte length excluding NUL terminator (or required length if out_buf == NULL || out_cap == 0)
 *   -1   : B8G_GHOST_CACHE_DIR is unset or empty
 *   -2   : out_cap <= required length
 */
int32_t b8g_uvpy_resolve_python_dir(char* out_buf, size_t out_cap);

#ifdef __cplusplus
}
#endif

#endif /* LIBUVPY_H */
