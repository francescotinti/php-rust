//! ext/xml host side: `__xml_tokenize(data, separator|null) -> events`.
//!
//! The expat-style `xml_parser_*` API lives in the prelude (`dom.php`): PHP
//! keeps the parser object, its options and handlers, buffers the chunks
//! `xml_parse` receives, and on the final chunk asks this tokenizer for the
//! whole event stream, dispatching handler callbacks from PHP (the same
//! delegate-to-builtin pattern the DOM classes use). Events are arrays:
//!
//! - `['o', name, attrs]` — start element (attrs an ordered name→value hash)
//! - `['c', name]` — end element (self-closing tags emit both)
//! - `['t', text]` — coalesced character data (text + refs + CDATA runs)
//! - `['p', target, data]` — processing instruction
//! - `['x', code, line, column, byte]` — ALWAYS last: 0 = clean EOF, else the
//!   expat error code at the failure position
//!
//! With a separator the parser is namespace-aware (`xml_parser_create_ns`):
//! qualified names expand to `URI<sep>local`, `xmlns` declaration attributes
//! disappear from the attribute list. Without it names stay raw. Case folding
//! and whitespace skipping are applied by the PHP side.
//!
//! DOCTYPE internal subsets contribute `<!ENTITY name "value">` declarations
//! (SimplePie declares the whole HTML 4 set that way); parameter entities and
//! external DTDs are out of scope. Target-encoding conversion is out of scope
//! too: input and output are UTF-8.

use std::collections::HashMap;
use std::rc::Rc;

use php_types::{PhpArray, PhpStr, Zval};

use super::dom::line_col;
use super::{PhpError, Vm};

/// libxml error codes (PHP's ext/xml sits on the libxml compat layer, so
/// `xml_get_error_code` reports xmlParserErrors values — oracle-probed:
/// empty/truncated input 5, mismatched tag 76, undeclared entity 26, invalid
/// character reference 9).
const ERR_DOCUMENT_END: i64 = 5;
const ERR_INVALID_CHAR: i64 = 9;
const ERR_UNDECLARED_ENTITY: i64 = 26;
const ERR_TAG_MISMATCH: i64 = 76;

/// Resolve one reference body (`amp`, `#8211`, `#x2013`, or a declared name).
fn resolve_ref(body: &[u8], entities: &HashMap<Vec<u8>, Vec<u8>>) -> Result<Vec<u8>, i64> {
    let cp_bytes = |cp: u32, err: i64| -> Result<Vec<u8>, i64> {
        char::from_u32(cp)
            .filter(|c| {
                // XML Char production: no NULs/control noise.
                !matches!(*c as u32, 0 | 0xFFFE | 0xFFFF) && !(*c as u32 >= 0xD800 && (*c as u32) < 0xE000)
            })
            .map(|c| c.to_string().into_bytes())
            .ok_or(err)
    };
    if let Some(hex) = body.strip_prefix(b"#x").or_else(|| body.strip_prefix(b"#X")) {
        let cp = u32::from_str_radix(&String::from_utf8_lossy(hex), 16)
            .map_err(|_| ERR_INVALID_CHAR)?;
        return cp_bytes(cp, ERR_INVALID_CHAR);
    }
    if let Some(dec) = body.strip_prefix(b"#") {
        let cp: u32 =
            String::from_utf8_lossy(dec).parse().map_err(|_| ERR_INVALID_CHAR)?;
        return cp_bytes(cp, ERR_INVALID_CHAR);
    }
    match body {
        b"amp" => Ok(b"&".to_vec()),
        b"lt" => Ok(b"<".to_vec()),
        b"gt" => Ok(b">".to_vec()),
        b"quot" => Ok(b"\"".to_vec()),
        b"apos" => Ok(b"'".to_vec()),
        _ => entities.get(body).cloned().ok_or(ERR_UNDECLARED_ENTITY),
    }
}

/// Decode every `&…;` in `raw` (attribute values, entity declaration values).
fn decode_refs(raw: &[u8], entities: &HashMap<Vec<u8>, Vec<u8>>) -> Result<Vec<u8>, i64> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'&' {
            if let Some(end) = raw[i + 1..].iter().position(|&b| b == b';') {
                let body = &raw[i + 1..i + 1 + end];
                out.extend_from_slice(&resolve_ref(body, entities)?);
                i += end + 2;
                continue;
            }
            return Err(ERR_DOCUMENT_END);
        }
        out.push(raw[i]);
        i += 1;
    }
    Ok(out)
}

