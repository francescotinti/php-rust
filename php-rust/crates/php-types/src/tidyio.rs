//! ext/tidy over the **system libtidy** (`/opt/homebrew/opt/tidy-html5`,
//! 5.8.0 — the very keg the PHP oracle links, see its configure line
//! `--with-tidy=/opt/homebrew/opt/tidy-html5`): thin FFI, so parsing, the
//! clean-and-repair pass, diagnostics text and the pretty-printed output all
//! run through the *same* C code and the produced bytes are identical.
//! Mirrors the zlibio/gdio/xsltio pattern: the FFI lives once, in the bottom
//! crate; the document handle table and the `__tidy_*` host builtins live in
//! vm/tidy.rs; the `tidy`/`tidyNode` classes in the prelude.
//!
//! ext/tidy specifics reproduced here (php-8.5.7/ext/tidy/tidy.c):
//! - every new doc gets `TidyForceOutput=yes`, `TidyMark=no` and an attached
//!   error buffer (tidy_object_new / php_tidy_quick_repair);
//! - buffers are consumed as `bp[0 .. size-1]` (tidy null-terminates and
//!   counts the terminator — PHP's FIX_BUFFER + `size-1` everywhere);
//! - a config "string" is a config FILE PATH (tidyLoadConfig), an options
//!   array is applied per-option with type-directed set calls.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint, c_void};

type TidyDoc = *mut c_void;
type TidyOption = *mut c_void;
type TidyNode = *mut c_void;
type TidyAttr = *mut c_void;
type TidyIterator = *mut c_void;

/// tidybuffio.h `TidyBuffer` — must match the C layout exactly (allocated by
/// us, written through by libtidy).
#[repr(C)]
struct TidyBuffer {
    allocator: *mut c_void,
    bp: *mut u8,
    size: c_uint,
    allocated: c_uint,
    next: c_uint,
}

impl TidyBuffer {
    /// A properly initialised buffer: `tidyBufInit` installs the default
    /// allocator — hand-zeroing leaves it NULL and `tidyBufFree` on a buffer
    /// libtidy never wrote to (e.g. an unparsed doc's error buffer)
    /// dereferences it.
    fn zeroed() -> Self {
        let mut b = TidyBuffer {
            allocator: std::ptr::null_mut(),
            bp: std::ptr::null_mut(),
            size: 0,
            allocated: 0,
            next: 0,
        };
        unsafe { tidyBufInit(&mut b) };
        b
    }
    /// The ext/tidy reading: `bp[0 .. size-1]` (drop the counted trailing
    /// NUL), empty when nothing was written.
    fn bytes(&self) -> Vec<u8> {
        if self.bp.is_null() || self.size == 0 {
            return Vec::new();
        }
        unsafe { std::slice::from_raw_parts(self.bp, (self.size - 1) as usize) }.to_vec()
    }
}

// TidyOptionType (tidyenum.h)
pub const OPT_STRING: c_int = 0;
pub const OPT_INTEGER: c_int = 1;
pub const OPT_BOOLEAN: c_int = 2;
/// TidyConfigCategory: TidyUnknownCategory=300, then the 11 public categories;
/// TidyInternalCategory is the 12th — options in it are read-only for PHP.
const TIDY_INTERNAL_CATEGORY: c_int = 312;

