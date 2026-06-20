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
mod date;
mod file;
mod format;
mod json;
mod math;
mod mbstring;
mod serialize;
mod string;

use std::rc::Rc;

use php_runtime::{Builtin, Ctx, Registry};
use php_types::{
    convert, dtoa, numstr, Closure, ClosureRender, Diag, Diags, Key, PhpArray, PhpError, PhpStr,
    PropVis, Zval,
};

/// Build the Tier 1 builtin registry.
pub fn registry() -> Registry {
    let mut r = Registry::new();
    let mut add = |name: &[u8], f: php_runtime::BuiltinFn| {
        r.insert(name.to_vec(), Builtin::Value(f));
    };
    add(b"count", array::count);
    add(b"sizeof", array::count);
    add(b"date", date::date);
    add(b"gmdate", date::gmdate);
    add(b"mktime", date::mktime);
    add(b"gmmktime", date::gmmktime);
    add(b"checkdate", date::checkdate);
    add(b"strtotime", date::strtotime);
    add(b"time", date::time);
    add(b"date_default_timezone_set", date::date_default_timezone_set);
    add(b"date_default_timezone_get", date::date_default_timezone_get);
    add(b"getdate", date::getdate);
    add(b"localtime", date::localtime);
    add(b"__interval_parse", date::__interval_parse);
    add(b"__interval_from_date_string", date::__interval_from_date_string);
    add(b"__date_diff", date::__date_diff);
    add(b"__interval_format", date::__interval_format);
    add(b"__date_from_format", date::__date_from_format);
    add(b"json_encode", json::json_encode);
    // File / stream builtins (step 51; `fopen` is evaluator-dispatched).
    add(b"fread", file::fread);
    add(b"fwrite", file::fwrite);
    add(b"fputs", file::fwrite);
    add(b"fclose", file::fclose);
    add(b"fgets", file::fgets);
    add(b"fgetc", file::fgetc);
    add(b"feof", file::feof);
    add(b"fseek", file::fseek);
    add(b"ftell", file::ftell);
    add(b"rewind", file::rewind);
    add(b"fflush", file::fflush);
    add(b"file_get_contents", file::file_get_contents);
    add(b"file_put_contents", file::file_put_contents);
    add(b"array_keys", array::array_keys);
    add(b"array_values", array::array_values);
    add(b"in_array", array::in_array);
    add(b"array_merge", array::array_merge);
    add(b"range", array::range);
    add(b"array_slice", array::array_slice);
    add(b"array_reverse", array::array_reverse);
    add(b"array_unique", array::array_unique);
    add(b"array_sum", array::array_sum);
    add(b"array_key_exists", array::array_key_exists);
    add(b"key_exists", array::array_key_exists);
    add(b"array_search", array::array_search);
    add(b"array_fill", array::array_fill);
    add(b"array_flip", array::array_flip);
    add(b"array_combine", array::array_combine);
    add(b"array_pad", array::array_pad);
    add(b"array_product", array::array_product);
    add(b"array_key_first", array::array_key_first);
    add(b"array_key_last", array::array_key_last);
    add(b"array_diff", array::array_diff);
    add(b"array_intersect", array::array_intersect);
    add(b"array_diff_key", array::array_diff_key);
    add(b"array_intersect_key", array::array_intersect_key);
    add(b"array_diff_assoc", array::array_diff_assoc);
    add(b"array_intersect_assoc", array::array_intersect_assoc);
    add(b"array_column", array::array_column);
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
    add(b"strrev", string::strrev);
    add(b"str_contains", string::str_contains);
    add(b"str_starts_with", string::str_starts_with);
    add(b"str_ends_with", string::str_ends_with);
    add(b"str_split", string::str_split);
    add(b"substr_count", string::substr_count);
    add(b"mb_strlen", mbstring::mb_strlen);
    add(b"mb_substr", mbstring::mb_substr);
    add(b"mb_str_split", mbstring::mb_str_split);
    add(b"mb_strtoupper", mbstring::mb_strtoupper);
    add(b"mb_strtolower", mbstring::mb_strtolower);
    add(b"mb_convert_case", mbstring::mb_convert_case);
    add(b"mb_ucfirst", mbstring::mb_ucfirst);
    add(b"mb_lcfirst", mbstring::mb_lcfirst);
    add(b"mb_strpos", mbstring::mb_strpos);
    add(b"mb_stripos", mbstring::mb_stripos);
    add(b"mb_strrpos", mbstring::mb_strrpos);
    add(b"mb_strripos", mbstring::mb_strripos);
    add(b"mb_strstr", mbstring::mb_strstr);
    add(b"mb_stristr", mbstring::mb_stristr);
    add(b"mb_strrchr", mbstring::mb_strrchr);
    add(b"mb_strrichr", mbstring::mb_strrichr);
    add(b"mb_substr_count", mbstring::mb_substr_count);
    add(b"mb_ord", mbstring::mb_ord);
    add(b"mb_chr", mbstring::mb_chr);
    add(b"mb_str_pad", mbstring::mb_str_pad);
    add(b"mb_trim", mbstring::mb_trim);
    add(b"mb_ltrim", mbstring::mb_ltrim);
    add(b"mb_rtrim", mbstring::mb_rtrim);
    add(b"mb_check_encoding", mbstring::mb_check_encoding);
    add(b"mb_strwidth", mbstring::mb_strwidth);
    add(b"mb_strimwidth", mbstring::mb_strimwidth);
    add(b"mb_strcut", mbstring::mb_strcut);
    add(b"mb_convert_encoding", mbstring::mb_convert_encoding);
    add(b"mb_detect_encoding", mbstring::mb_detect_encoding);
    add(b"number_format", string::number_format);
    add(b"sprintf", format::sprintf);
    add(b"printf", format::printf);
    add(b"abs", math::abs);
    add(b"max", math::max);
    add(b"min", math::min);
    add(b"intdiv", math::intdiv);
    add(b"pow", math::pow);
    add(b"sqrt", math::sqrt);
    add(b"floor", math::floor);
    add(b"ceil", math::ceil);
    add(b"round", math::round);
    add(b"var_dump", var_dump);
    add(b"var_export", var_export);
    add(b"serialize", serialize::serialize);
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
    add_ref(b"array_splice", array::array_splice);
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
        let mut seen = Vec::new();
        dump(ctx.out, v, 0, &mut seen);
    }
    Ok(Zval::Null)
}

