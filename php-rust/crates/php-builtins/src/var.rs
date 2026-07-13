//! var_dump / var_export / print_r / gettype / is_* / *val type helpers.
//! Split from lib.rs (kept only module decls + the registry() dispatch table).

use super::*;

pub(crate) fn var_dump(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.is_empty() {
        return Err(PhpError::Error(
            "var_dump() expects at least 1 argument, 0 given".to_string(),
        ));
    }
    for v in args {
        let mut seen = Vec::new();
        dump(ctx.out, v, 0, &mut seen, ctx.debug_info);
    }
    Ok(Zval::Null)
}
/// ext/pdo's classes are C structs with no visible instance props: the prelude
/// keeps their state in private `__…` slots, which debug dumps must hide (the
/// oracle shows `object(PDO)#1 (0) {}`, and only `queryString` on a statement).
pub(crate) fn pdo_hidden_prop(key: &[u8]) -> bool {
    key.starts_with(b"\0PDO\0__")
        || key.starts_with(b"\0PDOStatement\0__")
        || key.starts_with(b"\0PDORow\0__")
}
/// Recursive var_dump formatter. `indent` is the leading-space count for this
/// value's own block; nested entries indent by a further 2. `seen` holds the
/// addresses of containers currently being dumped, so a value that refers back
/// into its own subtree prints `*RECURSION*` instead of looping (step 19-7).
pub(crate) fn dump(
    out: &mut Vec<u8>,
    v: &Zval,
    indent: usize,
    seen: &mut Vec<usize>,
    debug: &std::collections::HashMap<u32, Zval>,
) {
    match v {
        Zval::Undef | Zval::Null | Zval::ArgPlace(_) => out.extend_from_slice(b"NULL\n"),
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
                        dump(out, &cell.borrow(), indent + 2, seen, debug);
                    }
                    _ => dump(out, val, indent + 2, seen, debug),
                }
            }
            seen.pop();
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
        // A top-level reference is dereferenced transparently (the `&` marker
        // only applies to reference *elements* inside a container).
        Zval::Ref(cell) => dump(out, &cell.borrow(), indent, seen, debug),
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
                dump(out, val, indent + 2, seen, debug);
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
            // An object with a `__debugInfo()` method (PHP 8.4): the VM already
            // invoked it (keyed by object id) and var_dump renders the returned
            // array under the object header, not the raw slots. A still-lazy
            // object keeps its `lazy ghost `/`lazy proxy ` prefix — the call
            // initializes it only if the method body touched its state.
            if let Some(Zval::Array(dbg)) = debug.get(&obj.id) {
                if let Some(kind) = obj.lazy {
                    out.extend_from_slice(match kind {
                        php_types::LazyKind::Ghost => b"lazy ghost ".as_slice(),
                        php_types::LazyKind::Proxy => b"lazy proxy ".as_slice(),
                    });
                }
                out.extend_from_slice(b"object(");
                out.extend_from_slice(class_display_name(obj.class_name.as_bytes()));
                out.extend_from_slice(format!(")#{} ({}) {{\n", obj.id, dbg.len()).as_bytes());
                for (key, val) in dbg.iter() {
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
                    dump(out, val, indent + 2, seen, debug);
                }
                drop(obj);
                seen.pop();
                spaces(out, indent);
                out.extend_from_slice(b"}\n");
                return;
            }
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
            // Opaque handle classes: PHP's native DeflateContext/InflateContext
            // have NO properties — phpr's backing `__id` (into the host context
            // table) is internal and hidden, matching `object(X)#N (0) {}`.
            if matches!(obj.class_name.as_bytes(), b"DeflateContext" | b"InflateContext") {
                out.extend_from_slice(
                    format!("object({})#{} (0) {{\n", String::from_utf8_lossy(obj.class_name.as_bytes()), obj.id)
                        .as_bytes(),
                );
                spaces(out, indent);
                out.extend_from_slice(b"}\n");
                drop(obj);
                seen.pop();
                return;
            }
            // A WeakReference renders its (weak) referent under an "object" key —
            // the upgraded object, or NULL once it has been collected. The backing
            // `__h` property is the internal weak handle.
            if obj.class_name.as_bytes() == b"WeakReference" {
                // `__h` is private → stored under its mangled key.
                let inner = obj
                    .props
                    .get(php_types::mangle_prop_key(b"WeakReference", b"__h").as_slice())
                    .or_else(|| obj.props.get(b"__h"))
                    .cloned()
                    .unwrap_or(Zval::Null);
                out.extend_from_slice(
                    format!("object(WeakReference)#{} (1) {{\n", obj.id).as_bytes(),
                );
                spaces(out, indent + 2);
                out.extend_from_slice(b"[\"object\"]=>\n");
                spaces(out, indent + 2);
                dump(out, &inner, indent + 2, seen, debug); // WeakHandle arm: object or NULL
                drop(obj);
                seen.pop();
                spaces(out, indent);
                out.extend_from_slice(b"}\n");
                return;
            }
            // A ReflectionAttribute renders with only its public `name` — the
            // private handle props (`__class`/`__index`/`__prop`/`__func`/
            // `__method`) the reflection hosts use to materialise it lazily are
            // internal and hidden, matching PHP's native single-property dump.
            if obj.class_name.as_bytes() == b"ReflectionAttribute" {
                let name = obj.props.get(b"name").cloned().unwrap_or(Zval::Null);
                out.extend_from_slice(format!("object(ReflectionAttribute)#{} (1) {{\n", obj.id).as_bytes());
                spaces(out, indent + 2);
                out.extend_from_slice(b"[\"name\"]=>\n");
                spaces(out, indent + 2);
                drop(obj);
                dump(out, &name, indent + 2, seen, debug);
                seen.pop();
                spaces(out, indent);
                out.extend_from_slice(b"}\n");
                return;
            }
            // A WeakMap renders as its *live* key/value pairs, not its internal
            // storage property: `[i] => array(2){ ["key"]=>K, ["value"]=>V }`, in
            // insertion order, with collected keys pruned (mirrors PHP's native
            // handler). The backing `__entries` maps spl_object_id => [weak, value].
            if obj.class_name.as_bytes() == b"WeakMap" {
                let mut live: Vec<(Zval, Zval)> = Vec::new();
                // `__entries` is private → stored under its mangled key.
                if let Some(Zval::Array(a)) = obj
                    .props
                    .get(php_types::mangle_prop_key(b"WeakMap", b"__entries").as_slice())
                    .or_else(|| obj.props.get(b"__entries"))
                {
                    for (_, entry) in a.iter() {
                        if let Zval::Array(pair) = entry {
                            let mut it = pair.iter().map(|(_, v)| v.clone());
                            let h = it.next().unwrap_or(Zval::Null);
                            let value = it.next().unwrap_or(Zval::Null);
                            match h {
                                Zval::WeakHandle(w) => {
                                    if let Some(o) = w.upgrade() {
                                        live.push((Zval::Object(o), value));
                                    }
                                }
                                other => live.push((other, value)), // strong fallback
                            }
                        }
                    }
                }
                out.extend_from_slice(
                    format!("object(WeakMap)#{} ({}) {{\n", obj.id, live.len()).as_bytes(),
                );
                for (i, (key, value)) in live.iter().enumerate() {
                    spaces(out, indent + 2);
                    out.extend_from_slice(format!("[{i}]=>\n").as_bytes());
                    spaces(out, indent + 2);
                    out.extend_from_slice(b"array(2) {\n");
                    spaces(out, indent + 4);
                    out.extend_from_slice(b"[\"key\"]=>\n");
                    spaces(out, indent + 4);
                    dump(out, key, indent + 4, seen, debug);
                    spaces(out, indent + 4);
                    out.extend_from_slice(b"[\"value\"]=>\n");
                    spaces(out, indent + 4);
                    dump(out, value, indent + 4, seen, debug);
                    spaces(out, indent + 2);
                    out.extend_from_slice(b"}\n");
                }
                drop(obj);
                seen.pop();
                spaces(out, indent);
                out.extend_from_slice(b"}\n");
                return;
            }
            // An *initialized* lazy proxy renders as a single synthetic
            // `["instance"]` slot pointing at the real object it forwards to
            // (PHP 8.4) — its own property slots are irrelevant once forwarding.
            if matches!(obj.lazy, Some(php_types::LazyKind::Proxy)) {
                if let Some(inst) = &obj.proxy_instance {
                    out.extend_from_slice(b"lazy proxy object(");
                    out.extend_from_slice(class_display_name(obj.class_name.as_bytes()));
                    out.extend_from_slice(format!(")#{} (1) {{\n", obj.id).as_bytes());
                    spaces(out, indent + 2);
                    out.extend_from_slice(b"[\"instance\"]=>\n");
                    spaces(out, indent + 2);
                    let inst = (**inst).clone();
                    drop(obj);
                    dump(out, &inst, indent + 2, seen, debug);
                    seen.pop();
                    spaces(out, indent);
                    out.extend_from_slice(b"}\n");
                    return;
                }
            }
            // An *uninitialized* lazy object is prefixed `lazy ghost `/`lazy proxy `
            // (PHP 8.4); var_dump itself does not trigger initialization.
            if let Some(kind) = obj.lazy {
                out.extend_from_slice(match kind {
                    php_types::LazyKind::Ghost => b"lazy ghost ".as_slice(),
                    php_types::LazyKind::Proxy => b"lazy proxy ".as_slice(),
                });
            }
            out.extend_from_slice(b"object(");
            // An anonymous class's name is `class@anonymous\0…`; displays show only
            // the part before the NUL (`class@anonymous`), like PHP. A no-op for
            // ordinary class names.
            out.extend_from_slice(class_display_name(obj.class_name.as_bytes()));
            // The header count excludes uninitialized typed properties (PHP).
            let count = obj
                .props
                .iter()
                .filter(|(k, v)| !matches!(v, Zval::Undef) && !pdo_hidden_prop(k))
                .count();
            out.extend_from_slice(format!(")#{} ({}) {{\n", obj.id, count).as_bytes());
            for (k, val) in obj.props.iter() {
                if pdo_hidden_prop(k) {
                    continue;
                }
                let (disp, vis) = php_types::unmangle_prop_key(k, &obj.info);
                spaces(out, indent + 2);
                out.extend_from_slice(b"[\"");
                out.extend_from_slice(disp);
                match vis {
                    PropVis::Public => out.extend_from_slice(b"\"]=>\n"),
                    PropVis::Protected => out.extend_from_slice(b"\":protected]=>\n"),
                    PropVis::Private(cls) => {
                        out.extend_from_slice(b"\":\"");
                        out.extend_from_slice(cls.as_bytes());
                        out.extend_from_slice(b"\":private]=>\n");
                    }
                }
                spaces(out, indent + 2);
                // An uninitialized typed property renders as `uninitialized(type)`.
                if matches!(val, Zval::Undef) {
                    out.extend_from_slice(b"uninitialized(");
                    // Type displays are keyed by the *storage* key (mangled for a
                    // private); fall back to the display name for plain slots.
                    out.extend_from_slice(obj.info.type_of(k).or_else(|| obj.info.type_of(disp)).unwrap_or(b"mixed"));
                    out.extend_from_slice(b")\n");
                } else {
                    // A property slot holding a reference with a live alias is
                    // marked `&` (`$r = &$obj->p`), like the array-element arm;
                    // a sole-holder reference prints as a plain value.
                    match val {
                        Zval::Ref(cell) if std::rc::Rc::strong_count(cell) >= 2 => {
                            out.push(b'&');
                            dump(out, &cell.borrow(), indent + 2, seen, debug);
                        }
                        _ => dump(out, val, indent + 2, seen, debug),
                    }
                }
            }
            drop(obj);
            seen.pop();
            spaces(out, indent);
            out.extend_from_slice(b"}\n");
        }
        // A bare weak handle (only reached if one ever escapes the WeakReference/
        // WeakMap special-casing): the live object, or NULL once collected.
        Zval::WeakHandle(w) => match w.upgrade() {
            Some(o) => dump(out, &Zval::Object(o), indent, seen, debug),
            None => out.extend_from_slice(b"NULL\n"),
        },
    }
}
/// The `var_dump`/`print_r` pseudo-properties of a closure, in PHP's order
/// (step 18-7, D-18.9): `name`/`file`/`line` for an anonymous/arrow closure or a
/// single `function` for a first-class callable, then a `parameter` array iff
/// the closure has any parameters.
pub(crate) fn closure_properties(c: &Closure) -> Vec<(Vec<u8>, Zval)> {
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
pub(crate) fn spaces(out: &mut Vec<u8>, n: usize) {
    out.resize(out.len() + n, b' ');
}
/// `var_export($value, $return = false)` (step 47). Renders a value as a
/// PHP-parsable literal. With a truthy `$return` the rendering is returned as a
/// string instead of being printed. A direct port of PHP's `php_var_export_ex`.
pub(crate) fn var_export(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
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
pub(crate) fn export_into(out: &mut Vec<u8>, v: &Zval, level: usize, seen: &mut Vec<usize>, diags: &mut Diags) {
    match v {
        Zval::Undef | Zval::Null | Zval::ArgPlace(_) => out.extend_from_slice(b"NULL"),
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
            // An enum case exports as `\Enum::Case` (PHP 8.1), not `__set_state`.
            if obj.info.is_enum_case {
                out.push(b'\\');
                out.extend_from_slice(class_display_name(obj.class_name.as_bytes()));
                out.extend_from_slice(b"::");
                if let Some(Zval::Str(s)) = obj.props.get(b"name") {
                    out.extend_from_slice(s.as_bytes());
                }
                drop(obj);
                seen.pop();
                return;
            }
            // `stdClass` renders as a cast; any other class via `__set_state`.
            let is_std = obj.class_name.as_bytes() == b"stdClass";
            if is_std {
                out.extend_from_slice(b"(object) array(\n");
            } else {
                out.push(b'\\');
                out.extend_from_slice(class_display_name(obj.class_name.as_bytes()));
                out.extend_from_slice(b"::__set_state(array(\n");
            }
            // All properties are exported by value, with no visibility markers
            // (a private property is exported under its plain, unmangled name).
            for (k, val) in obj.props.iter() {
                let (disp, _) = php_types::unmangle_prop_key(k, &obj.info);
                spaces(out, level + 2);
                export_str(out, disp);
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
        Zval::Closure(_) | Zval::Generator(_) | Zval::Resource(_) | Zval::WeakHandle(_) => {
            out.extend_from_slice(b"NULL")
        }
    }
}
/// `var_export` float: shortest round-trip, but always a valid PHP float literal
/// — if the result has no `.`/`e`/`E` and is finite, append `.0` (`1.0`, `-0.0`).
pub(crate) fn export_float(d: f64) -> Vec<u8> {
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
pub(crate) fn export_str(out: &mut Vec<u8>, s: &[u8]) {
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
pub(crate) fn print_r(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
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
pub(crate) fn print_r_into(out: &mut Vec<u8>, v: &Zval, indent: usize, ctx: &mut Ctx, seen: &mut Vec<usize>) {
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
            out.extend_from_slice(class_display_name(obj.class_name.as_bytes()));
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
                if pdo_hidden_prop(k) {
                    continue;
                }
                let (disp, vis) = php_types::unmangle_prop_key(k, &obj.info);
                spaces(out, indent + 4);
                out.push(b'[');
                out.extend_from_slice(disp);
                match vis {
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
pub(crate) fn strlen(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "strlen")?;
    if matches!(v, Zval::Array(_)) {
        return Err(PhpError::TypeError(
            "strlen(): Argument #1 ($string) must be of type string, array given".to_string(),
        ));
    }
    let s = convert::to_zstr(v, ctx.diags);
    Ok(Zval::Long(s.len() as i64))
}
pub(crate) fn gettype(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "gettype")?;
    Ok(Zval::Str(PhpStr::from_str(v.gettype())))
}
pub(crate) fn is_int(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_int")?, Zval::Long(_))))
}
pub(crate) fn is_float(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_float")?, Zval::Double(_))))
}
pub(crate) fn is_string(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_string")?, Zval::Str(_))))
}
pub(crate) fn is_bool(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_bool")?, Zval::Bool(_))))
}
pub(crate) fn is_null(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(
        arg1(args, "is_null")?,
        Zval::Null | Zval::Undef
    )))
}
pub(crate) fn is_array(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(arg1(args, "is_array")?, Zval::Array(_))))
}
/// The displayable class name: an anonymous class is stored as `class@anonymous\0…`
/// and shown (in `var_dump`/`print_r`/`var_export`) only up to the NUL, matching
/// PHP. A no-op for ordinary class names (which contain no NUL).
pub(crate) fn class_display_name(name: &[u8]) -> &[u8] {
    match name.iter().position(|&b| b == 0) {
        Some(i) => &name[..i],
        None => name,
    }
}
pub(crate) fn is_object(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    // Closures and Generators are objects in PHP (oracle-confirmed).
    Ok(Zval::Bool(matches!(
        arg1(args, "is_object")?,
        Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_)
    )))
}
pub(crate) fn is_resource(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    // A *closed* resource (post-`fclose`) is no longer a resource (oracle-confirmed).
    Ok(Zval::Bool(matches!(
        arg1(args, "is_resource")?,
        Zval::Resource(r) if r.borrow().is_open()
    )))
}
pub(crate) fn is_scalar(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(matches!(
        arg1(args, "is_scalar")?,
        Zval::Long(_) | Zval::Double(_) | Zval::Str(_) | Zval::Bool(_)
    )))
}
pub(crate) fn is_numeric(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "is_numeric")?;
    let numeric = match v {
        Zval::Long(_) | Zval::Double(_) => true,
        // A string is numeric iff it parses fully (no trailing non-numeric bytes).
        Zval::Str(s) => numstr::parse_numeric_ex(s.as_bytes(), true).is_some_and(|i| !i.trailing),
        _ => false,
    };
    Ok(Zval::Bool(numeric))
}
pub(crate) fn intval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(convert::to_long_cast(arg1(args, "intval")?, ctx.diags)))
}
pub(crate) fn floatval(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Double(convert::to_double(arg1(args, "floatval")?)))
}
pub(crate) fn strval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = arg1(args, "strval")?;
    // Honor the VM-precomputed `__toString` of a Stringable object argument
    // (Ctx::stringify) — the pure funnel below cannot invoke user methods.
    if let Zval::Object(o) = &v.deref_clone() {
        if let Some(s) = ctx.stringify.get(&o.borrow().id) {
            return Ok(Zval::Str(std::rc::Rc::clone(s)));
        }
    }
    Ok(Zval::Str(convert::to_zstr_cast(v, ctx.diags)))
}
pub(crate) fn boolval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(convert::to_bool(arg1(args, "boolval")?, ctx.diags)))
}
/// `filter_var($value, $filter = FILTER_DEFAULT, $options = 0)` — the validate
/// filters Composer and its dependencies use. The oracle build lacks ext/filter,
/// so this is implemented to the documented PHP semantics rather than diffed.
/// `$options` accepts the flags int or an `array{flags?: int}`; only the
/// `FILTER_NULL_ON_FAILURE` flag affects a validation miss here.
pub(crate) fn filter_var(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const FILTER_DEFAULT: i64 = 516;
    const VALIDATE_INT: i64 = 257;
    const VALIDATE_BOOL: i64 = 258;
    const VALIDATE_FLOAT: i64 = 259;
    const NULL_ON_FAILURE: i64 = 134_217_728;
    let value = arg1(args, "filter_var")?;
    let filter = args.get(1).map_or(FILTER_DEFAULT, |f| convert::to_long_cast(f, ctx.diags));
    let flags = match args.get(2) {
        Some(Zval::Array(a)) => a
            .get(&Key::from_bytes(b"flags"))
            .map_or(0, |v| convert::to_long_cast(v, ctx.diags)),
        Some(other) => convert::to_long_cast(other, ctx.diags),
        None => 0,
    };
    // The `options` sub-array: min_range/max_range/default for VALIDATE_INT
    // (and `default` on any validator's miss).
    let opt = |name: &[u8]| -> Option<Zval> {
        match args.get(2) {
            Some(Zval::Array(a)) => match a.get(&Key::from_bytes(b"options")) {
                Some(o) => match o.deref_clone() {
                    Zval::Array(opts) => opts.get(&Key::from_bytes(name)).map(|v| v.deref_clone()),
                    _ => None,
                },
                None => None,
            },
            _ => None,
        }
    };
    let default_or_miss = || match opt(b"default") {
        Some(d) => d,
        None => {
            if flags & NULL_ON_FAILURE != 0 {
                Zval::Null
            } else {
                Zval::Bool(false)
            }
        }
    };
    let miss = || if flags & NULL_ON_FAILURE != 0 { Zval::Null } else { Zval::Bool(false) };
    let s = convert::to_zstr_cast(value, ctx.diags);
    let text = String::from_utf8_lossy(s.as_bytes());
    let trimmed = text.trim();
    match filter {
        VALIDATE_BOOL => {
            // Recognised true/false words (case-insensitive, trimmed); anything else
            // is the validation miss (false, or null under FILTER_NULL_ON_FAILURE).
            match trimmed.to_ascii_lowercase().as_str() {
                "1" | "true" | "on" | "yes" => Ok(Zval::Bool(true)),
                "0" | "false" | "off" | "no" | "" => Ok(Zval::Bool(false)),
                _ => Ok(default_or_miss()),
            }
        }
        VALIDATE_INT => {
            const ALLOW_OCTAL: i64 = 1;
            const ALLOW_HEX: i64 = 2;
            // Decimal, or (behind their flags) 0x-hex and 0-octal literals.
            let parsed: Option<i64> = if flags & ALLOW_HEX != 0
                && (trimmed.starts_with("0x") || trimmed.starts_with("0X"))
            {
                i64::from_str_radix(&trimmed[2..], 16).ok()
            } else if flags & ALLOW_OCTAL != 0 && trimmed.len() > 1 && trimmed.starts_with('0') {
                i64::from_str_radix(&trimmed[1..], 8).ok()
            } else {
                trimmed.parse::<i64>().ok()
            };
            // min_range / max_range bound the accepted value (PHP's
            // php_filter_int): out of bounds is the validation miss.
            let min = opt(b"min_range").map(|v| convert::to_long_cast(&v, ctx.diags));
            let max = opt(b"max_range").map(|v| convert::to_long_cast(&v, ctx.diags));
            match parsed {
                Some(n) if min.is_none_or(|m| n >= m) && max.is_none_or(|m| n <= m) => {
                    Ok(Zval::Long(n))
                }
                _ => Ok(default_or_miss()),
            }
        }
        VALIDATE_FLOAT => match trimmed.parse::<f64>() {
            Ok(f) => Ok(Zval::Double(f)),
            Err(_) => Ok(miss()),
        },
        // FILTER_VALIDATE_URL: port of php_filter_url (logical_filters.c) over
        // the same php_url_parse the parse_url builtin uses. A URL must parse,
        // carry a scheme, and (unless mailto/news/file) a host; an http(s)
        // host is further restricted to alnum/-/./_ (monolog's SlackRecord
        // relies on 'ghost' NOT validating so it emits icon_emoji).
        273 => {
            let bytes = s.as_bytes();
            let ok = match crate::url::php_url_parse(bytes) {
                None => false,
                Some(u) => {
                    let scheme_ok = u.scheme.is_some();
                    let sch = u.scheme.as_deref().unwrap_or(b"");
                    let hostless_ok = sch.eq_ignore_ascii_case(b"mailto")
                        || sch.eq_ignore_ascii_case(b"news")
                        || sch.eq_ignore_ascii_case(b"file");
                    let host_ok = match &u.host {
                        Some(h) if !h.is_empty() => {
                            if sch.eq_ignore_ascii_case(b"http") || sch.eq_ignore_ascii_case(b"https") {
                                h.iter().all(|&c| {
                                    c.is_ascii_alphanumeric()
                                        || c == b'-'
                                        || c == b'.'
                                        || c == b'_'
                                        || c == b'['
                                        || c == b']'
                                        || c == b':'
                                })
                            } else {
                                true
                            }
                        }
                        _ => hostless_ok,
                    };
                    scheme_ok && host_ok
                }
            };
            if ok { Ok(Zval::Str(s)) } else { Ok(miss()) }
        }
        // FILTER_VALIDATE_IP: faithful port of php_filter_validate_ip
        // (ext/filter/logical_filters.c) — strict v4 (no leading zeros, no
        // whitespace), the hand-rolled v6 grammar (`::` compression, bundled
        // v4 tail), and the RFC 6890 special-purpose tables behind the
        // IPV4/IPV6/NO_PRIV_RANGE/NO_RES_RANGE/GLOBAL_RANGE flags.
        275 => {
            if validate_ip(s.as_bytes(), flags) { Ok(Zval::Str(s)) } else { Ok(miss()) }
        }
        // FILTER_VALIDATE_MAC (same file): 17-byte `xx:xx:…`/`xx-xx-…` or
        // 14-byte EUI-64 `xxxx.xxxx.xxxx`. The `separator` option must be one
        // character; a mismatch is a plain validation miss.
        276 => {
            let sep = match args.get(2) {
                Some(Zval::Array(a)) => match a.get(&Key::from_bytes(b"options")) {
                    Some(o) => match o.deref_clone() {
                        Zval::Array(opts) => match opts.get(&Key::from_bytes(b"separator")) {
                            Some(v) => {
                                let sv = convert::to_zstr_cast(&v.deref_clone(), ctx.diags);
                                if sv.as_bytes().len() != 1 {
                                    return Err(PhpError::ValueError(
                                        "filter_var(): \"separator\" option must be one character long"
                                            .to_string(),
                                    ));
                                }
                                Some(sv.as_bytes()[0])
                            }
                            None => None,
                        },
                        _ => None,
                    },
                    None => None,
                },
                _ => None,
            };
            if validate_mac(s.as_bytes(), sep) { Ok(Zval::Str(s)) } else { Ok(miss()) }
        }
        // FILTER_SANITIZE_NUMBER_INT: strip everything but digits, `+`, `-`
        // (php_filter_number_int's [^0-9+-] deny list).
        519 => Ok(Zval::Str(PhpStr::new(
            s.as_bytes()
                .iter()
                .copied()
                .filter(|b| b.is_ascii_digit() || *b == b'+' || *b == b'-')
                .collect::<Vec<u8>>(),
        ))),
        // FILTER_DEFAULT / FILTER_UNSAFE_RAW and the unimplemented validators return
        // the value as a string (no sanitisation), the documented default behaviour.
        _ => Ok(Zval::Str(s)),
    }
}

