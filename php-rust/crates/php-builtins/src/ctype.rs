//! `ctype` extension: C-locale (ASCII) character-class predicates. Mirrors
//! `ext/ctype/ctype.c` — including the deprecated non-string fallback where an
//! integer is interpreted as a byte / its decimal representation.

use php_runtime::Ctx;
use php_types::{Diag, PhpError, Zval};

// One predicate per class, matching the C-locale `is*` macros exactly. Note
// `is_print` adds the space (0x20) to `graph`, and `is_space` includes the
// vertical tab (0x0B) which Rust's `is_ascii_whitespace` omits.
fn is_alnum(b: u8) -> bool { b.is_ascii_alphanumeric() }
fn is_alpha(b: u8) -> bool { b.is_ascii_alphabetic() }
fn is_cntrl(b: u8) -> bool { b.is_ascii_control() }
fn is_digit(b: u8) -> bool { b.is_ascii_digit() }
fn is_lower(b: u8) -> bool { b.is_ascii_lowercase() }
fn is_graph(b: u8) -> bool { b.is_ascii_graphic() }
fn is_print(b: u8) -> bool { b == b' ' || b.is_ascii_graphic() }
fn is_punct(b: u8) -> bool { b.is_ascii_punctuation() }
fn is_space(b: u8) -> bool { b == b' ' || (0x09..=0x0D).contains(&b) }
fn is_upper(b: u8) -> bool { b.is_ascii_uppercase() }
fn is_xdigit(b: u8) -> bool { b.is_ascii_hexdigit() }

/// Shared body for every `ctype_*` builtin (`ctype_impl` / `ctype_fallback`).
/// A string is `true` iff non-empty and every byte satisfies `pred`. A
/// non-string argument is deprecated since 8.1 and falls back: an int in
/// `-128..=255` is the corresponding byte; a larger non-negative int has an
/// all-digits decimal form (`allow_digits`); a smaller negative int's decimal
/// form starts with `-` (`allow_minus`); any other type is `false`.
fn ctype(
    args: &[Zval],
    ctx: &mut Ctx,
    name: &str,
    pred: fn(u8) -> bool,
    allow_digits: bool,
    allow_minus: bool,
) -> Result<Zval, PhpError> {
    let v = args
        .first()
        .ok_or_else(|| PhpError::Error(format!("{name}() expects exactly 1 argument, 0 given")))?;
    let r = match v {
        Zval::Str(s) => {
            let b = s.as_bytes();
            !b.is_empty() && b.iter().all(|&c| pred(c))
        }
        other => {
            ctx.diags.push(Diag::Deprecated(format!(
                "{name}(): Argument of type {} will be interpreted as string in the future",
                other.type_name_for_error()
            )));
            match other {
                Zval::Long(n) => {
                    let n = *n;
                    if (0..=255).contains(&n) {
                        pred(n as u8)
                    } else if (-128..0).contains(&n) {
                        pred((n + 256) as u8)
                    } else if n >= 0 {
                        allow_digits
                    } else {
                        allow_minus
                    }
                }
                _ => false,
            }
        }
    };
    Ok(Zval::Bool(r))
}

macro_rules! ctype_fn {
    ($fn:ident, $name:literal, $pred:path, $digits:literal, $minus:literal) => {
        pub fn $fn(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
            ctype(args, ctx, $name, $pred, $digits, $minus)
        }
    };
}

ctype_fn!(ctype_alnum, "ctype_alnum", is_alnum, true, false);
ctype_fn!(ctype_alpha, "ctype_alpha", is_alpha, false, false);
ctype_fn!(ctype_cntrl, "ctype_cntrl", is_cntrl, false, false);
ctype_fn!(ctype_digit, "ctype_digit", is_digit, true, false);
ctype_fn!(ctype_lower, "ctype_lower", is_lower, false, false);
ctype_fn!(ctype_graph, "ctype_graph", is_graph, true, true);
ctype_fn!(ctype_print, "ctype_print", is_print, true, true);
ctype_fn!(ctype_punct, "ctype_punct", is_punct, false, false);
ctype_fn!(ctype_space, "ctype_space", is_space, false, false);
ctype_fn!(ctype_upper, "ctype_upper", is_upper, false, false);
ctype_fn!(ctype_xdigit, "ctype_xdigit", is_xdigit, true, false);
