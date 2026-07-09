//! Minimal JSON parser for `json_decode` (step 26).
//!
//! Produces a [`Json`] tree; the evaluator converts it to a `Zval`, building
//! either PHP arrays (associative mode) or `stdClass` instances (default). The
//! parser is strict: trailing non-whitespace makes the whole input invalid, in
//! which case `json_decode` returns `null`.

/// A parsed JSON value. Object key order is preserved (PHP keeps insertion
/// order for both arrays and `stdClass`).
pub enum Json {
    Null,
    Bool(bool),
    Long(i64),
    Double(f64),
    Str(Vec<u8>),
    Array(Vec<Json>),
    Object(Vec<(Vec<u8>, Json)>),
}

/// Why a parse failed: a malformed document (`JSON_ERROR_SYNTAX`) versus nesting
/// deeper than the requested limit (`JSON_ERROR_DEPTH`).
pub enum JsonError {
    Syntax,
    Depth,
}

/// Parse a complete JSON document. Returns `None` on any syntax error or
/// trailing garbage. Uses PHP's default nesting limit (512); callers that must
/// distinguish a depth overflow use [`parse_depth`].
pub fn parse(input: &[u8]) -> Option<Json> {
    parse_depth(input, 512).ok()
}

/// Parse with an explicit maximum nesting depth (`json_validate`/`json_decode`'s
/// `$depth`). PHP counts one level per `{`/`[` entered and errors once the count
/// reaches `max_depth`, so a flat container needs `max_depth >= 2`. Distinguishes
/// a depth overflow from a syntax error.
pub fn parse_depth(input: &[u8], max_depth: u32) -> Result<Json, JsonError> {
    let mut p = Parser { s: input, i: 0, depth: 0, max_depth, depth_exceeded: false };
    p.skip_ws();
    let value = p.value();
    if p.depth_exceeded {
        return Err(JsonError::Depth);
    }
    let v = value.ok_or(JsonError::Syntax)?;
    p.skip_ws();
    if p.i == p.s.len() {
        Ok(v)
    } else {
        Err(JsonError::Syntax)
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    /// Current nesting level (0 at the document root, +1 per open container).
    depth: u32,
    /// Requested limit; entering a container that makes `depth >= max_depth`
    /// aborts the parse with a depth error rather than a syntax error.
    max_depth: u32,
    depth_exceeded: bool,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.i += 1;
        }
    }

    fn eat(&mut self, lit: &[u8]) -> bool {
        if self.s[self.i..].starts_with(lit) {
            self.i += lit.len();
            true
        } else {
            false
        }
    }

    fn value(&mut self) -> Option<Json> {
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => Some(Json::Str(self.string()?)),
            b't' => self.eat(b"true").then_some(Json::Bool(true)),
            b'f' => self.eat(b"false").then_some(Json::Bool(false)),
            b'n' => self.eat(b"null").then_some(Json::Null),
            b'-' | b'0'..=b'9' => self.number(),
            _ => None,
        }
    }

    /// Enter a container: deepen one level and flag (returning `false`) if that
    /// reaches the configured `max_depth`.
    fn enter(&mut self) -> bool {
        self.depth += 1;
        if self.depth >= self.max_depth {
            self.depth_exceeded = true;
            return false;
        }
        true
    }

    fn object(&mut self) -> Option<Json> {
        self.i += 1; // '{'
        if !self.enter() {
            return None;
        }
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek()? == b'}' {
            self.i += 1;
            self.depth -= 1;
            return Some(Json::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek()? != b'"' {
                return None;
            }
            let key = self.string()?;
            self.skip_ws();
            if self.peek()? != b':' {
                return None;
            }
            self.i += 1;
            self.skip_ws();
            let val = self.value()?;
            entries.push((key, val));
            self.skip_ws();
            match self.peek()? {
                b',' => self.i += 1,
                b'}' => {
                    self.i += 1;
                    self.depth -= 1;
                    return Some(Json::Object(entries));
                }
                _ => return None,
            }
        }
    }

    fn array(&mut self) -> Option<Json> {
        self.i += 1; // '['
        if !self.enter() {
            return None;
        }
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek()? == b']' {
            self.i += 1;
            self.depth -= 1;
            return Some(Json::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value()?);
            self.skip_ws();
            match self.peek()? {
                b',' => self.i += 1,
                b']' => {
                    self.i += 1;
                    self.depth -= 1;
                    return Some(Json::Array(items));
                }
                _ => return None,
            }
        }
    }

    fn string(&mut self) -> Option<Vec<u8>> {
        self.i += 1; // opening '"'
        let mut out = Vec::new();
        loop {
            let c = self.peek()?;
            self.i += 1;
            match c {
                b'"' => return Some(out),
                b'\\' => {
                    let e = self.peek()?;
                    self.i += 1;
                    match e {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0C),
                        b'u' => {
                            let cp = self.hex4()?;
                            let scalar = if (0xD800..=0xDBFF).contains(&cp) {
                                // High surrogate: expect a following \uXXXX low surrogate.
                                if !self.eat(b"\\u") {
                                    return None;
                                }
                                let lo = self.hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&lo) {
                                    return None;
                                }
                                0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00)
                            } else if (0xDC00..=0xDFFF).contains(&cp) {
                                return None; // lone low surrogate
                            } else {
                                cp
                            };
                            let ch = char::from_u32(scalar)?;
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return None,
                    }
                }
                // Unescaped control characters are invalid in JSON strings.
                0x00..=0x1F => return None,
                other => out.push(other),
            }
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let mut v = 0u32;
        for _ in 0..4 {
            let d = self.peek()?;
            self.i += 1;
            v = v * 16 + (d as char).to_digit(16)?;
        }
        Some(v)
    }

    fn number(&mut self) -> Option<Json> {
        let start = self.i;
        let mut is_float = false;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.i += 1;
        }
        if self.peek() == Some(b'.') {
            is_float = true;
            self.i += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.i += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        let text = std::str::from_utf8(&self.s[start..self.i]).ok()?;
        if is_float {
            text.parse::<f64>().ok().map(Json::Double)
        } else {
            // An integer that overflows i64 decodes as a float, like PHP.
            match text.parse::<i64>() {
                Ok(n) => Some(Json::Long(n)),
                Err(_) => text.parse::<f64>().ok().map(Json::Double),
            }
        }
    }
}
