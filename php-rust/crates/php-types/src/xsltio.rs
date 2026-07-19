//! ext/xsl over the **system libxslt/libexslt** (`/usr/lib/libxslt.1.dylib`,
//! the very dylibs the PHP oracle links — see `otool -L php`): thin FFI, so
//! stylesheet compilation, the transform itself and the result serialization
//! all run through the *same* C code and the produced bytes are identical.
//! Mirrors the zlibio/gdio pattern: the FFI lives once, in the bottom crate;
//! the stylesheet handle table and the `__xslt_*` host builtins live in
//! vm/xslt.rs.
//!
//! Interchange is by *serialized document*: phpr's DOM is its own tree, so
//! the VM hands us the `saveXML()` bytes of the stylesheet/source documents
//! and we re-parse them with the system libxml2 (`/usr/lib/libxml2.2.dylib`,
//! also what the oracle links). The transform depends on the parsed infoset,
//! not on the serialized form, so the round-trip is invisible to the result.
//!
//! libxslt reports problems through the variadic `xsltGenericError` callback,
//! which stable Rust cannot implement; instead we install the *default*
//! handler with an `open_memstream(3)` FILE* as its context (the default
//! vfprintf's into the context), then split the captured stream on newlines —
//! the same message granularity PHP's buffering error handler produces.

use std::cell::Cell;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

type XmlDocPtr = *mut c_void;
type XsltStylesheetPtr = *mut c_void;
type XsltCtxtPtr = *mut c_void;
type XsltSecurityPrefsPtr = *mut c_void;

// xmlReadMemory options (libxml2 parser.h). The stylesheet is parsed the way
// ext/xsl's importStylesheet sets the loader up (substitute entities, load
// external DTD subsets, default attributes); NOERROR/NOWARNING because the
// document was already parsed once by the DOM layer — our re-parse must not
// leak diagnostics the oracle never shows.
const XML_PARSE_NOENT: c_int = 1 << 1;
const XML_PARSE_DTDLOAD: c_int = 1 << 2;
const XML_PARSE_DTDATTR: c_int = 1 << 3;
const XML_PARSE_NOERROR: c_int = 1 << 5;
const XML_PARSE_NOWARNING: c_int = 1 << 6;

// xsltSecurityOption (libxslt security.h) and the PHP-side XSL_SECPREF_* bits
// they are driven by (php_xsl.h).
const XSLT_SECPREF_READ_FILE: c_int = 1;
const XSLT_SECPREF_WRITE_FILE: c_int = 2;
const XSLT_SECPREF_CREATE_DIRECTORY: c_int = 3;
const XSLT_SECPREF_READ_NETWORK: c_int = 4;
const XSLT_SECPREF_WRITE_NETWORK: c_int = 5;
pub const SECPREF_READ_FILE: i64 = 2;
pub const SECPREF_WRITE_FILE: i64 = 4;
pub const SECPREF_CREATE_DIRECTORY: i64 = 8;
pub const SECPREF_READ_NETWORK: i64 = 16;
pub const SECPREF_WRITE_NETWORK: i64 = 32;