extern "C" {
    // /opt/homebrew/opt/tidy-html5/lib/libtidy.dylib
    fn tidyCreate() -> TidyDoc;
    fn tidyRelease(doc: TidyDoc);
    fn tidyBufInit(buf: *mut TidyBuffer);
    fn tidyBufFree(buf: *mut TidyBuffer);
    fn tidyBufAttach(buf: *mut TidyBuffer, bp: *const u8, size: c_uint);
    fn tidyBufDetach(buf: *mut TidyBuffer);
    fn tidySetErrorBuffer(doc: TidyDoc, errbuf: *mut TidyBuffer) -> c_int;
    fn tidyOptSetBool(doc: TidyDoc, opt_id: c_int, val: c_int) -> c_int;
    fn tidyOptSetInt(doc: TidyDoc, opt_id: c_int, val: u64) -> c_int;
    fn tidyOptSetValue(doc: TidyDoc, opt_id: c_int, val: *const c_char) -> c_int;
    fn tidyOptGetBool(doc: TidyDoc, opt_id: c_int) -> c_int;
    fn tidyOptGetInt(doc: TidyDoc, opt_id: c_int) -> u64;
    fn tidyOptGetValue(doc: TidyDoc, opt_id: c_int) -> *const c_char;
    fn tidyGetOptionByName(doc: TidyDoc, name: *const c_char) -> TidyOption;
    fn tidyOptGetId(opt: TidyOption) -> c_int;
    fn tidyOptGetIdForName(name: *const c_char) -> c_int;
    fn tidyOptGetName(opt: TidyOption) -> *const c_char;
    fn tidyOptGetType(opt: TidyOption) -> c_int;
    fn tidyOptGetCategory(opt: TidyOption) -> c_int;
    fn tidyOptGetDoc(doc: TidyDoc, opt: TidyOption) -> *const c_char;
    fn tidyGetOptionList(doc: TidyDoc) -> TidyIterator;
    fn tidyGetNextOption(doc: TidyDoc, it: *mut TidyIterator) -> TidyOption;
    fn tidyLoadConfig(doc: TidyDoc, config_file: *const c_char) -> c_int;
    fn tidySetCharEncoding(doc: TidyDoc, enc: *const c_char) -> c_int;
    fn tidyParseBuffer(doc: TidyDoc, buf: *mut TidyBuffer) -> c_int;
    fn tidyCleanAndRepair(doc: TidyDoc) -> c_int;
    fn tidyRunDiagnostics(doc: TidyDoc) -> c_int;
    fn tidySaveBuffer(doc: TidyDoc, buf: *mut TidyBuffer) -> c_int;
    fn tidyStatus(doc: TidyDoc) -> c_int;
    fn tidyDetectedHtmlVersion(doc: TidyDoc) -> c_int;
    fn tidyDetectedXhtml(doc: TidyDoc) -> c_int;
    fn tidyDetectedGenericXml(doc: TidyDoc) -> c_int;
    fn tidyErrorCount(doc: TidyDoc) -> c_uint;
    fn tidyWarningCount(doc: TidyDoc) -> c_uint;
    fn tidyAccessWarningCount(doc: TidyDoc) -> c_uint;
    fn tidyConfigErrorCount(doc: TidyDoc) -> c_uint;
    fn tidyReleaseDate() -> *const c_char;
    fn tidyLibraryVersion() -> *const c_char;

    fn tidyGetRoot(doc: TidyDoc) -> TidyNode;
    fn tidyGetHtml(doc: TidyDoc) -> TidyNode;
    fn tidyGetHead(doc: TidyDoc) -> TidyNode;
    fn tidyGetBody(doc: TidyDoc) -> TidyNode;
    fn tidyGetChild(node: TidyNode) -> TidyNode;
    fn tidyGetNext(node: TidyNode) -> TidyNode;
    fn tidyGetPrev(node: TidyNode) -> TidyNode;
    fn tidyGetParent(node: TidyNode) -> TidyNode;
    fn tidyNodeGetType(node: TidyNode) -> c_int;
    fn tidyNodeGetName(node: TidyNode) -> *const c_char;
    fn tidyNodeLine(node: TidyNode) -> c_uint;
    fn tidyNodeColumn(node: TidyNode) -> c_uint;
    fn tidyNodeGetId(node: TidyNode) -> c_int;
    fn tidyNodeIsProp(doc: TidyDoc, node: TidyNode) -> c_int;
    fn tidyNodeGetText(doc: TidyDoc, node: TidyNode, buf: *mut TidyBuffer) -> c_int;
    fn tidyAttrFirst(node: TidyNode) -> TidyAttr;
    fn tidyAttrNext(attr: TidyAttr) -> TidyAttr;
    fn tidyAttrName(attr: TidyAttr) -> *const c_char;
    fn tidyAttrValue(attr: TidyAttr) -> *const c_char;
}

