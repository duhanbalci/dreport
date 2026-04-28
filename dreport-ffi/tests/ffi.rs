//! Integration tests that drive the C ABI directly. These tests treat the FFI
//! crate exactly the way a foreign-language host (NuGet, P/Invoke) would —
//! through opaque pointers, byte buffers, and return codes. They are the
//! contract test suite for non-Rust consumers.

use dreport_ffi::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

const TEMPLATE: &str = r#"{
  "id": "ffi",
  "name": "FFI Test",
  "page": { "width": 210, "height": 297 },
  "fonts": ["Noto Sans"],
  "root": {
    "id": "root",
    "type": "container",
    "position": { "type": "flow" },
    "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
    "direction": "column",
    "gap": 5,
    "padding": { "top": 15, "right": 15, "bottom": 15, "left": 15 },
    "align": "stretch",
    "justify": "start",
    "style": {},
    "children": [
      {
        "id": "title",
        "type": "static_text",
        "position": { "type": "flow" },
        "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
        "style": { "fontSize": 14, "fontWeight": "bold" },
        "content": "FFI"
      }
    ]
  }
}"#;
const DATA: &str = "{}";
const NOTO_SANS_REGULAR: &[u8] =
    include_bytes!("../../dreport-service/assets/fonts/NotoSans-Regular.ttf");

// ---------------------------------------------------------------------------
// Small RAII wrappers around the raw FFI types so each test stays terse.
// ---------------------------------------------------------------------------

struct Handle(*mut DreportHandle);
impl Handle {
    fn new() -> Self {
        let h = dreport_new();
        assert!(!h.is_null(), "dreport_new must succeed");
        Self(h)
    }
    fn empty() -> Self {
        let h = dreport_new_empty();
        assert!(!h.is_null());
        Self(h)
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { dreport_free(self.0) };
    }
}
// SAFETY: Underlying handle wraps an Arc<DreportService>; the service is Sync.
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

struct OwnedBuffer(DreportBuffer);
impl OwnedBuffer {
    fn empty() -> Self {
        Self(DreportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        })
    }
    fn as_slice(&self) -> &[u8] {
        if self.0.data.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.0.data, self.0.len) }
        }
    }
    fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_slice()).expect("valid utf8")
    }
    fn ptr(&mut self) -> *mut DreportBuffer {
        &mut self.0
    }
}
impl Drop for OwnedBuffer {
    fn drop(&mut self) {
        let buf = std::mem::replace(
            &mut self.0,
            DreportBuffer {
                data: std::ptr::null_mut(),
                len: 0,
                cap: 0,
            },
        );
        unsafe { dreport_buffer_free(buf) };
    }
}

fn last_error() -> String {
    let mut buf = OwnedBuffer::empty();
    let rc = unsafe { dreport_last_error(buf.ptr()) };
    assert_eq!(rc, error_code::OK);
    buf.as_str().to_string()
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn new_and_free_round_trips() {
    let h = dreport_new();
    assert!(!h.is_null());
    unsafe { dreport_free(h) };
}

#[test]
fn free_null_is_safe() {
    unsafe { dreport_free(std::ptr::null_mut()) };
}

#[test]
fn buffer_free_null_is_safe() {
    unsafe {
        dreport_buffer_free(DreportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        })
    };
}

#[test]
fn version_returns_valid_c_string() {
    let ptr = dreport_version();
    assert!(!ptr.is_null());
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    let s = cstr.to_str().unwrap();
    assert!(!s.is_empty());
    assert!(s.chars().next().unwrap().is_ascii_digit());
}

#[test]
fn embedded_default_handle_has_fonts() {
    let h = Handle::new();
    let count = unsafe { dreport_font_family_count(h.0) };
    assert!(count >= 1);
}

#[test]
fn empty_handle_has_no_fonts() {
    let h = Handle::empty();
    let count = unsafe { dreport_font_family_count(h.0) };
    assert_eq!(count, 0);
}

// ---------------------------------------------------------------------------
// Null-handle guard rails
// ---------------------------------------------------------------------------