extern "C" {
    // /usr/lib/libxml2.2.dylib
    fn xmlReadMemory(
        buf: *const c_char, size: c_int, url: *const c_char, encoding: *const c_char, opts: c_int,
    ) -> XmlDocPtr;
    fn xmlNewDoc(version: *const c_char) -> XmlDocPtr;
    fn xmlFreeDoc(doc: XmlDocPtr);
    fn xmlCanonicPath(path: *const c_char) -> *mut c_char;
    fn xmlPathToURI(path: *const c_char) -> *mut c_char;
    // Global variable of function-pointer type (the allocator table entry the
    // result buffer of xsltSaveResultToString must be released through).
    static xmlFree: unsafe extern "C" fn(*mut c_void);

    // /usr/lib/libxslt.1.dylib
    fn xsltParseStylesheetDoc(doc: XmlDocPtr) -> XsltStylesheetPtr;
    fn xsltFreeStylesheet(style: XsltStylesheetPtr);
    fn xsltNewTransformContext(style: XsltStylesheetPtr, doc: XmlDocPtr) -> XsltCtxtPtr;
    fn xsltFreeTransformContext(ctxt: XsltCtxtPtr);
    fn xsltQuoteOneUserParam(
        ctxt: XsltCtxtPtr, name: *const c_char, value: *const c_char,
    ) -> c_int;
    fn xsltApplyStylesheetUser(
        style: XsltStylesheetPtr, doc: XmlDocPtr, params: *const *const c_char,
        output: *const c_char, profile: *mut c_void, ctxt: XsltCtxtPtr,
    ) -> XmlDocPtr;
    fn xsltSaveResultToString(
        doc_txt: *mut *mut c_char, doc_txt_len: *mut c_int, result: XmlDocPtr,
        style: XsltStylesheetPtr,
    ) -> c_int;
    fn xsltSetGenericErrorFunc(ctx: *mut c_void, handler: *const c_void);
    fn xsltNewSecurityPrefs() -> XsltSecurityPrefsPtr;
    fn xsltFreeSecurityPrefs(sec: XsltSecurityPrefsPtr);
    fn xsltSetSecurityPrefs(sec: XsltSecurityPrefsPtr, option: c_int, func: *const c_void)
        -> c_int;
    fn xsltSetCtxtSecurityPrefs(sec: XsltSecurityPrefsPtr, ctxt: XsltCtxtPtr) -> c_int;
    fn xsltSecurityForbid(
        sec: XsltSecurityPrefsPtr, ctxt: XsltCtxtPtr, value: *const c_char,
    ) -> c_int;
    static mut xsltMaxDepth: c_int;
    static mut xsltMaxVars: c_int;

    // /usr/lib/libexslt.0.dylib
    fn exsltRegisterAll();

    // libSystem
    fn open_memstream(bufp: *mut *mut c_char, sizep: *mut usize) -> *mut c_void;
    fn fflush(f: *mut c_void) -> c_int;
    fn fclose(f: *mut c_void) -> c_int;
    fn free(p: *mut c_void);
}

thread_local! {
    static EXSLT_READY: Cell<bool> = const { Cell::new(false) };
}

/// Register the EXSLT extension functions once (PHP does this at MINIT when
/// HAVE_XSL_EXSLT — hasExsltSupport() is true on the oracle).
fn ensure_exslt() {
    EXSLT_READY.with(|r| {
        if !r.get() {
            unsafe { exsltRegisterAll() };
            r.set(true);
        }
    });
}

/// Scoped capture of everything libxslt prints through `xsltGenericError`:
/// the default handler vfprintf's into its context FILE*, which we point at
/// an `open_memstream` buffer for the duration of the call.
struct ErrCapture {
    file: *mut c_void,
    // open_memstream keeps *pointers* to these two cells and writes through
    // them on every flush/close — they must live at a stable heap address,
    // not inside the (movable) capture struct itself.
    slots: Box<ErrSlots>,
}

struct ErrSlots {
    buf: *mut c_char,
    size: usize,
}

impl ErrCapture {
    fn begin() -> Option<ErrCapture> {
        let mut slots = Box::new(ErrSlots { buf: std::ptr::null_mut(), size: 0 });
        let file = unsafe { open_memstream(&mut slots.buf, &mut slots.size) };
        if file.is_null() {
            return None;
        }
        unsafe { xsltSetGenericErrorFunc(file, std::ptr::null()) };
        Some(ErrCapture { file, slots })
    }