fn cstr_opt(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { std::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }
}

/// NUL-safe CString: a config value / path with an embedded NUL cannot reach
/// the C side — treat it as its truncated-at-NUL form, like C would read it.
fn cstring(bytes: &[u8]) -> CString {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    CString::new(&bytes[..end]).expect("NUL stripped above")
}

/// `is_numeric_string` for the integer-option path: the WHOLE trimmed string
/// must be numeric (int or float literal → truncated), else `None` (enum
/// name).
fn full_numeric_long(s: &[u8]) -> Option<i64> {
    let t = std::str::from_utf8(s).ok()?.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(i) = t.parse::<i64>() {
        return Some(i);
    }
    t.parse::<f64>().ok().map(|d| d as i64)
}

/// `zval_get_long` on a string: the leading numeric prefix, 0 when none.
fn prefix_numeric_long(s: &[u8]) -> i64 {
    let t = std::str::from_utf8(s).unwrap_or("").trim_start();
    let mut end = 0;
    let bytes = t.as_bytes();
    if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    t[..end].parse::<i64>().unwrap_or(0)
}

/// One live TidyDoc with its attached error buffer (PHPTidyDoc). Freed on
/// drop; node pointers handed out stay valid for the doc's lifetime, exactly
/// as in ext/tidy (nodes hold a ref on the doc, modelled PHP-side by the
/// keeper object).
pub struct TidyDocH {
    doc: TidyDoc,
    errbuf: Box<TidyBuffer>,
    pub initialized: bool,
}

/// The result of setting one option (php_tidy_set_tidy_opt's error split).
pub enum OptSetResult {
    Ok,
    UnknownOption,
    ReadOnly,
    /// tidyOptSetValue refused the value (string / enum-string case): PHP
    /// raises a TypeError naming option and value.
    BadValue,
    /// tidyOptSetInt / tidyOptSetBool refused: PHP just returns false.
    Failed,
}

pub enum OptValue {
    Str(Vec<u8>),
    Int(i64),
    Bool(bool),
}

impl Default for TidyDocH {
    fn default() -> Self {
        Self::new()
    }
}

impl TidyDocH {
    /// tidy_object_new(is_doc): fresh doc + error buffer + the two forced
    /// options. The tidy.default_config file (if any) is loaded by the caller.
    pub fn new() -> TidyDocH {
        let doc = unsafe { tidyCreate() };
        let mut errbuf = Box::new(TidyBuffer::zeroed());
        unsafe {
            tidySetErrorBuffer(doc, &mut *errbuf);
            tidyOptSetBool(doc, tidyOptGetIdForName(c"force-output".as_ptr()), 1);
            tidyOptSetBool(doc, tidyOptGetIdForName(c"tidy-mark".as_ptr()), 0);
        }
        TidyDocH { doc, errbuf, initialized: false }
    }

    /// tidyLoadConfig return: 0 ok, <0 could-not-load, >0 parse errors.
    pub fn load_config(&mut self, path: &[u8]) -> i32 {
        let c = cstring(path);
        unsafe { tidyLoadConfig(self.doc, c.as_ptr()) }
    }

    pub fn set_encoding(&mut self, enc: &[u8]) -> bool {
        let c = cstring(enc);
        unsafe { tidySetCharEncoding(self.doc, c.as_ptr()) >= 0 }
    }