/// Collect `<!ENTITY name "value">` declarations from a DOCTYPE internal
/// subset (values decoded on declaration — they may reference earlier ones).
fn collect_entities(doctype: &[u8], entities: &mut HashMap<Vec<u8>, Vec<u8>>) {
    let mut i = 0;
    while let Some(p) = doctype[i..]
        .windows(8)
        .position(|w| w.eq_ignore_ascii_case(b"<!ENTITY"))
    {
        let mut k = i + p + 8;
        while doctype.get(k).is_some_and(|b| b.is_ascii_whitespace()) {
            k += 1;
        }
        // Parameter entities (`%`) are out of scope: skip past.
        if doctype.get(k) == Some(&b'%') {
            i = k;
            continue;
        }
        let name_start = k;
        while doctype.get(k).is_some_and(|&b| !b.is_ascii_whitespace() && b != b'>') {
            k += 1;
        }
        let name = doctype[name_start..k].to_vec();
        while doctype.get(k).is_some_and(|b| b.is_ascii_whitespace()) {
            k += 1;
        }
        if let Some(&q @ (b'"' | b'\'')) = doctype.get(k) {
            k += 1;
            let v_start = k;
            while doctype.get(k).is_some_and(|&b| b != q) {
                k += 1;
            }
            let value = &doctype[v_start..k.min(doctype.len())];
            if let Ok(decoded) = decode_refs(value, entities) {
                entities.entry(name).or_insert(decoded);
            }
        }
        i = k.min(doctype.len());
    }
}

/// Namespace scope stack: one frame of `prefix → URI` bindings per element.
struct NsScope {
    frames: Vec<Vec<(Vec<u8>, Vec<u8>)>>,
}

impl NsScope {
    fn lookup(&self, prefix: &[u8]) -> Option<&[u8]> {
        if prefix == b"xml" {
            return Some(b"http://www.w3.org/XML/1998/namespace");
        }
        for frame in self.frames.iter().rev() {
            for (p, uri) in frame.iter().rev() {
                if p == prefix {
                    return if uri.is_empty() { None } else { Some(uri) };
                }
            }
        }
        None
    }

    /// Expand `qname` with `sep`; `use_default` applies the default namespace
    /// (elements yes, attributes no).
    fn expand(&self, qname: &[u8], sep: &[u8], use_default: bool) -> Vec<u8> {
        let (prefix, local): (&[u8], &[u8]) = match qname.iter().position(|&b| b == b':') {
            Some(c) => (&qname[..c], &qname[c + 1..]),
            None => (b"", qname),
        };
        if prefix.is_empty() && !use_default {
            return qname.to_vec();
        }
        match self.lookup(prefix) {
            Some(uri) => {
                let mut out = uri.to_vec();
                out.extend_from_slice(sep);
                out.extend_from_slice(local);
                out
            }
            // Unbound prefix: keep the raw name (expat would error; lenient).
            None => qname.to_vec(),
        }
    }
}

fn ev(items: Vec<Zval>) -> Zval {
    let mut arr = PhpArray::new();
    for it in items {
        let _ = arr.append(it);
    }
    Zval::Array(Rc::new(arr))
}

fn zstr(bytes: Vec<u8>) -> Zval {
    Zval::Str(PhpStr::new(bytes))
}

