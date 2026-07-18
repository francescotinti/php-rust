//! ext/xsl host side (`__xslt_*`), backing the prelude `XSLTProcessor` class
//! in lower/prelude/dom.php. Thin wrappers over the **system libxslt** FFI in
//! `php_types::xsltio` (the same /usr/lib dylibs the PHP oracle links, so the
//! compiled stylesheets and the transform output bytes are identical).
//!
//! Model (pattern `__gd_*`): each imported stylesheet is an `xsltio::XsltSheet`
//! in `Vm.xslt_sheets`, addressed by an int handle in a hidden prop of the
//! prelude class (freed by its `__destruct` via `__xslt_free`). Diagnostics
//! are NOT emitted here: the libxslt messages ride back in the `errs` slot and
//! the prelude turns them into call-site Warnings (`__warning_from_caller`),
//! matching ext/xsl's xsltSetGenericErrorFunc + php_libxml error-buffer split.

use php_types::xsltio::{self, TransformOpts};
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

fn put(arr: &mut PhpArray, key: &str, v: Zval) {
    arr.insert(Key::from_bytes(key.as_bytes()), v);
}

fn errs_zval(errs: Vec<String>) -> Zval {
    let mut list = PhpArray::new();
    for e in errs {
        let _ = list.append(Zval::Str(PhpStr::new(e.into_bytes())));
    }
    Zval::Array(std::rc::Rc::new(list))
}

impl<'m> Vm<'m> {
    fn xslt_arg_str(&mut self, args: &[Zval], idx: usize) -> Vec<u8> {
        convert::to_zstr_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec()
    }
    fn xslt_arg_long(&mut self, args: &[Zval], idx: usize) -> i64 {
        convert::to_long_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags)
    }

    /// `__xslt_import($xml, $base_url)` → `['h' => int|false, 'errs' => [...]]`
    /// — the compile half of XSLTProcessor::importStylesheet.
    pub(super) fn ho_xslt_import(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let xml = self.xslt_arg_str(&args, 0);
        let base = self.xslt_arg_str(&args, 1);
        let (sheet, errs) = xsltio::parse_stylesheet(&xml, &base);
        let mut out = PhpArray::new();
        match sheet {
            Some(s) => {
                let id = self.next_xslt;
                self.next_xslt += 1;
                self.xslt_sheets.insert(id, s);
                put(&mut out, "h", Zval::Long(id as i64));
            }
            None => put(&mut out, "h", Zval::Bool(false)),
        }
        put(&mut out, "errs", errs_zval(errs));
        Ok(Zval::Array(std::rc::Rc::new(out)))
    }

    /// `__xslt_free($h)` — the XSLTProcessor `__destruct` / re-import path.
    pub(super) fn ho_xslt_free(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.xslt_arg_long(&args, 0) as u32;
        Ok(Zval::Bool(self.xslt_sheets.remove(&id).is_some()))
    }

    /// `__xslt_transform($h, $doc_xml, $doc_url, $params, $max_depth,
    /// $max_vars, $secprefs)` → `['out' => string|false, 'errs' => [...]]`
    /// — the apply half shared by transformToXml/transformToDoc/transformToUri
    /// (`false` = PHP's false; empty string is the prelude's null case).
    pub(super) fn ho_xslt_transform(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let doc_xml = self.xslt_arg_str(&args, 1);
        let doc_url = self.xslt_arg_str(&args, 2);
        let mut params: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        if let Some(Zval::Array(a)) = args.get(3) {
            for (k, v) in a.iter() {
                let name = match k {
                    Key::Str(s) => s.as_bytes().to_vec(),
                    Key::Int(i) => i.to_string().into_bytes(),
                };
                let value = convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec();
                params.push((name, value));
            }
        }
        let max_depth = self.xslt_arg_long(&args, 4);
        let max_vars = self.xslt_arg_long(&args, 5);
        let secprefs = self.xslt_arg_long(&args, 6);
        let id = self.xslt_arg_long(&args, 0) as u32;
        let mut out = PhpArray::new();
        let Some(sheet) = self.xslt_sheets.get(&id) else {
            put(&mut out, "out", Zval::Bool(false));
            put(&mut out, "errs", errs_zval(Vec::new()));
            return Ok(Zval::Array(std::rc::Rc::new(out)));
        };
        let opts = TransformOpts {
            params: &params,
            max_depth,
            max_vars,
            security_prefs: secprefs,
        };
        let (result, errs) = xsltio::transform(sheet, &doc_xml, &doc_url, &opts);
        match result {
            Ok(bytes) => put(&mut out, "out", Zval::Str(PhpStr::new(bytes))),
            Err(()) => put(&mut out, "out", Zval::Bool(false)),
        }
        put(&mut out, "errs", errs_zval(errs));
        Ok(Zval::Array(std::rc::Rc::new(out)))
    }
}