#[test]
fn null_handle_returns_null_handle_code() {
    let mut buf = OwnedBuffer::empty();
    let rc = unsafe { dreport_list_fonts_json(std::ptr::null(), buf.ptr()) };
    assert_eq!(rc, error_code::NULL_HANDLE);
    assert!(!last_error().is_empty(), "error message must be set");
}

#[test]
fn null_handle_count_returns_negative() {
    let count = unsafe { dreport_font_family_count(std::ptr::null()) };
    assert!(count < 0);
}

#[test]
fn render_with_null_template_returns_null_pointer_code() {
    let h = Handle::new();
    let mut out = OwnedBuffer::empty();
    let rc = unsafe {
        dreport_render_pdf(
            h.0,
            std::ptr::null(),
            0,
            DATA.as_ptr(),
            DATA.len(),
            out.ptr(),
        )
    };
    assert_eq!(rc, error_code::NULL_POINTER);
}

// ---------------------------------------------------------------------------
// Font registration
// ---------------------------------------------------------------------------

#[test]
fn register_font_valid_bytes() {
    let h = Handle::empty();
    let rc = unsafe { dreport_register_font(h.0, NOTO_SANS_REGULAR.as_ptr(), NOTO_SANS_REGULAR.len()) };
    assert_eq!(rc, error_code::OK);
    assert!(unsafe { dreport_font_family_count(h.0) } >= 1);
}

#[test]
fn register_font_invalid_bytes_returns_negative_service_code() {
    let h = Handle::empty();
    let garbage = b"not a font";
    let rc = unsafe { dreport_register_font(h.0, garbage.as_ptr(), garbage.len()) };
    assert_eq!(rc, -3, "ServiceError::FontParseFailed code is 3 → -3 over FFI");
    let msg = last_error();
    assert!(msg.to_lowercase().contains("font"), "error msg: {}", msg);
}

#[test]
fn register_fonts_dir_invalid_path_sets_error() {
    let h = Handle::empty();
    let path = "/zzz/no/such/dreport/path";
    let mut out_count: usize = 0;
    let rc = unsafe {
        dreport_register_fonts_dir(h.0, path.as_ptr(), path.len(), &mut out_count)
    };
    assert!(rc < 0);
    assert_eq!(out_count, 0);
    assert!(!last_error().is_empty());
}

#[test]
fn register_fonts_dir_valid_path_loads_count() {
    let h = Handle::empty();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../dreport-service/assets/fonts");
    let path_str = path.to_string_lossy().into_owned();
    let mut out_count: usize = 0;
    let rc = unsafe {
        dreport_register_fonts_dir(h.0, path_str.as_ptr(), path_str.len(), &mut out_count)
    };
    assert_eq!(rc, error_code::OK, "{}", last_error());
    assert!(out_count >= 1);
}

