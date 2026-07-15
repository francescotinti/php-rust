//! ext/gd over the **system libgd** (the Homebrew keg the PHP oracle itself
//! links): thin FFI, so every decode/encode goes through the *same* codec
//! stack (libjpeg-turbo, libpng, libwebp, libavif) and the generated file
//! bytes are identical to PHP's — a Rust-crate reimplementation (`image`)
//! would diverge on every encode (resampling weights, encoder settings).
//! Mirrors the zlibio pattern: the FFI lives once, in the bottom crate; the
//! GdImage handle table and the `__gd_*` host builtins live in vm/gd.rs.
//!
//! libgd reports codec problems through a `gdSetErrorMethod` callback with a
//! `va_list`; [`take_errors`] drains the formatted messages recorded during
//! the last call so the VM can turn them into PHP Warnings (PHP's
//! `php_gd_error_method` does exactly this).

use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_int, c_uint, c_void};

pub const MAX_COLORS: usize = 256;

/// `gdImageStruct` from the keg's gd.h (2.3.3). Only read directly for the
/// accessor-macro fields (`gdImageSX`, `gdImageTrueColor`, …); everything
/// else goes through exported functions.
#[repr(C)]
pub struct GdImageRaw {
    pub pixels: *mut *mut u8,
    pub sx: c_int,
    pub sy: c_int,
    pub colors_total: c_int,
    pub red: [c_int; MAX_COLORS],
    pub green: [c_int; MAX_COLORS],
    pub blue: [c_int; MAX_COLORS],
    pub open: [c_int; MAX_COLORS],
    pub transparent: c_int,
    pub poly_ints: *mut c_int,
    pub poly_allocated: c_int,
    pub brush: *mut GdImageRaw,
    pub tile: *mut GdImageRaw,
    pub brush_color_map: [c_int; MAX_COLORS],
    pub tile_color_map: [c_int; MAX_COLORS],
    pub style_length: c_int,
    pub style_pos: c_int,
    pub style: *mut c_int,
    pub interlace: c_int,
    pub thick: c_int,
    pub alpha: [c_int; MAX_COLORS],
    pub true_color: c_int,
    pub tpixels: *mut *mut c_int,
    pub alpha_blending_flag: c_int,
    pub save_alpha_flag: c_int,
    pub aa: c_int,
    pub aa_color: c_int,
    pub aa_dont_blend: c_int,
    pub cx1: c_int,
    pub cy1: c_int,
    pub cx2: c_int,
    pub cy2: c_int,
    pub res_x: c_uint,
    pub res_y: c_uint,
    pub palette_quantization_method: c_int,
    pub palette_quantization_speed: c_int,
    pub palette_quantization_min_quality: c_int,
    pub palette_quantization_max_quality: c_int,
    pub interpolation_id: c_int,
    pub interpolation: *mut c_void,
}

/// `gdRect` (crop rectangle).
#[repr(C)]
pub struct GdRect {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
}

pub type GdImagePtr = *mut GdImageRaw;

// The gdSetErrorMethod callback is `void (*)(int, const char *, va_list)`.
// On aarch64-darwin `va_list` is a single pointer-sized value, so it can be
// carried as `*mut c_void` straight into `vsnprintf`.
type GdErrorMethod = unsafe extern "C" fn(c_int, *const c_char, *mut c_void);

