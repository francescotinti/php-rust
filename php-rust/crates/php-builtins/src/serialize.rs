//! `serialize()` (step 50a) — render a value to PHP's serialization byte format.
//!
//! Grammar (the subset Tier 1 produces):
//!   N;                       null
//!   b:0;  b:1;               bool
//!   i:<int>;                 integer
//!   d:<shortest>;            float (serialize_precision = -1: shortest round-trip)
//!   s:<bytelen>:"<bytes>";   string (byte length, raw bytes)
//!   a:<n>:{<k><v>...};       array (n key/value pairs, in order)
//!   O:<len>:"<class>":<n>:{<propname-str><v>...};   object
//!
//! `unserialize()` is the evaluator-dispatched inverse (step 50b): it must
//! instantiate objects, which a pure builtin cannot reach.

use php_runtime::Ctx;
use php_types::{dtoa, Key, PhpError, PhpStr, Zval};

pub fn serialize(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = args
        .first()
        .ok_or_else(|| PhpError::Error("serialize() expects exactly 1 argument, 0 given".into()))?;
    let mut out = Vec::new();
    let mut sc = SerCtx::default();
    ser_into(&mut out, v, &mut sc)?;
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Zend's `var_hash`: every serialized value slot gets a pre-order number
/// (starting at 1; array keys and property names don't count). A repeated
/// object emits `r:<first>;` (and still consumes a number); a repeated
/// reference cell emits `R:<first>;` (consuming none). A first-seen reference
/// shares its single number with its pointee.
#[derive(Default)]
struct SerCtx {
    count: i64,
    /// Object identity (`Rc` address) → the slot number it first appeared at.
    objs: std::collections::HashMap<usize, i64>,
    /// Reference-cell identity (`Rc` address) → its slot number.
    refs: std::collections::HashMap<usize, i64>,
}

/// Append `s:<bytelen>:"<bytes>";` for a raw byte string (used for both string
/// values and string array keys / property names).
fn ser_str(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(b"s:");
    out.extend_from_slice(bytes.len().to_string().as_bytes());
    out.extend_from_slice(b":\"");
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\";");
}

/// Serialize one value slot: assign its number, resolve `r:`/`R:` repeats,
/// then render the concrete form via [`ser_body`].
fn ser_into(out: &mut Vec<u8>, v: &Zval, sc: &mut SerCtx) -> Result<(), PhpError> {
    match v {
        Zval::Ref(cell) => {
            let key = std::rc::Rc::as_ptr(cell) as usize;
            if let Some(&n) = sc.refs.get(&key) {
                out.extend_from_slice(b"R:");
                out.extend_from_slice(n.to_string().as_bytes());
                out.push(b';');
                return Ok(());
            }
            let inner = cell.borrow();
            // A fresh cell around an already-seen object still aliases it.
            if let Zval::Object(o) = &*inner {
                let okey = std::rc::Rc::as_ptr(o) as usize;
                if let Some(&n) = sc.objs.get(&okey) {
                    sc.refs.insert(key, n);
                    out.extend_from_slice(b"R:");
                    out.extend_from_slice(n.to_string().as_bytes());
                    out.push(b';');
                    return Ok(());
                }
            }
            // First occurrence: the cell and its pointee share one number.
            sc.count += 1;
            let n = sc.count;
            sc.refs.insert(key, n);
            if let Zval::Object(o) = &*inner {
                sc.objs.insert(std::rc::Rc::as_ptr(o) as usize, n);
            }
            ser_body(out, &inner, sc)
        }
        Zval::Object(o) => {
            let key = std::rc::Rc::as_ptr(o) as usize;
            if let Some(&n) = sc.objs.get(&key) {
                // A repeated object emits `r:` and still consumes a number.
                sc.count += 1;
                out.extend_from_slice(b"r:");
                out.extend_from_slice(n.to_string().as_bytes());
                out.push(b';');
                return Ok(());
            }
            sc.count += 1;
            sc.objs.insert(key, sc.count);
            ser_body(out, v, sc)
        }
        _ => {
            sc.count += 1;
            ser_body(out, v, sc)
        }
    }
}

