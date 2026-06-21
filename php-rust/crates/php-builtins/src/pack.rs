//! `pack` / `unpack` (step 63) — binary string packing.
//!
//! Faithful port of `ext/standard/pack.c` for a little-endian host (the oracle is
//! built on macOS x86_64/arm64). PHP strings are bytes, so every value here is
//! `[u8]`. Integer codes coerce through PHP's `zval_get_long`/`zval_get_double`.
//!
//! Endianness: a machine-order or little code emits the low `size` bytes in
//! little-endian order (`to_le_bytes()[..size]`); a big-endian code emits the
//! same bytes reversed. `sizeof(int)` and `sizeof(float)` are 4, `sizeof(double)`
//! is 8.

use php_runtime::Ctx;
use php_types::{convert, Diag, Key, PhpArray, PhpError, PhpStr, Zval};

const INT_MAX: i64 = i32::MAX as i64; // PHP's pack uses C `int` for format sizes

// ---------------------------------------------------------------------------
// byte emit/read helpers (host is little-endian)
// ---------------------------------------------------------------------------

fn put_int(out: &mut [u8], pos: usize, val: i64, size: usize, big_endian: bool) {
    let bytes = (val as u64).to_le_bytes();
    if big_endian {
        for k in 0..size {
            out[pos + k] = bytes[size - 1 - k];
        }
    } else {
        out[pos..pos + size].copy_from_slice(&bytes[..size]);
    }
}

fn put_f32(out: &mut [u8], pos: usize, v: f32, big_endian: bool) {
    let b = if big_endian {
        v.to_be_bytes()
    } else {
        v.to_le_bytes()
    };
    out[pos..pos + 4].copy_from_slice(&b);
}

fn put_f64(out: &mut [u8], pos: usize, v: f64, big_endian: bool) {
    let b = if big_endian {
        v.to_be_bytes()
    } else {
        v.to_le_bytes()
    };
    out[pos..pos + 8].copy_from_slice(&b);
}

/// Read `size` bytes as an integer; sign-extend when `signed`. For an 8-byte
/// unsigned value the u64 bit pattern is reinterpreted as i64 (PHP semantics).
fn read_int(input: &[u8], pos: usize, size: usize, big_endian: bool, signed: bool) -> i64 {
    let mut u: u64 = 0;
    if big_endian {
        for k in 0..size {
            u = (u << 8) | input[pos + k] as u64;
        }
    } else {
        for k in (0..size).rev() {
            u = (u << 8) | input[pos + k] as u64;
        }
    }
    if signed && size < 8 {
        let shift = 64 - size * 8;
        ((u << shift) as i64) >> shift
    } else {
        u as i64
    }
}

fn read_f32(input: &[u8], pos: usize, big_endian: bool) -> f32 {
    let mut b = [0u8; 4];
    b.copy_from_slice(&input[pos..pos + 4]);
    if big_endian {
        f32::from_be_bytes(b)
    } else {
        f32::from_le_bytes(b)
    }
}

fn read_f64(input: &[u8], pos: usize, big_endian: bool) -> f64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&input[pos..pos + 8]);
    if big_endian {
        f64::from_be_bytes(b)
    } else {
        f64::from_le_bytes(b)
    }
}

// ---------------------------------------------------------------------------
// pack
// ---------------------------------------------------------------------------

