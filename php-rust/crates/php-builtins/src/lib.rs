//! PHP builtin functions (Tier 1 nucleus, plan step 5).
//!
//! Each builtin has the [`php_runtime::BuiltinFn`] signature and is registered
//! by name in [`registry`]. The evaluator dispatches to them through the
//! injected registry (see `php_runtime::builtin`), so this crate depends on
//! php-runtime, not the other way around.
//!
//! Scope: `var_dump`, `strlen`, `gettype`, the `is_*` predicate family, and the
//! `*val` cast helpers. Frequency-driven expansion (implode, count, substr,
//! sprintf, array_*) is step 10.

mod array;
mod format;
mod math;
mod string;

use php_runtime::{Builtin, Ctx, Registry};
use php_types::{convert, dtoa, numstr, Key, PhpError, PhpStr, Zval};

/// Build the Tier 1 builtin registry.
pub fn registry() -> Registry {
    let mut r = Registry::new();
    let mut add = |name: &[u8], f: php_runtime::BuiltinFn| {
        r.insert(name.to_vec(), Builtin::Value(f));
    };
    add(b"count", array::count);
    add(b"sizeof", array::count);
    add(b"array_keys", array::array_keys);
    add(b"array_values", array::array_values);
    add(b"in_array", array::in_array);
    add(b"array_merge", array::array_merge);
    add(b"implode", string::implode);
    add(b"join", string::implode);
    add(b"explode", string::explode);
    add(b"substr", string::substr);
    add(b"strpos", string::strpos);
    add(b"str_replace", string::str_replace);
    add(b"strtoupper", string::strtoupper);
    add(b"strtolower", string::strtolower);
    add(b"ucfirst", string::ucfirst);
    add(b"lcfirst", string::lcfirst);
    add(b"ucwords", string::ucwords);
    add(b"str_repeat", string::str_repeat);
    add(b"str_pad", string::str_pad);
    add(b"chr", string::chr);
    add(b"ord", string::ord);
    add(b"trim", string::trim);
    add(b"ltrim", string::ltrim);
    add(b"rtrim", string::rtrim);
    add(b"sprintf", format::sprintf);
    add(b"printf", format::printf);
    add(b"abs", math::abs);
    add(b"max", math::max);
    add(b"min", math::min);
    add(b"var_dump", var_dump);
    add(b"strlen", strlen);
    add(b"gettype", gettype);
    add(b"is_int", is_int);
    add(b"is_integer", is_int);
    add(b"is_long", is_int);
    add(b"is_float", is_float);
    add(b"is_double", is_float);
    add(b"is_string", is_string);
    add(b"is_bool", is_bool);
    add(b"is_null", is_null);
    add(b"is_array", is_array);
    add(b"is_scalar", is_scalar);
    add(b"is_numeric", is_numeric);
    add(b"intval", intval);
    add(b"floatval", floatval);
    add(b"doubleval", floatval);
    add(b"strval", strval);
    add(b"boolval", boolval);
    add(b"print_r", print_r);
    // By-reference builtins (step 11c): their first argument binds the caller's
    // variable cell (D-R7).
    let mut add_ref = |name: &[u8], f: php_runtime::BuiltinRefFn| {
        r.insert(name.to_vec(), Builtin::RefFirst(f));
    };
    add_ref(b"array_push", array::array_push);
    add_ref(b"sort", array::sort);
    add_ref(b"array_pop", array::array_pop);
    add_ref(b"array_shift", array::array_shift);
    r
}

/// First positional argument, or an `ArgumentCountError`-style fatal.
fn arg1<'a>(args: &'a [Zval], fname: &str) -> Result<&'a Zval, PhpError> {
    args.first()
        .ok_or_else(|| PhpError::Error(format!("{fname}() expects exactly 1 argument, 0 given")))
}

// --- output ---

fn var_dump(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.is_empty() {
        return Err(PhpError::Error(
            "var_dump() expects at least 1 argument, 0 given".to_string(),
        ));
    }
    for v in args {
        dump(ctx.out, v, 0);
    }
    Ok(Zval::Null)
}