    /// php_tidy_set_tidy_opt, minus the diagnostics (the caller renders them).
    /// Value/type mismatches follow PHP's coercions: a string option
    /// stringifies any value; an integer option takes numeric strings as
    /// numbers and other strings as enum NAMES (tidyOptSetValue); a boolean
    /// option long-casts (leading-numeric prefix for strings, like
    /// zval_get_long).
    pub fn opt_set(&mut self, name: &[u8], value: &OptValue) -> OptSetResult {
        let cname = cstring(name);
        let opt = unsafe { tidyGetOptionByName(self.doc, cname.as_ptr()) };
        if opt.is_null() {
            return OptSetResult::UnknownOption;
        }
        if unsafe { tidyOptGetCategory(opt) } == TIDY_INTERNAL_CATEGORY {
            return OptSetResult::ReadOnly;
        }
        let id = unsafe { tidyOptGetId(opt) };
        let ty = unsafe { tidyOptGetType(opt) };
        match ty {
            OPT_STRING => {
                // zval_get_tmp_string: any value stringifies.
                let s: Vec<u8> = match value {
                    OptValue::Str(s) => s.clone(),
                    OptValue::Int(i) => i.to_string().into_bytes(),
                    OptValue::Bool(b) => if *b { b"1".to_vec() } else { Vec::new() },
                };
                let cv = cstring(&s);
                if unsafe { tidyOptSetValue(self.doc, id, cv.as_ptr()) } != 0 {
                    OptSetResult::Ok
                } else {
                    OptSetResult::BadValue
                }
            }
            OPT_INTEGER => match value {
                OptValue::Int(i) => {
                    if unsafe { tidyOptSetInt(self.doc, id, *i as u64) } != 0 {
                        OptSetResult::Ok
                    } else {
                        OptSetResult::Failed
                    }
                }
                // A numeric string is a number; anything else is an enum NAME.
                OptValue::Str(s) => match full_numeric_long(s) {
                    Some(i) => {
                        if unsafe { tidyOptSetInt(self.doc, id, i as u64) } != 0 {
                            OptSetResult::Ok
                        } else {
                            OptSetResult::Failed
                        }
                    }
                    None => {
                        let cv = cstring(s);
                        if unsafe { tidyOptSetValue(self.doc, id, cv.as_ptr()) } != 0 {
                            OptSetResult::Ok
                        } else {
                            OptSetResult::BadValue
                        }
                    }
                },
                OptValue::Bool(b) => {
                    if unsafe { tidyOptSetInt(self.doc, id, *b as u64) } != 0 {
                        OptSetResult::Ok
                    } else {
                        OptSetResult::Failed
                    }
                }
            },
            _ => {
                // zval_get_long, then non-zero → yes.
                let lval = match value {
                    OptValue::Int(i) => *i,
                    OptValue::Bool(b) => *b as i64,
                    OptValue::Str(s) => prefix_numeric_long(s),
                };
                if unsafe { tidyOptSetBool(self.doc, id, (lval != 0) as c_int) } != 0 {
                    OptSetResult::Ok
                } else {
                    OptSetResult::Failed
                }
            }
        }
    }

    /// tidy_getopt / tidy::getOpt: `None` = unknown option.
    pub fn opt_get(&self, name: &[u8]) -> Option<OptValue> {
        let cname = cstring(name);
        let opt = unsafe { tidyGetOptionByName(self.doc, cname.as_ptr()) };
        if opt.is_null() {
            return None;
        }
        Some(self.opt_value(opt))
    }

    fn opt_value(&self, opt: TidyOption) -> OptValue {
        let id = unsafe { tidyOptGetId(opt) };
        match unsafe { tidyOptGetType(opt) } {
            OPT_STRING => {
                let v = unsafe { tidyOptGetValue(self.doc, id) };
                OptValue::Str(cstr_opt(v).unwrap_or_default().into_bytes())
            }
            OPT_INTEGER => OptValue::Int(unsafe { tidyOptGetInt(self.doc, id) } as i64),
            _ => OptValue::Bool(unsafe { tidyOptGetBool(self.doc, id) } != 0),
        }
    }

