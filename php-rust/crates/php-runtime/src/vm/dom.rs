//! ext/dom subset: an arena-backed XML tree (host side of the prelude `DOM*`
//! classes) with a quick-xml parser, a `saveXML` serializer, and an XPath 1.0
//! subset evaluator.
//!
//! Model: each `DOMDocument` owns a [`DomDoc`] in `Vm.dom_docs`, addressed by a
//! `docId`; every node is an arena index (`nodeId`, index 0 = the document
//! node). The prelude classes carry `(docId, nodeId)` handles and delegate to
//! the `__dom_*` host builtins registered in `vm/mod.rs`. Attributes live
//! inline on their element (PHP's `DOMAttr` objects address them as
//! `(docId, elemId, name)`).
//!
//! Real ext/dom wraps libxml2; this arena mirrors the observable W3C DOM
//! behaviour needed by the app-compat roadmap (phpunit.xml loading, phar-io
//! manifests, Composer XML paths). Known scope-outs: DTD/entity machinery,
//! namespace-aware creation (`createElementNS`), schema/relaxNG validation,
//! HTML parsing, and full XPath (functions beyond the common core).

use std::rc::Rc;

use php_types::{Key, PhpArray, PhpStr, Zval};

/// One parsed document (or an empty one from `new DOMDocument`).
pub(super) struct DomDoc {
    pub nodes: Vec<DomNode>,
    pub version: Vec<u8>,
    pub encoding: Option<Vec<u8>>,
    /// The document encoding an HTML parse resolved (override argument, `<meta
    /// charset>` sniff, or the UTF-8 default) — `Dom\Document::$inputEncoding`.
    pub input_encoding: Option<Vec<u8>>,
}

pub(super) struct DomNode {
    pub kind: DomKind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

pub(super) enum DomKind {
    Document,
    Element {
        name: Vec<u8>,
        /// Attributes in document order as `(qualified name, value)`.
        attrs: Vec<(Vec<u8>, Vec<u8>)>,
    },
    Text(Vec<u8>),
    Cdata(Vec<u8>),
    Comment(Vec<u8>),
    Pi { target: Vec<u8>, data: Vec<u8> },
    DocType { name: Vec<u8> },
    Fragment,
}

impl DomKind {
    /// W3C `nodeType` codes (XML_*_NODE constants).
    pub(super) fn node_type(&self) -> i64 {
        match self {
            DomKind::Element { .. } => 1,
            DomKind::Text(_) => 3,
            DomKind::Cdata(_) => 4,
            DomKind::Pi { .. } => 7,
            DomKind::Comment(_) => 8,
            DomKind::Document => 9,
            DomKind::DocType { .. } => 10,
            DomKind::Fragment => 11,
        }
    }

    /// W3C `nodeName`.
    pub(super) fn node_name(&self) -> Vec<u8> {
        match self {
            DomKind::Element { name, .. } => name.clone(),
            DomKind::Text(_) => b"#text".to_vec(),
            DomKind::Cdata(_) => b"#cdata-section".to_vec(),
            DomKind::Comment(_) => b"#comment".to_vec(),
            DomKind::Pi { target, .. } => target.clone(),
            DomKind::Document => b"#document".to_vec(),
            DomKind::DocType { name } => name.clone(),
            DomKind::Fragment => b"#document-fragment".to_vec(),
        }
    }

    /// W3C `nodeValue` (`None` for element/document/fragment/doctype → PHP null).
    pub(super) fn node_value(&self) -> Option<Vec<u8>> {
        match self {
            DomKind::Text(d) | DomKind::Cdata(d) | DomKind::Comment(d) => Some(d.clone()),
            DomKind::Pi { data, .. } => Some(data.clone()),
            _ => None,
        }
    }
}

impl DomDoc {
    pub(super) fn new() -> DomDoc {
        DomDoc {
            nodes: vec![DomNode { kind: DomKind::Document, parent: None, children: Vec::new() }],
            version: b"1.0".to_vec(),
            encoding: None,
            input_encoding: None,
        }
    }

    fn push(&mut self, kind: DomKind, parent: Option<usize>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(DomNode { kind, parent, children: Vec::new() });
        if let Some(p) = parent {
            self.nodes[p].children.push(id);
        }
        id
    }

    /// Append text under `parent`, merging into a trailing text sibling (libxml
    /// yields one text node around resolved entity references).
    fn push_text(&mut self, text: Vec<u8>, parent: usize) {
        if let Some(&last) = self.nodes[parent].children.last() {
            if let DomKind::Text(existing) = &mut self.nodes[last].kind {
                existing.extend_from_slice(&text);
                return;
            }
        }
        self.push(DomKind::Text(text), Some(parent));
    }

    /// Detach `child` from its current parent (if any).
    pub(super) fn detach(&mut self, child: usize) {
        if let Some(p) = self.nodes[child].parent.take() {
            self.nodes[p].children.retain(|&c| c != child);
        }
    }

    /// Append `child` under `parent` (moving it if attached elsewhere). A
    /// fragment splices its children instead, as the DOM specifies.
    pub(super) fn append(&mut self, parent: usize, child: usize) {
        if matches!(self.nodes[child].kind, DomKind::Fragment) {
            let kids: Vec<usize> = std::mem::take(&mut self.nodes[child].children);
            for k in kids {
                self.nodes[k].parent = None;
                self.append(parent, k);
            }
            return;
        }
        self.detach(child);
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.push(child);
    }

    /// Insert `child` under `parent` immediately before `reference`.
    pub(super) fn insert_before(&mut self, parent: usize, child: usize, reference: usize) {
        if matches!(self.nodes[child].kind, DomKind::Fragment) {
            let kids: Vec<usize> = std::mem::take(&mut self.nodes[child].children);
            for k in kids {
                self.nodes[k].parent = None;
                self.insert_before(parent, k, reference);
            }
            return;
        }
        self.detach(child);
        self.nodes[child].parent = Some(parent);
        let pos = self.nodes[parent].children.iter().position(|&c| c == reference);
        match pos {
            Some(i) => self.nodes[parent].children.insert(i, child),
            None => self.nodes[parent].children.push(child),
        }
    }

    /// Deep/shallow copy of `src` (possibly from another arena) into `self`,
    /// returning the new detached node id. Serves `cloneNode` and `importNode`.
    pub(super) fn copy_from(&mut self, src_doc: &[DomNode], src: usize, deep: bool) -> usize {
        let kind = match &src_doc[src].kind {
            DomKind::Document | DomKind::Fragment => DomKind::Fragment,
            DomKind::Element { name, attrs } => {
                DomKind::Element { name: name.clone(), attrs: attrs.clone() }
            }
            DomKind::Text(d) => DomKind::Text(d.clone()),
            DomKind::Cdata(d) => DomKind::Cdata(d.clone()),
            DomKind::Comment(d) => DomKind::Comment(d.clone()),
            DomKind::Pi { target, data } => {
                DomKind::Pi { target: target.clone(), data: data.clone() }
            }
            DomKind::DocType { name } => DomKind::DocType { name: name.clone() },
        };
        let id = self.push(kind, None);
        if deep {
            let kids = src_doc[src].children.clone();
            for k in kids {
                let copied = self.copy_from(src_doc, k, true);
                self.nodes[copied].parent = Some(id);
                self.nodes[id].children.push(copied);
            }
        }
        id
    }

    /// The document element (first element child of the document node).
    pub(super) fn document_element(&self) -> Option<usize> {
        self.nodes[0]
            .children
            .iter()
            .copied()
            .find(|&c| matches!(self.nodes[c].kind, DomKind::Element { .. }))
    }

    /// Concatenated text of all descendant text/cdata nodes (W3C textContent).
    pub(super) fn text_content(&self, n: usize) -> Vec<u8> {
        match &self.nodes[n].kind {
            DomKind::Text(d) | DomKind::Cdata(d) | DomKind::Comment(d) => d.clone(),
            DomKind::Pi { data, .. } => data.clone(),
            _ => {
                let mut out = Vec::new();
                for &c in &self.nodes[n].children {
                    if !matches!(self.nodes[c].kind, DomKind::Comment(_) | DomKind::Pi { .. }) {
                        out.extend_from_slice(&self.text_content(c));
                    }
                }
                out
            }
        }
    }

    /// Descendant elements (document order) matching the qualified `name`
    /// (`*` matches all), excluding the context node itself.
    pub(super) fn elements_by_tag(&self, ctx: usize, name: &[u8], out: &mut Vec<usize>) {
        for &c in &self.nodes[ctx].children {
            if let DomKind::Element { name: n, .. } = &self.nodes[c].kind {
                // ext/dom matches the *local* name too (oracle: `b` finds `m:b`).
                let local = match n.iter().position(|&b| b == b':') {
                    Some(p) => &n[p + 1..],
                    None => &n[..],
                };
                if name == b"*" || n == name || local == name {
                    out.push(c);
                }
            }
            self.elements_by_tag(c, name, out);
        }
    }

    /// Attribute lookup on an element.
    pub(super) fn attr(&self, n: usize, name: &[u8]) -> Option<&[u8]> {
        match &self.nodes[n].kind {
            DomKind::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_slice()),
            _ => None,
        }
    }

    /// Resolve the namespace URI in scope for `prefix` at element `n` by
    /// walking `xmlns`/`xmlns:p` declarations up the ancestor chain. An empty
    /// prefix resolves the default namespace.
    pub(super) fn resolve_ns(&self, n: usize, prefix: &[u8]) -> Option<Vec<u8>> {
        let key: Vec<u8> = if prefix.is_empty() {
            b"xmlns".to_vec()
        } else {
            let mut k = b"xmlns:".to_vec();
            k.extend_from_slice(prefix);
            k
        };
        let mut cur = Some(n);
        while let Some(c) = cur {
            if let Some(v) = self.attr(c, &key) {
                return Some(v.to_vec());
            }
            cur = self.nodes[c].parent;
        }
        None
    }

    // ----- parsing -----

