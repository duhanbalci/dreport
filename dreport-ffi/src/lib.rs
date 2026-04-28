//! dreport-ffi
//!
//! C ABI exposing `dreport_service::DreportService` to non-Rust hosts
//! (.NET / NuGet, Node N-API, Python ctypes, etc.).
//!
//! ## Conventions
//!
//! - All exported symbols are prefixed `dreport_`.
//! - Functions return `i32`: `0 == success`, negative values are error codes.
//!   See [`error_code`] constants. The detailed message for the most recent
//!   error on the calling thread is retrievable via [`dreport_last_error`].
//! - Outbound dynamic data is returned as a [`DreportBuffer`] (ptr + len + cap).
//!   The caller MUST hand the buffer back to [`dreport_buffer_free`] to release it.
//! - Inbound strings are passed as `(ptr, len)` byte pairs and interpreted as UTF-8.
//! - Handles ([`DreportHandle`]) are opaque pointers. Pass them to
//!   [`dreport_free`] exactly once when done. Never use after free.
//! - All exported functions are safe to call from any thread; the underlying
//!   service is `Sync`.

#![allow(clippy::missing_safety_doc)] // safety contract documented at module level

use std::cell::RefCell;
use std::ffi::c_char;
use std::sync::Arc;

use dreport_service::{DreportService, ServiceError};

// ---------------------------------------------------------------------------
// Return codes
// ---------------------------------------------------------------------------

pub mod error_code {
    pub const OK: i32 = 0;
    pub const NULL_HANDLE: i32 = -100;
    pub const NULL_POINTER: i32 = -101;
    pub const INVALID_UTF8: i32 = -102;
    pub const PANIC: i32 = -103;

    // Service-level errors are exposed as the negation of `ServiceError::code()`.
    // E.g. ServiceError::FontParseFailed (3) → -3 here.
}

// ---------------------------------------------------------------------------
// Opaque handle
// ---------------------------------------------------------------------------

/// Opaque handle backing a `DreportService` shared across the FFI boundary.
/// Internally an `Arc<DreportService>`, so the same engine can be cloned and
/// driven from multiple threads.
pub struct DreportHandle {
    inner: Arc<DreportService>,
}

// ---------------------------------------------------------------------------
// Outbound buffer
// ---------------------------------------------------------------------------

/// Owned byte buffer returned across the FFI boundary. Released with
/// [`dreport_buffer_free`].
#[repr(C)]
pub struct DreportBuffer {
    pub data: *mut u8,
    pub len: usize,
    pub cap: usize,
}

impl DreportBuffer {
    fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    fn from_vec(mut v: Vec<u8>) -> Self {
        v.shrink_to_fit();
        let buf = Self {
            data: v.as_mut_ptr(),
            len: v.len(),
            cap: v.capacity(),
        };
        std::mem::forget(v);
        buf
    }
}

// ---------------------------------------------------------------------------
// Thread-local error state
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_last_error(msg: impl Into<String>) {
    LAST_ERROR.with(|cell| *cell.borrow_mut() = Some(msg.into()));
}

fn clear_last_error() {
    LAST_ERROR.with(|cell| *cell.borrow_mut() = None);
}

fn map_service_error(err: ServiceError) -> i32 {
    let code = -err.code();
    set_last_error(err.to_string());
    code
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

unsafe fn handle_ref<'a>(handle: *const DreportHandle) -> Option<&'a DreportHandle> {
    if handle.is_null() {
        set_last_error("null handle");
        None
    } else {
        Some(unsafe { &*handle })
    }
}

unsafe fn slice_from_raw<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        set_last_error("null pointer for input slice");
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(ptr, len) })
    }
}

unsafe fn str_from_raw<'a>(ptr: *const u8, len: usize) -> Result<&'a str, i32> {
    if ptr.is_null() {
        set_last_error("null pointer for input string");
        return Err(error_code::NULL_POINTER);
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes).map_err(|e| {
        set_last_error(format!("invalid utf-8: {}", e));
        error_code::INVALID_UTF8
    })
}

unsafe fn write_buffer(out: *mut DreportBuffer, buffer: DreportBuffer) -> i32 {
    if out.is_null() {
        set_last_error("null out buffer pointer");
        return error_code::NULL_POINTER;
    }
    unsafe { *out = buffer };
    error_code::OK
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

/// Allocate a new service handle with default embedded fonts.
#[unsafe(no_mangle)]
pub extern "C" fn dreport_new() -> *mut DreportHandle {
    clear_last_error();
    Box::into_raw(Box::new(DreportHandle {
        inner: Arc::new(DreportService::new()),
    }))
}

/// Allocate an empty service handle (no embedded fonts).
#[unsafe(no_mangle)]
pub extern "C" fn dreport_new_empty() -> *mut DreportHandle {
    clear_last_error();
    Box::into_raw(Box::new(DreportHandle {
        inner: Arc::new(DreportService::empty()),
    }))
}

/// Release a service handle previously returned by `dreport_new` /
/// `dreport_new_empty`. Calling with `NULL` is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_free(handle: *mut DreportHandle) {
    if handle.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(handle) });
}

/// Release a buffer previously produced by an FFI call. Calling with a buffer
/// whose `data` is NULL or whose `cap` is 0 is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_buffer_free(buffer: DreportBuffer) {
    if buffer.data.is_null() || buffer.cap == 0 {
        return;
    }
    drop(unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.cap) });
}

