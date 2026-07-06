//! Parser for PHP's serialization format (step 50b), the inverse of the
//! `serialize()` builtin. Pure: it produces an intermediate [`Ser`] tree from
//! bytes; the evaluator turns that into a `Zval` (objects need the class table).
//!
//! Grammar handled (the subset `serialize()` emits):
//!   N;  b:[01];  i:<int>;  d:<float>;  s:<len>:"<bytes>";
//!   a:<n>:{<k><v>...}      O:<len>:"<class>":<n>:{<propname><v>...}
//!
//! Shared-reference markers `r:`/`R:` are not handled (step-50 scope-out, D-50):
//! input using them parses to `None`, so `unserialize()` returns `false`.

/// An intermediate node decoded from a serialized string.
#[derive(Debug, Clone, PartialEq)]
pub enum Ser {
    Null,
    Bool(bool),
    Long(i64),
    Double(f64),
    Str(Vec<u8>),
    /// Ordered (key, value) pairs; a key is only ever `Long` or `Str`.
    Array(Vec<(Ser, Ser)>),
    /// Class name and ordered (property-name, value) pairs.
    Object(Vec<u8>, Vec<(Vec<u8>, Ser)>),
    /// `C:<len>:"<class>":<len>:{<payload>}` — a legacy `Serializable` record:
    /// class name and the raw opaque payload its `unserialize()` receives.
    CObject(Vec<u8>, Vec<u8>),
}

/// Parse a complete serialized value. Returns `None` on any malformed input or
/// trailing garbage (PHP's `unserialize()` then yields `false` + a notice).
pub fn parse(bytes: &[u8]) -> Option<Ser> {
    let mut p = Parser { b: bytes, i: 0 };
    let v = p.value()?;
    // PHP tolerates nothing after the top-level value.
    if p.i == p.b.len() {
        Some(v)
    } else {
        None
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn eat(&mut self, c: u8) -> Option<()> {
        if self.peek() == Some(c) {
            self.i += 1;
            Some(())
        } else {
            None
        }
    }

    /// Read up to (not including) `delim`, advancing past it.
    fn take_until(&mut self, delim: u8) -> Option<&[u8]> {
        let start = self.i;
        while self.peek().is_some_and(|c| c != delim) {
            self.i += 1;
        }
        let slice = self.b.get(start..self.i)?;
        self.eat(delim)?;
        Some(slice)
    }

    fn int_until(&mut self, delim: u8) -> Option<i64> {
        let s = self.take_until(delim)?;
        std::str::from_utf8(s).ok()?.parse::<i64>().ok()
    }

    fn value(&mut self) -> Option<Ser> {
        match self.peek()? {
            b'N' => {
                self.i += 1;
                self.eat(b';')?;
                Some(Ser::Null)
            }
            b'b' => {
                self.i += 1;
                self.eat(b':')?;
                let v = match self.peek()? {
                    b'0' => false,
                    b'1' => true,
                    _ => return None,
                };
                self.i += 1;
                self.eat(b';')?;
                Some(Ser::Bool(v))
            }
            b'i' => {
                self.i += 1;
                self.eat(b':')?;
                Some(Ser::Long(self.int_until(b';')?))
            }
            b'd' => {
                self.i += 1;
                self.eat(b':')?;
                let s = self.take_until(b';')?;
                Some(Ser::Double(parse_double(s)?))
            }
            b's' => {
                self.i += 1;
                self.eat(b':')?;
                Some(Ser::Str(self.string_body()?))
            }
            b'a' => {
                self.i += 1;
                self.eat(b':')?;
                let n = self.usize_until(b':')?;
                self.eat(b'{')?;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    let k = self.value()?;
                    // A key is only valid as int or string.
                    if !matches!(k, Ser::Long(_) | Ser::Str(_)) {
                        return None;
                    }
                    let v = self.value()?;
                    items.push((k, v));
                }
                self.eat(b'}')?;
                Some(Ser::Array(items))
            }
            b'O' => {
                self.i += 1;
                self.eat(b':')?;
                // Class name: `<len>:"<class>"` followed by `:` then the count.
                let class = self.quoted_bytes()?;
                self.eat(b':')?;
                let n = self.usize_until(b':')?;
                self.eat(b'{')?;
                let mut props = Vec::with_capacity(n);
                for _ in 0..n {
                    // Property names are serialized strings; an `__serialize()`
                    // record may carry *int* keys (`i:0;`) — kept as their
                    // decimal form (the array builder re-canonicalizes them).
                    let name = match self.value()? {
                        Ser::Str(s) => s,
                        Ser::Long(i) => i.to_string().into_bytes(),
                        _ => return None,
                    };
                    let v = self.value()?;
                    props.push((name, v));
                }
                self.eat(b'}')?;
                Some(Ser::Object(class, props))
            }
            b'C' => {
                // Legacy Serializable record: the braces wrap `<len>` raw
                // payload bytes, NOT nested serialized values.
                self.i += 1;
                self.eat(b':')?;
                let class = self.quoted_bytes()?;
                self.eat(b':')?;
                let len = self.usize_until(b':')?;
                self.eat(b'{')?;
                let payload = self.b.get(self.i..self.i.checked_add(len)?)?.to_vec();
                self.i += len;
                self.eat(b'}')?;
                Some(Ser::CObject(class, payload))
            }
            _ => None,
        }
    }

    /// Read a `<len>:"<bytes>"` chunk (byte count, then the verbatim bytes),
    /// stopping right after the closing quote. The terminator differs by context:
    /// a string value ends with `;`, but an object's class name is followed by
    /// `:` (the property count), so callers consume the terminator themselves.
    fn quoted_bytes(&mut self) -> Option<Vec<u8>> {
        let len = self.usize_until(b':')?;
        self.eat(b'"')?;
        let bytes = self.b.get(self.i..self.i.checked_add(len)?)?.to_vec();
        self.i += len;
        self.eat(b'"')?;
        Some(bytes)
    }

    /// A string value / array key / property name: `<len>:"<bytes>";`.
    fn string_body(&mut self) -> Option<Vec<u8>> {
        let bytes = self.quoted_bytes()?;
        self.eat(b';')?;
        Some(bytes)
    }

    fn usize_until(&mut self, delim: u8) -> Option<usize> {
        let s = self.take_until(delim)?;
        std::str::from_utf8(s).ok()?.parse::<usize>().ok()
    }
}