    /// Parse `xml` into a fresh arena. `Err(message)` mirrors a libxml parse
    /// error (recorded by the caller for `libxml_get_errors`).
    pub(super) fn parse(xml: &[u8]) -> Result<DomDoc, String> {
        use quick_xml::events::Event;
        let mut doc = DomDoc::new();
        let mut reader = quick_xml::Reader::from_reader(xml);
        // PHP's default `preserveWhiteSpace = true` keeps blank text nodes.
        reader.config_mut().trim_text(false);
        // Mirror libxml's well-formedness checking (mismatched tags error out).
        reader.config_mut().check_end_names = true;
        let mut stack: Vec<usize> = vec![0];
        let mut saw_element = false;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => return Err(format!("{e}")),
                Ok(Event::Eof) => break,
                Ok(Event::Decl(d)) => {
                    if let Ok(v) = d.version() {
                        doc.version = v.to_vec();
                    }
                    if let Some(Ok(enc)) = d.encoding() {
                        doc.encoding = Some(enc.to_vec());
                    }
                }
                Ok(Event::Start(e)) => {
                    let parent = *stack.last().unwrap();
                    if parent == 0 && saw_element {
                        return Err("Extra content at the end of the document".to_string());
                    }
                    let id = doc.start_element(&e, parent)?;
                    saw_element = true;
                    stack.push(id);
                }
                Ok(Event::Empty(e)) => {
                    let parent = *stack.last().unwrap();
                    if parent == 0 && saw_element {
                        return Err("Extra content at the end of the document".to_string());
                    }
                    doc.start_element(&e, parent)?;
                    saw_element = true;
                }
                Ok(Event::End(_)) => {
                    stack.pop();
                    if stack.is_empty() {
                        return Err("unexpected closing tag".to_string());
                    }
                }
                Ok(Event::Text(t)) => {
                    let raw = String::from_utf8_lossy(&t.into_inner()).into_owned();
                    let text = quick_xml::escape::unescape(&raw)
                        .map_err(|e| format!("{e}"))?
                        .into_owned()
                        .into_bytes();
                    let parent = *stack.last().unwrap();
                    // Text directly under the document node: only whitespace is
                    // well-formed there, and libxml drops it.
                    if parent == 0 {
                        if text.iter().any(|b| !b.is_ascii_whitespace()) {
                            return Err("Start tag expected, '<' not found".to_string());
                        }
                        continue;
                    }
                    doc.push_text(text, parent);
                }
                Ok(Event::CData(c)) => {
                    let parent = *stack.last().unwrap();
                    if parent != 0 {
                        doc.push(DomKind::Cdata(c.to_vec()), Some(parent));
                    }
                }
                Ok(Event::Comment(c)) => {
                    // Comment content is raw: libxml does not expand entities here.
                    let text = c.into_inner().into_owned();
                    let parent = *stack.last().unwrap();
                    doc.push(DomKind::Comment(text), Some(parent));
                }
                Ok(Event::PI(pi)) => {
                    let raw = pi.to_vec();
                    let split = raw.iter().position(|b| b.is_ascii_whitespace());
                    let (target, data) = match split {
                        Some(i) => {
                            let mut d = raw[i..].to_vec();
                            while d.first().is_some_and(|b| b.is_ascii_whitespace()) {
                                d.remove(0);
                            }
                            (raw[..i].to_vec(), d)
                        }
                        None => (raw, Vec::new()),
                    };
                    let parent = *stack.last().unwrap();
                    doc.push(DomKind::Pi { target, data }, Some(parent));
                }
                Ok(Event::DocType(d)) => {
                    let raw = d.to_vec();
                    let name_end =
                        raw.iter().position(|b| b.is_ascii_whitespace()).unwrap_or(raw.len());
                    doc.push(DomKind::DocType { name: raw[..name_end].to_vec() }, Some(0));
                }
                Ok(Event::GeneralRef(r)) => {
                    // quick-xml 0.41 reports every `&name;` / `&#NN;` in text as
                    // its own event. Resolve predefined + numeric references and
                    // merge into the surrounding text (libxml yields ONE text
                    // node for `a&amp;b`); an unknown entity is a parse error,
                    // as libxml without NOENT/DTD.
                    let name = r.into_inner();
                    let resolved: Vec<u8> = if let Some(hex) = name.strip_prefix(b"#x") {
                        let cp = u32::from_str_radix(&String::from_utf8_lossy(hex), 16)
                            .map_err(|_| "invalid character reference".to_string())?;
                        char::from_u32(cp)
                            .map(|c| c.to_string().into_bytes())
                            .ok_or("invalid character reference".to_string())?
                    } else if let Some(dec) = name.strip_prefix(b"#") {
                        let cp: u32 = String::from_utf8_lossy(dec)
                            .parse()
                            .map_err(|_| "invalid character reference".to_string())?;
                        char::from_u32(cp)
                            .map(|c| c.to_string().into_bytes())
                            .ok_or("invalid character reference".to_string())?
                    } else {
                        match quick_xml::escape::resolve_predefined_entity(
                            &String::from_utf8_lossy(&name),
                        ) {
                            Some(s) => s.as_bytes().to_vec(),
                            None => {
                                return Err(format!(
                                    "Entity '{}' not defined",
                                    String::from_utf8_lossy(&name)
                                ))
                            }
                        }
                    };
                    let parent = *stack.last().unwrap();
                    if parent != 0 {
                        doc.push_text(resolved, parent);
                    }
                }
            }
            buf.clear();
        }
        if stack.len() != 1 {
            return Err("Premature end of data in tag".to_string());
        }
        if !saw_element {
            return Err("Start tag expected, '<' not found".to_string());
        }
        Ok(doc)
    }

    fn start_element(
        &mut self,
        e: &quick_xml::events::BytesStart<'_>,
        parent: usize,
    ) -> Result<usize, String> {
        let name = e.name().as_ref().to_vec();
        let mut attrs = Vec::new();
        for a in e.attributes() {
            let a = a.map_err(|e| format!("{e}"))?;
            let value = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_err(|e| format!("{e}"))?
                .into_owned()
                .into_bytes();
            attrs.push((a.key.as_ref().to_vec(), value));
        }
        Ok(self.push(DomKind::Element { name, attrs }, Some(parent)))
    }

    // ----- serialization -----

    /// `saveXML()`: the document (or the subtree at `node`) as XML text. The
    /// whole document gets the `<?xml … ?>` declaration and a trailing newline,
    /// exactly as libxml emits it.
    pub(super) fn save_xml(&self, node: Option<usize>) -> Vec<u8> {
        let mut out = Vec::new();
        match node {
            None => {
                out.extend_from_slice(b"<?xml version=\"");
                out.extend_from_slice(&self.version);
                out.extend_from_slice(b"\"");
                if let Some(enc) = &self.encoding {
                    out.extend_from_slice(b" encoding=\"");
                    out.extend_from_slice(enc);
                    out.extend_from_slice(b"\"");
                }
                out.extend_from_slice(b"?>\n");
                for &c in &self.nodes[0].children {
                    self.serialize(c, &mut out);
                }
                out.push(b'\n');
            }
            Some(n) => self.serialize(n, &mut out),
        }
        out
    }

    fn serialize(&self, n: usize, out: &mut Vec<u8>) {
        match &self.nodes[n].kind {
            DomKind::Document | DomKind::Fragment => {
                for &c in &self.nodes[n].children {
                    self.serialize(c, out);
                }
            }
            DomKind::Element { name, attrs } => {
                out.push(b'<');
                out.extend_from_slice(name);
                for (k, v) in attrs {
                    out.push(b' ');
                    out.extend_from_slice(k);
                    out.extend_from_slice(b"=\"");
                    escape_into(v, true, out);
                    out.push(b'"');
                }
                if self.nodes[n].children.is_empty() {
                    out.extend_from_slice(b"/>");
                } else {
                    out.push(b'>');
                    for &c in &self.nodes[n].children {
                        self.serialize(c, out);
                    }
                    out.extend_from_slice(b"</");
                    out.extend_from_slice(name);
                    out.push(b'>');
                }
            }
            DomKind::Text(d) => escape_into(d, false, out),
            DomKind::Cdata(d) => {
                out.extend_from_slice(b"<![CDATA[");
                out.extend_from_slice(d);
                out.extend_from_slice(b"]]>");
            }
            DomKind::Comment(d) => {
                out.extend_from_slice(b"<!--");
                out.extend_from_slice(d);
                out.extend_from_slice(b"-->");
            }
            DomKind::Pi { target, data } => {
                out.extend_from_slice(b"<?");
                out.extend_from_slice(target);
                if !data.is_empty() {
                    out.push(b' ');
                    out.extend_from_slice(data);
                }
                out.extend_from_slice(b"?>");
            }
            DomKind::DocType { name } => {
                out.extend_from_slice(b"<!DOCTYPE ");
                out.extend_from_slice(name);
                out.extend_from_slice(b">\n");
            }
        }
    }
}

// ----- HTML5-lite parser (`Dom\HTMLDocument::createFromString`) -----
//
// A lenient HTML tree builder covering the surface the new `Dom\` API needs for
// app-compat (symfony/dom-crawler parses every BrowserKit response body through
// it): implied html/head/body structure, void elements, raw-text elements
// (script/style + RCDATA title/textarea), comments, doctype, bogus comments,
// attribute quoting forms, `<p>`/`<li>`-style auto-closing, numeric + core
// named character references, and `<meta charset>` sniffing with the WHATWG
// label→canonical mapping for the Latin-1 family. Real PHP wraps lexbor's full
// HTML5 algorithm; scope-outs (adoption agency, table fostering, template
// contents, the full named-entity table) are documented in
// PHPR_DIVERGENCES_FROM_PHP.md.

/// Void elements: start tags that never take children (HTML5 §13.1.2).
fn html_void(name: &[u8]) -> bool {
    matches!(
        name,
        b"area" | b"base" | b"basefont" | b"bgsound" | b"br" | b"col" | b"embed" | b"hr"
            | b"img" | b"input" | b"keygen" | b"link" | b"meta" | b"param" | b"source"
            | b"track" | b"wbr"
    )
}

/// Start tags that implicitly close an open `<p>` (HTML5 "close a p element").
fn html_closes_p(name: &[u8]) -> bool {
    matches!(
        name,
        b"address" | b"article" | b"aside" | b"blockquote" | b"details" | b"div" | b"dl"
            | b"fieldset" | b"figcaption" | b"figure" | b"footer" | b"form" | b"h1" | b"h2"
            | b"h3" | b"h4" | b"h5" | b"h6" | b"header" | b"hgroup" | b"hr" | b"main"
            | b"menu" | b"nav" | b"ol" | b"p" | b"pre" | b"section" | b"table" | b"ul"
    )
}

/// Metadata tags routed into `<head>` while no body content has started.
fn html_head_tag(name: &[u8]) -> bool {
    matches!(
        name,
        b"base" | b"basefont" | b"bgsound" | b"link" | b"meta" | b"title" | b"style"
            | b"script" | b"template" | b"noframes"
    )
}

/// Raw-text elements: their content runs to the matching end tag with no tag
/// tokenization. Returns `Some(decode_entities)` (RCDATA decodes references).
fn html_rawtext(name: &[u8]) -> Option<bool> {
    match name {
        b"script" | b"style" => Some(false),
        b"title" | b"textarea" => Some(true),
        _ => None,
    }
}

