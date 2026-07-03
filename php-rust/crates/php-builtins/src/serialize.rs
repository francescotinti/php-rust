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
    ser_into(&mut out, v)?;
    Ok(Zval::Str(PhpStr::new(out)))
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

fn ser_into(out: &mut Vec<u8>, v: &Zval) -> Result<(), PhpError> {
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
                ser_into(out, val)?;
            }
            out.push(b'}');
        }
        Zval::Object(o) => {
            let obj = o.borrow();
            let cname = obj.class_name.as_bytes();
            let n = obj.props.iter().count();
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
                ser_into(out, val)?;
            }
            out.push(b'}');
        }
        // A reference is transparent here: serialize the pointee. PHP's r:/R:
        // shared-reference markers are a step-50 scope-out (D-50).
        Zval::Ref(cell) => ser_into(out, &cell.borrow())?,
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
