#![allow(unsafe_code)]

use std::ffi::c_char;
use std::path::PathBuf;

/// Canonical Awesome-OS B8G ABI v3 magic (`"B8G3"` little-endian).
pub const B8G_ABI_MAGIC_V3: u32 = 0x3356_4742;

static UVPY_VERSION_CSTR: &[u8] = b"0.13.10-awesome-os.1\0";

#[unsafe(no_mangle)]
pub extern "C" fn b8g_uvpy_abi_magic() -> u32 {
    B8G_ABI_MAGIC_V3
}

#[unsafe(no_mangle)]
pub extern "C" fn b8g_uvpy_version() -> *const c_char {
    UVPY_VERSION_CSTR.as_ptr().cast::<c_char>()
}

fn write_path_to_buf(path: PathBuf, out_buf: *mut c_char, out_cap: usize) -> i32 {
    let s = path.to_string_lossy();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let Ok(len_i32) = i32::try_from(len) else {
        return -3;
    };

    if out_buf.is_null() || out_cap == 0 {
        return len_i32;
    }
    if out_cap <= len {
        return -2;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf.cast::<u8>(), len);
        *out_buf.add(len) = 0;
    }
    len_i32
}

/// Resolves `<B8G_GHOST_CACHE_DIR>/uv` into `out_buf`.
///
/// Returns:
/// - `>= 0`: UTF-8 byte length (excluding NUL terminator)
/// - `-1`: `B8G_GHOST_CACHE_DIR` is unset or empty
/// - `-2`: `out_cap` is too small (must be `>= len + 1`)
#[unsafe(no_mangle)]
pub extern "C" fn b8g_uvpy_resolve_cas_root(out_buf: *mut c_char, out_cap: usize) -> i32 {
    let Some(root) = std::env::var_os("B8G_GHOST_CACHE_DIR") else {
        return -1;
    };
    if root.is_empty() {
        return -1;
    }
    write_path_to_buf(PathBuf::from(root).join("uv"), out_buf, out_cap)
}

/// Resolves `<B8G_GHOST_CACHE_DIR>/uv/python` into `out_buf`.
///
/// Returns:
/// - `>= 0`: UTF-8 byte length (excluding NUL terminator)
/// - `-1`: `B8G_GHOST_CACHE_DIR` is unset or empty
/// - `-2`: `out_cap` is too small (must be `>= len + 1`)
#[unsafe(no_mangle)]
pub extern "C" fn b8g_uvpy_resolve_python_dir(out_buf: *mut c_char, out_cap: usize) -> i32 {
    let Some(root) = std::env::var_os("B8G_GHOST_CACHE_DIR") else {
        return -1;
    };
    if root.is_empty() {
        return -1;
    }
    write_path_to_buf(PathBuf::from(root).join("uv").join("python"), out_buf, out_cap)
}