/// WHATWG encoding label → the canonical name phpr can transcode. `None` for
/// an unknown label (PHP raises ValueError for an invalid override).
pub(super) fn html_encoding_canonical(label: &[u8]) -> Option<&'static [u8]> {
    let mut l = label.to_ascii_lowercase();
    l.retain(|b| !b.is_ascii_whitespace());
    Some(match l.as_slice() {
        b"utf-8" | b"utf8" | b"unicode-1-1-utf-8" | b"unicode11utf8" | b"unicode20utf8"
        | b"x-unicode20utf8" => b"UTF-8",
        b"windows-1252" | b"cp1252" | b"x-cp1252" | b"iso-8859-1" | b"iso8859-1"
        | b"iso88591" | b"iso_8859-1" | b"iso_8859-1:1987" | b"latin1" | b"l1"
        | b"csisolatin1" | b"ibm819" | b"cp819" | b"ansi_x3.4-1968" | b"ascii"
        | b"us-ascii" => b"windows-1252",
        b"iso-8859-15" | b"iso8859-15" | b"iso885915" | b"iso_8859-15" | b"csisolatin9"
        | b"l9" | b"latin9" => b"ISO-8859-15",
        _ => return None,
    })
}

/// Transcode `bytes` in `canonical` encoding to UTF-8 (the tree's encoding).
fn html_to_utf8(bytes: &[u8], canonical: &[u8]) -> Vec<u8> {
    // windows-1252's 0x80–0x9F block (WHATWG index); other bytes are Latin-1.
    const W1252: [u32; 32] = [
        0x20AC, 0x0081, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030,
        0x0160, 0x2039, 0x0152, 0x008D, 0x017D, 0x008F, 0x0090, 0x2018, 0x2019, 0x201C,
        0x201D, 0x2022, 0x2013, 0x2014, 0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x009D,
        0x017E, 0x0178,
    ];
    match canonical {
        b"windows-1252" | b"ISO-8859-15" => {
            let mut out = Vec::with_capacity(bytes.len());
            for &b in bytes {
                let cp = match (canonical, b) {
                    (b"windows-1252", 0x80..=0x9F) => W1252[(b - 0x80) as usize],
                    // ISO-8859-15 revisions over Latin-1.
                    (b"ISO-8859-15", 0xA4) => 0x20AC,
                    (b"ISO-8859-15", 0xA6) => 0x0160,
                    (b"ISO-8859-15", 0xA8) => 0x0161,
                    (b"ISO-8859-15", 0xB4) => 0x017D,
                    (b"ISO-8859-15", 0xB8) => 0x017E,
                    (b"ISO-8859-15", 0xBC) => 0x0152,
                    (b"ISO-8859-15", 0xBD) => 0x0153,
                    (b"ISO-8859-15", 0xBE) => 0x0178,
                    _ => b as u32,
                };
                match char::from_u32(cp) {
                    Some(c) => out.extend_from_slice(c.to_string().as_bytes()),
                    None => out.push(b),
                }
            }
            out
        }
        _ => bytes.to_vec(),
    }
}

/// Sniff a `<meta charset=X>` / `<meta … content="…charset=X">` label in the
/// first 1024 bytes (the HTML5 prescan), returning the canonical name.
fn html_sniff_meta_charset(html: &[u8]) -> Option<&'static [u8]> {
    let window = &html[..html.len().min(1024)];
    let lower = window.to_ascii_lowercase();
    let mut from = 0;
    while let Some(p) = find_sub(&lower[from..], b"<meta") {
        let start = from + p;
        let end = lower[start..].iter().position(|&b| b == b'>').map(|e| start + e)?;
        let tag = &lower[start..end];
        if let Some(cp) = find_sub(tag, b"charset") {
            let mut i = cp + b"charset".len();
            while tag.get(i).is_some_and(|b| b.is_ascii_whitespace()) { i += 1 }
            if tag.get(i) == Some(&b'=') {
                i += 1;
                while tag.get(i).is_some_and(|b| b.is_ascii_whitespace()) { i += 1 }
                let quote = match tag.get(i) {
                    Some(&q @ (b'"' | b'\'')) => { i += 1; Some(q) }
                    _ => None,
                };
                let vs = i;
                while i < tag.len() {
                    let b = tag[i];
                    let stop = match quote {
                        Some(q) => b == q,
                        None => b.is_ascii_whitespace() || b == b';' || b == b'/',
                    };
                    if stop { break }
                    i += 1;
                }
                if let Some(c) = html_encoding_canonical(&tag[vs..i]) {
                    return Some(c);
                }
            }
        }
        from = end + 1;
    }
    None
}

/// Naive substring find (ASCII haystacks from the prescan window).
fn find_sub(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Decode numeric character references plus the core named set; an unmatched
/// `&` stays literal, as the HTML5 tokenizer specifies. (The full 2 200-entry
/// named table is a documented scope-out.)
fn html_decode_entities(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i] == b'&' {
            if let Some((rep, len)) = html_entity_at(&text[i..]) {
                out.extend_from_slice(&rep);
                i += len;
                continue;
            }
        }
        out.push(text[i]);
        i += 1;
    }
    out
}

/// One reference starting at `&`: `(replacement, bytes consumed)`.
fn html_entity_at(s: &[u8]) -> Option<(Vec<u8>, usize)> {
    let end = s.iter().take(34).position(|&b| b == b';')?;
    let body = &s[1..end];
    let cp_bytes = |cp: u32| char::from_u32(cp).unwrap_or('\u{FFFD}').to_string().into_bytes();
    let rep: Vec<u8> = if let Some(hex) = body.strip_prefix(b"#x").or_else(|| body.strip_prefix(b"#X")) {
        cp_bytes(u32::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?)
    } else if let Some(dec) = body.strip_prefix(b"#") {
        cp_bytes(std::str::from_utf8(dec).ok()?.parse().ok()?)
    } else {
        match body {
            b"amp" => b"&".to_vec(),
            b"lt" => b"<".to_vec(),
            b"gt" => b">".to_vec(),
            b"quot" => b"\"".to_vec(),
            b"apos" => b"'".to_vec(),
            b"nbsp" => "\u{A0}".as_bytes().to_vec(),
            _ => return None,
        }
    };
    Some((rep, end + 1))
}

/// Tree-construction state: the implied html/head/body skeleton plus the open
/// element stack for body content.
struct HtmlTree {
    doc: DomDoc,
    html: Option<usize>,
    head: Option<usize>,
    body: Option<usize>,
    /// Open elements under body (body itself excluded).
    stack: Vec<usize>,
    in_body: bool,
}

impl HtmlTree {
    fn ensure_html(&mut self) -> usize {
        match self.html {
            Some(h) => h,
            None => {
                let h = self.doc.push(DomKind::Element { name: b"html".to_vec(), attrs: Vec::new() }, Some(0));
                self.html = Some(h);
                h
            }
        }
    }

    fn ensure_head(&mut self) -> usize {
        let html = self.ensure_html();
        match self.head {
            Some(h) => h,
            None => {
                let h = self.doc.push(DomKind::Element { name: b"head".to_vec(), attrs: Vec::new() }, Some(html));
                self.head = Some(h);
                h
            }
        }
    }

    fn ensure_body(&mut self) -> usize {
        self.ensure_head();
        let html = self.ensure_html();
        match self.body {
            Some(b) => b,
            None => {
                let b = self.doc.push(DomKind::Element { name: b"body".to_vec(), attrs: Vec::new() }, Some(html));
                self.body = Some(b);
                b
            }
        }
    }

    /// Where body-mode content inserts: the innermost open element, else body.
    fn insertion(&mut self) -> usize {
        match self.stack.last() {
            Some(&t) => t,
            None => self.ensure_body(),
        }
    }

    /// Merge attributes onto an existing element, keeping present ones (the
    /// HTML5 rule for repeated `<html>`/`<body>` tags).
    fn merge_attrs(&mut self, el: usize, new_attrs: Vec<(Vec<u8>, Vec<u8>)>) {
        if let DomKind::Element { attrs, .. } = &mut self.doc.nodes[el].kind {
            for (n, v) in new_attrs {
                if !attrs.iter().any(|(en, _)| *en == n) {
                    attrs.push((n, v));
                }
            }
        }
    }

    /// A start tag. Returns the created element (rawtext callers append the
    /// pending text under it).
    fn start_tag(&mut self, name: &[u8], attrs: Vec<(Vec<u8>, Vec<u8>)>) -> usize {
        match name {
            b"html" => {
                let h = self.ensure_html();
                self.merge_attrs(h, attrs);
                h
            }
            b"head" => {
                let h = self.ensure_head();
                self.merge_attrs(h, attrs);
                h
            }
            b"body" => {
                let b = self.ensure_body();
                self.merge_attrs(b, attrs);
                self.in_body = true;
                b
            }
            _ if html_head_tag(name) && !self.in_body => {
                let head = self.ensure_head();
                self.doc.push(DomKind::Element { name: name.to_vec(), attrs }, Some(head))
            }
            _ => {
                self.ensure_body();
                self.in_body = true;
                if html_closes_p(name) {
                    if let Some(pos) = self.stack.iter().rposition(|&e| {
                        matches!(&self.doc.nodes[e].kind, DomKind::Element { name, .. } if name.as_slice() == b"p")
                    }) {
                        self.stack.truncate(pos);
                    }
                }
                // `<li><li>`, `<dt><dd>`, `<tr><tr>`, `<td><th>`… auto-close
                // their open counterpart.
                let group: &[&[u8]] = match name {
                    b"li" => &[b"li"],
                    b"dt" | b"dd" => &[b"dt", b"dd"],
                    b"tr" => &[b"tr", b"td", b"th"],
                    b"td" | b"th" => &[b"td", b"th"],
                    b"option" | b"optgroup" => &[b"option"],
                    _ => &[],
                };
                if !group.is_empty() {
                    if let Some(pos) = self.stack.iter().rposition(|&e| {
                        matches!(&self.doc.nodes[e].kind, DomKind::Element { name, .. } if group.contains(&name.as_slice()))
                    }) {
                        self.stack.truncate(pos);
                    }
                }
                let parent = self.insertion();
                let id = self.doc.push(DomKind::Element { name: name.to_vec(), attrs }, Some(parent));
                if !html_void(name) {
                    self.stack.push(id);
                }
                id
            }
        }
    }

    fn end_tag(&mut self, name: &[u8]) {
        if matches!(name, b"html" | b"body" | b"head") {
            return;
        }
        if let Some(pos) = self.stack.iter().rposition(|&e| {
            matches!(&self.doc.nodes[e].kind, DomKind::Element { name: n, .. } if n.as_slice() == name)
        }) {
            self.stack.truncate(pos);
        }
    }

    fn text(&mut self, decoded: Vec<u8>) {
        if !self.in_body && self.stack.is_empty() {
            // Pre-body whitespace is dropped by the before-head insertion
            // modes; the first non-blank character opens the body.
            let start = decoded.iter().position(|b| !b.is_ascii_whitespace());
            let Some(start) = start else { return };
            self.ensure_body();
            self.in_body = true;
            let parent = self.insertion();
            self.doc.push_text(decoded[start..].to_vec(), parent);
            return;
        }
        let parent = self.insertion();
        self.doc.push_text(decoded, parent);
    }