/// The tokenizer proper.
fn xml_tokenize(data: &[u8], separator: Option<&[u8]>) -> Zval {
    use quick_xml::events::Event;
    let mut events = PhpArray::new();
    let mut reader = quick_xml::Reader::from_reader(data);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_end_names = true;
    reader.config_mut().expand_empty_elements = true;
    let mut entities: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
    let mut ns = NsScope { frames: Vec::new() };
    let mut saw_element = false;
    let mut depth: i64 = 0;
    let mut buf = Vec::new();
    let mut error: Option<i64> = None;

    // Character data is NOT coalesced: expat/libxml deliver one callback per
    // literal run, per resolved reference and per CDATA section, and the
    // probe-visible chunking matches that.
    macro_rules! text_event {
        ($bytes:expr) => {
            let _ = events.append(ev(vec![zstr(b"t".to_vec()), zstr($bytes)]));
        };
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                use quick_xml::errors::{Error as QxError, IllFormedError};
                error = Some(match &e {
                    QxError::IllFormed(IllFormedError::MismatchedEndTag { .. })
                    | QxError::IllFormed(IllFormedError::UnmatchedEndTag(_)) => ERR_TAG_MISMATCH,
                    _ => ERR_DOCUMENT_END,
                });
                break;
            }
            Ok(Event::Eof) => {
                if depth > 0 {
                    // Unclosed elements at EOF: "Invalid document end".
                    error = Some(ERR_DOCUMENT_END);
                }
                break;
            }
            Ok(Event::Decl(_)) => {}
            Ok(Event::DocType(d)) => {
                collect_entities(&d.into_inner(), &mut entities);
            }
            Ok(Event::Start(e)) => {
                saw_element = true;
                depth += 1;
                // Gather raw attributes first (namespace declarations bind on
                // the element that carries them).
                let mut raw_attrs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
                let mut had_err = None;
                for a in e.attributes() {
                    match a {
                        Ok(a) => {
                            let name = a.key.as_ref().to_vec();
                            match decode_refs(&a.value, &entities) {
                                Ok(v) => raw_attrs.push((name, v)),
                                Err(code) => {
                                    had_err = Some(code);
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            had_err = Some(ERR_DOCUMENT_END);
                            break;
                        }
                    }
                }
                if let Some(code) = had_err {
                    error = Some(code);
                    break;
                }
                let name = e.name().as_ref().to_vec();
                let (out_name, out_attrs) = match separator {
                    Some(sep) => {
                        let mut frame: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
                        for (an, av) in &raw_attrs {
                            if an.as_slice() == b"xmlns" {
                                frame.push((Vec::new(), av.clone()));
                            } else if let Some(p) = an.strip_prefix(b"xmlns:") {
                                frame.push((p.to_vec(), av.clone()));
                            }
                        }
                        // xml_set_start_namespace_decl_handler: one event per
                        // declaration, BEFORE the element's open event; the
                        // default namespace's prefix is `false` (oracle-pinned).
                        // No end event: the libxml compat never fires it.
                        for (p, uri) in &frame {
                            let prefix = if p.is_empty() {
                                Zval::Bool(false)
                            } else {
                                zstr(p.clone())
                            };
                            let _ = events.append(ev(vec![
                                zstr(b"n".to_vec()),
                                prefix,
                                zstr(uri.clone()),
                            ]));
                        }
                        ns.frames.push(frame);
                        let out_name = ns.expand(&name, sep, true);
                        let mut out_attrs = PhpArray::new();
                        for (an, av) in raw_attrs {
                            if an.as_slice() == b"xmlns" || an.starts_with(b"xmlns:") {
                                continue;
                            }
                            let expanded = ns.expand(&an, sep, false);
                            out_attrs.insert(
                                php_types::Key::Str(PhpStr::new(expanded)),
                                zstr(av),
                            );
                        }
                        (out_name, out_attrs)
                    }
                    None => {
                        let mut out_attrs = PhpArray::new();
                        for (an, av) in raw_attrs {
                            out_attrs.insert(php_types::Key::Str(PhpStr::new(an)), zstr(av));
                        }
                        (name, out_attrs)
                    }
                };
                let _ = events.append(ev(vec![
                    zstr(b"o".to_vec()),
                    zstr(out_name),
                    Zval::Array(Rc::new(out_attrs)),
                ]));
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                let name = e.name().as_ref().to_vec();
                let out_name = match separator {
                    Some(sep) => {
                        let n = ns.expand(&name, sep, true);
                        ns.frames.pop();
                        n
                    }
                    None => name,
                };
                let _ = events.append(ev(vec![zstr(b"c".to_vec()), zstr(out_name)]));
            }
            Ok(Event::Empty(_)) => unreachable!("expand_empty_elements is on"),
            Ok(Event::Text(t)) => {
                // Prolog/epilog character data (outside the root element) is
                // never delivered — libxml drops it (oracle-pinned, WP-18).
                let raw = t.into_inner().into_owned();
                if !raw.is_empty() && depth > 0 {
                    text_event!(raw);
                }
            }
            Ok(Event::Comment(c)) => {
                // A comment goes to the DEFAULT handler as its full raw text,
                // `<!--…-->` included, in one call (oracle-pinned).
                let mut raw = b"<!--".to_vec();
                raw.extend_from_slice(&c.into_inner());
                raw.extend_from_slice(b"-->");
                let _ = events.append(ev(vec![zstr(b"d".to_vec()), zstr(raw)]));
            }
            Ok(Event::GeneralRef(r)) => match resolve_ref(&r.into_inner(), &entities) {
                Ok(resolved) => { text_event!(resolved); }
                Err(code) => {
                    error = Some(code);
                    break;
                }
            },
            Ok(Event::CData(c)) => {
                text_event!(c.into_inner().into_owned());
            }

            Ok(Event::PI(pi)) => {
                let raw = pi.to_vec();
                let split = raw.iter().position(|b| b.is_ascii_whitespace());
                let (target, pdata) = match split {
                    Some(i) => {
                        let mut d = raw[i..].to_vec();
                        while d.first().is_some_and(|b| b.is_ascii_whitespace()) {
                            d.remove(0);
                        }
                        (raw[..i].to_vec(), d)
                    }
                    None => (raw, Vec::new()),
                };
                let _ = events
                    .append(ev(vec![zstr(b"p".to_vec()), zstr(target), zstr(pdata)]));
            }
        }
        buf.clear();
    }

    let code = match error {
        Some(c) => c,
        None if !saw_element => ERR_DOCUMENT_END,
        None => 0,
    };
    // A document with no root element reports the START position (libxml stops
    // before consuming the blanks).
    let byte = if code == ERR_DOCUMENT_END && !saw_element {
        0
    } else {
        reader.buffer_position() as usize
    };
    let (line, col) = line_col(data, byte, false);
    let _ = events.append(ev(vec![
        zstr(b"x".to_vec()),
        Zval::Long(code),
        Zval::Long(line),
        Zval::Long(col),
        Zval::Long(byte.min(data.len()) as i64),
    ]));
    Zval::Array(Rc::new(events))
}

impl<'m> Vm<'m> {
    /// `__xml_tokenize(data, separator|null) -> events` (see module docs).
    pub(super) fn ho_xml_tokenize(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let data = self.dom_str(&args, 0);
        let separator = match args.get(1) {
            None | Some(Zval::Null) => None,
            _ => Some(self.dom_str(&args, 1)),
        };
        Ok(xml_tokenize(&data, separator.as_deref()))
    }
}