    /// Stop capturing and split the stream into per-line messages, applying
    /// the two literal rewrites PHP's xsl_libxslt_error_handler performs so
    /// diagnostics speak of the PHP property names.
    fn end(self) -> Vec<String> {
        unsafe {
            xsltSetGenericErrorFunc(std::ptr::null_mut(), std::ptr::null());
            fflush(self.file);
            fclose(self.file);
        }
        let mut out: Vec<String> = Vec::new();
        if !self.slots.buf.is_null() {
            let bytes =
                unsafe { std::slice::from_raw_parts(self.slots.buf as *const u8, self.slots.size) };
            for line in bytes.split(|&b| b == b'\n') {
                if line.is_empty() {
                    continue;
                }
                let msg = String::from_utf8_lossy(line)
                    .replace("xsltMaxDepth (--maxdepth)", "$maxTemplateDepth")
                    .replace("maxTemplateVars (--maxvars)", "$maxTemplateVars");
                // The recursion-limit report is ONE xsltTransformError call
                // with an embedded newline ("… detected.\nYou can adjust …"):
                // PHP raises it as a single two-line warning, but the FILE*
                // funnel here loses call boundaries — restitch the known
                // continuation line onto its opener (bug71571).
                if msg.starts_with("You can adjust ") {
                    if let Some(prev) = out.last_mut() {
                        prev.push('\n');
                        prev.push_str(&msg);
                        continue;
                    }
                }
                out.push(msg);
            }
            unsafe { free(self.slots.buf as *mut c_void) };
        }
        out
    }
}

/// A compiled stylesheet (owns the underlying document, like
/// xsltParseStylesheetDoc's success path).
pub struct XsltSheet {
    ptr: XsltStylesheetPtr,
}

impl Drop for XsltSheet {
    fn drop(&mut self) {
        unsafe { xsltFreeStylesheet(self.ptr) };
    }
}

/// libxml's canonic form of a load path — what the oracle stamps on
/// `doc->URL` and `DOMDocument::$documentURI`: a local `file://` is reduced
/// to its plain path and URI-invalid bytes are escaped (`%20` for a space),
/// so relative `xsl:include`/`document()` resolution against it works even
/// under paths with spaces (bug53965). Unconvertible input passes through.
pub fn canonic_path(path: &[u8]) -> Vec<u8> {
    // A local `file://` URI reduces to its plain path first (the oracle's
    // documentURI never keeps the scheme), then xmlPathToURI escapes the
    // URI-invalid bytes exactly as libxml stamped doc->URL.
    let local: &[u8] = match path.strip_prefix(b"file://".as_slice()) {
        Some(rest) if rest.first() == Some(&b'/') => rest,
        Some(rest) if rest.starts_with(b"localhost/") => &rest[b"localhost".len()..],
        _ => path,
    };
    let Ok(c) = CString::new(local.to_vec()) else {
        return path.to_vec();
    };
    let p = unsafe { xmlPathToURI(c.as_ptr()) };
    if p.is_null() {
        return local.to_vec();
    }
    let out = unsafe { std::ffi::CStr::from_ptr(p).to_bytes().to_vec() };
    unsafe { free(p as *mut c_void) };
    out
}

/// Whether the serialized document is empty beyond an optional XML
/// declaration — the round-trip shape of a never-loaded `DOMDocument`.
fn decl_only(xml: &[u8]) -> bool {
    let mut s = xml;
    while let [b, rest @ ..] = s {
        if b.is_ascii_whitespace() {
            s = rest;
        } else {
            break;
        }
    }
    if s.starts_with(b"<?xml") {
        match s.windows(2).position(|w| w == b"?>") {
            Some(p) => s = &s[p + 2..],
            None => return false,
        }
    }
    s.iter().all(|b| b.is_ascii_whitespace())
}

fn parse_doc(xml: &[u8], url: &[u8], opts: c_int) -> XmlDocPtr {
    let curl = CString::new(url.to_vec()).unwrap_or_default();
    unsafe {
        xmlReadMemory(
            xml.as_ptr() as *const c_char,
            xml.len().min(c_int::MAX as usize) as c_int,
            curl.as_ptr(),
            std::ptr::null(),
            opts,
        )
    }
}