/// Recursive var_dump formatter. `indent` is the leading-space count for this
/// value's own block; nested entries indent by a further 2. `seen` holds the
/// addresses of containers currently being dumped, so a value that refers back
/// into its own subtree prints `*RECURSION*` instead of looping (step 19-7).
fn dump(out: &mut Vec<u8>, v: &Zval, indent: usize, seen: &mut Vec<usize>) {
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
            let ptr = Rc::as_ptr(a) as usize;
            if seen.contains(&ptr) {
                out.extend_from_slice(b"*RECURSION*\n");
                return;
            }
            seen.push(ptr);
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
                        dump(out, &cell.borrow(), indent + 2, seen);
                    }
                    _ => dump(out, val, indent + 2, seen),
                }
            }
            seen.pop();
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
        // A top-level reference is dereferenced transparently (the `&` marker
        // only applies to reference *elements* inside a container).
        Zval::Ref(cell) => dump(out, &cell.borrow(), indent, seen),
        // A closure dumps as a `Closure` object with its name/file/line (or the
        // wrapped `function`) plus a `parameter` pseudo-property (step 18-7).
        Zval::Closure(c) => {
            let props = closure_properties(c);
            out.extend_from_slice(
                format!("object(Closure)#{} ({}) {{\n", c.id, props.len()).as_bytes(),
            );
            for (k, val) in &props {
                spaces(out, indent + 2);
                out.extend_from_slice(b"[\"");
                out.extend_from_slice(k);
                out.extend_from_slice(b"\"]=>\n");
                spaces(out, indent + 2);
                dump(out, val, indent + 2, seen);
            }
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
        // A `Generator` dumps with a single `function` pseudo-property naming the
        // generator function (step 39-7).
        Zval::Generator(g) => {
            let g = g.borrow();
            out.extend_from_slice(format!("object(Generator)#{} (1) {{\n", g.id).as_bytes());
            spaces(out, indent + 2);
            out.extend_from_slice(b"[\"function\"]=>\n");
            spaces(out, indent + 2);
            out.extend_from_slice(
                format!("string({}) \"", g.func_name.len()).as_bytes(),
            );
            out.extend_from_slice(&g.func_name);
            out.extend_from_slice(b"\"\n");
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
        // A resource (step 51): `resource(N) of type (stream)` while open,
        // `resource(N) of type (Unknown)` once closed (oracle-verified, D-51.5).
        Zval::Resource(r) => {
            let r = r.borrow();
            out.extend_from_slice(
                format!("resource({}) of type ({})\n", r.id, r.dump_type()).as_bytes(),
            );
        }
        // A class instance (step 19-7): `object(C)#N (k) { ["p"]=>…,
        // ["p":protected]=>…, ["p":"C":private]=>… }`, with a recursion guard.
        Zval::Object(o) => {
            let ptr = Rc::as_ptr(o) as usize;
            if seen.contains(&ptr) {
                out.extend_from_slice(b"*RECURSION*\n");
                return;
            }
            seen.push(ptr);
            let obj = o.borrow();
            // An enum case renders as `enum(Name::Case)` (step 23, D-23.5); the
            // backing value is intentionally not shown.
            if obj.info.is_enum_case {
                out.extend_from_slice(b"enum(");
                out.extend_from_slice(obj.class_name.as_bytes());
                out.extend_from_slice(b"::");
                if let Some(Zval::Str(s)) = obj.props.get(b"name") {
                    out.extend_from_slice(s.as_bytes());
                }
                out.extend_from_slice(b")\n");
                drop(obj);
                seen.pop();
                return;
            }
            out.extend_from_slice(b"object(");
            out.extend_from_slice(obj.class_name.as_bytes());
            out.extend_from_slice(format!(")#{} ({}) {{\n", obj.id, obj.props.len()).as_bytes());
            for (k, val) in obj.props.iter() {
                spaces(out, indent + 2);
                out.extend_from_slice(b"[\"");
                out.extend_from_slice(k);
                match obj.info.vis_of(k) {
                    PropVis::Public => out.extend_from_slice(b"\"]=>\n"),
                    PropVis::Protected => out.extend_from_slice(b"\":protected]=>\n"),
                    PropVis::Private(cls) => {
                        out.extend_from_slice(b"\":\"");
                        out.extend_from_slice(cls.as_bytes());
                        out.extend_from_slice(b"\":private]=>\n");
                    }
                }
                spaces(out, indent + 2);
                dump(out, val, indent + 2, seen);
            }
            drop(obj);
            seen.pop();
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
    }
}

/// The `var_dump`/`print_r` pseudo-properties of a closure, in PHP's order
/// (step 18-7, D-18.9): `name`/`file`/`line` for an anonymous/arrow closure or a
/// single `function` for a first-class callable, then a `parameter` array iff
/// the closure has any parameters.
fn closure_properties(c: &Closure) -> Vec<(Vec<u8>, Zval)> {
    let mut props: Vec<(Vec<u8>, Zval)> = Vec::new();
    match &c.info.kind {
        ClosureRender::Closure { name, file, line } => {
            props.push((b"name".to_vec(), Zval::Str(Rc::clone(name))));
            props.push((b"file".to_vec(), Zval::Str(Rc::clone(file))));
            props.push((b"line".to_vec(), Zval::Long(*line as i64)));
        }
        ClosureRender::Function(name) => {
            props.push((b"function".to_vec(), Zval::Str(Rc::clone(name))));
        }
    }
    if !c.info.params.is_empty() {
        let mut parr = PhpArray::new();
        for p in &c.info.params {
            let mut key = Vec::with_capacity(p.name.len() + 1);
            key.push(b'$');
            key.extend_from_slice(&p.name);
            let marker = if p.optional { "<optional>" } else { "<required>" };
            parr.insert(Key::from_bytes(&key), Zval::str_from(marker));
        }
        props.push((b"parameter".to_vec(), Zval::Array(Rc::new(parr))));
    }
    props
}

fn spaces(out: &mut Vec<u8>, n: usize) {
    out.resize(out.len() + n, b' ');
}

/// `var_export($value, $return = false)` (step 47). Renders a value as a
/// PHP-parsable literal. With a truthy `$return` the rendering is returned as a
/// string instead of being printed. A direct port of PHP's `php_var_export_ex`.
fn var_export(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "var_export")?;
    let want_return = matches!(args.get(1), Some(r) if convert::is_true_silent(r));
    let mut buf = Vec::new();
    let mut seen = Vec::new();
    export_into(&mut buf, v, 1, &mut seen, ctx.diags);
    if want_return {
        Ok(Zval::Str(PhpStr::new(buf)))
    } else {
        ctx.out.extend_from_slice(&buf);
        Ok(Zval::Null)
    }
}