fn ser_body(out: &mut Vec<u8>, v: &Zval, sc: &mut SerCtx) -> Result<(), PhpError> {
    match v {
        Zval::Undef | Zval::Null => out.extend_from_slice(b"N;"),
        Zval::Bool(b) => {
            out.extend_from_slice(b"b:");
            out.push(if *b { b'1' } else { b'0' });
            out.push(b';');
        }
        Zval::Long(n) => {
            out.extend_from_slice(b"i:");
            out.extend_from_slice(n.to_string().as_bytes());
            out.push(b';');
        }
        // serialize_precision = -1: the shortest representation that round-trips.
        Zval::Double(d) => {
            out.extend_from_slice(b"d:");
            out.extend_from_slice(&dtoa::double_to_shortest(*d));
            out.push(b';');
        }
        Zval::Str(s) => ser_str(out, s.as_bytes()),
        Zval::Array(a) => {
            let n = a.iter().count();
            out.extend_from_slice(b"a:");
            out.extend_from_slice(n.to_string().as_bytes());
            out.extend_from_slice(b":{");
            for (k, val) in a.iter() {
                match k {
                    Key::Int(i) => {
                        out.extend_from_slice(b"i:");
                        out.extend_from_slice(i.to_string().as_bytes());
                        out.push(b';');
                    }
                    Key::Str(s) => ser_str(out, s.as_bytes()),
                }
                ser_into(out, val, sc)?;
            }
            out.push(b'}');
        }
        Zval::Object(o) => {
            let obj = o.borrow();
            let cname = obj.class_name.as_bytes();
            // A record staged by the VM for a legacy `Serializable` object
            // (its `serialize()` already produced the raw payload): emit
            // `C:<len>:"<class>":<len>:{<payload>}`; a missing payload (PHP:
            // `serialize()` returned NULL) serializes the object as plain `N;`.
            if cname == b"\0__phpr_cformat" {
                match (obj.props.get(b"class"), obj.props.get(b"payload")) {
                    (Some(Zval::Str(c)), Some(Zval::Str(p))) => {
                        out.extend_from_slice(b"C:");
                        out.extend_from_slice(c.as_bytes().len().to_string().as_bytes());
                        out.extend_from_slice(b":\"");
                        out.extend_from_slice(c.as_bytes());
                        out.extend_from_slice(b"\":");
                        out.extend_from_slice(p.as_bytes().len().to_string().as_bytes());
                        out.extend_from_slice(b":{");
                        out.extend_from_slice(p.as_bytes());
                        out.push(b'}');
                    }
                    _ => out.extend_from_slice(b"N;"),
                }
                return Ok(());
            }
            // An *uninitialized* typed property (`Undef`) is absent from the
            // wire format (Zend skips it; a lazy wrapper serialized with
            // SKIP_INITIALIZATION_ON_SERIALIZE keeps only materialized slots).
            let n = obj.props.iter().filter(|(_, v)| !matches!(v, Zval::Undef)).count();
            out.extend_from_slice(b"O:");
            out.extend_from_slice(cname.len().to_string().as_bytes());
            out.extend_from_slice(b":\"");
            out.extend_from_slice(cname);
            out.extend_from_slice(b"\":");
            out.extend_from_slice(n.to_string().as_bytes());
            out.extend_from_slice(b":{");
            // Wire-format name mangling: a private slot is stored mangled
            // (`\0Class\0p`) and serializes verbatim; a protected one is stored
            // PLAIN and gains Zend's `\0*\0` prefix on the wire (unserialize
            // strips it back off). Public/dynamic names pass through.
            for (pname, val) in obj.props.iter() {
                if matches!(val, Zval::Undef) {
                    continue;
                }
                // `__serialize()` payload: keys keep array semantics, so a
                // canonical integer key serializes as `i:N`, not `s:…`.
                if obj.info.opaque_array_keys {
                    let as_int = std::str::from_utf8(pname)
                        .ok()
                        .and_then(|s| s.parse::<i64>().ok().filter(|i| i.to_string() == s));
                    match as_int {
                        Some(i) => {
                            out.extend_from_slice(b"i:");
                            out.extend_from_slice(i.to_string().as_bytes());
                            out.push(b';');
                        }
                        None => ser_str(out, pname),
                    }
                    ser_into(out, val, sc)?;
                    continue;
                }
                let is_plain = !pname.starts_with(b"\0");
                if is_plain
                    && matches!(
                        php_types::unmangle_prop_key(pname, &obj.info).1,
                        php_types::PropVis::Protected
                    )
                {
                    let mut mangled = Vec::with_capacity(pname.len() + 3);
                    mangled.extend_from_slice(b"\0*\0");
                    mangled.extend_from_slice(pname);
                    ser_str(out, &mangled);
                } else {
                    ser_str(out, pname);
                }
                ser_into(out, val, sc)?;
            }
            out.push(b'}');
        }
        // References are resolved in `ser_into` before the body renders; a
        // nested cell (not producible by the VM) just re-enters the resolver.
        Zval::Ref(cell) => ser_into(out, &cell.borrow(), sc)?,
        // PHP throws for these (Zend/zend_closures.c / generators).
        Zval::Closure(_) => {
            return Err(PhpError::Error(
                "Serialization of 'Closure' is not allowed".into(),
            ))
        }
        Zval::Generator(_) => {
            return Err(PhpError::Error(
                "Serialization of 'Generator' is not allowed".into(),
            ))
        }
        // PHP serializes a resource as the integer 0 (step 51, D-51.5).
        Zval::Resource(_) => out.extend_from_slice(b"i:0;"),
        Zval::WeakHandle(_) => {
            return Err(PhpError::Error(
                "Serialization of 'WeakReference' is not allowed".into(),
            ))
        }
    }
    Ok(())
}