extern "C" {
    fn gdImageCreate(sx: c_int, sy: c_int) -> GdImagePtr;
    fn gdImageCreateTrueColor(sx: c_int, sy: c_int) -> GdImagePtr;
    fn gdImageDestroy(im: GdImagePtr);

    fn gdImageCreateFromJpegPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromPngPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromGifPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromWebpPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromAvifPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromWBMPPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromBmpPtr(size: c_int, data: *const c_void) -> GdImagePtr;
    fn gdImageCreateFromTgaPtr(size: c_int, data: *const c_void) -> GdImagePtr;

    fn gdImageJpegPtr(im: GdImagePtr, size: *mut c_int, quality: c_int) -> *mut c_void;
    fn gdImagePngPtrEx(im: GdImagePtr, size: *mut c_int, level: c_int) -> *mut c_void;
    fn gdImageGifPtr(im: GdImagePtr, size: *mut c_int) -> *mut c_void;
    fn gdImageWebpPtrEx(im: GdImagePtr, size: *mut c_int, quality: c_int) -> *mut c_void;
    fn gdImageAvifPtrEx(im: GdImagePtr, size: *mut c_int, quality: c_int, speed: c_int)
        -> *mut c_void;
    fn gdFree(ptr: *mut c_void);

    fn gdImageColorAllocate(im: GdImagePtr, r: c_int, g: c_int, b: c_int) -> c_int;
    fn gdImageColorAllocateAlpha(im: GdImagePtr, r: c_int, g: c_int, b: c_int, a: c_int) -> c_int;
    fn gdImageColorTransparent(im: GdImagePtr, color: c_int);
    fn gdImageColorClosest(im: GdImagePtr, r: c_int, g: c_int, b: c_int) -> c_int;
    fn gdImageColorClosestAlpha(im: GdImagePtr, r: c_int, g: c_int, b: c_int, a: c_int) -> c_int;
    fn gdImageColorExact(im: GdImagePtr, r: c_int, g: c_int, b: c_int) -> c_int;
    fn gdImageColorExactAlpha(im: GdImagePtr, r: c_int, g: c_int, b: c_int, a: c_int) -> c_int;
    fn gdImageColorResolve(im: GdImagePtr, r: c_int, g: c_int, b: c_int) -> c_int;
    fn gdImageColorResolveAlpha(im: GdImagePtr, r: c_int, g: c_int, b: c_int, a: c_int) -> c_int;

    fn gdImageSetPixel(im: GdImagePtr, x: c_int, y: c_int, color: c_int);
    fn gdImageGetPixel(im: GdImagePtr, x: c_int, y: c_int) -> c_int;
    fn gdImageGetTrueColorPixel(im: GdImagePtr, x: c_int, y: c_int) -> c_int;
    fn gdImageBoundsSafe(im: GdImagePtr, x: c_int, y: c_int) -> c_int;

    fn gdImageFilledRectangle(im: GdImagePtr, x1: c_int, y1: c_int, x2: c_int, y2: c_int, c: c_int);
    fn gdImageRectangle(im: GdImagePtr, x1: c_int, y1: c_int, x2: c_int, y2: c_int, c: c_int);
    fn gdImageLine(im: GdImagePtr, x1: c_int, y1: c_int, x2: c_int, y2: c_int, c: c_int);
    fn gdImageFilledEllipse(im: GdImagePtr, cx: c_int, cy: c_int, w: c_int, h: c_int, c: c_int);
    fn gdImageEllipse(im: GdImagePtr, cx: c_int, cy: c_int, w: c_int, h: c_int, c: c_int);
    fn gdImageFill(im: GdImagePtr, x: c_int, y: c_int, c: c_int);
    fn gdImageFillToBorder(im: GdImagePtr, x: c_int, y: c_int, border: c_int, c: c_int);

    fn gdImageCopy(
        dst: GdImagePtr, src: GdImagePtr, dst_x: c_int, dst_y: c_int, src_x: c_int, src_y: c_int,
        w: c_int, h: c_int,
    );
    fn gdImageCopyResampled(
        dst: GdImagePtr, src: GdImagePtr, dst_x: c_int, dst_y: c_int, src_x: c_int, src_y: c_int,
        dst_w: c_int, dst_h: c_int, src_w: c_int, src_h: c_int,
    );
    fn gdImageCopyResized(
        dst: GdImagePtr, src: GdImagePtr, dst_x: c_int, dst_y: c_int, src_x: c_int, src_y: c_int,
        dst_w: c_int, dst_h: c_int, src_w: c_int, src_h: c_int,
    );
    fn gdImageRotateInterpolated(src: GdImagePtr, angle: f32, bgcolor: c_int) -> GdImagePtr;
    fn gdImageFlipHorizontal(im: GdImagePtr);
    fn gdImageFlipVertical(im: GdImagePtr);
    fn gdImageFlipBoth(im: GdImagePtr);
    fn gdImageCrop(src: GdImagePtr, crop: *const GdRect) -> GdImagePtr;
    fn gdImageScale(src: GdImagePtr, new_width: c_uint, new_height: c_uint) -> GdImagePtr;
    fn gdImageSetInterpolationMethod(im: GdImagePtr, id: c_int) -> c_int;

    fn gdImageAlphaBlending(im: GdImagePtr, blending: c_int);
    fn gdImageSaveAlpha(im: GdImagePtr, save: c_int);
    fn gdImageInterlace(im: GdImagePtr, interlace: c_int);
    fn gdImageTrueColorToPalette(im: GdImagePtr, dither: c_int, colors: c_int) -> c_int;
    fn gdImagePaletteToTrueColor(im: GdImagePtr) -> c_int;

    fn gdImageString(
        im: GdImagePtr, font: *mut c_void, x: c_int, y: c_int, s: *mut u8, color: c_int,
    );
    fn gdImageStringUp(
        im: GdImagePtr, font: *mut c_void, x: c_int, y: c_int, s: *mut u8, color: c_int,
    );
    fn gdImageChar(im: GdImagePtr, font: *mut c_void, x: c_int, y: c_int, c: c_int, color: c_int);
    fn gdImageCharUp(im: GdImagePtr, font: *mut c_void, x: c_int, y: c_int, c: c_int, color: c_int);
    fn gdFontGetTiny() -> *mut c_void;
    fn gdFontGetSmall() -> *mut c_void;
    fn gdFontGetMediumBold() -> *mut c_void;
    fn gdFontGetLarge() -> *mut c_void;
    fn gdFontGetGiant() -> *mut c_void;

    fn gdVersionString() -> *const c_char;
    fn gdSetErrorMethod(m: GdErrorMethod);

    fn vsnprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ap: *mut c_void) -> c_int;
}