    /// tidy_get_config: every option in tidy's iteration order.
    pub fn config(&self) -> Vec<(Vec<u8>, OptValue)> {
        let mut out = Vec::new();
        let mut it = unsafe { tidyGetOptionList(self.doc) };
        while !it.is_null() {
            let opt = unsafe { tidyGetNextOption(self.doc, &mut it) };
            if opt.is_null() {
                break;
            }
            let name = cstr_opt(unsafe { tidyOptGetName(opt) }).unwrap_or_default();
            out.push((name.into_bytes(), self.opt_value(opt)));
        }
        out
    }

    /// tidy_get_opt_doc: `None` = unknown option, `Some(None)` = no doc text.
    #[allow(clippy::option_option)]
    pub fn opt_doc(&self, name: &[u8]) -> Option<Option<Vec<u8>>> {
        let cname = cstring(name);
        let opt = unsafe { tidyGetOptionByName(self.doc, cname.as_ptr()) };
        if opt.is_null() {
            return None;
        }
        Some(cstr_opt(unsafe { tidyOptGetDoc(self.doc, opt) }).map(String::into_bytes))
    }

    /// tidyParseBuffer over an attached (zero-copy) buffer; `false` = parse
    /// error (< 0), the caller warns with the error-buffer text. Mirrors
    /// php_tidy_parse_string: `initialized` flips BEFORE the parse (a failed
    /// parse still counts as initialized).
    pub fn parse(&mut self, data: &[u8]) -> bool {
        self.initialized = true;
        let mut buf = TidyBuffer::zeroed();
        unsafe {
            tidyBufAttach(&mut buf, data.as_ptr(), data.len() as c_uint);
            let rc = tidyParseBuffer(self.doc, &mut buf);
            // Attached memory is ours — detach so nothing frees it.
            tidyBufDetach(&mut buf);
            rc >= 0
        }
    }

    pub fn clean_repair(&mut self) -> bool {
        unsafe { tidyCleanAndRepair(self.doc) >= 0 }
    }

    pub fn diagnose(&mut self) -> bool {
        self.initialized && unsafe { tidyRunDiagnostics(self.doc) >= 0 }
    }

    /// tidySaveBuffer → the pretty-printed document bytes.
    pub fn output(&self) -> Vec<u8> {
        let mut buf = TidyBuffer::zeroed();
        unsafe {
            tidySaveBuffer(self.doc, &mut buf);
            let out = buf.bytes();
            tidyBufFree(&mut buf);
            out
        }
    }

    /// The error buffer's current text; `None` when nothing was ever written
    /// (tidy_get_error_buffer's `false`).
    pub fn error_buffer(&self) -> Option<Vec<u8>> {
        if self.errbuf.bp.is_null() {
            return None;
        }
        Some(self.errbuf.bytes())
    }

    pub fn status(&self) -> i64 {
        unsafe { tidyStatus(self.doc) as i64 }
    }
    pub fn html_ver(&self) -> i64 {
        unsafe { tidyDetectedHtmlVersion(self.doc) as i64 }
    }
    pub fn is_xhtml(&self) -> bool {
        unsafe { tidyDetectedXhtml(self.doc) != 0 }
    }
    pub fn is_xml(&self) -> bool {
        unsafe { tidyDetectedGenericXml(self.doc) != 0 }
    }
    pub fn error_count(&self) -> i64 {
        unsafe { tidyErrorCount(self.doc) as i64 }
    }
    pub fn warning_count(&self) -> i64 {
        unsafe { tidyWarningCount(self.doc) as i64 }
    }
    pub fn access_count(&self) -> i64 {
        unsafe { tidyAccessWarningCount(self.doc) as i64 }
    }
    pub fn config_count(&self) -> i64 {
        unsafe { tidyConfigErrorCount(self.doc) as i64 }
    }