    fn comment(&mut self, data: Vec<u8>) {
        let parent = if self.in_body || !self.stack.is_empty() {
            self.insertion()
        } else if self.head.is_some() {
            self.head.unwrap()
        } else if self.html.is_some() {
            self.html.unwrap()
        } else {
            0
        };
        self.doc.push(DomKind::Comment(data), Some(parent));
    }
}

impl DomDoc {
    /// Parse `html` (already UTF-8) into an implied-structure tree. Lenient:
    /// never fails (HTML5 defines error recovery for every input).
    pub(super) fn parse_html(html: &[u8]) -> DomDoc {
        let mut t = HtmlTree {
            doc: DomDoc::new(),
            html: None,
            head: None,
            body: None,
            stack: Vec::new(),
            in_body: false,
        };
        let s = html;
        let mut i = 0;
        let mut saw_doctype = false;
        while i < s.len() {
            if s[i] != b'<' {
                let end = s[i..].iter().position(|&b| b == b'<').map_or(s.len(), |p| i + p);
                t.text(html_decode_entities(&s[i..end]));
                i = end;
                continue;
            }
            let rest = &s[i + 1..];
            if rest.starts_with(b"!--") {
                // Comment: data up to `-->` (unterminated runs to EOF).
                let (data, consumed) = match find_sub(&rest[3..], b"-->") {
                    Some(p) => (&rest[3..3 + p], 1 + 3 + p + 3),
                    None => (&rest[3..], 1 + rest.len()),
                };
                t.comment(data.to_vec());
                i += consumed;
            } else if rest.first() == Some(&b'!') {
                let close = rest.iter().position(|&b| b == b'>').map_or(rest.len(), |p| p + 1);
                let body = &rest[1..close.saturating_sub(1).max(1)];
                if body.len() >= 7 && body[..7].eq_ignore_ascii_case(b"doctype") {
                    if !saw_doctype && t.html.is_none() {
                        let name = body[7..]
                            .split(|b| b.is_ascii_whitespace())
                            .find(|w| !w.is_empty())
                            .unwrap_or(b"html")
                            .to_ascii_lowercase();
                        t.doc.push(DomKind::DocType { name }, Some(0));
                        saw_doctype = true;
                    }
                } else {
                    // `<!…>` otherwise (incl. `<![CDATA[…]]>` in HTML) is a
                    // bogus comment holding everything after `<!`.
                    t.comment(body.to_vec());
                }
                i += 1 + close;
            } else if rest.first() == Some(&b'?') {
                // `<?…>` is a bogus comment in HTML (data includes the `?`).
                let close = rest.iter().position(|&b| b == b'>').map_or(rest.len(), |p| p + 1);
                t.comment(rest[..close.saturating_sub(1)].to_vec());
                i += 1 + close;
            } else if rest.first() == Some(&b'/') {
                match html_tag_name(&rest[1..]) {
                    Some((name, mut j)) => {
                        // Skip to `>` (end tags carry no attributes).
                        while rest.get(1 + j).is_some_and(|&b| b != b'>') { j += 1 }
                        t.end_tag(&name);
                        i += 1 + 1 + j + 1;
                    }
                    None => {
                        // `</>` or `</3…>`: bogus comment per the spec.
                        let close = rest.iter().position(|&b| b == b'>').map_or(rest.len(), |p| p + 1);
                        t.comment(rest[1..close.saturating_sub(1)].to_vec());
                        i += 1 + close;
                    }
                }
            } else if rest.first().is_some_and(|b| b.is_ascii_alphabetic()) {
                let (name, mut j) = html_tag_name(rest).expect("alphabetic start");
                let mut attrs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
                // Attribute list up to `>`.
                loop {
                    while rest.get(j).is_some_and(|b| b.is_ascii_whitespace()) { j += 1 }
                    match rest.get(j) {
                        None => break,
                        Some(b'>') => { j += 1; break }
                        Some(b'/') => { j += 1; continue }
                        _ => {}
                    }
                    let an_start = j;
                    while rest.get(j).is_some_and(|&b| {
                        !b.is_ascii_whitespace() && b != b'=' && b != b'>' && b != b'/'
                    }) { j += 1 }
                    let an = rest[an_start..j].to_ascii_lowercase();
                    while rest.get(j).is_some_and(|b| b.is_ascii_whitespace()) { j += 1 }
                    let av: Vec<u8> = if rest.get(j) == Some(&b'=') {
                        j += 1;
                        while rest.get(j).is_some_and(|b| b.is_ascii_whitespace()) { j += 1 }
                        match rest.get(j) {
                            Some(&q @ (b'"' | b'\'')) => {
                                j += 1;
                                let v_start = j;
                                while rest.get(j).is_some_and(|&b| b != q) { j += 1 }
                                let v = &rest[v_start..j.min(rest.len())];
                                if rest.get(j).is_some() { j += 1 }
                                html_decode_entities(v)
                            }
                            _ => {
                                let v_start = j;
                                while rest.get(j).is_some_and(|&b| !b.is_ascii_whitespace() && b != b'>') { j += 1 }
                                html_decode_entities(&rest[v_start..j])
                            }
                        }
                    } else {
                        Vec::new()
                    };
                    if !an.is_empty() && !attrs.iter().any(|(n, _)| *n == an) {
                        attrs.push((an, av));
                    }
                    if rest.get(j) == Some(&b'>') { j += 1; break }
                }
                let el = t.start_tag(&name, attrs);
                i += 1 + j;
                if let Some(decode) = html_rawtext(&name) {
                    // Raw text runs to the matching case-insensitive end tag.
                    let mut close_pat = Vec::with_capacity(name.len() + 2);
                    close_pat.extend_from_slice(b"</");
                    close_pat.extend_from_slice(&name);
                    let lower = s[i..].to_ascii_lowercase();
                    let (raw_end, after) = match find_sub(&lower, &close_pat) {
                        Some(p) => {
                            let gt = lower[p..].iter().position(|&b| b == b'>').map_or(lower.len() - p, |g| g + 1);
                            (i + p, i + p + gt)
                        }
                        None => (s.len(), s.len()),
                    };
                    let mut raw = s[i..raw_end].to_vec();
                    // `<pre>`/`<textarea>` drop one leading newline (§13.1.2.5
                    // covers pre; the tree builder covers textarea).
                    if name == b"textarea" && raw.first() == Some(&b'\n') {
                        raw.remove(0);
                    }
                    if decode {
                        raw = html_decode_entities(&raw);
                    }
                    if !raw.is_empty() {
                        t.doc.push_text(raw, el);
                    }
                    // Rawtext elements never join the open stack, but
                    // start_tag pushed non-void names: pop it back off.
                    if t.stack.last() == Some(&el) {
                        t.stack.pop();
                    }
                    i = after;
                } else if name == b"pre" {
                    if s.get(i) == Some(&b'\n') {
                        i += 1;
                    }
                }
            } else {
                // A lone `<` (not a tag): literal text.
                t.text(b"<".to_vec());
                i += 1;
            }
        }
        t.ensure_body();
        t.doc
    }
}