thread_local! {
    /// Messages recorded by the libgd error callback since the last drain.
    static GD_ERRORS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static HANDLER_SET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// gd.h error priorities (syslog-style): GD_ERROR=3, GD_WARNING=4, GD_NOTICE=5.
/// PHP's `php_gd_error_method` maps ERROR/WARNING to E_WARNING and NOTICE to
/// E_NOTICE; INFO/DEBUG are dropped. We record everything up to NOTICE.
unsafe extern "C" fn record_error(code: c_int, fmt: *const c_char, ap: *mut c_void) {
    if code > 5 {
        return;
    }
    let mut buf = [0u8; 1024];
    let n = vsnprintf(buf.as_mut_ptr() as *mut c_char, buf.len(), fmt, ap);
    if n < 0 {
        return;
    }
    let end = (n as usize).min(buf.len() - 1);
    let msg = String::from_utf8_lossy(&buf[..end]).into_owned();
    GD_ERRORS.with(|e| e.borrow_mut().push(msg));
}

/// Install the recording error handler once per thread (before any gd call).
pub fn ensure_error_handler() {
    HANDLER_SET.with(|h| {
        if !h.get() {
            unsafe { gdSetErrorMethod(record_error) };
            h.set(true);
        }
    });
}

/// Drain the libgd messages recorded since the last call.
pub fn take_errors() -> Vec<String> {
    GD_ERRORS.with(|e| std::mem::take(&mut *e.borrow_mut()))
}

/// The libgd version string (`"2.3.3"`).
pub fn version() -> String {
    unsafe { CStr::from_ptr(gdVersionString()).to_string_lossy().into_owned() }
}

/// An owned gd image; destroys the underlying `gdImage` on drop.
pub struct GdImg(GdImagePtr);

impl Drop for GdImg {
    fn drop(&mut self) {
        unsafe { gdImageDestroy(self.0) };
    }
}

impl GdImg {
    fn from_raw(p: GdImagePtr) -> Option<GdImg> {
        if p.is_null() {
            None
        } else {
            Some(GdImg(p))
        }
    }
    fn raw(&self) -> &GdImageRaw {
        unsafe { &*self.0 }
    }
    pub fn sx(&self) -> i32 {
        self.raw().sx
    }
    pub fn sy(&self) -> i32 {
        self.raw().sy
    }
    pub fn true_color(&self) -> bool {
        self.raw().true_color != 0
    }
    pub fn colors_total(&self) -> i32 {
        self.raw().colors_total
    }
    pub fn transparent(&self) -> i32 {
        self.raw().transparent
    }
    pub fn interlace(&self) -> bool {
        self.raw().interlace != 0
    }
    /// Palette entry (r, g, b, a) — caller checks the index range.
    pub fn palette_entry(&self, i: usize) -> (i32, i32, i32, i32) {
        let r = self.raw();
        (r.red[i], r.green[i], r.blue[i], r.alpha[i])
    }
    pub fn palette_open(&self, i: usize) -> bool {
        self.raw().open[i] != 0
    }
    /// `imageantialias` writes the AA flag directly (as ext/gd does).
    pub fn set_antialias(&mut self, on: bool) {
        unsafe { (*self.0).aa = on as c_int };
    }

    pub fn create(sx: i32, sy: i32, true_color: bool) -> Option<GdImg> {
        ensure_error_handler();
        Self::from_raw(unsafe {
            if true_color {
                gdImageCreateTrueColor(sx, sy)
            } else {
                gdImageCreate(sx, sy)
            }
        })
    }

    /// Decode `data` with the given codec ("jpeg", "png", …).
    pub fn decode(kind: &str, data: &[u8]) -> Option<GdImg> {
        ensure_error_handler();
        let n = data.len() as c_int;
        let p = data.as_ptr() as *const c_void;
        Self::from_raw(unsafe {
            match kind {
                "jpeg" => gdImageCreateFromJpegPtr(n, p),
                "png" => gdImageCreateFromPngPtr(n, p),
                "gif" => gdImageCreateFromGifPtr(n, p),
                "webp" => gdImageCreateFromWebpPtr(n, p),
                "avif" => gdImageCreateFromAvifPtr(n, p),
                "wbmp" => gdImageCreateFromWBMPPtr(n, p),
                "bmp" => gdImageCreateFromBmpPtr(n, p),
                "tga" => gdImageCreateFromTgaPtr(n, p),
                _ => return None,
            }
        })
    }

    /// Encode with the given codec; `q1`/`q2` are the codec's quality/speed
    /// knobs (jpeg quality, png level, webp quality, avif quality+speed).
    pub fn encode(&self, kind: &str, q1: i32, q2: i32) -> Option<Vec<u8>> {
        ensure_error_handler();
        let mut size: c_int = 0;
        let ptr = unsafe {
            match kind {
                "jpeg" => gdImageJpegPtr(self.0, &mut size, q1),
                "png" => gdImagePngPtrEx(self.0, &mut size, q1),
                "gif" => gdImageGifPtr(self.0, &mut size),
                "webp" => gdImageWebpPtrEx(self.0, &mut size, q1),
                "avif" => gdImageAvifPtrEx(self.0, &mut size, q1, q2),
                _ => return None,
            }
        };
        if ptr.is_null() {
            return None;
        }
        let out =
            unsafe { std::slice::from_raw_parts(ptr as *const u8, size.max(0) as usize) }.to_vec();
        unsafe { gdFree(ptr) };
        Some(out)
    }

    pub fn color_allocate(&mut self, r: i32, g: i32, b: i32, a: Option<i32>) -> i32 {
        unsafe {
            match a {
                Some(a) => gdImageColorAllocateAlpha(self.0, r, g, b, a),
                None => gdImageColorAllocate(self.0, r, g, b),
            }
        }
    }
    pub fn color_closest(&mut self, r: i32, g: i32, b: i32, a: Option<i32>) -> i32 {
        unsafe {
            match a {
                Some(a) => gdImageColorClosestAlpha(self.0, r, g, b, a),
                None => gdImageColorClosest(self.0, r, g, b),
            }
        }
    }
    pub fn color_exact(&mut self, r: i32, g: i32, b: i32, a: Option<i32>) -> i32 {
        unsafe {
            match a {
                Some(a) => gdImageColorExactAlpha(self.0, r, g, b, a),
                None => gdImageColorExact(self.0, r, g, b),
            }
        }
    }
    pub fn color_resolve(&mut self, r: i32, g: i32, b: i32, a: Option<i32>) -> i32 {
        unsafe {
            match a {
                Some(a) => gdImageColorResolveAlpha(self.0, r, g, b, a),
                None => gdImageColorResolve(self.0, r, g, b),
            }
        }
    }
    pub fn set_transparent(&mut self, color: i32) {
        unsafe { gdImageColorTransparent(self.0, color) };
    }

    pub fn bounds_safe(&self, x: i32, y: i32) -> bool {
        unsafe { gdImageBoundsSafe(self.0, x, y) != 0 }
    }
    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        unsafe { gdImageGetPixel(self.0, x, y) }
    }
    pub fn get_true_color_pixel(&self, x: i32, y: i32) -> i32 {
        unsafe { gdImageGetTrueColorPixel(self.0, x, y) }
    }
    pub fn set_pixel(&mut self, x: i32, y: i32, color: i32) {
        unsafe { gdImageSetPixel(self.0, x, y, color) };
    }

    pub fn filled_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i32) {
        unsafe { gdImageFilledRectangle(self.0, x1, y1, x2, y2, c) };
    }
    pub fn rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i32) {
        unsafe { gdImageRectangle(self.0, x1, y1, x2, y2, c) };
    }
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i32) {
        unsafe { gdImageLine(self.0, x1, y1, x2, y2, c) };
    }
    pub fn filled_ellipse(&mut self, cx: i32, cy: i32, w: i32, h: i32, c: i32) {
        unsafe { gdImageFilledEllipse(self.0, cx, cy, w, h, c) };
    }
    pub fn ellipse(&mut self, cx: i32, cy: i32, w: i32, h: i32, c: i32) {
        unsafe { gdImageEllipse(self.0, cx, cy, w, h, c) };
    }
    pub fn fill(&mut self, x: i32, y: i32, c: i32) {
        unsafe { gdImageFill(self.0, x, y, c) };
    }
    pub fn fill_to_border(&mut self, x: i32, y: i32, border: i32, c: i32) {
        unsafe { gdImageFillToBorder(self.0, x, y, border, c) };
    }

    /// The raw pointer, for the two-image copy entry points below (the map in
    /// vm/gd.rs can't lend `&mut dst` and `&src` at once; the pixel data is
    /// C-owned, so C-level aliasing rules apply, not Rust's).
    pub fn as_raw(&self) -> GdImagePtr {
        self.0
    }
    pub fn rotate(&self, angle: f64, bgcolor: i32) -> Option<GdImg> {
        ensure_error_handler();
        Self::from_raw(unsafe { gdImageRotateInterpolated(self.0, angle as f32, bgcolor) })
    }
    pub fn flip(&mut self, mode: i32) {
        unsafe {
            match mode {
                1 => gdImageFlipHorizontal(self.0),
                2 => gdImageFlipVertical(self.0),
                _ => gdImageFlipBoth(self.0),
            }
        }
    }
    pub fn crop(&self, x: i32, y: i32, w: i32, h: i32) -> Option<GdImg> {
        let rect = GdRect { x, y, width: w, height: h };
        Self::from_raw(unsafe { gdImageCrop(self.0, &rect) })
    }
    pub fn scale(&self, w: u32, h: u32) -> Option<GdImg> {
        ensure_error_handler();
        Self::from_raw(unsafe { gdImageScale(self.0, w, h) })
    }
    pub fn set_interpolation_method(&mut self, id: i32) -> bool {
        unsafe { gdImageSetInterpolationMethod(self.0, id) != 0 }
    }

    pub fn alpha_blending(&mut self, on: bool) {
        unsafe { gdImageAlphaBlending(self.0, on as c_int) };
    }
    pub fn save_alpha(&mut self, on: bool) {
        unsafe { gdImageSaveAlpha(self.0, on as c_int) };
    }
    pub fn set_interlace(&mut self, on: bool) {
        unsafe { gdImageInterlace(self.0, on as c_int) };
    }
    pub fn true_color_to_palette(&mut self, dither: bool, colors: i32) -> bool {
        unsafe { gdImageTrueColorToPalette(self.0, dither as c_int, colors) != 0 }
    }
    pub fn palette_to_true_color(&mut self) -> bool {
        unsafe { gdImagePaletteToTrueColor(self.0) != 0 }
    }

    /// `imagestring`/`imagestringup`/`imagechar`/`imagecharup` over the five
    /// built-in bitmap fonts (font 1..=5; >5 clamps to giant, <1 to tiny —
    /// as ext/gd's `php_find_gd_font`).
    pub fn draw_string(&mut self, font: i32, x: i32, y: i32, s: &[u8], color: i32, up: bool) {
        let f = font_ptr(font);
        let mut buf = s.to_vec();
        buf.push(0);
        unsafe {
            if up {
                gdImageStringUp(self.0, f, x, y, buf.as_mut_ptr(), color);
            } else {
                gdImageString(self.0, f, x, y, buf.as_mut_ptr(), color);
            }
        }
    }
    pub fn draw_char(&mut self, font: i32, x: i32, y: i32, ch: i32, color: i32, up: bool) {
        let f = font_ptr(font);
        unsafe {
            if up {
                gdImageCharUp(self.0, f, x, y, ch, color);
            } else {
                gdImageChar(self.0, f, x, y, ch, color);
            }
        }
    }
}