/// `_php_filter_validate_ipv4`: strict dotted-quad — exactly 4 decimal octets
/// 0..=255, no leading zeros (octal ambiguity), no surrounding whitespace.
fn parse_ipv4_strict(b: &[u8]) -> Option<[i32; 4]> {
    let mut ip = [0i32; 4];
    let mut i = 0usize;
    let mut n = 0usize;
    while i < b.len() {
        if !b[i].is_ascii_digit() {
            return None;
        }
        let leading_zero = b[i] == b'0';
        let mut m = 1;
        let mut num = (b[i] - b'0') as i32;
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            num = num * 10 + (b[i] - b'0') as i32;
            m += 1;
            if num > 255 || m > 3 {
                return None;
            }
            i += 1;
        }
        if leading_zero && (num != 0 || m > 1) {
            return None;
        }
        ip[n] = num;
        n += 1;
        if n == 4 {
            return if i == b.len() { Some(ip) } else { None };
        }
        if i >= b.len() || b[i] != b'.' {
            return None;
        }
        i += 1;
    }
    None
}

/// `_php_filter_validate_ipv6`: PHP's hand-rolled v6 grammar. Returns the 8
/// 16-bit blocks (after `::` expansion / bundled-v4 fix-up) on success.
fn parse_ipv6_strict(bytes: &[u8]) -> Option<[i32; 8]> {
    if !bytes.contains(&b':') {
        return None;
    }
    let mut ip = [0i32; 8];
    let mut len = bytes.len();
    let mut ip4elm: Option<[i32; 4]> = None;
    // Bundled IPv4 tail (`::ffff:1.2.3.4`): backtrack from the first '.' to the
    // preceding ':', validate as v4, and shorten the v6 part.
    if let Some(dot) = bytes.iter().position(|&c| c == b'.') {
        let mut v4start = dot;
        while v4start > 0 && bytes[v4start - 1] != b':' {
            v4start -= 1;
        }
        let v4 = parse_ipv4_strict(&bytes[v4start..])?;
        ip4elm = Some(v4);
        len = v4start;
        if len < 2 {
            return None;
        }
        if bytes[v4start - 2] != b':' {
            // don't include the ':' before the v4 unless it's a '::'
            len -= 1;
        }
    }
    let s = &bytes[..len];
    let mut blocks: i32 = if ip4elm.is_some() { 2 } else { 0 };
    let mut compressed_pos: i32 = -1;
    let mut i = 0usize;
    let mut goto_fixup = false;
    while i < s.len() {
        if s[i] == b':' {
            i += 1;
            if i >= s.len() {
                return None; // cannot end in ':' without previous ':'
            }
            if s[i] == b':' {
                if compressed_pos >= 0 {
                    return None;
                }
                if (blocks as usize) < 8 {
                    ip[blocks as usize] = -1;
                }
                compressed_pos = blocks;
                blocks += 1;
                i += 1;
                if i == s.len() {
                    if blocks > 8 {
                        return None;
                    }
                    goto_fixup = true;
                    break;
                }
            } else if i == 1 {
                // leading ':' without another ':' following
                return None;
            }
        }
        let mut num: i32 = 0;
        let mut n = 0usize;
        while i < s.len() {
            let d = match s[i] {
                c @ b'0'..=b'9' => (c - b'0') as i32,
                c @ b'a'..=b'f' => (c - b'a') as i32 + 10,
                c @ b'A'..=b'F' => (c - b'A') as i32 + 10,
                _ => break,
            };
            num = 16 * num + d;
            n += 1;
            i += 1;
        }
        if (blocks as usize) < 8 {
            ip[blocks as usize] = num;
        }
        if n < 1 || n > 4 {
            return None;
        }
        blocks += 1;
        if blocks > 8 {
            return None;
        }
    }
    let _ = goto_fixup;
    if let Some(v4) = ip4elm {
        ip = [0, 0, 0, 0, 0, 0xffff, 256 * v4[0] + v4[1], 256 * v4[2] + v4[3]];
    } else if compressed_pos >= 0 && blocks <= 8 {
        let offset = (8 - blocks) as usize;
        let cp = compressed_pos as usize;
        let mut j = 7usize;
        while j > cp + offset {
            ip[j] = ip[j - offset];
            j -= 1;
        }
        let mut j = cp + offset;
        loop {
            ip[j] = 0;
            if j == cp {
                break;
            }
            j -= 1;
        }
    }
    if (compressed_pos >= 0 && blocks <= 8) || blocks == 8 {
        Some(ip)
    } else {
        None
    }
}