/// Recursive `var_export` formatter (port of `php_var_export_ex`). `level` starts
/// at 1; PHP's indentation is derived from it: array members indent by
/// `(level+1)` spaces, object members by `(level+2)`, and a nested value recurses
/// at `level+2`. The opening `array (` / closing `)` of a *nested* container
/// (level > 1) is preceded by `(level-1)` spaces (and, for the opener, a newline).
fn export_into(out: &mut Vec<u8>, v: &Zval, level: usize, seen: &mut Vec<usize>, diags: &mut Diags) {
    match v {
        Zval::Undef | Zval::Null => out.extend_from_slice(b"NULL"),
        Zval::Bool(true) => out.extend_from_slice(b"true"),
        Zval::Bool(false) => out.extend_from_slice(b"false"),
        Zval::Long(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Zval::Double(d) => out.extend_from_slice(&export_float(*d)),
        Zval::Str(s) => export_str(out, s.as_bytes()),
        Zval::Ref(cell) => export_into(out, &cell.borrow(), level, seen, diags),
        Zval::Array(a) => {
            let ptr = Rc::as_ptr(a) as usize;
            if seen.contains(&ptr) {
                // PHP emits a Warning and `NULL` on a circular reference.
                diags.push(Diag::Warning(
                    "var_export does not handle circular references".to_string(),
                ));
                out.extend_from_slice(b"NULL");
                return;
            }
            seen.push(ptr);
            if level > 1 {
                out.push(b'\n');
                spaces(out, level - 1);
            }
            out.extend_from_slice(b"array (\n");
            for (key, val) in a.iter() {
                spaces(out, level + 1);
                match key {
                    Key::Int(i) => out.extend_from_slice(i.to_string().as_bytes()),
                    Key::Str(s) => export_str(out, s.as_bytes()),
                }
                out.extend_from_slice(b" => ");
                export_into(out, val, level + 2, seen, diags);
                out.extend_from_slice(b",\n");
            }
            seen.pop();
            spaces(out, level - 1);
            out.push(b')');
        }
        Zval::Object(o) => {
            let ptr = Rc::as_ptr(o) as usize;
            if seen.contains(&ptr) {
                diags.push(Diag::Warning(
                    "var_export does not handle circular references".to_string(),
                ));
                out.extend_from_slice(b"NULL");
                return;
            }
            seen.push(ptr);
            let obj = o.borrow();
            if level > 1 {
                out.push(b'\n');
                spaces(out, level - 1);
            }
            // `stdClass` renders as a cast; any other class via `__set_state`.
            let is_std = obj.class_name.as_bytes() == b"stdClass";
            if is_std {
                out.extend_from_slice(b"(object) array(\n");
            } else {
                out.push(b'\\');
                out.extend_from_slice(obj.class_name.as_bytes());
                out.extend_from_slice(b"::__set_state(array(\n");
            }
            // All properties are exported by value, with no visibility markers.
            for (k, val) in obj.props.iter() {
                spaces(out, level + 2);
                export_str(out, k);
                out.extend_from_slice(b" => ");
                export_into(out, val, level + 2, seen, diags);
                out.extend_from_slice(b",\n");
            }
            drop(obj);
            seen.pop();
            spaces(out, level - 1);
            if is_std {
                out.push(b')');
            } else {
                out.extend_from_slice(b"))");
            }
        }
        // Closures / generators / resources have no `var_export` form
        // (D-47.1 scope-out; PHP warns and yields NULL for a resource).
        Zval::Closure(_) | Zval::Generator(_) | Zval::Resource(_) => {
            out.extend_from_slice(b"NULL")
        }
    }
}

/// `var_export` float: shortest round-trip, but always a valid PHP float literal
/// — if the result has no `.`/`e`/`E` and is finite, append `.0` (`1.0`, `-0.0`).
fn export_float(d: f64) -> Vec<u8> {
    let mut s = dtoa::double_to_shortest(d);
    if d.is_finite() && !s.iter().any(|&b| matches!(b, b'.' | b'e' | b'E')) {
        s.extend_from_slice(b".0");
    }
    s
}

/// `var_export` string: single-quoted, escaping only `'` and `\` (other bytes,
/// including newlines, are emitted verbatim). A NUL byte cannot appear in a
/// single-quoted literal, so PHP splits on it and joins the single-quoted
/// segments with a double-quoted `"\0"`, e.g. `'' . "\0" . 'Hello'`.
fn export_str(out: &mut Vec<u8>, s: &[u8]) {
    let mut first = true;
    for seg in s.split(|&b| b == 0) {
        if !first {
            out.extend_from_slice(b" . \"\\0\" . ");
        }
        first = false;
        out.push(b'\'');
        for &b in seg {
            if b == b'\'' || b == b'\\' {
                out.push(b'\\');
            }
            out.push(b);
        }
        out.push(b'\'');
    }
}

/// print_r($value, $return = false). Human-readable dump; with a truthy
/// `$return` the rendering is returned as a string instead of being printed.
fn print_r(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "print_r")?;
    let want_return = matches!(args.get(1), Some(r) if convert::is_true_silent(r));
    let mut buf = Vec::new();
    let mut seen = Vec::new();
    print_r_into(&mut buf, v, 0, ctx, &mut seen);
    if want_return {
        Ok(Zval::Str(PhpStr::new(buf)))
    } else {
        ctx.out.extend_from_slice(&buf);
        Ok(Zval::Bool(true))
    }
}