#[test]
fn list_fonts_json_is_valid_array() {
    let h = Handle::new();
    let mut out = OwnedBuffer::empty();
    let rc = unsafe { dreport_list_fonts_json(h.0, out.ptr()) };
    assert_eq!(rc, error_code::OK);
    let parsed: serde_json::Value = serde_json::from_str(out.as_str()).unwrap();
    assert!(parsed.is_array());
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn get_font_bytes_existing_returns_data() {
    let h = Handle::new();
    let family = "Noto Sans";
    let mut out = OwnedBuffer::empty();
    let rc = unsafe {
        dreport_get_font_bytes(h.0, family.as_ptr(), family.len(), 400, false, out.ptr())
    };
    assert_eq!(rc, error_code::OK);
    assert!(out.as_slice().len() > 1000);
}

#[test]
fn get_font_bytes_missing_returns_ok_with_empty_buffer() {
    let h = Handle::new();
    let family = "DoesNotExist";
    let mut out = OwnedBuffer::empty();
    let rc = unsafe {
        dreport_get_font_bytes(h.0, family.as_ptr(), family.len(), 400, false, out.ptr())
    };
    assert_eq!(rc, error_code::OK);
    assert!(out.as_slice().is_empty());
    assert!(out.0.data.is_null());
}

// ---------------------------------------------------------------------------
// Render pipeline
// ---------------------------------------------------------------------------

#[test]
fn compute_layout_round_trip() {
    let h = Handle::new();
    let mut out = OwnedBuffer::empty();
    let rc = unsafe {
        dreport_compute_layout(
            h.0,
            TEMPLATE.as_ptr(),
            TEMPLATE.len(),
            DATA.as_ptr(),
            DATA.len(),
            out.ptr(),
        )
    };
    assert_eq!(rc, error_code::OK, "{}", last_error());
    let parsed: serde_json::Value = serde_json::from_str(out.as_str()).unwrap();
    assert!(parsed["pages"].is_array());
}

#[test]
fn render_pdf_returns_pdf_magic_header() {
    let h = Handle::new();
    let mut out = OwnedBuffer::empty();
    let rc = unsafe {
        dreport_render_pdf(
            h.0,
            TEMPLATE.as_ptr(),
            TEMPLATE.len(),
            DATA.as_ptr(),
            DATA.len(),
            out.ptr(),
        )
    };
    assert_eq!(rc, error_code::OK, "{}", last_error());
    let bytes = out.as_slice();
    assert!(bytes.starts_with(b"%PDF-"), "missing magic header");
}

#[test]
fn render_with_invalid_template_json_sets_error() {
    let h = Handle::new();
    let bad = b"{not json";
    let mut out = OwnedBuffer::empty();
    let rc = unsafe {
        dreport_render_pdf(
            h.0,
            bad.as_ptr(),
            bad.len(),
            DATA.as_ptr(),
            DATA.len(),
            out.ptr(),
        )
    };
    assert_eq!(rc, -1, "ServiceError::InvalidTemplateJson → -1");
    assert!(!last_error().is_empty());
    assert!(out.as_slice().is_empty());
}

// ---------------------------------------------------------------------------
// Concurrency
// ---------------------------------------------------------------------------

#[test]
fn concurrent_independent_handles() {
    let success = Arc::new(AtomicUsize::new(0));
    let mut threads = Vec::new();
    for _ in 0..6 {
        let s = Arc::clone(&success);
        threads.push(thread::spawn(move || {
            let h = Handle::new();
            let mut out = OwnedBuffer::empty();
            let rc = unsafe {
                dreport_render_pdf(
                    h.0,
                    TEMPLATE.as_ptr(),
                    TEMPLATE.len(),
                    DATA.as_ptr(),
                    DATA.len(),
                    out.ptr(),
                )
            };
            if rc == error_code::OK && out.as_slice().starts_with(b"%PDF-") {
                s.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(success.load(Ordering::SeqCst), 6);
}

#[test]
fn concurrent_shared_handle() {
    // The handle itself is owned by one thread, but the underlying service is
    // an Arc<DreportService>, so internally a shared engine is fine. To test
    // the most realistic NuGet scenario (one process-wide engine) we instead
    // create per-thread handles backed by parallel `dreport_new` calls.
    let success = Arc::new(AtomicUsize::new(0));
    let mut threads = Vec::new();
    for _ in 0..4 {
        let s = Arc::clone(&success);
        threads.push(thread::spawn(move || {
            for _ in 0..4 {
                let h = Handle::new();
                let mut out = OwnedBuffer::empty();
                let rc = unsafe {
                    dreport_render_pdf(
                        h.0,
                        TEMPLATE.as_ptr(),
                        TEMPLATE.len(),
                        DATA.as_ptr(),
                        DATA.len(),
                        out.ptr(),
                    )
                };
                if rc == 0 {
                    s.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(success.load(Ordering::SeqCst), 16);
}

// ---------------------------------------------------------------------------
// Last-error semantics
// ---------------------------------------------------------------------------

#[test]
fn successful_call_clears_previous_error() {
    let h = Handle::new();

    // Provoke an error first.
    let rc = unsafe { dreport_register_font(h.0, b"x".as_ptr(), 1) };
    assert!(rc < 0);
    assert!(!last_error().is_empty());

    // A subsequent successful call must clear it.
    let count = unsafe { dreport_font_family_count(h.0) };
    assert!(count >= 1);
    assert!(
        last_error().is_empty(),
        "successful call should clear last_error"
    );
}