/// `ipv4_get_status_flags` (RFC 6890 table): `Some((global, reserved, private))`
/// when the address falls in a special-purpose block, `None` otherwise.
fn ipv4_status(ip: &[i32; 4]) -> Option<(bool, bool, bool)> {
    let (g, r, p) = match ip {
        [0, ..] => (false, true, false),                            // this network
        [10, ..] => (false, false, true),                           // private
        [100, b, ..] if (64..=127).contains(b) => (false, false, false), // shared space
        [127, ..] => (false, true, false),                          // loopback
        [169, 254, ..] => (false, true, false),                     // link local
        [172, b, ..] if (16..=31).contains(b) => (false, false, true), // private
        [192, 0, 0, _] => (false, false, false),                    // IETF assignments / DS-Lite
        [192, 0, 2, _] => (false, false, false),                    // documentation
        [192, 88, 99, _] => (true, false, false),                   // 6to4 relay
        [192, 168, ..] => (false, false, true),                     // private
        [198, b, ..] if (18..=19).contains(b) => (false, false, false), // benchmarking
        [198, 51, 100, _] => (false, false, false),                 // documentation
        [203, 0, 113, _] => (false, false, false),                  // documentation
        [a, ..] if *a >= 240 => (false, true, false),               // reserved (incl. broadcast)
        _ => return None,
    };
    Some((g, r, p))
}