/// Recursive print_r renderer. `indent` is the leading-space count of this
/// value's `(` block (0 at the top level); nested arrays add 8.
fn print_r_into(out: &mut Vec<u8>, v: &Zval, indent: usize, ctx: &mut Ctx, seen: &mut Vec<usize>) {
    match v {
        Zval::Array(a) => {
            let ptr = Rc::as_ptr(a) as usize;
            out.extend_from_slice(b"Array\n");
            spaces(out, indent);
            out.extend_from_slice(b"(\n");
            if seen.contains(&ptr) {
                out.extend_from_slice(b" *RECURSION*");
                return;
            }
            seen.push(ptr);
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
                print_r_into(out, val, indent + 8, ctx, seen);
                out.push(b'\n');
            }
            seen.pop();
            spaces(out, indent);
            out.extend_from_slice(b")\n");
        }
        // print_r is reference-transparent: deref and recurse, no `&` marker.
        Zval::Ref(cell) => print_r_into(out, &cell.borrow(), indent, ctx, seen),
        // A closure prints as a `Closure Object` with the same pseudo-properties
        // var_dump uses (step 18-7).
        Zval::Closure(c) => {
            let props = closure_properties(c);
            out.extend_from_slice(b"Closure Object\n");
            spaces(out, indent);
            out.extend_from_slice(b"(\n");
            for (k, val) in &props {
                spaces(out, indent + 4);
                out.push(b'[');
                out.extend_from_slice(k);
                out.extend_from_slice(b"] => ");
                print_r_into(out, val, indent + 8, ctx, seen);
                out.push(b'\n');
            }
            spaces(out, indent);
            out.extend_from_slice(b")\n");
        }
        // A `Generator` prints with its `function` pseudo-property (step 39-7).
        Zval::Generator(g) => {
            let g = g.borrow();
            out.extend_from_slice(b"Generator Object\n");
            spaces(out, indent);
            out.extend_from_slice(b"(\n");
            spaces(out, indent + 4);
            out.extend_from_slice(b"[function] => ");
            out.extend_from_slice(&g.func_name);
            out.push(b'\n');
            spaces(out, indent);
            out.extend_from_slice(b")\n");
        }
        // A resource prints as "Resource id #N" (step 51, like echo).
        Zval::Resource(r) => {
            out.extend_from_slice(format!("Resource id #{}", r.borrow().id).as_bytes());
        }
        // A class instance (step 19-7): `C Object ( [p] => …, [p:protected] => …,
        // [p:C:private] => … )`, with a recursion guard.
        Zval::Object(o) => {
            let ptr = Rc::as_ptr(o) as usize;
            let obj = o.borrow();
            out.extend_from_slice(obj.class_name.as_bytes());
            // An enum case prints `C Enum` / `C Enum:int` / `C Enum:string`
            // instead of `C Object`; its properties render as usual (step 23,
            // D-23.5).
            if obj.info.is_enum_case {
                out.extend_from_slice(b" Enum");
                match obj.props.get(b"value") {
                    Some(Zval::Long(_)) => out.extend_from_slice(b":int"),
                    Some(Zval::Str(_)) => out.extend_from_slice(b":string"),
                    _ => {}
                }
                out.push(b'\n');
            } else {
                out.extend_from_slice(b" Object\n");
            }
            spaces(out, indent);
            out.extend_from_slice(b"(\n");
            if seen.contains(&ptr) {
                out.extend_from_slice(b" *RECURSION*");
                return;
            }
            seen.push(ptr);
            for (k, val) in obj.props.iter() {
                spaces(out, indent + 4);
                out.push(b'[');
                out.extend_from_slice(k);
                match obj.info.vis_of(k) {
                    PropVis::Public => {}
                    PropVis::Protected => out.extend_from_slice(b":protected"),
                    PropVis::Private(cls) => {
                        out.push(b':');
                        out.extend_from_slice(cls.as_bytes());
                        out.extend_from_slice(b":private");
                    }
                }
                out.extend_from_slice(b"] => ");
                print_r_into(out, val, indent + 8, ctx, seen);
                out.push(b'\n');
            }
            seen.pop();
            spaces(out, indent);
            out.extend_from_slice(b")\n");
        }
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
