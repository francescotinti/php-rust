//! CSV engine (step 54c): line parsing and field quoting shared by
//! `str_getcsv`, `fgetcsv`, and `fputcsv`. Semantics verified byte-exact against
//! PHP 8.5.7. PHP uses only the first byte of the separator / enclosure / escape
//! strings; an empty escape string disables escaping. The parser handles
//! doubled-enclosure (`""` → `"`), the escape char inside a quoted field, and
//! embedded separators/newlines within quotes.

use php_runtime::Ctx;
use php_types::{convert, Diag, PhpArray, PhpError, PhpStr, Zval};

/// The PHP `$escape` deprecation (PHP 8.5): emitted when the caller omits the
/// `$escape` argument. `arg_count` is the number of arguments actually passed;
/// `escape_index` is the position of the `$escape` parameter.
pub fn maybe_escape_deprecation(ctx: &mut Ctx, fname: &str, arg_count: usize, escape_index: usize) {
    if arg_count <= escape_index {
        ctx.diags.push(Diag::Deprecated(format!(
            "{fname}(): the $escape parameter must be provided as its default value will change"
        )));
    }
}

/// The first byte of a CSV control-string argument, or `default` when absent.
pub fn first_byte(arg: Option<&Zval>, ctx: &mut Ctx, default: u8) -> u8 {
    match arg {
        Some(v) => {
            let s = convert::to_zstr(v, ctx.diags);
            s.as_bytes().first().copied().unwrap_or(default)
        }
        None => default,
    }
}

/// The escape byte, or `None` when the argument is an explicit empty string
/// (which disables escaping in PHP).
pub fn escape_byte(arg: Option<&Zval>, ctx: &mut Ctx) -> Option<u8> {
    match arg {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().first().copied(),
        None => Some(b'\\'),
    }
}

/// Parse one CSV record's fields. `line` must have any trailing newline already
/// stripped by the caller (fgetcsv) or absent (str_getcsv).
pub fn parse_csv_line(line: &[u8], sep: u8, enc: u8, esc: Option<u8>) -> Vec<Vec<u8>> {
    let mut fields = Vec::new();
    let mut i = 0;
    loop {
        let (field, ni) = parse_field(line, i, sep, enc, esc);
        fields.push(field);
        i = ni;
        if i < line.len() && line[i] == sep {
            i += 1; // consume the separator; a trailing one yields a final empty field
        } else {
            break;
        }
    }
    fields
}

/// Parse a single field starting at `i`; returns `(field_bytes, index_at_sep_or_end)`.
fn parse_field(line: &[u8], mut i: usize, sep: u8, enc: u8, esc: Option<u8>) -> (Vec<u8>, usize) {
    let mut field = Vec::new();
    if line.get(i) == Some(&enc) {
        // Quoted field.
        i += 1;
        while i < line.len() {
            let b = line[i];
            if Some(b) == esc && i + 1 < line.len() {
                // Escape: the escape byte and the next byte are kept literally.
                field.push(b);
                field.push(line[i + 1]);
                i += 2;
            } else if b == enc {
                if line.get(i + 1) == Some(&enc) {
                    field.push(enc); // doubled enclosure → literal
                    i += 2;
                } else {
                    i += 1; // closing enclosure
                    break;
                }
            } else {
                field.push(b);
                i += 1;
            }
        }
        // Any bytes between the closing enclosure and the next separator are
        // appended verbatim (PHP behaviour).
        while i < line.len() && line[i] != sep {
            field.push(line[i]);
            i += 1;
        }
    } else {
        // Unquoted field: read up to the separator; the escape char is inert here.
        while i < line.len() && line[i] != sep {
            field.push(line[i]);
            i += 1;
        }
    }
    (field, i)
}

/// Build a `Zval` array from parsed CSV fields. An empty input is the single
/// `[null]` PHP returns for a blank line.
pub fn fields_to_array(input: &[u8], sep: u8, enc: u8, esc: Option<u8>) -> Zval {
    let mut arr = PhpArray::new();
    if input.is_empty() {
        let _ = arr.append(Zval::Null);
        return Zval::Array(std::rc::Rc::new(arr));
    }
    for f in parse_csv_line(input, sep, enc, esc) {
        let _ = arr.append(Zval::Str(PhpStr::new(f)));
    }
    Zval::Array(std::rc::Rc::new(arr))
}

/// Whether a field must be quoted on output: it contains the separator,
/// enclosure, escape char, or whitespace / NUL (PHP `fputcsv` qualify set).
fn needs_quote(field: &[u8], sep: u8, enc: u8, esc: Option<u8>) -> bool {
    field.iter().any(|&b| {
        b == sep
            || b == enc
            || Some(b) == esc
            || matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0)
    })
}

/// Render one field for `fputcsv`: quote it when required, doubling any embedded
/// enclosure byte.
pub fn format_csv_field(field: &[u8], sep: u8, enc: u8, esc: Option<u8>) -> Vec<u8> {
    if !needs_quote(field, sep, enc, esc) {
        return field.to_vec();
    }
    let mut out = Vec::with_capacity(field.len() + 2);
    out.push(enc);
    for &b in field {
        if b == enc {
            out.push(enc); // double it
        }
        out.push(b);
    }
    out.push(enc);
    out
}

/// Join CSV fields into a record line (no trailing EOL).
pub fn format_csv_line(fields: &[Vec<u8>], sep: u8, enc: u8, esc: Option<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push(sep);
        }
        out.extend_from_slice(&format_csv_field(f, sep, enc, esc));
    }
    out
}

/// `str_getcsv($string, $separator = ",", $enclosure = "\"", $escape = "\\")`:
/// parse one CSV record into an array. Emits the PHP 8.5 `$escape` deprecation
/// when that argument is omitted.
pub fn str_getcsv(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    maybe_escape_deprecation(ctx, "str_getcsv", argv.len(), 3);
    let input = convert::to_zstr(
        argv.first().ok_or_else(|| {
            PhpError::ArgumentCountError("str_getcsv() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    )
    .as_bytes()
    .to_vec();
    let sep = first_byte(argv.get(1), ctx, b',');
    let enc = first_byte(argv.get(2), ctx, b'"');
    let esc = escape_byte(argv.get(3), ctx);
    Ok(fields_to_array(&input, sep, enc, esc))
}