/// `ipv6_get_status_flags` (RFC 6890 table).
fn ipv6_status(ip: &[i32; 8]) -> Option<(bool, bool, bool)> {
    let (g, r, p) = match ip {
        [0, 0, 0, 0, 0, 0, 0, 0] => (false, true, false),           // unspecified
        [0, 0, 0, 0, 0, 0, 0, 1] => (false, true, false),           // loopback
        [0x0064, 0xff9b, ..] => (true, false, false),               // v4-v6 translation
        [0, 0, 0, 0, 0, 0xffff, ..] => (false, true, false),        // v4-mapped
        [0x0100, 0, 0, 0, ..] => (false, false, false),             // discard-only
        [0x2001, 0x0000, ..] => (false, false, false),              // TEREDO
        [0x2001, b, ..] if *b <= 0x01ff => (false, false, false),   // IETF assignments
        [0x2001, 0x0002, 0, ..] => (false, false, false),           // benchmarking
        [0x2001, 0x0db8, ..] => (false, false, false),              // documentation
        [0x2001, b, ..] if (0x0010..=0x001f).contains(b) => (false, false, false), // ORCHID
        [0x2002, ..] => (false, false, false),                      // 6to4
        [a, ..] if (0xfc00..=0xfdff).contains(a) => (false, false, true), // unique-local
        [a, ..] if (0xfe80..=0xfebf).contains(a) => (false, true, false), // link-scoped
        _ => return None,
    };
    Some((g, r, p))
}