/// Compile a stylesheet from its serialized XML. Returns the sheet (None on
/// failure — PHP's importStylesheet returns false) plus every diagnostic line
/// libxslt emitted, for the VM to raise as call-site Warnings.
pub fn parse_stylesheet(xml: &[u8], base_url: &[u8]) -> (Option<XsltSheet>, Vec<String>) {
    ensure_exslt();
    let cap = ErrCapture::begin();
    let doc = parse_doc(
        xml,
        base_url,
        XML_PARSE_NOENT | XML_PARSE_DTDLOAD | XML_PARSE_DTDATTR | XML_PARSE_NOERROR
            | XML_PARSE_NOWARNING,
    );
    if doc.is_null() {
        let errs = cap.map(ErrCapture::end).unwrap_or_default();
        return (None, errs);
    }
    let sheet = unsafe { xsltParseStylesheetDoc(doc) };
    let errs = cap.map(ErrCapture::end).unwrap_or_default();
    if sheet.is_null() {
        // On failure the caller keeps ownership of the doc (PHP frees it by
        // releasing the cloned DOM object).
        unsafe { xmlFreeDoc(doc) };
        return (None, errs);
    }
    (Some(XsltSheet { ptr: sheet }), errs)
}

/// Everything php_xsl_apply_stylesheet reads from the processor object.
pub struct TransformOpts<'a> {
    /// setParameter() pairs, already flattened to clark-notation keys.
    pub params: &'a [(Vec<u8>, Vec<u8>)],
    pub max_depth: i64,
    pub max_vars: i64,
    /// XSL_SECPREF_* bitmask (0 = XSL_SECPREF_NONE skips the prefs entirely).
    pub security_prefs: i64,
    /// Whether registerPHPFunctions() was called on the processor: registers
    /// `php:function` / `php:functionString` on the transform context, routed
    /// through the [`set_php_callback`] trampoline.
    pub php_functions: bool,
}

// ---- php:function / php:functionString trampoline (xsl_ext_function_php) --

/// One XPath argument, converted for the PHP side.
pub enum XslArg {
    Str(Vec<u8>),
    Num(f64),
    Bool(bool),
    /// The object-mode nodeset: per node (xmlElementType, name, text content,
    /// serialized XML).
    NodeSet(Vec<XslNodeInfo>),
}

pub struct XslNodeInfo {
    pub kind: i32,
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub xml: Vec<u8>,
}

/// What the PHP callback hands back to be pushed as the XPath result.
pub enum XslRet {
    Str(Vec<u8>),
    Num(f64),
    Bool(bool),
    /// A DOM node, serialized: re-parsed into a transform-lifetime temp doc
    /// and pushed as a real NODESET, so the stylesheet can apply further
    /// XPath steps to it (xslt011's `php:function('nodeSet',/doc)/i`).
    Node(Vec<u8>),
    /// Callback raised: the VM parked the error; an empty string is pushed and
    /// the transform runs on (PHP's EG(exception) shape).
    Err,
}

pub type XslPhpCallback = Box<dyn Fn(bool, Vec<XslArg>) -> XslRet>;