    /// The four base nodes (tidy_get_root/html/head/body): 0/1/2/3.
    pub fn base_node(&self, which: i64) -> Option<usize> {
        let n = unsafe {
            match which {
                0 => tidyGetRoot(self.doc),
                1 => tidyGetHtml(self.doc),
                2 => tidyGetHead(self.doc),
                _ => tidyGetBody(self.doc),
            }
        };
        if n.is_null() {
            None
        } else {
            Some(n as usize)
        }
    }

    /// Relative navigation from a node this doc handed out: 0 parent,
    /// 1 previous sibling, 2 next sibling, 3 first child.
    pub fn node_rel(&self, node: usize, rel: i64) -> Option<usize> {
        let node = node as TidyNode;
        let n = unsafe {
            match rel {
                0 => tidyGetParent(node),
                1 => tidyGetPrev(node),
                2 => tidyGetNext(node),
                _ => tidyGetChild(node),
            }
        };
        if n.is_null() {
            None
        } else {
            Some(n as usize)
        }
    }

    /// tidy_add_node_default_properties' data for one node: value, name,
    /// type, line, column, proprietary, id (None for root/doctype/text/
    /// comment), attributes (None when the node has none), child node
    /// pointers (empty = PHP's NULL child).
    #[allow(clippy::type_complexity)]
    pub fn node_info(
        &self,
        node: usize,
    ) -> (Vec<u8>, Vec<u8>, i64, i64, i64, bool, Option<i64>, Option<Vec<(Vec<u8>, Vec<u8>)>>, Vec<usize>)
    {
        let n = node as TidyNode;
        let mut buf = TidyBuffer::zeroed();
        let value = unsafe {
            tidyNodeGetText(self.doc, n, &mut buf);
            let v = buf.bytes();
            tidyBufFree(&mut buf);
            v
        };
        let name = cstr_opt(unsafe { tidyNodeGetName(n) }).unwrap_or_default().into_bytes();
        let ty = unsafe { tidyNodeGetType(n) } as i64;
        let line = unsafe { tidyNodeLine(n) } as i64;
        let column = unsafe { tidyNodeColumn(n) } as i64;
        let proprietary = unsafe { tidyNodeIsProp(self.doc, n) } != 0;
        // TidyNode_Root=0, DocType=1, Comment=2, Text=4 → id NULL (tidy.c).
        let id = if matches!(ty, 0 | 1 | 2 | 4) {
            None
        } else {
            Some(unsafe { tidyNodeGetId(n) } as i64)
        };
        let mut attrs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        let mut a = unsafe { tidyAttrFirst(n) };
        let has_attrs = !a.is_null();
        while !a.is_null() {
            if let Some(an) = cstr_opt(unsafe { tidyAttrName(a) }) {
                let av = cstr_opt(unsafe { tidyAttrValue(a) }).unwrap_or_default();
                attrs.push((an.into_bytes(), av.into_bytes()));
            }
            a = unsafe { tidyAttrNext(a) };
        }
        let mut children = Vec::new();
        let mut c = unsafe { tidyGetChild(n) };
        while !c.is_null() {
            children.push(c as usize);
            c = unsafe { tidyGetNext(c) };
        }
        (value, name, ty, line, column, proprietary, id, has_attrs.then_some(attrs), children)
    }
}

impl Drop for TidyDocH {
    fn drop(&mut self) {
        // tidy_object_free_storage's order: buffer first, then the doc.
        unsafe {
            tidyBufFree(&mut *self.errbuf);
            tidyRelease(self.doc);
        }
    }
}

/// tidy_get_release / tidy::getRelease.
pub fn release_date() -> Vec<u8> {
    cstr_opt(unsafe { tidyReleaseDate() }).unwrap_or_else(|| "unknown".into()).into_bytes()
}

/// phpinfo's "libTidy Version" (not surfaced yet; kept for completeness).
pub fn library_version() -> Vec<u8> {
    cstr_opt(unsafe { tidyLibraryVersion() }).unwrap_or_default().into_bytes()
}