/// `php_filter_validate_ip`: format picked by the first ':' (v6) else '.' (v4);
/// no trimming. The RFC 6890 range flags only apply to special-purpose blocks.
fn validate_ip(b: &[u8], flags: i64) -> bool {
    const FLAG_IPV4: i64 = 1_048_576;
    const FLAG_IPV6: i64 = 2_097_152;
    const FLAG_NO_RES: i64 = 4_194_304;
    const FLAG_NO_PRIV: i64 = 8_388_608;
    const FLAG_GLOBAL: i64 = 536_870_912;
    let v6 = b.contains(&b':');
    if !v6 && !b.contains(&b'.') {
        return false;
    }
    let both = (flags & FLAG_IPV4 != 0) == (flags & FLAG_IPV6 != 0);
    if !both {
        if flags & FLAG_IPV4 != 0 && v6 {
            return false;
        }
        if flags & FLAG_IPV6 != 0 && !v6 {
            return false;
        }
    }
    let status = if v6 {
        match parse_ipv6_strict(b) {
            Some(ip) => ipv6_status(&ip),
            None => return false,
        }
    } else {
        match parse_ipv4_strict(b) {
            Some(ip) => ipv4_status(&ip),
            None => return false,
        }
    };
    let Some((global, reserved, private)) = status else {
        return true; // no special block: every range flag passes
    };
    if flags & FLAG_GLOBAL != 0 && !global {
        return false;
    }
    if flags & FLAG_NO_PRIV != 0 && private {
        return false;
    }
    if flags & FLAG_NO_RES != 0 && reserved {
        return false;
    }
    true
}