/// Returns the static crate version string. Pointer remains valid for the
/// lifetime of the loaded library.
#[unsafe(no_mangle)]
pub extern "C" fn dreport_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// Copy the most recent error message produced on this thread into `out`.
/// Returns `error_code::OK` on success (even if there is no error — the buffer
/// will simply be empty). The buffer must be released with `dreport_buffer_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_last_error(out: *mut DreportBuffer) -> i32 {
    let msg = LAST_ERROR.with(|cell| cell.borrow().clone()).unwrap_or_default();
    let buf = if msg.is_empty() {
        DreportBuffer::empty()
    } else {
        DreportBuffer::from_vec(msg.into_bytes())
    };
    unsafe { write_buffer(out, buf) }
}

// ---------------------------------------------------------------------------
// Font registry operations
// ---------------------------------------------------------------------------

/// Register a font from raw TTF/OTF bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_register_font(
    handle: *const DreportHandle,
    data: *const u8,
    len: usize,
) -> i32 {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE;
    };
    let Some(bytes) = (unsafe { slice_from_raw(data, len) }) else {
        return error_code::NULL_POINTER;
    };
    match h.inner.register_font_bytes(bytes.to_vec()) {
        Ok(_) => error_code::OK,
        Err(e) => map_service_error(e),
    }
}

/// Register every font file in `path` (UTF-8 directory path).
/// Returns the count via `out_count` (negative on error).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_register_fonts_dir(
    handle: *const DreportHandle,
    path: *const u8,
    path_len: usize,
    out_count: *mut usize,
) -> i32 {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE;
    };
    let p = match unsafe { str_from_raw(path, path_len) } {
        Ok(s) => s,
        Err(rc) => return rc,
    };
    match h.inner.register_fonts_directory(p) {
        Ok(n) => {
            if !out_count.is_null() {
                unsafe { *out_count = n };
            }
            error_code::OK
        }
        Err(e) => map_service_error(e),
    }
}

/// List all registered font families as a JSON array
/// `[{"family":"Noto Sans","variants":[{"weight":400,"italic":false}, ...]}]`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_list_fonts_json(
    handle: *const DreportHandle,
    out: *mut DreportBuffer,
) -> i32 {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE;
    };
    let families = h.inner.list_font_families();
    match serde_json::to_vec(&families) {
        Ok(v) => unsafe { write_buffer(out, DreportBuffer::from_vec(v)) },
        Err(e) => {
            set_last_error(format!("serialize fonts: {}", e));
            -ServiceError::SerializationFailed(String::new()).code()
        }
    }
}

/// Get the raw bytes for a specific font variant. Sets `out` to an empty buffer
/// (data=NULL,len=0) and returns OK if the variant does not exist; this lets
/// the caller distinguish "missing" from "error" by inspecting `out.data`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_get_font_bytes(
    handle: *const DreportHandle,
    family: *const u8,
    family_len: usize,
    weight: u16,
    italic: bool,
    out: *mut DreportBuffer,
) -> i32 {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE;
    };
    let fam = match unsafe { str_from_raw(family, family_len) } {
        Ok(s) => s,
        Err(rc) => return rc,
    };
    let buf = match h.inner.get_font_bytes(fam, weight, italic) {
        Some(v) => DreportBuffer::from_vec(v),
        None => DreportBuffer::empty(),
    };
    unsafe { write_buffer(out, buf) }
}

// ---------------------------------------------------------------------------
// Render pipeline
// ---------------------------------------------------------------------------

/// Compute layout. Returns the LayoutResult JSON via `out`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_compute_layout(
    handle: *const DreportHandle,
    template: *const u8,
    template_len: usize,
    data: *const u8,
    data_len: usize,
    out: *mut DreportBuffer,
) -> i32 {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE;
    };
    let tpl = match unsafe { str_from_raw(template, template_len) } {
        Ok(s) => s,
        Err(rc) => return rc,
    };
    let d = match unsafe { str_from_raw(data, data_len) } {
        Ok(s) => s,
        Err(rc) => return rc,
    };
    match h.inner.compute_layout_json(tpl, d) {
        Ok(json) => unsafe { write_buffer(out, DreportBuffer::from_vec(json.into_bytes())) },
        Err(e) => map_service_error(e),
    }
}

/// Render PDF. Returns PDF bytes via `out`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_render_pdf(
    handle: *const DreportHandle,
    template: *const u8,
    template_len: usize,
    data: *const u8,
    data_len: usize,
    out: *mut DreportBuffer,
) -> i32 {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE;
    };
    let tpl = match unsafe { str_from_raw(template, template_len) } {
        Ok(s) => s,
        Err(rc) => return rc,
    };
    let d = match unsafe { str_from_raw(data, data_len) } {
        Ok(s) => s,
        Err(rc) => return rc,
    };
    match h.inner.render_pdf_json(tpl, d) {
        Ok(pdf) => unsafe { write_buffer(out, DreportBuffer::from_vec(pdf)) },
        Err(e) => map_service_error(e),
    }
}

/// Number of distinct font families currently registered. Returns a negative
/// value if the handle is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreport_font_family_count(handle: *const DreportHandle) -> isize {
    clear_last_error();
    let Some(h) = (unsafe { handle_ref(handle) }) else {
        return error_code::NULL_HANDLE as isize;
    };
    h.inner.font_family_count() as isize
}