/// `pack(string $format, mixed ...$values): string`.
pub fn pack(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("pack() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let format = fmt.as_bytes().to_vec();
    let argv = &args[1..];
    let num_args = argv.len() as i64;

    // Pass 1: parse format into (code, arg), validating argument counts.
    let mut codes: Vec<(u8, i64)> = Vec::new();
    let mut currentarg: i64 = 0;
    let mut i = 0usize;
    let flen = format.len();
    while i < flen {
        let code = format[i];
        i += 1;
        let mut arg: i64 = 1;
        if i < flen {
            let c = format[i];
            if c == b'*' {
                arg = -1;
                i += 1;
            } else if c.is_ascii_digit() {
                let start = i;
                while i < flen && format[i].is_ascii_digit() {
                    i += 1;
                }
                arg = std::str::from_utf8(&format[start..i])
                    .ok()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(INT_MAX);
            }
        }

        match code {
            b'x' | b'X' | b'@' => {
                if arg < 0 {
                    ctx.diags.push(Diag::Warning(format!(
                        "pack(): Type {}: '*' ignored",
                        code as char
                    )));
                    arg = 1;
                }
            }
            b'a' | b'A' | b'Z' | b'h' | b'H' => {
                if currentarg >= num_args {
                    return Err(PhpError::ValueError(format!(
                        "Type {}: not enough arguments",
                        code as char
                    )));
                }
                if arg < 0 {
                    let s = convert::to_zstr(&argv[currentarg as usize], ctx.diags);
                    arg = s.as_bytes().len() as i64;
                    if code == b'Z' {
                        arg += 1;
                    }
                }
                currentarg += 1;
            }
            b'q' | b'Q' | b'J' | b'P' | b'c' | b'C' | b's' | b'S' | b'i' | b'I' | b'l' | b'L'
            | b'n' | b'N' | b'v' | b'V' | b'f' | b'g' | b'G' | b'd' | b'e' | b'E' => {
                if arg < 0 {
                    arg = num_args - currentarg;
                }
                currentarg += arg;
                if currentarg > num_args {
                    return Err(PhpError::ValueError(format!(
                        "Type {}: too few arguments",
                        code as char
                    )));
                }
            }
            _ => {
                return Err(PhpError::ValueError(format!(
                    "Type {}: unknown format code",
                    code as char
                )))
            }
        }
        codes.push((code, arg));
    }

    if currentarg < num_args {
        ctx.diags.push(Diag::Warning(format!(
            "pack(): {} arguments unused",
            num_args - currentarg
        )));
    }

    // Pass 2: compute output size (X/@ move the cursor; guard overflow like
    // INC_OUTPUTPOS to avoid huge allocations).
    let mut outputpos: i64 = 0;
    let mut outputsize: i64 = 0;
    let inc = |outputpos: &mut i64, a: i64, b: i64, code: u8| -> Result<(), PhpError> {
        if a < 0 || (INT_MAX - *outputpos) / b < a {
            return Err(PhpError::ValueError(format!(
                "Type {}: integer overflow in format string",
                code as char
            )));
        }
        *outputpos += a * b;
        Ok(())
    };
    for &(code, arg) in &codes {
        match code {
            b'h' | b'H' => inc(&mut outputpos, (arg / 2) + (arg % 2), 1, code)?,
            b'a' | b'A' | b'Z' | b'c' | b'C' | b'x' => inc(&mut outputpos, arg, 1, code)?,
            b's' | b'S' | b'n' | b'v' => inc(&mut outputpos, arg, 2, code)?,
            b'i' | b'I' => inc(&mut outputpos, arg, 4, code)?,
            b'l' | b'L' | b'N' | b'V' => inc(&mut outputpos, arg, 4, code)?,
            b'q' | b'Q' | b'J' | b'P' => inc(&mut outputpos, arg, 8, code)?,
            b'f' | b'g' | b'G' => inc(&mut outputpos, arg, 4, code)?,
            b'd' | b'e' | b'E' => inc(&mut outputpos, arg, 8, code)?,
            b'X' => {
                outputpos -= arg;
                if outputpos < 0 {
                    ctx.diags.push(Diag::Warning(format!(
                        "pack(): Type {}: outside of string",
                        code as char
                    )));
                    outputpos = 0;
                }
            }
            b'@' => outputpos = arg,
            _ => {}
        }
        if outputsize < outputpos {
            outputsize = outputpos;
        }
    }

    // Pass 3: pack.
    let mut out = vec![0u8; outputsize as usize];
    let mut outputpos: usize = 0;
    let mut currentarg: usize = 0;
    for &(code, arg) in &codes {
        let arg_u = arg.max(0) as usize;
        match code {
            b'a' | b'A' | b'Z' => {
                let arg_cp = if code != b'Z' { arg } else { (arg - 1).max(0) } as usize;
                let s = convert::to_zstr(&argv[currentarg], ctx.diags);
                currentarg += 1;
                let pad = if code == b'A' { b' ' } else { 0u8 };
                for b in out.iter_mut().skip(outputpos).take(arg_u) {
                    *b = pad;
                }
                let n = s.as_bytes().len().min(arg_cp);
                out[outputpos..outputpos + n].copy_from_slice(&s.as_bytes()[..n]);
                outputpos += arg_u;
            }
            b'h' | b'H' => {
                let mut nibbleshift = if code == b'h' { 0 } else { 4 };
                let mut first = true;
                let s = convert::to_zstr(&argv[currentarg], ctx.diags);
                currentarg += 1;
                let v = s.as_bytes();
                let mut arg_n = arg;
                let mut op: i64 = outputpos as i64 - 1;
                if (arg as usize) > v.len() {
                    ctx.diags.push(Diag::Warning(format!(
                        "pack(): Type {}: not enough characters in string",
                        code as char
                    )));
                    arg_n = v.len() as i64;
                }
                let mut vi = 0usize;
                while arg_n > 0 {
                    arg_n -= 1;
                    let c = v[vi];
                    vi += 1;
                    let n = match c {
                        b'0'..=b'9' => c - b'0',
                        b'A'..=b'F' => c - (b'A' - 10),
                        b'a'..=b'f' => c - (b'a' - 10),
                        _ => {
                            ctx.diags.push(Diag::Warning(format!(
                                "pack(): Type {}: illegal hex digit {}",
                                code as char, c as char
                            )));
                            0
                        }
                    };
                    if first {
                        op += 1;
                        out[op as usize] = 0;
                        first = false;
                    } else {
                        first = true;
                    }
                    out[op as usize] |= n << nibbleshift;
                    nibbleshift = (nibbleshift + 4) & 7;
                }
                outputpos = (op + 1) as usize;
            }
            b'c' | b'C' => {
                for _ in 0..arg_u {
                    let val = convert::to_long_cast(&argv[currentarg], ctx.diags);
                    currentarg += 1;
                    put_int(&mut out, outputpos, val, 1, false);
                    outputpos += 1;
                }
            }
            b's' | b'S' | b'n' | b'v' => {
                let big = code == b'n';
                for _ in 0..arg_u {
                    let val = convert::to_long_cast(&argv[currentarg], ctx.diags);
                    currentarg += 1;
                    put_int(&mut out, outputpos, val, 2, big);
                    outputpos += 2;
                }
            }
            b'i' | b'I' => {
                for _ in 0..arg_u {
                    let val = convert::to_long_cast(&argv[currentarg], ctx.diags);
                    currentarg += 1;
                    put_int(&mut out, outputpos, val, 4, false);
                    outputpos += 4;
                }
            }
            b'l' | b'L' | b'N' | b'V' => {
                let big = code == b'N';
                for _ in 0..arg_u {
                    let val = convert::to_long_cast(&argv[currentarg], ctx.diags);
                    currentarg += 1;
                    put_int(&mut out, outputpos, val, 4, big);
                    outputpos += 4;
                }
            }
            b'q' | b'Q' | b'J' | b'P' => {
                let big = code == b'J';
                for _ in 0..arg_u {
                    let val = convert::to_long_cast(&argv[currentarg], ctx.diags);
                    currentarg += 1;
                    put_int(&mut out, outputpos, val, 8, big);
                    outputpos += 8;
                }
            }
            b'f' | b'g' | b'G' => {
                let big = code == b'G';
                for _ in 0..arg_u {
                    let val = convert::to_double(&argv[currentarg]) as f32;
                    currentarg += 1;
                    put_f32(&mut out, outputpos, val, big);
                    outputpos += 4;
                }
            }
            b'd' | b'e' | b'E' => {
                let big = code == b'E';
                for _ in 0..arg_u {
                    let val = convert::to_double(&argv[currentarg]);
                    currentarg += 1;
                    put_f64(&mut out, outputpos, val, big);
                    outputpos += 8;
                }
            }
            b'x' => {
                for b in out.iter_mut().skip(outputpos).take(arg_u) {
                    *b = 0;
                }
                outputpos += arg_u;
            }
            b'X' => {
                outputpos = outputpos.saturating_sub(arg_u);
            }
            b'@' => {
                if arg_u > outputpos {
                    for b in out.iter_mut().take(arg_u).skip(outputpos) {
                        *b = 0;
                    }
                }
                outputpos = arg_u;
            }
            _ => {}
        }
    }

    out.truncate(outputpos);
    Ok(Zval::Str(PhpStr::new(out)))
}

// ---------------------------------------------------------------------------
// unpack
// ---------------------------------------------------------------------------

/// `unpack(string $format, string $string, int $offset = 0): array|false`.
pub fn unpack(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("unpack() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let data = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("unpack() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let offset = match args.get(2) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => 0,
    };

    let format = fmt.as_bytes();
    let full = data.as_bytes();
    if offset < 0 || offset > full.len() as i64 {
        return Err(PhpError::ValueError(
            "unpack(): Argument #3 ($offset) must be contained in argument #2 ($data)".to_string(),
        ));
    }
    let input = &full[offset as usize..];
    let inputlen = input.len() as i64;
    let mut inputpos: i64 = 0;

    let mut result = PhpArray::new();
    let fbytes = format;
    let mut fi = 0usize;
    let flen = fbytes.len();

    while fi < flen {
        let ty = fbytes[fi];
        fi += 1;
        let mut repetitions: i64 = 1;

        // Optional repeater.
        if fi < flen {
            let c = fbytes[fi];
            if c.is_ascii_digit() {
                let start = fi;
                while fi < flen && fbytes[fi].is_ascii_digit() {
                    fi += 1;
                }
                match std::str::from_utf8(&fbytes[start..fi])
                    .ok()
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    Some(n) if n <= INT_MAX => repetitions = n,
                    _ => {
                        ctx.diags.push(Diag::Warning(format!(
                            "unpack(): Type {}: integer overflow",
                            ty as char
                        )));
                        return Ok(Zval::Bool(false));
                    }
                }
            } else if c == b'*' {
                repetitions = -1;
                fi += 1;
            }
        }

        // Name = bytes up to '/'.
        let name_start = fi;
        let argb = repetitions;
        while fi < flen && fbytes[fi] != b'/' {
            fi += 1;
        }
        let mut name = &fbytes[name_start..fi];
        if name.len() > 200 {
            name = &name[..200];
        }

        // Per-type element size.
        let mut size: i64;
        match ty {
            b'X' => {
                size = -1;
                if repetitions < 0 {
                    ctx.diags.push(Diag::Warning(format!(
                        "unpack(): Type {}: '*' ignored",
                        ty as char
                    )));
                    repetitions = 1;
                }
            }
            b'@' => size = 0,
            b'a' | b'A' | b'Z' => {
                size = repetitions;
                repetitions = 1;
            }
            b'h' | b'H' => {
                size = if repetitions > 0 {
                    (repetitions + 1) / 2
                } else {
                    repetitions
                };
                repetitions = 1;
            }
            b'c' | b'C' | b'x' => size = 1,
            b's' | b'S' | b'n' | b'v' => size = 2,
            b'i' | b'I' => size = 4,
            b'l' | b'L' | b'N' | b'V' => size = 4,
            b'q' | b'Q' | b'J' | b'P' => size = 8,
            b'f' | b'g' | b'G' => size = 4,
            b'd' | b'e' | b'E' => size = 8,
            _ => {
                return Err(PhpError::ValueError(format!(
                    "Invalid format type {}",
                    ty as char
                )))
            }
        }

        let mut iidx: i64 = 0;
        while iidx != repetitions {
            if size != 0 && size != -1 && INT_MAX - size + 1 < inputpos {
                ctx.diags.push(Diag::Warning(format!(
                    "unpack(): Type {}: integer overflow",
                    ty as char
                )));
                return Ok(Zval::Bool(false));
            }

            if inputpos + size <= inputlen {
                let pos = inputpos as usize;
                let mut emit = true;
                let val: Zval = match ty {
                    b'a' => {
                        let mut len = inputlen - inputpos;
                        if size >= 0 && len > size {
                            len = size;
                        }
                        size = len;
                        Zval::Str(PhpStr::new(input[pos..pos + len as usize].to_vec()))
                    }
                    b'A' => {
                        let mut len = inputlen - inputpos;
                        if size >= 0 && len > size {
                            len = size;
                        }
                        size = len;
                        let mut l = len;
                        while l > 0 {
                            let b = input[pos + (l - 1) as usize];
                            if b != b'\0' && b != b' ' && b != b'\t' && b != b'\r' && b != b'\n' {
                                break;
                            }
                            l -= 1;
                        }
                        Zval::Str(PhpStr::new(input[pos..pos + l as usize].to_vec()))
                    }
                    b'Z' => {
                        let mut len = inputlen - inputpos;
                        if size >= 0 && len > size {
                            len = size;
                        }
                        size = len;
                        let mut s = 0i64;
                        while s < len && input[pos + s as usize] != b'\0' {
                            s += 1;
                        }
                        Zval::Str(PhpStr::new(input[pos..pos + s as usize].to_vec()))
                    }
                    b'h' | b'H' => {
                        let mut len = (inputlen - inputpos) * 2;
                        let mut nibbleshift = if ty == b'h' { 0 } else { 4 };
                        let mut first = true;
                        if size >= 0 && len > size * 2 {
                            len = size * 2;
                        }
                        if len > 0 && argb > 0 {
                            len -= argb % 2;
                        }
                        let mut buf = Vec::with_capacity(len.max(0) as usize);
                        let mut ipos = 0usize;
                        let mut opos = 0i64;
                        while opos < len {
                            let cc = (input[pos + ipos] >> nibbleshift) & 0xf;
                            let ch = if cc < 10 { cc + b'0' } else { cc + (b'a' - 10) };
                            buf.push(ch);
                            nibbleshift = (nibbleshift + 4) & 7;
                            if first {
                                first = false;
                            } else {
                                ipos += 1;
                                first = true;
                            }
                            opos += 1;
                        }
                        Zval::Str(PhpStr::new(buf))
                    }
                    b'c' => Zval::Long(read_int(input, pos, 1, false, true)),
                    b'C' => Zval::Long(read_int(input, pos, 1, false, false)),
                    b's' => Zval::Long(read_int(input, pos, 2, false, true)),
                    b'S' | b'v' => Zval::Long(read_int(input, pos, 2, false, false)),
                    b'n' => Zval::Long(read_int(input, pos, 2, true, false)),
                    b'i' => Zval::Long(read_int(input, pos, 4, false, true)),
                    b'I' => Zval::Long(read_int(input, pos, 4, false, false)),
                    b'l' => Zval::Long(read_int(input, pos, 4, false, true)),
                    b'L' | b'V' => Zval::Long(read_int(input, pos, 4, false, false)),
                    b'N' => Zval::Long(read_int(input, pos, 4, true, false)),
                    b'q' => Zval::Long(read_int(input, pos, 8, false, true)),
                    b'Q' | b'P' => Zval::Long(read_int(input, pos, 8, false, false)),
                    b'J' => Zval::Long(read_int(input, pos, 8, true, false)),
                    b'f' | b'g' => Zval::Double(read_f32(input, pos, false) as f64),
                    b'G' => Zval::Double(read_f32(input, pos, true) as f64),
                    b'd' | b'e' => Zval::Double(read_f64(input, pos, false)),
                    b'E' => Zval::Double(read_f64(input, pos, true)),
                    b'x' => {
                        emit = false;
                        Zval::Null
                    }
                    b'X' => {
                        emit = false;
                        if inputpos < size {
                            inputpos = -size;
                            iidx = repetitions - 1;
                            if repetitions >= 0 {
                                ctx.diags.push(Diag::Warning(format!(
                                    "unpack(): Type {}: outside of string",
                                    ty as char
                                )));
                            }
                        }
                        Zval::Null
                    }
                    b'@' => {
                        emit = false;
                        if repetitions <= inputlen {
                            inputpos = repetitions;
                        } else {
                            ctx.diags.push(Diag::Warning(format!(
                                "unpack(): Type {}: outside of string",
                                ty as char
                            )));
                        }
                        iidx = repetitions - 1;
                        Zval::Null
                    }
                    _ => unreachable!(),
                };

                if emit {
                    let key = if name.is_empty() {
                        Key::Int(iidx + 1)
                    } else if repetitions == 1 {
                        Key::from_bytes(name)
                    } else {
                        let mut k = name.to_vec();
                        k.extend_from_slice((iidx + 1).to_string().as_bytes());
                        Key::from_bytes(&k)
                    };
                    result.insert(key, val);
                }

                inputpos += size;
                if inputpos < 0 {
                    if size != -1 {
                        ctx.diags.push(Diag::Warning(format!(
                            "unpack(): Type {}: outside of string",
                            ty as char
                        )));
                    }
                    inputpos = 0;
                }
            } else if repetitions < 0 {
                break;
            } else {
                let remaining = inputlen - inputpos;
                ctx.diags.push(Diag::Warning(format!(
                    "unpack(): Type {}: not enough input values, need {} values but only {} {} provided",
                    ty as char,
                    size,
                    remaining,
                    if remaining == 1 { "was" } else { "were" }
                )));
                return Ok(Zval::Bool(false));
            }

            iidx += 1;
        }

        if fi < flen {
            // Skip the '/' separator.
            fi += 1;
        }
    }

    Ok(Zval::Array(std::rc::Rc::new(result)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::Diags;

    fn call(f: fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>, args: &[Zval]) -> Zval {
        let mut out = Vec::new();
        let mut diags: Diags = Vec::new();
        let mut ctx = Ctx {
            out: &mut out,
            diags: &mut diags,
        };
        f(args, &mut ctx).unwrap()
    }

    fn s(x: &str) -> Zval {
        Zval::Str(PhpStr::new(x.as_bytes().to_vec()))
    }

    fn bytes(z: &Zval) -> Vec<u8> {
        match z {
            Zval::Str(p) => p.as_bytes().to_vec(),
            _ => panic!("expected string, got {z:?}"),
        }
    }

    #[test]
    fn pack_big_endian_ints() {
        assert_eq!(bytes(&call(pack, &[s("N"), Zval::Long(65534)])), vec![0, 0, 0xff, 0xfe]);
        assert_eq!(bytes(&call(pack, &[s("n"), Zval::Long(0x1234)])), vec![0x12, 0x34]);
        assert_eq!(bytes(&call(pack, &[s("V"), Zval::Long(0x01020304)])), vec![4, 3, 2, 1]);
    }

    #[test]
    fn pack_a_and_z_padding() {
        assert_eq!(bytes(&call(pack, &[s("A5"), s("foo ")])), b"foo  ".to_vec());
        assert_eq!(bytes(&call(pack, &[s("a5"), s("foo")])), b"foo\0\0".to_vec());
        assert_eq!(bytes(&call(pack, &[s("Z5"), s("foo")])), b"foo\0\0".to_vec());
    }

    #[test]
    fn pack_hex() {
        // H packs high nibble first.
        assert_eq!(bytes(&call(pack, &[s("H*"), s("48656c6c6f")])), b"Hello".to_vec());
    }

    #[test]
    fn unpack_roundtrip_named() {
        // pack() has no field names; "N" packs one big-endian 32-bit value.
        let packed = call(pack, &[s("N"), Zval::Long(65534)]);
        assert_eq!(bytes(&packed), vec![0, 0, 0xff, 0xfe]);
        let arr = call(unpack, &[s("Nlen"), packed]);
        match arr {
            Zval::Array(a) => {
                let (k, v) = a.iter().next().unwrap();
                assert_eq!(k, &Key::from_bytes(b"len"));
                assert!(matches!(v, Zval::Long(65534)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn unpack_a_keeps_padding_capital_strips() {
        let arr = call(unpack, &[s("A*"), s("foo  ")]);
        match arr {
            Zval::Array(a) => {
                let (_, v) = a.iter().next().unwrap();
                assert_eq!(bytes(v), b"foo".to_vec());
            }
            _ => panic!(),
        }
        let arr = call(unpack, &[s("a*"), s("foo  ")]);
        match arr {
            Zval::Array(a) => {
                let (_, v) = a.iter().next().unwrap();
                assert_eq!(bytes(v), b"foo  ".to_vec());
            }
            _ => panic!(),
        }
    }
}