thread_local! {
    /// The VM's dispatcher for php:function(String) calls, installed around
    /// [`transform`] (pattern: pdo.rs ACTIVE_VM). First argument: string-mode.
    static PHP_CB: std::cell::RefCell<Option<XslPhpCallback>> =
        const { std::cell::RefCell::new(None) };
    /// Temp documents backing `XslRet::Node` nodesets: they must outlive the
    /// XPath evaluation that received them, so they are freed only when the
    /// enclosing [`transform`] finishes.
    static TMP_DOCS: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Install / clear the callback for one transform. Returns the previous one
/// so nested transforms (a callback running another transform) restore it.
pub fn set_php_callback(cb: Option<XslPhpCallback>) -> Option<XslPhpCallback> {
    PHP_CB.with(|c| std::mem::replace(&mut *c.borrow_mut(), cb))
}

// libxml2 xpath FFI for the handlers. xmlXPathObject's layout is ABI-frozen
// (public header since forever): type at 0, nodesetval at 8, boolval at 16,
// floatval at 24, stringval at 32. xmlNodeSet: nodeNr 0, nodeMax 4, nodeTab 8.
// xmlNode: _private 0, type 8, name 16, children 24, last 32, parent 40,
// next 48, prev 56, doc 64.
const XPATH_NODESET: c_int = 1;
const XPATH_BOOLEAN: c_int = 2;
const XPATH_NUMBER: c_int = 3;
const XPATH_STRING: c_int = 4;

extern "C" {
    fn valuePop(ctxt: *mut c_void) -> *mut c_void;
    fn valuePush(ctxt: *mut c_void, value: *mut c_void) -> c_int;
    fn xmlXPathFreeObject(obj: *mut c_void);
    fn xmlXPathCastToString(obj: *mut c_void) -> *mut c_char;
    fn xmlXPathCastToNumber(obj: *mut c_void) -> f64;
    fn xmlXPathNewString(val: *const c_char) -> *mut c_void;
    fn xmlXPathNewFloat(val: f64) -> *mut c_void;
    fn xmlXPathNewBoolean(val: c_int) -> *mut c_void;
    fn xmlNodeGetContent(node: *mut c_void) -> *mut c_char;
    fn xmlBufferCreate() -> *mut c_void;
    fn xmlBufferFree(buf: *mut c_void);
    fn xmlBufferContent(buf: *mut c_void) -> *const c_char;
    fn xmlNodeDump(
        buf: *mut c_void, doc: *mut c_void, node: *mut c_void, level: c_int, format: c_int,
    ) -> c_int;
    fn xsltRegisterExtFunction(
        ctxt: XsltCtxtPtr, name: *const c_char, uri: *const c_char, func: *const c_void,
    ) -> c_int;
    fn xmlDocGetRootElement(doc: *mut c_void) -> *mut c_void;
    fn xmlXPathNewNodeSet(node: *mut c_void) -> *mut c_void;
}

unsafe fn cstr_bytes(p: *const c_char) -> Vec<u8> {
    if p.is_null() {
        Vec::new()
    } else {
        std::ffi::CStr::from_ptr(p).to_bytes().to_vec()
    }
}

/// Cast an xpath object to its string form (caller-owned copy).
unsafe fn obj_to_string(obj: *mut c_void) -> Vec<u8> {
    let s = xmlXPathCastToString(obj);
    let out = cstr_bytes(s);
    if !s.is_null() {
        free(s as *mut c_void);
    }
    out
}

unsafe fn obj_type(obj: *mut c_void) -> c_int {
    *(obj as *const c_int)
}

/// The object-mode nodeset conversion (xsl_ext_function_php's DOM handoff,
/// reduced to content: kind + name + text + serialized XML per node).
unsafe fn nodeset_info(obj: *mut c_void) -> Vec<XslNodeInfo> {
    let ns = *((obj as *const u8).add(8) as *const *mut c_void);
    let mut out = Vec::new();
    if ns.is_null() {
        return out;
    }
    let node_nr = *(ns as *const c_int);
    let node_tab = *((ns as *const u8).add(8) as *const *mut *mut c_void);
    if node_tab.is_null() {
        return out;
    }
    for i in 0..node_nr {
        let node = *node_tab.add(i as usize);
        if node.is_null() {
            continue;
        }
        let kind = *((node as *const u8).add(8) as *const c_int);
        let name = cstr_bytes(*((node as *const u8).add(16) as *const *const c_char));
        let content = xmlNodeGetContent(node);
        let value = cstr_bytes(content);
        if !content.is_null() {
            free(content as *mut c_void);
        }
        let doc = *((node as *const u8).add(64) as *const *mut c_void);
        let buf = xmlBufferCreate();
        let mut xml = Vec::new();
        if !buf.is_null() {
            xmlNodeDump(buf, doc, node, 0, 0);
            xml = cstr_bytes(xmlBufferContent(buf));
            xmlBufferFree(buf);
        }
        out.push(XslNodeInfo { kind, name, value, xml });
    }
    out
}

unsafe fn xsl_php_handler(ctxt: *mut c_void, nargs: c_int, string_mode: bool) {
    let mut objs: Vec<*mut c_void> = (0..nargs).map(|_| valuePop(ctxt)).collect();
    objs.reverse(); // call order: [fname, arg1, arg2, …]
    let push_empty = |ctxt: *mut c_void| {
        valuePush(ctxt, xmlXPathNewString(c"".as_ptr()));
    };
    let ret = PHP_CB.with(|c| {
        let cb = c.borrow();
        let Some(cb) = cb.as_ref() else {
            return XslRet::Err; // no VM installed: inert empty string
        };
        let mut args = Vec::with_capacity(objs.len());
        for (i, &obj) in objs.iter().enumerate() {
            if i == 0 || string_mode {
                // The handler NAME is always taken as a string; string-mode
                // casts every argument (xsl_ext_function_string_php). A
                // non-string name is flagged for the VM's diagnostics.
                if i == 0 && obj_type(obj) != XPATH_STRING {
                    args.push(XslArg::Bool(false)); // sentinel: name-not-string
                } else {
                    args.push(XslArg::Str(obj_to_string(obj)));
                }
                continue;
            }
            args.push(match obj_type(obj) {
                XPATH_NODESET => XslArg::NodeSet(nodeset_info(obj)),
                XPATH_BOOLEAN => XslArg::Bool(xmlXPathCastToNumber(obj) != 0.0),
                XPATH_NUMBER => XslArg::Num(xmlXPathCastToNumber(obj)),
                XPATH_STRING => XslArg::Str(obj_to_string(obj)),
                _ => XslArg::Str(obj_to_string(obj)),
            });
        }
        cb(string_mode, args)
    });
    for obj in objs {
        if !obj.is_null() {
            xmlXPathFreeObject(obj);
        }
    }
    match ret {
        XslRet::Str(s) => {
            let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
            match CString::new(&s[..end]) {
                Ok(c) => {
                    valuePush(ctxt, xmlXPathNewString(c.as_ptr()));
                }
                Err(_) => push_empty(ctxt),
            }
        }
        XslRet::Num(n) => {
            valuePush(ctxt, xmlXPathNewFloat(n));
        }
        XslRet::Bool(b) => {
            valuePush(ctxt, xmlXPathNewBoolean(b as c_int));
        }
        XslRet::Node(xml) => {
            let doc = parse_doc(&xml, b"", XML_PARSE_NOERROR | XML_PARSE_NOWARNING);
            let root = if doc.is_null() {
                std::ptr::null_mut()
            } else {
                TMP_DOCS.with(|d| d.borrow_mut().push(doc as usize));
                xmlDocGetRootElement(doc)
            };
            if root.is_null() {
                push_empty(ctxt);
            } else {
                valuePush(ctxt, xmlXPathNewNodeSet(root));
            }
        }
        XslRet::Err => push_empty(ctxt),
    }
}

unsafe extern "C" fn xsl_php_handler_string(ctxt: *mut c_void, nargs: c_int) {
    xsl_php_handler(ctxt, nargs, true);
}

unsafe extern "C" fn xsl_php_handler_object(ctxt: *mut c_void, nargs: c_int) {
    xsl_php_handler(ctxt, nargs, false);
}

/// Apply `sheet` to the serialized source document. `Ok(bytes)` is the
/// xsltSaveResultToString payload (empty → PHP returns null); `Err(())` is
/// the failure path (PHP returns false). Diagnostics ride alongside either way.
pub fn transform(
    sheet: &XsltSheet, doc_xml: &[u8], doc_url: &[u8], opts: &TransformOpts<'_>,
) -> (Result<Vec<u8>, ()>, Vec<String>) {
    ensure_exslt();
    let cap = ErrCapture::begin();
    let mut errs_pre = Vec::new();
    // Source documents keep their DOM-parse semantics: no entity substitution
    // or DTD defaulting beyond what the serialized form already carries.
    let mut doc = parse_doc(doc_xml, doc_url, XML_PARSE_NOERROR | XML_PARSE_NOWARNING);
    if doc.is_null() && decl_only(doc_xml) {
        // A never-loaded DOMDocument: Zend hands libxslt its live (rootless)
        // xmlDoc, but xmlReadMemory refuses contentless input — rebuild the
        // empty doc (bug71571's recursion-limit probes transform one).
        doc = unsafe { xmlNewDoc(c"1.0".as_ptr()) };
    }
    if doc.is_null() {
        if let Some(c) = cap {
            errs_pre = c.end();
        }
        return (Err(()), errs_pre);
    }

    // maxTemplateDepth/maxTemplateVars: xsltNewTransformContext seeds the
    // context from these globals, so scope-set them instead of poking at the
    // (layout-private) context struct.
    let (old_depth, old_vars) = unsafe { (xsltMaxDepth, xsltMaxVars) };
    unsafe {
        xsltMaxDepth = opts.max_depth.clamp(0, c_int::MAX as i64) as c_int;
        xsltMaxVars = opts.max_vars.clamp(0, c_int::MAX as i64) as c_int;
    }
    let ctxt = unsafe { xsltNewTransformContext(sheet.ptr, doc) };
    unsafe {
        xsltMaxDepth = old_depth;
        xsltMaxVars = old_vars;
    }
    if ctxt.is_null() {
        unsafe { xmlFreeDoc(doc) };
        if let Some(c) = cap {
            errs_pre = c.end();
        }
        return (Err(()), errs_pre);
    }

    // registerPHPFunctions: wire the two php-namespace extension functions
    // onto THIS context (PHP registers them per-transform the same way).
    if opts.php_functions {
        unsafe {
            xsltRegisterExtFunction(
                ctxt,
                c"functionString".as_ptr(),
                c"http://php.net/xsl".as_ptr(),
                xsl_php_handler_string as *const c_void,
            );
            xsltRegisterExtFunction(
                ctxt,
                c"function".as_ptr(),
                c"http://php.net/xsl".as_ptr(),
                xsl_php_handler_object as *const c_void,
            );
        }
    }

    let mut param_failed = false;
    for (name, value) in opts.params {
        let (Ok(cn), Ok(cv)) = (CString::new(name.clone()), CString::new(value.clone())) else {
            param_failed = true;
            break;
        };
        if unsafe { xsltQuoteOneUserParam(ctxt, cn.as_ptr(), cv.as_ptr()) } < 0 {
            param_failed = true;
            break;
        }
    }

    // Security prefs, exactly as php_xsl_apply_stylesheet wires them: each
    // XSL_SECPREF_* bit forbids the matching operation.
    let mut sec: XsltSecurityPrefsPtr = std::ptr::null_mut();
    let sp = opts.security_prefs;
    if sp != 0 {
        sec = unsafe { xsltNewSecurityPrefs() };
        let forbid = xsltSecurityForbid as *const c_void;
        unsafe {
            if sp & SECPREF_READ_FILE != 0 {
                xsltSetSecurityPrefs(sec, XSLT_SECPREF_READ_FILE, forbid);
            }
            if sp & SECPREF_WRITE_FILE != 0 {
                xsltSetSecurityPrefs(sec, XSLT_SECPREF_WRITE_FILE, forbid);
            }
            if sp & SECPREF_CREATE_DIRECTORY != 0 {
                xsltSetSecurityPrefs(sec, XSLT_SECPREF_CREATE_DIRECTORY, forbid);
            }
            if sp & SECPREF_READ_NETWORK != 0 {
                xsltSetSecurityPrefs(sec, XSLT_SECPREF_READ_NETWORK, forbid);
            }
            if sp & SECPREF_WRITE_NETWORK != 0 {
                xsltSetSecurityPrefs(sec, XSLT_SECPREF_WRITE_NETWORK, forbid);
            }
            xsltSetCtxtSecurityPrefs(sec, ctxt);
        }
    }

    let result = if param_failed {
        std::ptr::null_mut()
    } else {
        unsafe {
            xsltApplyStylesheetUser(
                sheet.ptr,
                doc,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                ctxt,
            )
        }
    };

    let mut outcome: Result<Vec<u8>, ()> = Err(());
    if !result.is_null() {
        let mut txt: *mut c_char = std::ptr::null_mut();
        let mut len: c_int = 0;
        let ret = unsafe { xsltSaveResultToString(&mut txt, &mut len, result, sheet.ptr) };
        if ret >= 0 {
            let mut bytes = Vec::new();
            if !txt.is_null() && len > 0 {
                bytes.extend_from_slice(unsafe {
                    std::slice::from_raw_parts(txt as *const u8, len as usize)
                });
            }
            outcome = Ok(bytes);
        }
        if !txt.is_null() {
            unsafe { xmlFree(txt as *mut c_void) };
        }
        unsafe { xmlFreeDoc(result) };
    }

    unsafe {
        xsltFreeTransformContext(ctxt);
        if !sec.is_null() {
            xsltFreeSecurityPrefs(sec);
        }
        xmlFreeDoc(doc);
    }
    // Temp docs backing XslRet::Node nodesets die with the transform.
    TMP_DOCS.with(|d| {
        for p in d.borrow_mut().drain(..) {
            unsafe { xmlFreeDoc(p as *mut c_void) };
        }
    });
    let errs = cap.map(ErrCapture::end).unwrap_or_default();
    (outcome, errs)
}

#[cfg(test)]
mod xsltio_smoke {
    use super::*;

    // The generic-error hook and xsltMaxDepth/MaxVars are process globals: the
    // VM is single-threaded per request, but cargo's default test harness is
    // not — serialize the smoke tests or two ErrCaptures race on the FILE*.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    const SHEET: &[u8] = br#"<?xml version="1.0"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:output method="xml" indent="no" encoding="UTF-8"/>
  <xsl:template match="/"><out><xsl:value-of select="/r/x"/></out></xsl:template>
</xsl:stylesheet>"#;

    #[test]
    fn transform_roundtrip() {
        let _g = LOCK.lock().unwrap();
        let (sheet, errs) = parse_stylesheet(SHEET, b"");
        assert!(errs.is_empty(), "{errs:?}");
        let sheet = sheet.expect("sheet compiles");
        let opts = TransformOpts {
            params: &[],
            max_depth: 3000,
            max_vars: 15000,
            security_prefs: 44,
            php_functions: false,
        };
        let (out, errs) = transform(&sheet, b"<r><x>hi</x></r>", b"", &opts);
        assert!(errs.is_empty(), "{errs:?}");
        let out = out.expect("transform succeeds");
        // indent="no" ⇒ libxslt emits no trailing newline (matches the
        // oracle's WP normalize-xml.xsl output byte-for-byte).
        assert_eq!(
            String::from_utf8_lossy(&out),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<out>hi</out>"
        );
    }

    #[test]
    fn param_and_not_a_stylesheet() {
        let _g = LOCK.lock().unwrap();
        let (none, errs) = parse_stylesheet(b"<not-a-stylesheet/>", b"");
        assert!(none.is_none());
        assert!(
            errs.iter().any(|e| e.contains("not a stylesheet")),
            "diagnostics captured: {errs:?}"
        );

        let psheet: &[u8] = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="text"/><xsl:param name="who" select="'nobody'"/><xsl:template match="/"><xsl:value-of select="$who"/></xsl:template></xsl:stylesheet>"#;
        let (sheet, _) = parse_stylesheet(psheet, b"");
        let sheet = sheet.expect("sheet compiles");
        let params = vec![(b"who".to_vec(), b"world".to_vec())];
        let opts = TransformOpts {
            params: &params,
            max_depth: 3000,
            max_vars: 15000,
            security_prefs: 44,
            php_functions: false,
        };
        let (out, _) = transform(&sheet, b"<x/>", b"", &opts);
        assert_eq!(String::from_utf8_lossy(&out.unwrap()), "world");
    }
}