/// A tag name at the start of `s`: lowercased name + index just past it.
fn html_tag_name(s: &[u8]) -> Option<(Vec<u8>, usize)> {
    if !s.first().is_some_and(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    let mut j = 1;
    while s.get(j).is_some_and(|&b| !b.is_ascii_whitespace() && b != b'>' && b != b'/') {
        j += 1;
    }
    Some((s[..j].to_ascii_lowercase(), j))
}

/// libxml text/attribute escaping: `&`, `<`, `>` always; `"` (as `&quot;`) and
/// the CR entity only inside attribute values.
fn escape_into(data: &[u8], in_attr: bool, out: &mut Vec<u8>) {
    for &b in data {
        match b {
            b'&' => out.extend_from_slice(b"&amp;"),
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'"' if in_attr => out.extend_from_slice(b"&quot;"),
            b'\r' => out.extend_from_slice(b"&#13;"),
            _ => out.push(b),
        }
    }
}

// ----- XPath 1.0 subset -----

/// An XPath result item: a tree node or an attribute of one.
#[derive(Clone, PartialEq)]
pub(super) enum XItem {
    Node(usize),
    Attr(usize, Vec<u8>),
}

/// An XPath value (node-set or scalar), for `evaluate()`.
pub(super) enum XVal {
    Nodes(Vec<XItem>),
    Str(Vec<u8>),
    Num(f64),
    Bool(bool),
}

/// One location step: axis + node test.
#[derive(Clone)]
enum Axis {
    Child,
    Descendant,
    DescendantOrSelf,
    Attribute,
    SelfAxis,
    Parent,
}

#[derive(Clone)]
enum NodeTest {
    Name(Vec<u8>),
    Any,
    Text,
    Comment,
    NodeAny,
}

/// Evaluate `expr` against `ctx` (document node when the PHP side passed none).
/// `ns` maps registered prefixes to namespace URIs (DOMXPath::registerNamespace).
/// Returns `Err(msg)` on a syntax error (PHP raises a warning + `false`).
pub(super) fn xpath_eval(
    doc: &DomDoc,
    ctx: usize,
    expr: &[u8],
    ns: &[(Vec<u8>, Vec<u8>)],
) -> Result<XVal, String> {
    let mut p = Parser { src: expr, pos: 0, doc, ns };
    let v = p.parse_or()?;
    p.skip_ws();
    if p.pos != p.src.len() {
        return Err(format!("Invalid expression near offset {}", p.pos));
    }
    p.eval_expr(&v, &[XItem::Node(ctx)], 0, 1)
}

/// Parsed expression tree (tiny XPath core).
enum Expr {
    Path { absolute: bool, from_root_descendant: bool, steps: Vec<(Axis, NodeTest, Vec<Expr>)> },
    Union(Vec<Expr>),
    Literal(Vec<u8>),
    Number(f64),
    Fn(Vec<u8>, Vec<Expr>),
    Cmp(Box<Expr>, Vec<u8>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
    doc: &'a DomDoc,
    ns: &'a [(Vec<u8>, Vec<u8>)],
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.src.get(self.pos).is_some_and(|b| b.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn eat(&mut self, s: &[u8]) -> bool {
        self.skip_ws();
        if self.src[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.src.get(self.pos).copied()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        loop {
            let save = self.pos;
            if self.eat(b"or")
                && !self.src.get(self.pos).is_some_and(|b| b.is_ascii_alphanumeric())
            {
                let right = self.parse_and()?;
                left = Expr::Or(Box::new(left), Box::new(right));
            } else {
                self.pos = save;
                return Ok(left);
            }
        }
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_cmp()?;
        loop {
            let save = self.pos;
            if self.eat(b"and")
                && !self.src.get(self.pos).is_some_and(|b| b.is_ascii_alphanumeric())
            {
                let right = self.parse_cmp()?;
                left = Expr::And(Box::new(left), Box::new(right));
            } else {
                self.pos = save;
                return Ok(left);
            }
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, String> {
        let left = self.parse_union()?;
        for op in [&b"!="[..], b"<=", b">=", b"=", b"<", b">"] {
            let save = self.pos;
            if self.eat(op) {
                let right = self.parse_union()?;
                return Ok(Expr::Cmp(Box::new(left), op.to_vec(), Box::new(right)));
            }
            self.pos = save;
        }
        Ok(left)
    }

    fn parse_union(&mut self) -> Result<Expr, String> {
        let first = self.parse_primary()?;
        let mut parts = vec![first];
        while self.eat(b"|") {
            parts.push(self.parse_primary()?);
        }
        if parts.len() == 1 {
            Ok(parts.pop().unwrap())
        } else {
            Ok(Expr::Union(parts))
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(b'\'') | Some(b'"') => {
                let q = self.src[self.pos];
                self.pos += 1;
                let start = self.pos;
                while self.pos < self.src.len() && self.src[self.pos] != q {
                    self.pos += 1;
                }
                let lit = self.src[start..self.pos].to_vec();
                self.pos += 1; // closing quote
                Ok(Expr::Literal(lit))
            }
            Some(b) if b.is_ascii_digit() => {
                let start = self.pos;
                while self
                    .src
                    .get(self.pos)
                    .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
                {
                    self.pos += 1;
                }
                let s = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
                Ok(Expr::Number(s.parse().map_err(|_| "bad number".to_string())?))
            }
            Some(b'(') => {
                self.pos += 1;
                let e = self.parse_or()?;
                if !self.eat(b")") {
                    return Err("expected )".to_string());
                }
                Ok(e)
            }
            Some(b'-') => {
                self.pos += 1;
                let e = self.parse_primary()?;
                Ok(Expr::Neg(Box::new(e)))
            }
            _ => {
                // A function call `name(args…)` — or a location path.
                let save = self.pos;
                if let Some(name) = self.try_ident() {
                    self.skip_ws();
                    if self.src.get(self.pos) == Some(&b'(')
                        && !matches!(&name[..], b"node" | b"text" | b"comment")
                    {
                        self.pos += 1;
                        let mut args = Vec::new();
                        if self.peek() != Some(b')') {
                            loop {
                                args.push(self.parse_or()?);
                                if !self.eat(b",") {
                                    break;
                                }
                            }
                        }
                        if !self.eat(b")") {
                            return Err("expected )".to_string());
                        }
                        return Ok(Expr::Fn(name, args));
                    }
                    self.pos = save;
                }
                self.parse_path()
            }
        }
    }

    /// One identifier (with `:`/`-`/`_`/`.` continuation, XPath QName-ish).
    fn try_ident(&mut self) -> Option<Vec<u8>> {
        self.skip_ws();
        let start = self.pos;
        let first = self.src.get(self.pos)?;
        if !(first.is_ascii_alphabetic() || *first == b'_') {
            return None;
        }
        while self.src.get(self.pos).is_some_and(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':')
        }) {
            self.pos += 1;
        }
        Some(self.src[start..self.pos].to_vec())
    }

    fn parse_path(&mut self) -> Result<Expr, String> {
        let mut absolute = false;
        let mut from_root_descendant = false;
        if self.eat(b"//") {
            absolute = true;
            from_root_descendant = true;
        } else if self.eat(b"/") {
            absolute = true;
        }
        let mut steps = Vec::new();
        loop {
            let step = self.parse_step()?;
            match step {
                Some(s) => steps.push(s),
                None => {
                    if steps.is_empty() && !absolute {
                        return Err("expected path step".to_string());
                    }
                    break;
                }
            }
            if self.eat(b"//") {
                steps.push((Axis::DescendantOrSelf, NodeTest::NodeAny, Vec::new()));
            } else if !self.eat(b"/") {
                break;
            }
        }
        Ok(Expr::Path { absolute, from_root_descendant, steps })
    }

    fn parse_step(&mut self) -> Result<Option<(Axis, NodeTest, Vec<Expr>)>, String> {
        self.skip_ws();
        let mut axis = Axis::Child;
        if self.eat(b"..") {
            return Ok(Some((Axis::Parent, NodeTest::NodeAny, self.parse_predicates()?)));
        }
        if self.src.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            return Ok(Some((Axis::SelfAxis, NodeTest::NodeAny, self.parse_predicates()?)));
        }
        if self.eat(b"@") {
            axis = Axis::Attribute;
        } else {
            let save = self.pos;
            if let Some(id) = self.try_ident() {
                if self.eat(b"::") {
                    axis = match &id[..] {
                        b"child" => Axis::Child,
                        b"descendant" => Axis::Descendant,
                        b"descendant-or-self" => Axis::DescendantOrSelf,
                        b"attribute" => Axis::Attribute,
                        b"self" => Axis::SelfAxis,
                        b"parent" => Axis::Parent,
                        other => {
                            return Err(format!(
                                "unsupported axis {}",
                                String::from_utf8_lossy(other)
                            ))
                        }
                    };
                } else {
                    self.pos = save;
                }
            }
        }
        // Node test.
        let test = if self.eat(b"*") {
            NodeTest::Any
        } else if let Some(name) = self.try_ident() {
            self.skip_ws();
            if self.src.get(self.pos) == Some(&b'(') {
                self.pos += 1;
                if !self.eat(b")") {
                    return Err("expected )".to_string());
                }
                match &name[..] {
                    b"text" => NodeTest::Text,
                    b"comment" => NodeTest::Comment,
                    b"node" => NodeTest::NodeAny,
                    other => {
                        return Err(format!(
                            "unsupported node test {}",
                            String::from_utf8_lossy(other)
                        ))
                    }
                }
            } else {
                NodeTest::Name(name)
            }
        } else {
            return Ok(None);
        };
        Ok(Some((axis, test, self.parse_predicates()?)))
    }

    fn parse_predicates(&mut self) -> Result<Vec<Expr>, String> {
        let mut preds = Vec::new();
        while self.eat(b"[") {
            preds.push(self.parse_or()?);
            if !self.eat(b"]") {
                return Err("expected ]".to_string());
            }
        }
        Ok(preds)
    }

    // ----- evaluation -----

    fn eval_expr(
        &self,
        e: &Expr,
        ctx: &[XItem],
        pos: usize,
        size: usize,
    ) -> Result<XVal, String> {
        match e {
            Expr::Literal(s) => Ok(XVal::Str(s.clone())),
            Expr::Number(n) => Ok(XVal::Num(*n)),
            Expr::Neg(inner) => {
                let v = self.eval_expr(inner, ctx, pos, size)?;
                Ok(XVal::Num(-self.to_num(&v)))
            }
            Expr::Or(a, b) => {
                let va = self.eval_expr(a, ctx, pos, size)?;
                if self.to_bool(&va) {
                    return Ok(XVal::Bool(true));
                }
                let vb = self.eval_expr(b, ctx, pos, size)?;
                Ok(XVal::Bool(self.to_bool(&vb)))
            }
            Expr::And(a, b) => {
                let va = self.eval_expr(a, ctx, pos, size)?;
                if !self.to_bool(&va) {
                    return Ok(XVal::Bool(false));
                }
                let vb = self.eval_expr(b, ctx, pos, size)?;
                Ok(XVal::Bool(self.to_bool(&vb)))
            }
            Expr::Cmp(a, op, b) => {
                let va = self.eval_expr(a, ctx, pos, size)?;
                let vb = self.eval_expr(b, ctx, pos, size)?;
                Ok(XVal::Bool(self.compare(&va, op, &vb)))
            }
            Expr::Union(parts) => {
                let mut all: Vec<XItem> = Vec::new();
                for p in parts {
                    if let XVal::Nodes(ns) = self.eval_expr(p, ctx, pos, size)? {
                        for n in ns {
                            if !all.contains(&n) {
                                all.push(n);
                            }
                        }
                    }
                }
                Ok(XVal::Nodes(all))
            }
            Expr::Fn(name, args) => self.eval_fn(name, args, ctx, pos, size),
            Expr::Path { .. } => {
                let cur = ctx.get(pos).cloned().into_iter().collect::<Vec<_>>();
                Ok(XVal::Nodes(self.eval_path(e, &cur)?))
            }
        }
    }

    fn eval_path(&self, e: &Expr, ctx: &[XItem]) -> Result<Vec<XItem>, String> {
        let Expr::Path { absolute, from_root_descendant, steps } = e else {
            return Err("not a path".to_string());
        };
        let mut current: Vec<XItem> = if *absolute {
            vec![XItem::Node(0)]
        } else {
            ctx.to_vec()
        };
        if *from_root_descendant {
            current = self.apply_step(
                &current,
                &(Axis::DescendantOrSelf, NodeTest::NodeAny, Vec::new()),
            )?;
        }
        for step in steps {
            current = self.apply_step(&current, step)?;
        }
        Ok(current)
    }

    fn apply_step(
        &self,
        ctx: &[XItem],
        step: &(Axis, NodeTest, Vec<Expr>),
    ) -> Result<Vec<XItem>, String> {
        let (axis, test, preds) = step;
        let mut out: Vec<XItem> = Vec::new();
        for item in ctx {
            let mut cand: Vec<XItem> = Vec::new();
            match (axis, item) {
                (Axis::Child, XItem::Node(n)) => {
                    cand.extend(self.doc.nodes[*n].children.iter().map(|&c| XItem::Node(c)));
                }
                (Axis::Descendant, XItem::Node(n)) => self.descendants(*n, false, &mut cand),
                (Axis::DescendantOrSelf, XItem::Node(n)) => {
                    self.descendants(*n, true, &mut cand)
                }
                (Axis::Attribute, XItem::Node(n)) => {
                    if let DomKind::Element { attrs, .. } = &self.doc.nodes[*n].kind {
                        cand.extend(attrs.iter().map(|(k, _)| XItem::Attr(*n, k.clone())));
                    }
                }
                (Axis::SelfAxis, it) => cand.push(it.clone()),
                (Axis::Parent, XItem::Node(n)) => {
                    if let Some(p) = self.doc.nodes[*n].parent {
                        cand.push(XItem::Node(p));
                    }
                }
                (Axis::Parent, XItem::Attr(n, _)) => cand.push(XItem::Node(*n)),
                _ => {}
            }
            // Node test.
            cand.retain(|it| self.test_matches(it, test));
            // Predicates, with position()/last() context per candidate list.
            for pred in preds {
                let size = cand.len();
                let mut kept = Vec::new();
                for (i, it) in cand.iter().enumerate() {
                    let v = self.eval_expr(pred, std::slice::from_ref(it), 0, size)?;
                    let keep = match v {
                        // A numeric predicate selects by 1-based position.
                        XVal::Num(n) => (i + 1) as f64 == n,
                        other => self.pred_bool_at(&other, i, size),
                    };
                    if keep {
                        kept.push(it.clone());
                    }
                }
                cand = kept;
            }
            for it in cand {
                if !out.contains(&it) {
                    out.push(it);
                }
            }
        }
        Ok(out)
    }

    /// position()/last() need the index inside the candidate list; scalar
    /// predicates reduce to a boolean.
    fn pred_bool_at(&self, v: &XVal, _i: usize, _size: usize) -> bool {
        self.to_bool(v)
    }

    fn descendants(&self, n: usize, include_self: bool, out: &mut Vec<XItem>) {
        if include_self {
            out.push(XItem::Node(n));
        }
        for &c in &self.doc.nodes[n].children {
            self.descendants(c, true, out);
        }
    }

    fn test_matches(&self, item: &XItem, test: &NodeTest) -> bool {
        match (item, test) {
            (_, NodeTest::NodeAny) => true,
            (XItem::Attr(_, name), NodeTest::Name(want)) => {
                self.qname_matches(None, name, want)
            }
            (XItem::Attr(..), NodeTest::Any) => true,
            (XItem::Attr(..), _) => false,
            (XItem::Node(n), t) => match (&self.doc.nodes[*n].kind, t) {
                (DomKind::Element { .. }, NodeTest::Any) => true,
                (DomKind::Element { name, .. }, NodeTest::Name(want)) => {
                    self.qname_matches(Some(*n), name, want)
                }
                (DomKind::Text(_) | DomKind::Cdata(_), NodeTest::Text) => true,
                (DomKind::Comment(_), NodeTest::Comment) => true,
                _ => false,
            },
        }
    }

    /// Match a node's qualified `name` against the test `want`. A prefixed test
    /// (`p:local`) resolves `p` through the registered namespaces and matches
    /// elements whose in-scope namespace has that URI and whose local name
    /// matches; an unprefixed test matches by full qualified name (like PHP
    /// with no registered default).
    fn qname_matches(&self, node: Option<usize>, name: &[u8], want: &[u8]) -> bool {
        match want.iter().position(|&b| b == b':') {
            None => name == want,
            Some(ci) => {
                let (wprefix, wlocal) = (&want[..ci], &want[ci + 1..]);
                let Some((_, want_uri)) = self.ns.iter().find(|(p, _)| p == wprefix) else {
                    return name == want;
                };
                let (nprefix, nlocal) = match name.iter().position(|&b| b == b':') {
                    Some(i) => (&name[..i], &name[i + 1..]),
                    None => (&name[..0], &name[..]),
                };
                if nlocal != wlocal {
                    return false;
                }
                match node {
                    Some(n) => {
                        self.doc.resolve_ns(n, nprefix).as_deref() == Some(&want_uri[..])
                    }
                    None => nprefix == wprefix,
                }
            }
        }
    }

    fn eval_fn(
        &self,
        name: &[u8],
        args: &[Expr],
        ctx: &[XItem],
        pos: usize,
        size: usize,
    ) -> Result<XVal, String> {
        let arg = |i: usize| -> Result<XVal, String> {
            self.eval_expr(&args[i], ctx, pos, size)
        };
        match name {
            b"last" => Ok(XVal::Num(size as f64)),
            b"position" => Ok(XVal::Num((pos + 1) as f64)),
            b"count" => {
                let v = arg(0)?;
                match v {
                    XVal::Nodes(ns) => Ok(XVal::Num(ns.len() as f64)),
                    _ => Err("count() expects a node-set".to_string()),
                }
            }
            b"not" => {
                let v = arg(0)?;
                Ok(XVal::Bool(!self.to_bool(&v)))
            }
            b"true" => Ok(XVal::Bool(true)),
            b"false" => Ok(XVal::Bool(false)),
            b"string" => {
                if args.is_empty() {
                    Ok(XVal::Str(self.item_string(ctx.get(pos))))
                } else {
                    let v = arg(0)?;
                    Ok(XVal::Str(self.to_str(&v)))
                }
            }
            b"number" => {
                let v = arg(0)?;
                Ok(XVal::Num(self.to_num(&v)))
            }
            b"boolean" => {
                let v = arg(0)?;
                Ok(XVal::Bool(self.to_bool(&v)))
            }
            b"concat" => {
                let mut out = Vec::new();
                for i in 0..args.len() {
                    out.extend_from_slice(&self.to_str(&arg(i)?));
                }
                Ok(XVal::Str(out))
            }
            b"contains" => {
                let hay = self.to_str(&arg(0)?);
                let needle = self.to_str(&arg(1)?);
                Ok(XVal::Bool(
                    needle.is_empty()
                        || hay.windows(needle.len().max(1)).any(|w| w == &needle[..]),
                ))
            }
            b"starts-with" => {
                let hay = self.to_str(&arg(0)?);
                let needle = self.to_str(&arg(1)?);
                Ok(XVal::Bool(hay.starts_with(&needle[..])))
            }
            b"string-length" => {
                let s = if args.is_empty() {
                    self.item_string(ctx.get(pos))
                } else {
                    self.to_str(&arg(0)?)
                };
                Ok(XVal::Num(String::from_utf8_lossy(&s).chars().count() as f64))
            }
            b"normalize-space" => {
                let s = if args.is_empty() {
                    self.item_string(ctx.get(pos))
                } else {
                    self.to_str(&arg(0)?)
                };
                let text = String::from_utf8_lossy(&s).into_owned();
                let norm = text.split_whitespace().collect::<Vec<_>>().join(" ");
                Ok(XVal::Str(norm.into_bytes()))
            }
            b"name" | b"local-name" => {
                let it = if args.is_empty() {
                    ctx.get(pos).cloned()
                } else {
                    match arg(0)? {
                        XVal::Nodes(ns) => ns.first().cloned(),
                        _ => None,
                    }
                };
                let full = match &it {
                    Some(XItem::Node(n)) => self.doc.nodes[*n].kind.node_name(),
                    Some(XItem::Attr(_, a)) => a.clone(),
                    None => Vec::new(),
                };
                let out = if name == b"local-name" {
                    match full.iter().position(|&b| b == b':') {
                        Some(i) => full[i + 1..].to_vec(),
                        None => full,
                    }
                } else {
                    full
                };
                Ok(XVal::Str(out))
            }
            other => Err(format!(
                "unsupported XPath function {}()",
                String::from_utf8_lossy(other)
            )),
        }
    }

    fn item_string(&self, it: Option<&XItem>) -> Vec<u8> {
        match it {
            Some(XItem::Node(n)) => self.doc.text_content(*n),
            Some(XItem::Attr(n, a)) => {
                self.doc.attr(*n, a).map(|v| v.to_vec()).unwrap_or_default()
            }
            None => Vec::new(),
        }
    }

    fn to_str(&self, v: &XVal) -> Vec<u8> {
        match v {
            XVal::Str(s) => s.clone(),
            XVal::Num(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64).into_bytes()
                } else {
                    format!("{n}").into_bytes()
                }
            }
            XVal::Bool(b) => if *b { b"true".to_vec() } else { b"false".to_vec() },
            XVal::Nodes(ns) => self.item_string(ns.first()),
        }
    }

    fn to_num(&self, v: &XVal) -> f64 {
        match v {
            XVal::Num(n) => *n,
            XVal::Bool(b) => *b as u8 as f64,
            other => {
                let s = self.to_str(other);
                String::from_utf8_lossy(&s).trim().parse().unwrap_or(f64::NAN)
            }
        }
    }

    fn to_bool(&self, v: &XVal) -> bool {
        match v {
            XVal::Bool(b) => *b,
            XVal::Num(n) => *n != 0.0 && !n.is_nan(),
            XVal::Str(s) => !s.is_empty(),
            XVal::Nodes(ns) => !ns.is_empty(),
        }
    }

    fn compare(&self, a: &XVal, op: &[u8], b: &XVal) -> bool {
        // Node-set comparison: true if ANY node's string satisfies the relation.
        if let XVal::Nodes(ns) = a {
            return ns.iter().any(|it| {
                let s = XVal::Str(self.item_string(Some(it)));
                self.compare(&s, op, b)
            });
        }
        if let XVal::Nodes(ns) = b {
            return ns.iter().any(|it| {
                let s = XVal::Str(self.item_string(Some(it)));
                self.compare(a, op, &s)
            });
        }
        match op {
            b"=" | b"!=" => {
                let eq = match (a, b) {
                    (XVal::Num(_), _) | (_, XVal::Num(_)) => self.to_num(a) == self.to_num(b),
                    (XVal::Bool(_), _) | (_, XVal::Bool(_)) => {
                        self.to_bool(a) == self.to_bool(b)
                    }
                    _ => self.to_str(a) == self.to_str(b),
                };
                if op == b"=" {
                    eq
                } else {
                    !eq
                }
            }
            _ => {
                let (x, y) = (self.to_num(a), self.to_num(b));
                match op {
                    b"<" => x < y,
                    b"<=" => x <= y,
                    b">" => x > y,
                    b">=" => x >= y,
                    _ => false,
                }
            }
        }
    }
}

/// Build the PHP-facing array for one XPath item: `[0 => kind, 1 => nodeId,
/// 2 => attrName?]` (`kind` "n" or "a"), decoded by the prelude's wrap factory.
pub(super) fn xitem_to_zval(it: &XItem) -> Zval {
    let mut arr = PhpArray::new();
    match it {
        XItem::Node(n) => {
            let _ = arr.append(Zval::Str(PhpStr::new(b"n".to_vec())));
            let _ = arr.append(Zval::Long(*n as i64));
        }
        XItem::Attr(n, name) => {
            let _ = arr.append(Zval::Str(PhpStr::new(b"a".to_vec())));
            let _ = arr.append(Zval::Long(*n as i64));
            let _ = arr.append(Zval::Str(PhpStr::new(name.clone())));
        }
    }
    Zval::Array(Rc::new(arr))
}

// ----- Vm host builtins (`__dom_*`, `libxml_*`) -----

use php_types::{convert, Diag, PhpError};

use super::Vm;

impl<'m> Vm<'m> {
    fn dom_doc(&self, args: &[Zval]) -> Result<(&DomDoc, u32), PhpError> {
        let id = args
            .first()
            .map(|a| a.deref_clone())
            .and_then(|v| if let Zval::Long(i) = v { Some(i) } else { None })
            .unwrap_or(-1);
        self.dom_docs
            .get(&(id as u32))
            .map(|d| (d, id as u32))
            .ok_or_else(|| PhpError::Error("invalid DOM document handle".to_string()))
    }

    /// Argument `i` as an i64 (missing → -1).
    pub(super) fn dom_arg(&mut self, args: &[Zval], i: usize) -> i64 {
        match args.get(i) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => -1,
        }
    }

    pub(super) fn dom_str(&mut self, args: &[Zval], i: usize) -> Vec<u8> {
        match args.get(i) {
            Some(a) => convert::to_zstr_cast(&a.deref_clone(), &mut self.diags)
                .as_bytes()
                .to_vec(),
            None => Vec::new(),
        }
    }

    fn dom_node(&self, doc_id: u32, n: i64) -> Result<usize, PhpError> {
        let doc = &self.dom_docs[&doc_id];
        let n = n as usize;
        if n < doc.nodes.len() {
            Ok(n)
        } else {
            Err(PhpError::Error("invalid DOM node handle".to_string()))
        }
    }

    /// `__dom_new_doc(version, encoding) -> docId`.
    pub(super) fn ho_dom_new_doc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut doc = DomDoc::new();
        let version = self.dom_str(&args, 0);
        if !version.is_empty() {
            doc.version = version;
        }
        let enc = self.dom_str(&args, 1);
        if !enc.is_empty() {
            doc.encoding = Some(enc);
        }
        let id = self.next_dom;
        self.next_dom += 1;
        self.dom_docs.insert(id, doc);
        Ok(Zval::Long(id as i64))
    }

    /// `__dom_load(docId, source, isFile) -> bool`: parse into the existing
    /// handle. A parse failure records a libxml error (and raises the PHP
    /// warning unless `libxml_use_internal_errors(true)`).
    pub(super) fn ho_dom_load(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (_, id) = self.dom_doc(&args)?;
        let source = self.dom_str(&args, 1);
        let is_file = self.dom_arg(&args, 2) != 0;
        let (xml, label) = if is_file {
            use std::os::unix::ffi::OsStrExt;
            let path = std::ffi::OsStr::from_bytes(&source);
            match std::fs::read(path) {
                Ok(d) => (d, String::from_utf8_lossy(&source).into_owned()),
                Err(_) => {
                    self.diags.push(Diag::Warning(format!(
                        "DOMDocument::load(): I/O warning : failed to load external entity \"{}\"",
                        String::from_utf8_lossy(&source)
                    )));
                    return Ok(Zval::Bool(false));
                }
            }
        } else {
            (source, "Entity".to_string())
        };
        // PHP: loading the empty string is a ValueError, not a parse warning.
        if xml.is_empty() {
            return Err(PhpError::ValueError(
                "DOMDocument::loadXML(): Argument #1 ($source) must not be empty".to_string(),
            ));
        }
        match DomDoc::parse(&xml) {
            Ok(doc) => {
                self.dom_docs.insert(id, doc);
                Ok(Zval::Bool(true))
            }
            Err(msg) => {
                self.libxml_errors.push(msg.clone());
                if !self.libxml_internal {
                    self.diags.push(Diag::Warning(format!(
                        "DOMDocument::loadXML(): {msg} in {label}, line: 1"
                    )));
                }
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `__dom_load_html(source, overrideEncoding|null) -> docId`: parse an
    /// HTML5 document into a fresh handle (`Dom\HTMLDocument::createFromString`).
    /// An unknown override label is the ValueError PHP raises.
    pub(super) fn ho_dom_load_html(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let source = self.dom_str(&args, 0);
        let override_enc = match args.get(1) {
            None | Some(Zval::Null) => None,
            _ => Some(self.dom_str(&args, 1)),
        };
        let enc: &[u8] = match override_enc {
            Some(label) => html_encoding_canonical(&label).ok_or_else(|| {
                PhpError::ValueError(
                    "Dom\\HTMLDocument::createFromString(): Argument #3 ($overrideEncoding) \
                     must be a valid document encoding"
                        .to_string(),
                )
            })?,
            None => html_sniff_meta_charset(&source).unwrap_or(b"UTF-8"),
        };
        let utf8 = html_to_utf8(&source, enc);
        let mut doc = DomDoc::parse_html(&utf8);
        doc.input_encoding = Some(enc.to_vec());
        let id = self.next_dom;
        self.next_dom += 1;
        self.dom_docs.insert(id, doc);
        Ok(Zval::Long(id as i64))
    }

    /// `__dom_save_xml(docId, nodeId|-1) -> string`.
    pub(super) fn ho_dom_save_xml(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let (doc, _) = self.dom_doc(&args)?;
        let node = if n < 0 { None } else { Some(n as usize) };
        Ok(Zval::Str(PhpStr::new(doc.save_xml(node))))
    }

    /// `__dom_info(docId, nodeId) -> [nodeType, nodeName, nodeValue|null]`.
    pub(super) fn ho_dom_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        let doc = &self.dom_docs[&id];
        let kind = &doc.nodes[n].kind;
        let mut arr = PhpArray::new();
        let _ = arr.append(Zval::Long(kind.node_type()));
        let _ = arr.append(Zval::Str(PhpStr::new(kind.node_name())));
        let _ = arr.append(match kind.node_value() {
            Some(v) => Zval::Str(PhpStr::new(v)),
            None => Zval::Null,
        });
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `__dom_nav(docId, nodeId, which) -> nodeId|-1`: 0 parent, 1 first child,
    /// 2 last child, 3 next sibling, 4 previous sibling.
    pub(super) fn ho_dom_nav(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let which = self.dom_arg(&args, 2);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        let doc = &self.dom_docs[&id];
        let result: Option<usize> = match which {
            0 => doc.nodes[n].parent,
            1 => doc.nodes[n].children.first().copied(),
            2 => doc.nodes[n].children.last().copied(),
            3 | 4 => doc.nodes[n].parent.and_then(|p| {
                let sibs = &doc.nodes[p].children;
                let i = sibs.iter().position(|&c| c == n)?;
                if which == 3 {
                    sibs.get(i + 1).copied()
                } else if i > 0 {
                    sibs.get(i - 1).copied()
                } else {
                    None
                }
            }),
            _ => None,
        };
        Ok(Zval::Long(result.map_or(-1, |x| x as i64)))
    }

    /// `__dom_children(docId, nodeId) -> array of nodeId`.
    pub(super) fn ho_dom_children(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        let doc = &self.dom_docs[&id];
        let mut arr = PhpArray::new();
        for &c in &doc.nodes[n].children {
            let _ = arr.append(Zval::Long(c as i64));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `__dom_ns($doc, $node, $attrName)` → `[namespaceURI|null, prefix, localName|null]`
    /// (oracle-pinned): an element resolves its prefix (or the in-scope default
    /// `xmlns`) up the ancestor chain; an *attribute* (`$attrName !== ''`, resolved
    /// against owner element `$node`) is only in a namespace when prefixed —
    /// `xml`/`xmlns` map to their fixed W3C URIs. `prefix` is always a string
    /// (`""` when none); `localName` is null for non-element/attr nodes.
    pub(super) fn ho_dom_ns(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let attr_name = self.dom_str(&args, 2);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        let doc = &self.dom_docs[&id];
        let split = |qn: &[u8]| -> (Vec<u8>, Vec<u8>) {
            match qn.iter().position(|&b| b == b':') {
                Some(p) => (qn[..p].to_vec(), qn[p + 1..].to_vec()),
                None => (Vec::new(), qn.to_vec()),
            }
        };
        let (uri, prefix, local): (Option<Vec<u8>>, Vec<u8>, Option<Vec<u8>>) =
            if !attr_name.is_empty() {
                let (prefix, local) = split(&attr_name);
                let uri = if prefix.is_empty() {
                    // An unprefixed attribute is never in the default namespace.
                    None
                } else if prefix == b"xml" {
                    Some(b"http://www.w3.org/XML/1998/namespace".to_vec())
                } else if prefix == b"xmlns" {
                    Some(b"http://www.w3.org/2000/xmlns/".to_vec())
                } else {
                    doc.resolve_ns(n, &prefix)
                };
                (uri, prefix, Some(local))
            } else {
                match &doc.nodes[n].kind {
                    DomKind::Element { name, .. } => {
                        let (prefix, local) = split(name);
                        let uri = if prefix == b"xml" {
                            Some(b"http://www.w3.org/XML/1998/namespace".to_vec())
                        } else {
                            // Empty prefix resolves the in-scope default `xmlns`;
                            // an explicit `xmlns=""` un-declares it (null).
                            doc.resolve_ns(n, &prefix).filter(|u| !u.is_empty())
                        };
                        (uri, prefix, Some(local))
                    }
                    _ => (None, Vec::new(), None),
                }
            };
        let mut arr = PhpArray::new();
        let _ = arr.append(match uri {
            Some(u) => Zval::Str(PhpStr::new(u)),
            None => Zval::Null,
        });
        let _ = arr.append(Zval::Str(PhpStr::new(prefix)));
        let _ = arr.append(match local {
            Some(l) => Zval::Str(PhpStr::new(l)),
            None => Zval::Null,
        });
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `__dom_text(docId, nodeId) -> string` (W3C textContent).
    pub(super) fn ho_dom_text(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        Ok(Zval::Str(PhpStr::new(self.dom_docs[&id].text_content(n))))
    }

    /// `__dom_set_value(docId, nodeId, value)`: set nodeValue (character data
    /// nodes) or replace an element/document's children with one text node
    /// (nodeValue/textContent assignment).
    pub(super) fn ho_dom_set_value(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let value = self.dom_str(&args, 2);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        let doc = self.dom_docs.get_mut(&id).unwrap();
        match &mut doc.nodes[n].kind {
            DomKind::Text(d) | DomKind::Cdata(d) | DomKind::Comment(d) => *d = value,
            DomKind::Pi { data, .. } => *data = value,
            _ => {
                for c in std::mem::take(&mut doc.nodes[n].children) {
                    doc.nodes[c].parent = None;
                }
                let t = doc.push(DomKind::Text(value), None);
                doc.nodes[t].parent = Some(n);
                doc.nodes[n].children.push(t);
            }
        }
        Ok(Zval::Bool(true))
    }

    /// `__dom_attr(docId, nodeId, op, name, value)`: op 0 get (string|false),
    /// 1 set, 2 has, 3 remove, 4 names (array).
    pub(super) fn ho_dom_attr(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.dom_arg(&args, 1);
        let op = self.dom_arg(&args, 2);
        let name = self.dom_str(&args, 3);
        let value = self.dom_str(&args, 4);
        let (_, id) = self.dom_doc(&args)?;
        let n = self.dom_node(id, n)?;
        let doc = self.dom_docs.get_mut(&id).unwrap();
        let DomKind::Element { attrs, .. } = &mut doc.nodes[n].kind else {
            return Ok(Zval::Bool(false));
        };
        Ok(match op {
            0 => match attrs.iter().find(|(k, _)| k == &name) {
                Some((_, v)) => Zval::Str(PhpStr::new(v.clone())),
                None => Zval::Bool(false),
            },
            1 => {
                match attrs.iter_mut().find(|(k, _)| k == &name) {
                    Some((_, v)) => *v = value,
                    None => attrs.push((name, value)),
                }
                Zval::Bool(true)
            }
            2 => Zval::Bool(attrs.iter().any(|(k, _)| k == &name)),
            3 => {
                let before = attrs.len();
                attrs.retain(|(k, _)| k != &name);
                Zval::Bool(attrs.len() != before)
            }
            _ => {
                let mut arr = PhpArray::new();
                for (k, _) in attrs.iter() {
                    let _ = arr.append(Zval::Str(PhpStr::new(k.clone())));
                }
                Zval::Array(Rc::new(arr))
            }
        })
    }

    /// `__dom_create(docId, kind, a, b) -> nodeId` (detached). kind: 1 element,
    /// 3 text, 4 cdata, 7 pi(target,data), 8 comment, 11 fragment.
    pub(super) fn ho_dom_create(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let kind_code = self.dom_arg(&args, 1);
        let a = self.dom_str(&args, 2);
        let b = self.dom_str(&args, 3);
        let (_, id) = self.dom_doc(&args)?;
        let kind = match kind_code {
            1 => {
                if a.is_empty()
                    || a.iter().any(|c| c.is_ascii_whitespace() || matches!(c, b'<' | b'>' | b'&'))
                {
                    // DOMDocument::createElement invalid name → DOMException(5).
                    return Ok(Zval::Long(-5));
                }
                DomKind::Element { name: a, attrs: vec![] }
            }
            3 => DomKind::Text(a),
            4 => DomKind::Cdata(a),
            7 => DomKind::Pi { target: a, data: b },
            8 => DomKind::Comment(a),
            _ => DomKind::Fragment,
        };
        let doc = self.dom_docs.get_mut(&id).unwrap();
        let n = doc.push(kind, None);
        Ok(Zval::Long(n as i64))
    }

    /// `__dom_mutate(docId, op, parent, child, ref)`: op 0 append, 1
    /// insertBefore, 2 removeChild. Returns bool (false = hierarchy error the
    /// prelude turns into DOMException).
    pub(super) fn ho_dom_mutate(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let op = self.dom_arg(&args, 1);
        let parent = self.dom_arg(&args, 2);
        let child = self.dom_arg(&args, 3);
        let reference = self.dom_arg(&args, 4);
        let (_, id) = self.dom_doc(&args)?;
        let parent = self.dom_node(id, parent)?;
        let child = self.dom_node(id, child)?;
        // Reject a cycle: `child` must not be `parent` or one of its ancestors.
        let doc = &self.dom_docs[&id];
        let mut cur = Some(parent);
        while let Some(c) = cur {
            if c == child {
                return Ok(Zval::Bool(false));
            }
            cur = doc.nodes[c].parent;
        }
        let doc = self.dom_docs.get_mut(&id).unwrap();
        match op {
            0 => doc.append(parent, child),
            1 => {
                if reference < 0 {
                    doc.append(parent, child);
                } else {
                    let r = reference as usize;
                    doc.insert_before(parent, child, r);
                }
            }
            _ => {
                if doc.nodes[child].parent != Some(parent) {
                    return Ok(Zval::Bool(false));
                }
                doc.detach(child);
            }
        }
        Ok(Zval::Bool(true))
    }

    /// `__dom_copy(dstDoc, srcDoc, nodeId, deep) -> nodeId` (detached clone in
    /// `dstDoc`; same doc = cloneNode, different = importNode).
    pub(super) fn ho_dom_copy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (_, dst) = self.dom_doc(&args)?;
        let src = self.dom_arg(&args, 1) as u32;
        let n = self.dom_arg(&args, 2);
        let deep = self.dom_arg(&args, 3) != 0;
        if !self.dom_docs.contains_key(&src) {
            return Err(PhpError::Error("invalid DOM document handle".to_string()));
        }
        let n = self.dom_node(src, n)?;
        if dst == src {
            let copied = self.dom_docs.get_mut(&dst).unwrap().copy_from_self(n, deep);
            Ok(Zval::Long(copied as i64))
        } else {
            let src_nodes = std::mem::take(&mut self.dom_docs.get_mut(&src).unwrap().nodes);
            let copied = self
                .dom_docs
                .get_mut(&dst)
                .unwrap()
                .copy_from(&src_nodes, n, deep);
            self.dom_docs.get_mut(&src).unwrap().nodes = src_nodes;
            Ok(Zval::Long(copied as i64))
        }
    }

    /// `__dom_doc_element(docId) -> nodeId|-1`.
    pub(super) fn ho_dom_doc_element(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (doc, _) = self.dom_doc(&args)?;
        Ok(Zval::Long(doc.document_element().map_or(-1, |n| n as i64)))
    }

    /// `__dom_by_tag(docId, ctx|-1, name) -> array of nodeId`.
    pub(super) fn ho_dom_by_tag(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ctx = self.dom_arg(&args, 1);
        let name = self.dom_str(&args, 2);
        let (doc, _) = self.dom_doc(&args)?;
        let ctx = if ctx < 0 { 0 } else { ctx as usize };
        let mut out = Vec::new();
        if ctx < doc.nodes.len() {
            doc.elements_by_tag(ctx, &name, &mut out);
        }
        let mut arr = PhpArray::new();
        for n in out {
            let _ = arr.append(Zval::Long(n as i64));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `__dom_xpath(docId, ctx|-1, expr, nsMap, wantScalar)`:
    /// node-set → array of xitem arrays; scalar (evaluate) → the scalar; a
    /// syntax error → warning + false.
    pub(super) fn ho_dom_xpath(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ctx = self.dom_arg(&args, 1);
        let expr = self.dom_str(&args, 2);
        let mut ns: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        if let Some(Zval::Array(a)) = args.get(3).map(|v| v.deref_clone()) {
            for (k, v) in a.iter() {
                if let Key::Str(s) = k {
                    ns.push((
                        s.as_bytes().to_vec(),
                        convert::to_zstr_cast(&v.deref_clone(), &mut self.diags)
                            .as_bytes()
                            .to_vec(),
                    ));
                }
            }
        }
        let (doc, _) = self.dom_doc(&args)?;
        // PHP's DOMXPath::query default context is the document *element*
        // (php.net: "defaults to the root element"), not the document node —
        // PHPUnit's Loader relies on it (`query('testsuites/testsuite')`).
        let ctx = if ctx < 0 { doc.document_element().unwrap_or(0) } else { ctx as usize };
        match xpath_eval(doc, ctx, &expr, &ns) {
            Ok(XVal::Nodes(items)) => {
                let mut arr = PhpArray::new();
                for it in &items {
                    let _ = arr.append(xitem_to_zval(it));
                }
                Ok(Zval::Array(Rc::new(arr)))
            }
            Ok(XVal::Str(s)) => Ok(Zval::Str(PhpStr::new(s))),
            Ok(XVal::Num(n)) => Ok(Zval::Double(n)),
            Ok(XVal::Bool(b)) => Ok(Zval::Bool(b)),
            Err(_) => {
                self.diags.push(Diag::Warning(
                    "DOMXPath::query(): Invalid expression".to_string(),
                ));
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `__dom_doc_meta(docId) -> [xmlVersion, xmlEncoding|null]`.
    pub(super) fn ho_dom_doc_meta(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (doc, _) = self.dom_doc(&args)?;
        let mut arr = PhpArray::new();
        let _ = arr.append(Zval::Str(PhpStr::new(doc.version.clone())));
        let _ = arr.append(match &doc.encoding {
            Some(e) => Zval::Str(PhpStr::new(e.clone())),
            None => Zval::Null,
        });
        let _ = arr.append(match &doc.input_encoding {
            Some(e) => Zval::Str(PhpStr::new(e.clone())),
            None => Zval::Null,
        });
        Ok(Zval::Array(Rc::new(arr)))
    }

    // ----- ext/libxml error surface -----

    pub(super) fn ho_libxml_use_internal_errors(
        &mut self,
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        let prev = self.libxml_internal;
        if let Some(a) = args.first() {
            if !matches!(a, Zval::Null) {
                self.libxml_internal = convert::to_bool(&a.deref_clone(), &mut self.diags);
            }
        }
        Ok(Zval::Bool(prev))
    }

    pub(super) fn ho_libxml_get_errors(&mut self) -> Result<Zval, PhpError> {
        let mut arr = PhpArray::new();
        for msg in &self.libxml_errors {
            let mut e = PhpArray::new();
            e.insert(Key::Str(PhpStr::new(b"level".to_vec())), Zval::Long(3)); // LIBXML_ERR_FATAL
            e.insert(Key::Str(PhpStr::new(b"code".to_vec())), Zval::Long(1));
            e.insert(Key::Str(PhpStr::new(b"column".to_vec())), Zval::Long(0));
            e.insert(
                Key::Str(PhpStr::new(b"message".to_vec())),
                Zval::Str(PhpStr::new(msg.clone().into_bytes())),
            );
            e.insert(
                Key::Str(PhpStr::new(b"file".to_vec())),
                Zval::Str(PhpStr::new(Vec::new())),
            );
            e.insert(Key::Str(PhpStr::new(b"line".to_vec())), Zval::Long(1));
            let _ = arr.append(Zval::Array(Rc::new(e)));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    pub(super) fn ho_libxml_clear_errors(&mut self) -> Result<Zval, PhpError> {
        self.libxml_errors.clear();
        Ok(Zval::Null)
    }
}

impl DomDoc {
    /// `cloneNode` within the same arena (avoids aliasing `copy_from`).
    fn copy_from_self(&mut self, src: usize, deep: bool) -> usize {
        let snapshot: Vec<DomNode> = self
            .nodes
            .iter()
            .map(|n| DomNode {
                kind: match &n.kind {
                    DomKind::Document => DomKind::Document,
                    DomKind::Element { name, attrs } => {
                        DomKind::Element { name: name.clone(), attrs: attrs.clone() }
                    }
                    DomKind::Text(d) => DomKind::Text(d.clone()),
                    DomKind::Cdata(d) => DomKind::Cdata(d.clone()),
                    DomKind::Comment(d) => DomKind::Comment(d.clone()),
                    DomKind::Pi { target, data } => {
                        DomKind::Pi { target: target.clone(), data: data.clone() }
                    }
                    DomKind::DocType { name } => DomKind::DocType { name: name.clone() },
                    DomKind::Fragment => DomKind::Fragment,
                },
                parent: n.parent,
                children: n.children.clone(),
            })
            .collect();
        self.copy_from(&snapshot, src, deep)
    }
}
