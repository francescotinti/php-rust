//! ext/gd host side (`__gd_*`), backing the prelude `GdImage` class and the
//! `image*` procedural functions in lower/prelude_gd.php. Thin wrappers over
//! the **system libgd** FFI in `php_types::gdio` (the same dylib the PHP
//! oracle links, so decoded pixels and encoded file bytes are identical).
//!
//! Model (pattern `__pdo_*`/`__mysqli_*`): each `GdImage` object owns a
//! `gdio::GdImg` in `Vm.gd_images`, addressed by an int handle in a hidden
//! prop of the prelude class (freed by its `__destruct` via `__gd_destroy`).
//! Diagnostics are NOT emitted here: decode/encode failures return the
//! collected libgd messages (`errs`) and the prelude turns them into the
//! call-site Warnings (`__warning_from_caller`), matching ext/gd's
//! `php_gd_error_method` + `php_error_docref` split.

use php_types::gdio::{self, GdImg};
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

fn zstr(s: impl AsRef<[u8]>) -> Zval {
    Zval::Str(PhpStr::new(s.as_ref().to_vec()))
}

fn put(arr: &mut PhpArray, key: &str, v: Zval) {
    arr.insert(Key::from_bytes(key.as_bytes()), v);
}

/// The `errs` payload for a failed decode/encode: the libgd messages recorded
/// during the call, in order.
fn errs_payload(errs: Vec<String>) -> Zval {
    let mut list = PhpArray::new();
    for e in errs {
        let _ = list.append(zstr(e.into_bytes()));
    }
    let mut a = PhpArray::new();
    put(&mut a, "errs", Zval::Array(std::rc::Rc::new(list)));
    Zval::Array(std::rc::Rc::new(a))
}

/// Sniff the image format of a byte buffer the way `_php_image_create_from_string`
/// does (php_getimagetype order), returning the gdio codec name.
pub(super) fn sniff_format(d: &[u8]) -> Option<&'static str> {
    if d.len() >= 3 && d[0] == 0xFF && d[1] == 0xD8 && d[2] == 0xFF {
        Some("jpeg")
    } else if d.len() >= 8 && d[..8] == *b"\x89PNG\r\n\x1a\n" {
        Some("png")
    } else if d.len() >= 3 && &d[..3] == b"GIF" {
        Some("gif")
    } else if d.len() >= 12 && &d[..4] == b"RIFF" && &d[8..12] == b"WEBP" {
        Some("webp")
    } else if d.len() >= 12 && &d[4..8] == b"ftyp" {
        Some("avif")
    } else if d.len() >= 2 && &d[..2] == b"BM" {
        Some("bmp")
    } else if d.len() >= 3 && d[0] == 0 && d[1] == 0 && d[2] <= 1 {
        // WBMP's signature is famously weak (0x00 type, 0x00 fixheader); PHP
        // tries it late for the same reason.
        Some("wbmp")
    } else {
        None
    }
}