/// `gdImageCopy` on raw handles (dst and src may be the same image).
#[allow(clippy::too_many_arguments)]
pub fn copy_raw(dst: GdImagePtr, src: GdImagePtr, dx: i32, dy: i32, sx: i32, sy: i32, w: i32, h: i32) {
    unsafe { gdImageCopy(dst, src, dx, dy, sx, sy, w, h) };
}

/// `gdImageCopyResampled` on raw handles.
#[allow(clippy::too_many_arguments)]
pub fn copy_resampled_raw(
    dst: GdImagePtr, src: GdImagePtr, dx: i32, dy: i32, sx: i32, sy: i32, dw: i32, dh: i32,
    sw: i32, sh: i32,
) {
    unsafe { gdImageCopyResampled(dst, src, dx, dy, sx, sy, dw, dh, sw, sh) };
}

/// `gdImageCopyResized` on raw handles.
#[allow(clippy::too_many_arguments)]
pub fn copy_resized_raw(
    dst: GdImagePtr, src: GdImagePtr, dx: i32, dy: i32, sx: i32, sy: i32, dw: i32, dh: i32,
    sw: i32, sh: i32,
) {
    unsafe { gdImageCopyResized(dst, src, dx, dy, sx, sy, dw, dh, sw, sh) };
}

fn font_ptr(font: i32) -> *mut c_void {
    unsafe {
        match font {
            i32::MIN..=1 => gdFontGetTiny(),
            2 => gdFontGetSmall(),
            3 => gdFontGetMediumBold(),
            4 => gdFontGetLarge(),
            5..=i32::MAX => gdFontGetGiant(),
        }
    }
}

/// (width, height) of a built-in bitmap font — `gdFont.w`/`gdFont.h` live at
/// offsets 8 and 12 (after `nchars`/`offset`).
pub fn font_metrics(font: i32) -> (i32, i32) {
    #[repr(C)]
    struct GdFontHead {
        nchars: c_int,
        offset: c_int,
        w: c_int,
        h: c_int,
    }
    let p = font_ptr(font) as *const GdFontHead;
    unsafe { ((*p).w, (*p).h) }
}

/// Ensure the double type is what gd expects for rotate (compile-time check).
const _: fn(c_double) = |_| {};