/// `php_filter_validate_mac`: `xx-xx-xx-xx-xx-xx`, `xx:xx:…` (17 bytes) or
/// EUI-64 `xxxx.xxxx.xxxx` (14 bytes); `sep`, when given, must match.
fn validate_mac(b: &[u8], sep: Option<u8>) -> bool {
    let (tokens, length, separator) = match (b.len(), b.get(2)) {
        (14, _) => (3usize, 4usize, b'.'),
        (17, Some(b'-')) => (6, 2, b'-'),
        (17, Some(b':')) => (6, 2, b':'),
        _ => return false,
    };
    if let Some(s) = sep {
        if s != separator {
            return false;
        }
    }
    for i in 0..tokens {
        let off = i * (length + 1);
        if i < tokens - 1 && b[off + length] != separator {
            return false;
        }
        if !b[off..off + length].iter().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    true
}
/// `filter_var_array(array $array, array|int $options = FILTER_DEFAULT, bool $add_empty = true)`
/// — apply filters to an array. A single filter int is applied to every element
/// (result keyed like `$array`); an array `$options` is a per-key spec (result
/// keyed by the spec's keys — a key absent from `$array` becomes `null` when
/// `$add_empty`). Reuses [`filter_var`] per element.
pub(crate) fn filter_var_array(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const FILTER_DEFAULT: i64 = 516;
    let arr = match args.first() {
        Some(Zval::Array(a)) => Rc::clone(a),
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "filter_var_array(): Argument #1 ($array) must be of type array, {} given",
                other.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "filter_var_array() expects at least 1 argument, 0 given".to_string(),
            ))
        }
    };
    let add_empty = args.get(2).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(true);
    let mut out = PhpArray::new();
    match args.get(1) {
        Some(Zval::Array(spec)) => {
            // Per-key: iterate the spec's keys, filtering the matching input.
            let entries: Vec<(Key, Zval)> = spec.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            for (k, filterspec) in entries {
                match arr.get(&k) {
                    Some(v) => {
                        let filtered = filter_one(&v.deref_clone(), &filterspec, ctx)?;
                        out.insert(k, filtered);
                    }
                    None if add_empty => out.insert(k, Zval::Null),
                    None => {}
                }
            }
        }
        other => {
            // A single filter int (or the default) applied to every element.
            let filter_int = other.map_or(FILTER_DEFAULT, |v| convert::to_long_cast(v, ctx.diags));
            let entries: Vec<(Key, Zval)> = arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            for (k, v) in entries {
                let filtered = filter_var(&[v.deref_clone(), Zval::Long(filter_int)], ctx)?;
                out.insert(k, filtered);
            }
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Filter one value against a `filter_var_array` per-key spec: an int filter id,
/// or an `array{filter?, flags?, options?}` passed through to [`filter_var`] as
/// its `$options` (so `flags` are honoured).
fn filter_one(value: &Zval, spec: &Zval, ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const FILTER_DEFAULT: i64 = 516;
    match spec {
        Zval::Array(a) => {
            let filter = a
                .get(&Key::from_bytes(b"filter"))
                .map_or(FILTER_DEFAULT, |v| convert::to_long_cast(v, ctx.diags));
            filter_var(&[value.clone(), Zval::Long(filter), spec.clone()], ctx)
        }
        other => {
            let filter = convert::to_long_cast(other, ctx.diags);
            filter_var(&[value.clone(), Zval::Long(filter)], ctx)
        }
    }
}

/// `localeconv(): array` — numeric/monetary formatting for the current locale.
/// phpr runs in the default "C" locale (no meaningful `setlocale`), so this is
/// the fixed C-locale snapshot the oracle returns there: "." decimal point, no
/// grouping, and the `CHAR_MAX` (127) sentinel for the unset monetary fields.
pub(crate) fn localeconv(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut a = PhpArray::new();
    let str_field = |a: &mut PhpArray, k: &[u8], v: &[u8]| {
        a.insert(Key::from_bytes(k), Zval::Str(PhpStr::new(v.to_vec())));
    };
    let int_field = |a: &mut PhpArray, k: &[u8], v: i64| {
        a.insert(Key::from_bytes(k), Zval::Long(v));
    };
    str_field(&mut a, b"decimal_point", b".");
    str_field(&mut a, b"thousands_sep", b"");
    str_field(&mut a, b"int_curr_symbol", b"");
    str_field(&mut a, b"currency_symbol", b"");
    str_field(&mut a, b"mon_decimal_point", b"");
    str_field(&mut a, b"mon_thousands_sep", b"");
    str_field(&mut a, b"positive_sign", b"");
    str_field(&mut a, b"negative_sign", b"");
    int_field(&mut a, b"int_frac_digits", 127);
    int_field(&mut a, b"frac_digits", 127);
    int_field(&mut a, b"p_cs_precedes", 127);
    int_field(&mut a, b"p_sep_by_space", 127);
    int_field(&mut a, b"n_cs_precedes", 127);
    int_field(&mut a, b"n_sep_by_space", 127);
    int_field(&mut a, b"p_sign_posn", 127);
    int_field(&mut a, b"n_sign_posn", 127);
    a.insert(Key::from_bytes(b"grouping"), Zval::Array(Rc::new(PhpArray::new())));
    a.insert(Key::from_bytes(b"mon_grouping"), Zval::Array(Rc::new(PhpArray::new())));
    Ok(Zval::Array(Rc::new(a)))
}
/// extension_loaded($name): whether a PHP extension is available (case-insensitive).
pub(crate) fn extension_loaded(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let name = convert::to_zstr(arg1(args, "extension_loaded")?, ctx.diags);
    let lc = name.as_bytes().to_ascii_lowercase();
    Ok(Zval::Bool(LOADED_EXTENSIONS.contains(&lc.as_slice())))
}
/// phpversion($extension = null): with no argument, the PHP version ("8.5.7");
/// with an extension name, that extension's version when loaded — phpr reports
/// the PHP version for its bundled extensions, matching the oracle (e.g.
/// `phpversion("openssl") === "8.5.7"`) — or `false` when the extension is absent.
pub(crate) fn phpversion(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match args.first() {
        None | Some(Zval::Null) => Ok(Zval::Str(PhpStr::new(b"8.5.7".to_vec()))),
        Some(v) => {
            let lc = convert::to_zstr(v, ctx.diags).as_bytes().to_ascii_lowercase();
            if LOADED_EXTENSIONS.contains(&lc.as_slice()) {
                Ok(Zval::Str(PhpStr::new(b"8.5.7".to_vec())))
            } else {
                Ok(Zval::Bool(false))
            }
        }
    }
}
/// get_loaded_extensions($zend_extensions = false): the extensions phpr models,
/// in the oracle's proper casing ("Core"/"SPL" capitalised). phpr has no Zend
/// extensions, so the flag yields the same list. Mirrors [`LOADED_EXTENSIONS`].
pub(crate) fn get_loaded_extensions(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut arr = PhpArray::new();
    for n in LOADED_EXTENSIONS_CASED {
        let _ = arr.append(Zval::Str(PhpStr::new(n.to_vec())));
    }
    Ok(Zval::Array(Rc::new(arr)))
}
// `inet_pton` moved to the `net` module (net.rs) — the lenient IPv4 parser there
// accepts leading zeros (`192.168.01.1`) as PHP does, which `std::net::IpAddr`
// rejects.
/// setlocale($category, ...$locales): we do not model real C locales — accept the
/// first non-empty candidate locale (a string arg, or an element of an array arg)
/// and echo it back; an empty / "0" locale (a query) yields the default "C".
pub(crate) fn setlocale(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let accept = |z: &Zval, ctx: &mut Ctx| -> Option<Zval> {
        let s = convert::to_zstr(z, ctx.diags);
        let b = s.as_bytes();
        if b.is_empty() || b == b"0" {
            None
        } else {
            Some(Zval::Str(PhpStr::new(b.to_vec())))
        }
    };
    for a in args.iter().skip(1) {
        match a.deref_clone() {
            Zval::Array(arr) => {
                for (_, v) in arr.iter() {
                    if let Some(z) = accept(&v, ctx) {
                        return Ok(z);
                    }
                }
            }
            v => {
                if let Some(z) = accept(&v, ctx) {
                    return Ok(z);
                }
            }
        }
    }
    Ok(Zval::str_from("C"))
}