/// Parse a serialized float body. PHP emits `INF` / `-INF` / `NAN` for the
/// non-finite cases and a shortest-round-trip decimal otherwise.
fn parse_double(s: &[u8]) -> Option<f64> {
    match s {
        b"INF" => return Some(f64::INFINITY),
        b"-INF" => return Some(f64::NEG_INFINITY),
        b"NAN" => return Some(f64::NAN),
        _ => {}
    }
    std::str::from_utf8(s).ok()?.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars() {
        assert_eq!(parse(b"N;"), Some(Ser::Null));
        assert_eq!(parse(b"b:1;"), Some(Ser::Bool(true)));
        assert_eq!(parse(b"i:42;"), Some(Ser::Long(42)));
        assert_eq!(parse(b"i:-7;"), Some(Ser::Long(-7)));
        assert_eq!(parse(b"d:2.5;"), Some(Ser::Double(2.5)));
        assert_eq!(parse(b"s:5:\"hello\";"), Some(Ser::Str(b"hello".to_vec())));
        // A string length is a byte count: an embedded ';' is data, not a delim.
        assert_eq!(parse(b"s:3:\"a;b\";"), Some(Ser::Str(b"a;b".to_vec())));
        // Embedded quote inside the counted bytes is fine too.
        assert_eq!(parse(b"s:4:\"a\";b\";"), Some(Ser::Str(b"a\";b".to_vec())));
        // A wrong byte count must not parse (closing quote lands mid-data).
        assert_eq!(parse(b"s:2:\"abc\";"), None);
    }

    #[test]
    fn non_finite_floats() {
        assert_eq!(parse(b"d:INF;"), Some(Ser::Double(f64::INFINITY)));
        assert_eq!(parse(b"d:-INF;"), Some(Ser::Double(f64::NEG_INFINITY)));
        assert!(matches!(parse(b"d:NAN;"), Some(Ser::Double(d)) if d.is_nan()));
    }

    #[test]
    fn arrays_and_objects() {
        assert_eq!(
            parse(b"a:2:{i:0;i:9;i:1;N;}"),
            Some(Ser::Array(vec![
                (Ser::Long(0), Ser::Long(9)),
                (Ser::Long(1), Ser::Null),
            ]))
        );
        assert_eq!(
            parse(b"O:8:\"stdClass\":1:{s:1:\"x\";i:5;}"),
            Some(Ser::Object(
                b"stdClass".to_vec(),
                vec![(b"x".to_vec(), Ser::Long(5))]
            ))
        );
    }

    #[test]
    fn malformed_and_trailing_garbage() {
        assert_eq!(parse(b"z"), None);
        assert_eq!(parse(b""), None);
        assert_eq!(parse(b"i:1;XX"), None); // trailing garbage
        assert_eq!(parse(b"b:2;"), None); // bad bool
        assert_eq!(parse(b"a:2:{i:0;i:9;}"), None); // count mismatch
        assert_eq!(parse(b"r:1;"), None); // reference markers unsupported (D-50)
    }
}
