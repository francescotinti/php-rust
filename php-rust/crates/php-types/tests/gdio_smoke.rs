//! Smoke tests for the libgd FFI: struct layout (sx/sy/trueColor via the
//! mirrored gdImageStruct), encode round-trip, and the va_list error callback.

use php_types::gdio::{self, GdImg};

#[test]
fn create_encode_decode_roundtrip() {
    let mut im = GdImg::create(40, 30, true).expect("create truecolor");
    assert_eq!(im.sx(), 40);
    assert_eq!(im.sy(), 30);
    assert!(im.true_color());
    assert_eq!(im.colors_total(), 0);
    assert_eq!(im.transparent(), -1);
    im.filled_rectangle(0, 0, 39, 29, 0x00FF00);
    let png = im.encode("png", -1, 0).expect("png encode");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    let back = GdImg::decode("png", &png).expect("png decode");
    assert_eq!(back.sx(), 40);
    assert_eq!(back.get_pixel(5, 5), 0x00FF00);

    let jpg = im.encode("jpeg", 82, 0).expect("jpeg encode");
    assert_eq!(&jpg[..3], &[0xFF, 0xD8, 0xFF]);
    assert!(GdImg::decode("jpeg", &jpg).is_some());
}

#[test]
fn palette_semantics() {
    let mut im = GdImg::create(8, 8, false).expect("create palette");
    assert!(!im.true_color());
    assert_eq!(im.colors_total(), 0);
    let white = im.color_allocate(255, 255, 255, None);
    let red = im.color_allocate(255, 0, 0, None);
    assert_eq!(white, 0);
    assert_eq!(red, 1);
    assert_eq!(im.colors_total(), 2);
    assert_eq!(im.palette_entry(1), (255, 0, 0, 0));
}

#[test]
fn error_callback_formats_va_args() {
    gdio::ensure_error_handler();
    let _ = gdio::take_errors();
    // Not a JPEG: libgd reports through the error method with printf args.
    assert!(GdImg::decode("jpeg", b"this is not a jpeg at all").is_none());
    let errors = gdio::take_errors();
    assert!(
        errors.iter().any(|e| e.contains("starts with 0x74 0x68")),
        "expected formatted libjpeg error, got: {errors:?}"
    );
}