/// Recursive var_dump formatter. `indent` is the leading-space count for this
/// value's own block; nested array entries indent by a further 2.
fn dump(out: &mut Vec<u8>, v: &Zval, indent: usize) {
    match v {
        Zval::Undef | Zval::Null => out.extend_from_slice(b"NULL\n"),
        Zval::Bool(true) => out.extend_from_slice(b"bool(true)\n"),
        Zval::Bool(false) => out.extend_from_slice(b"bool(false)\n"),
        Zval::Long(n) => out.extend_from_slice(format!("int({n})\n").as_bytes()),
        Zval::Double(d) => {
            // var_dump uses serialize_precision=-1 → shortest roundtrip.
            out.extend_from_slice(b"float(");
            out.extend_from_slice(&dtoa::double_to_shortest(*d));
            out.extend_from_slice(b")\n");
        }
        Zval::Str(s) => {
            out.extend_from_slice(format!("string({}) \"", s.len()).as_bytes());
            out.extend_from_slice(s.as_bytes());
            out.extend_from_slice(b"\"\n");
        }
        Zval::Array(a) => {
            out.extend_from_slice(format!("array({}) {{\n", a.len()).as_bytes());
            for (key, val) in a.iter() {
                spaces(out, indent + 2);
                match key {
                    Key::Int(i) => out.extend_from_slice(format!("[{i}]=>\n").as_bytes()),
                    Key::Str(s) => {
                        out.extend_from_slice(b"[\"");
                        out.extend_from_slice(s.as_bytes());
                        out.extend_from_slice(b"\"]=>\n");
                    }
                }
                spaces(out, indent + 2);
                // A reference element shared with at least one live alias is
                // marked `&` (D-R14); a sole-holder reference (strong_count 1)
                // prints as a plain value, matching PHP's var_dump.
                match val {
                    Zval::Ref(cell) if std::rc::Rc::strong_count(cell) >= 2 => {
                        out.push(b'&');
                        dump(out, &cell.borrow(), indent + 2);
                    }
                    _ => dump(out, val, indent + 2),
                }
            }
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
        // A top-level reference is dereferenced transparently (the `&` marker
        // only applies to reference *elements* inside a container).
        Zval::Ref(cell) => dump(out, &cell.borrow(), indent),
    }
}

fn spaces(out: &mut Vec<u8>, n: usize) {
    out.resize(out.len() + n, b' ');
}

/// print_r($value, $return = false). Human-readable dump; with a truthy
/// `$return` the rendering is returned as a string instead of being printed.
fn print_r(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "print_r")?;
    let want_return = matches!(args.get(1), Some(r) if convert::is_true_silent(r));
    let mut buf = Vec::new();
    print_r_into(&mut buf, v, 0, ctx);
    if want_return {
        Ok(Zval::Str(PhpStr::new(buf)))
    } else {
        ctx.out.extend_from_slice(&buf);
        Ok(Zval::Bool(true))
    }
}

/// Recursive print_r renderer. `indent` is the leading-space count of this
/// value's `(` block (0 at the top level); nested arrays add 8.
fn print_r_into(out: &mut Vec<u8>, v: &Zval, indent: usize, ctx: &mut Ctx) {
    match v {
        Zval::Array(a) => {
            out.extend_from_slice(b"Array\n");
            spaces(out, indent);
            out.extend_from_slice(b"(\n");
            for (key, val) in a.iter() {
                spaces(out, indent + 4);
                match key {
                    Key::Int(i) => out.extend_from_slice(format!("[{i}] => ").as_bytes()),
                    Key::Str(s) => {
                        out.push(b'[');
                        out.extend_from_slice(s.as_bytes());
                        out.extend_from_slice(b"] => ");
                    }
                }
                print_r_into(out, val, indent + 8, ctx);
                out.push(b'\n');
            }
            spaces(out, indent);
            out.extend_from_slice(b")\n");
        }
        // print_r is reference-transparent: deref and recurse, no `&` marker.
        Zval::Ref(cell) => print_r_into(out, &cell.borrow(), indent, ctx),
        scalar => out.extend_from_slice(convert::to_zstr(scalar, ctx.diags).as_bytes()),
    }
}

// --- string / type inspection ---

fn strlen(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "strlen")?;
    if matches!(v, Zval::Array(_)) {
        return Err(PhpError::TypeError(
            "strlen(): Argument #1 ($string) must be of type string, array given".to_string(),
        ));
    }
    let s = convert::to_zstr(v, ctx.diags);
    Ok(Zval::Long(s.len() as i64))
}

fn gettype(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "gettype")?;
    Ok(Zval::Str(PhpStr::from_str(v.gettype())))
}

// --- type predicates ---

fn is_int(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_int")?, Zval::Long(_))))
}

fn is_float(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_float")?, Zval::Double(_))))
}

fn is_string(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_string")?, Zval::Str(_))))
}

fn is_bool(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_bool")?, Zval::Bool(_))))
}

fn is_null(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(
        arg1(args, "is_null")?,
        Zval::Null | Zval::Undef
    )))
}

fn is_array(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_array")?, Zval::Array(_))))
}

fn is_scalar(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(
        arg1(args, "is_scalar")?,
        Zval::Long(_) | Zval::Double(_) | Zval::Str(_) | Zval::Bool(_)
    )))
}

fn is_numeric(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "is_numeric")?;
    let numeric = match v {
        Zval::Long(_) | Zval::Double(_) => true,
        // A string is numeric iff it parses fully (no trailing non-numeric bytes).
        Zval::Str(s) => numstr::parse_numeric_ex(s.as_bytes(), true).is_some_and(|i| !i.trailing),
        _ => false,
    };
    Ok(Zval::Bool(numeric))
}

// --- value casts ---

fn intval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(convert::to_long_cast(arg1(args, "intval")?, ctx.diags)))
}

fn floatval(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Double(convert::to_double(arg1(args, "floatval")?)))
}

fn strval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(convert::to_zstr_cast(arg1(args, "strval")?, ctx.diags)))
}

fn boolval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(convert::to_bool(arg1(args, "boolval")?, ctx.diags)))
}