impl<'m> Vm<'m> {
    fn gd_arg_id(&mut self, args: &[Zval], idx: usize) -> u32 {
        convert::to_long_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags) as u32
    }
    fn gd_arg_i32(&mut self, args: &[Zval], idx: usize) -> i32 {
        convert::to_long_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags) as i32
    }
    fn gd_arg_str(&mut self, args: &[Zval], idx: usize) -> Vec<u8> {
        convert::to_zstr_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec()
    }
    fn gd_put(&mut self, im: GdImg) -> u32 {
        let id = self.next_gd;
        self.next_gd += 1;
        self.gd_images.insert(id, im);
        id
    }

    /// `__gd_create($w, $h, $truecolor)` → handle (dims validated in the prelude).
    pub(super) fn ho_gd_create(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let w = self.gd_arg_i32(&args, 0);
        let h = self.gd_arg_i32(&args, 1);
        let tc = convert::to_bool(args.get(2).unwrap_or(&Zval::Null), &mut self.diags);
        match GdImg::create(w, h, tc) {
            Some(im) => Ok(Zval::Long(self.gd_put(im) as i64)),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `__gd_destroy($h)` — the GdImage `__destruct` path.
    pub(super) fn ho_gd_destroy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        Ok(Zval::Bool(self.gd_images.remove(&id).is_some()))
    }

    /// `__gd_decode($kind, $data)` → `['h'=>id]` | `['errs'=>[...]]`.
    pub(super) fn ho_gd_decode(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let kind = self.gd_arg_str(&args, 0);
        let data = self.gd_arg_str(&args, 1);
        let kind = String::from_utf8_lossy(&kind).into_owned();
        let _ = gdio::take_errors();
        match GdImg::decode(&kind, &data) {
            Some(im) => {
                let _ = gdio::take_errors();
                let mut a = PhpArray::new();
                let id = self.gd_put(im);
                put(&mut a, "h", Zval::Long(id as i64));
                Ok(Zval::Array(std::rc::Rc::new(a)))
            }
            None => Ok(errs_payload(gdio::take_errors())),
        }
    }

    /// `__gd_decode_auto($data)` → `['h'=>id]` | `['errs'=>[...], 'unknown'=>bool]`
    /// (imagecreatefromstring: sniff, then decode).
    pub(super) fn ho_gd_decode_auto(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let data = self.gd_arg_str(&args, 0);
        let _ = gdio::take_errors();
        let Some(kind) = sniff_format(&data) else {
            let mut a = PhpArray::new();
            put(&mut a, "unknown", Zval::Bool(true));
            return Ok(Zval::Array(std::rc::Rc::new(a)));
        };
        match GdImg::decode(kind, &data) {
            Some(im) => {
                let _ = gdio::take_errors();
                let mut a = PhpArray::new();
                let id = self.gd_put(im);
                put(&mut a, "h", Zval::Long(id as i64));
                Ok(Zval::Array(std::rc::Rc::new(a)))
            }
            None => Ok(errs_payload(gdio::take_errors())),
        }
    }

    /// `__gd_encode($h, $kind, $q1, $q2)` → bytes | `['errs'=>[...]]`.
    pub(super) fn ho_gd_encode(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let kind = self.gd_arg_str(&args, 1);
        let q1 = self.gd_arg_i32(&args, 2);
        let q2 = self.gd_arg_i32(&args, 3);
        let kind = String::from_utf8_lossy(&kind).into_owned();
        let _ = gdio::take_errors();
        let Some(im) = self.gd_images.get(&id) else {
            return Ok(Zval::Bool(false));
        };
        match im.encode(&kind, q1, q2) {
            Some(bytes) => {
                let _ = gdio::take_errors();
                Ok(Zval::Str(PhpStr::new(bytes)))
            }
            None => Ok(errs_payload(gdio::take_errors())),
        }
    }

    /// `__gd_stat($h)` → the accessor-macro fields in one array.
    pub(super) fn ho_gd_stat(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let Some(im) = self.gd_images.get(&id) else {
            return Ok(Zval::Bool(false));
        };
        let mut a = PhpArray::new();
        put(&mut a, "sx", Zval::Long(im.sx() as i64));
        put(&mut a, "sy", Zval::Long(im.sy() as i64));
        put(&mut a, "tc", Zval::Bool(im.true_color()));
        put(&mut a, "colors", Zval::Long(im.colors_total() as i64));
        put(&mut a, "transparent", Zval::Long(im.transparent() as i64));
        put(&mut a, "interlace", Zval::Bool(im.interlace()));
        Ok(Zval::Array(std::rc::Rc::new(a)))
    }

    /// `__gd_flag($h, $which, $on)`: alphablending / savealpha / interlace / aa.
    pub(super) fn ho_gd_flag(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let which = self.gd_arg_str(&args, 1);
        let on = convert::to_bool(args.get(2).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        match which.as_slice() {
            b"blend" => im.alpha_blending(on),
            b"savealpha" => im.save_alpha(on),
            b"interlace" => im.set_interlace(on),
            b"aa" => im.set_antialias(on),
            _ => return Ok(Zval::Bool(false)),
        }
        Ok(Zval::Bool(true))
    }

    /// `__gd_color($h, $op, $r, $g, $b, $a)`: allocate/closest/exact/resolve,
    /// with or without alpha (`$a < 0` = the 3-arg form).
    pub(super) fn ho_gd_color(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let op = self.gd_arg_str(&args, 1);
        let r = self.gd_arg_i32(&args, 2);
        let g = self.gd_arg_i32(&args, 3);
        let b = self.gd_arg_i32(&args, 4);
        let a_raw = self.gd_arg_i32(&args, 5);
        let a = if a_raw < 0 { None } else { Some(a_raw) };
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        let c = match op.as_slice() {
            b"allocate" => im.color_allocate(r, g, b, a),
            b"closest" => im.color_closest(r, g, b, a),
            b"exact" => im.color_exact(r, g, b, a),
            b"resolve" => im.color_resolve(r, g, b, a),
            _ => -1,
        };
        // gdImageColorAllocate returns -1 when the palette is full → false.
        if c < 0 && op.as_slice() == b"allocate" {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Long(c as i64))
    }

    /// `__gd_colortransparent($h, $color)`: set (+get); `$color = -2` = get only.
    pub(super) fn ho_gd_colortransparent(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let color = self.gd_arg_i32(&args, 1);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        if color != -2 {
            im.set_transparent(color);
        }
        Ok(Zval::Long(im.transparent() as i64))
    }

    /// `__gd_colorsforindex($h, $i)` → `[red, green, blue, alpha]` | false (out
    /// of range — the prelude throws the ValueError).
    pub(super) fn ho_gd_colorsforindex(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let idx = self.gd_arg_i32(&args, 1);
        let Some(im) = self.gd_images.get(&id) else {
            return Ok(Zval::Bool(false));
        };
        let (r, g, b, a) = if im.true_color() {
            let c = idx;
            ((c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF, (c >> 24) & 0x7F)
        } else {
            if idx < 0 || idx >= im.colors_total() {
                return Ok(Zval::Bool(false));
            }
            im.palette_entry(idx as usize)
        };
        let mut a_out = PhpArray::new();
        put(&mut a_out, "red", Zval::Long(r as i64));
        put(&mut a_out, "green", Zval::Long(g as i64));
        put(&mut a_out, "blue", Zval::Long(b as i64));
        put(&mut a_out, "alpha", Zval::Long(a as i64));
        Ok(Zval::Array(std::rc::Rc::new(a_out)))
    }

    /// `__gd_colorat($h, $x, $y)` → int | false (out of bounds).
    pub(super) fn ho_gd_colorat(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let x = self.gd_arg_i32(&args, 1);
        let y = self.gd_arg_i32(&args, 2);
        let Some(im) = self.gd_images.get(&id) else {
            return Ok(Zval::Bool(false));
        };
        if !im.bounds_safe(x, y) {
            return Ok(Zval::Bool(false));
        }
        if im.true_color() {
            Ok(Zval::Long(im.get_true_color_pixel(x, y) as i64))
        } else {
            Ok(Zval::Long(im.get_pixel(x, y) as i64))
        }
    }

    /// `__gd_setpixel($h, $x, $y, $c)`.
    pub(super) fn ho_gd_setpixel(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let x = self.gd_arg_i32(&args, 1);
        let y = self.gd_arg_i32(&args, 2);
        let c = self.gd_arg_i32(&args, 3);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        im.set_pixel(x, y, c);
        Ok(Zval::Bool(true))
    }

    /// `__gd_draw($h, $op, ...$ints)`: line/rect/filledrect/ellipse/
    /// filledellipse/fill/filltoborder.
    pub(super) fn ho_gd_draw(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let op = self.gd_arg_str(&args, 1);
        let mut v = [0i32; 5];
        for (i, slot) in v.iter_mut().enumerate() {
            *slot = self.gd_arg_i32(&args, 2 + i);
        }
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        match op.as_slice() {
            b"line" => im.line(v[0], v[1], v[2], v[3], v[4]),
            b"rect" => im.rectangle(v[0], v[1], v[2], v[3], v[4]),
            b"filledrect" => im.filled_rectangle(v[0], v[1], v[2], v[3], v[4]),
            b"ellipse" => im.ellipse(v[0], v[1], v[2], v[3], v[4]),
            b"filledellipse" => im.filled_ellipse(v[0], v[1], v[2], v[3], v[4]),
            b"fill" => im.fill(v[0], v[1], v[2]),
            b"filltoborder" => im.fill_to_border(v[0], v[1], v[2], v[3]),
            _ => return Ok(Zval::Bool(false)),
        }
        Ok(Zval::Bool(true))
    }

    /// `__gd_copy($dst, $src, $op, ...)`: op ∈ copy/resampled/resized; the two
    /// handles may name the same image (raw-pointer entry points).
    pub(super) fn ho_gd_copy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let dst = self.gd_arg_id(&args, 0);
        let src = self.gd_arg_id(&args, 1);
        let op = self.gd_arg_str(&args, 2);
        let mut v = [0i32; 8];
        for (i, slot) in v.iter_mut().enumerate() {
            *slot = self.gd_arg_i32(&args, 3 + i);
        }
        let (Some(d), Some(s)) = (self.gd_images.get(&dst), self.gd_images.get(&src)) else {
            return Ok(Zval::Bool(false));
        };
        let (dp, sp) = (d.as_raw(), s.as_raw());
        match op.as_slice() {
            b"copy" => gdio::copy_raw(dp, sp, v[0], v[1], v[2], v[3], v[4], v[5]),
            b"resampled" => {
                gdio::copy_resampled_raw(dp, sp, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])
            }
            b"resized" => {
                gdio::copy_resized_raw(dp, sp, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])
            }
            _ => return Ok(Zval::Bool(false)),
        }
        Ok(Zval::Bool(true))
    }

    /// `__gd_rotate($h, $angle, $bg)` → new handle | false.
    pub(super) fn ho_gd_rotate(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let angle = convert::to_double(args.get(1).unwrap_or(&Zval::Null));
        let bg = self.gd_arg_i32(&args, 2);
        let Some(im) = self.gd_images.get(&id) else {
            return Ok(Zval::Bool(false));
        };
        match im.rotate(angle, bg) {
            Some(out) => Ok(Zval::Long(self.gd_put(out) as i64)),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `__gd_flip($h, $mode)`.
    pub(super) fn ho_gd_flip(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let mode = self.gd_arg_i32(&args, 1);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        im.flip(mode);
        Ok(Zval::Bool(true))
    }

    /// `__gd_crop($h, $x, $y, $w, $h)` → new handle | false.
    pub(super) fn ho_gd_crop(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let x = self.gd_arg_i32(&args, 1);
        let y = self.gd_arg_i32(&args, 2);
        let w = self.gd_arg_i32(&args, 3);
        let h = self.gd_arg_i32(&args, 4);
        let Some(im) = self.gd_images.get(&id) else {
            return Ok(Zval::Bool(false));
        };
        match im.crop(x, y, w, h) {
            Some(out) => Ok(Zval::Long(self.gd_put(out) as i64)),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `__gd_scale($h, $w, $h2, $method)` → new handle | false. As ext/gd:
    /// set the interpolation method for the call, then restore the old one on
    /// the *source* (gdImageScale reads `im->interpolation_id`).
    pub(super) fn ho_gd_scale(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let w = self.gd_arg_i32(&args, 1);
        let h = self.gd_arg_i32(&args, 2);
        let method = self.gd_arg_i32(&args, 3);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        if w < 0 || h < 0 {
            return Ok(Zval::Bool(false));
        }
        if !im.set_interpolation_method(method) {
            return Ok(Zval::Bool(false));
        }
        let out = im.scale(w as u32, h as u32);
        // ext/gd restores GD_DEFAULT (bilinear-fixed) semantics by leaving the
        // method on the image; imagescale explicitly re-sets the old method.
        match out {
            Some(o) => Ok(Zval::Long(self.gd_put(o) as i64)),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `__gd_setinterpolation($h, $method)`.
    pub(super) fn ho_gd_setinterpolation(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let method = self.gd_arg_i32(&args, 1);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Bool(im.set_interpolation_method(method)))
    }

    /// `__gd_t2p($h, $dither, $ncolors)` — imagetruecolortopalette.
    pub(super) fn ho_gd_t2p(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let dither = convert::to_bool(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let ncolors = self.gd_arg_i32(&args, 2);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        if !im.true_color() {
            return Ok(Zval::Bool(true));
        }
        Ok(Zval::Bool(im.true_color_to_palette(dither, ncolors)))
    }

    /// `__gd_p2t($h)` — imagepalettetotruecolor.
    pub(super) fn ho_gd_p2t(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        if im.true_color() {
            return Ok(Zval::Bool(true));
        }
        Ok(Zval::Bool(im.palette_to_true_color()))
    }

    /// `__gd_string($h, $font, $x, $y, $s, $color, $up)`.
    pub(super) fn ho_gd_string(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let font = self.gd_arg_i32(&args, 1);
        let x = self.gd_arg_i32(&args, 2);
        let y = self.gd_arg_i32(&args, 3);
        let s = self.gd_arg_str(&args, 4);
        let color = self.gd_arg_i32(&args, 5);
        let up = convert::to_bool(args.get(6).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        im.draw_string(font, x, y, &s, color, up);
        Ok(Zval::Bool(true))
    }

    /// `__gd_char($h, $font, $x, $y, $ch, $color, $up)`.
    pub(super) fn ho_gd_char(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.gd_arg_id(&args, 0);
        let font = self.gd_arg_i32(&args, 1);
        let x = self.gd_arg_i32(&args, 2);
        let y = self.gd_arg_i32(&args, 3);
        let ch = self.gd_arg_str(&args, 4);
        let color = self.gd_arg_i32(&args, 5);
        let up = convert::to_bool(args.get(6).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(im) = self.gd_images.get_mut(&id) else {
            return Ok(Zval::Bool(false));
        };
        im.draw_char(font, x, y, *ch.first().unwrap_or(&0) as i32, color, up);
        Ok(Zval::Bool(true))
    }

    /// `__gd_fontsize($font)` → `[w, h]` (built-in bitmap fonts).
    pub(super) fn ho_gd_fontsize(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let font = self.gd_arg_i32(&args, 0);
        let (w, h) = gdio::font_metrics(font);
        let mut a = PhpArray::new();
        let _ = a.append(Zval::Long(w as i64));
        let _ = a.append(Zval::Long(h as i64));
        Ok(Zval::Array(std::rc::Rc::new(a)))
    }

    /// `__gd_version()` → the linked libgd's version string.
    pub(super) fn ho_gd_version(&mut self) -> Result<Zval, PhpError> {
        Ok(zstr(gdio::version().into_bytes()))
    }

    /// `getimagesize($f, &$image_info)` / `getimagesizefromstring($d, &$info)`:
    /// the CallHostBuiltinOut path. Delegates to the registry pair builtins
    /// (`__getimagesize_info` in php-builtins/image.rs) and splits the
    /// `[result, info]` pair into (return value, out-param value).
    pub(super) fn ho_getimagesize_out(
        &mut self,
        args: Vec<Zval>,
        from_string: bool,
    ) -> Result<(Zval, Zval), PhpError> {
        let name: &[u8] =
            if from_string { b"__getimagesizefromstring_info" } else { b"__getimagesize_info" };
        let f = match self.registry.get(name) {
            Some(crate::builtin::Builtin::Value(f)) => *f,
            _ => return Err(PhpError::Error("getimagesize builtin unavailable".to_string())),
        };
        let line = self.cur_line(self.frames.len() - 1);
        let pair = self.run_value_builtin(f, &args[..args.len().min(1)], line)?;
        if let Zval::Array(a) = pair {
            let res = a.get(&php_types::Key::Int(0)).cloned().unwrap_or(Zval::Bool(false));
            let info = a
                .get(&php_types::Key::Int(1))
                .cloned()
                .unwrap_or_else(|| Zval::Array(std::rc::Rc::new(PhpArray::new())));
            Ok((res, info))
        } else {
            Ok((Zval::Bool(false), Zval::Array(std::rc::Rc::new(PhpArray::new()))))
        }
    }
}
