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

use std::cell::{Cell, RefCell};

use php_types::xsltio::{self, TransformOpts, XslArg, XslPhpCallback, XslRet};
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

thread_local! {
    /// VM re-entry pointer for php:function / php:functionString callbacks
    /// (pattern: pdo.rs ACTIVE_VM): installed around the transform; the VM is
    /// single-threaded and the outer `&mut self` is suspended inside libxslt
    /// while the callback runs.
    static XSL_ACTIVE_VM: Cell<*mut ()> = const { Cell::new(std::ptr::null_mut()) };
    /// The FIRST PhpError a callback raised during the transform: libxslt
    /// carries no error out of the apply loop, so it is parked here and
    /// re-raised by ho_xslt_transform (PHP's EG(exception) shape — later
    /// handler invocations short-circuit while it is pending).
    static XSL_CB_ERROR: RefCell<Option<PhpError>> = const { RefCell::new(None) };
}

/// The php-functions registration state, captured per transform.
/// `None` = registerPHPFunctions was never called; `All` = null-registration;
/// `Set` = restricted map name → callable.
enum XslPhpMode {
    Never,
    All,
    Set(std::rc::Rc<PhpArray>),
}

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
    /// $max_vars, $secprefs, $php_funcs)` → `['out' => string|false,
    /// 'errs' => [...]]` — the apply half shared by transformToXml/
    /// transformToDoc/transformToUri (`false` = PHP's false; empty string is
    /// the prelude's null case). `$php_funcs`: null = registerPHPFunctions
    /// never called, true = all allowed, array = restricted name → callable
    /// map. A callback's exception is re-raised here after the transform.
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
        let mode = match args.get(7).map(Zval::deref_clone) {
            Some(Zval::Array(a)) => XslPhpMode::Set(a),
            Some(Zval::Bool(true)) => XslPhpMode::All,
            _ => XslPhpMode::Never,
        };
        let id = self.xslt_arg_long(&args, 0) as u32;
        let mut out = PhpArray::new();
        // Move the sheet OUT of the table for the duration: the callback
        // re-enters `self` through the raw pointer, so no live borrow of the
        // sheets map may exist while libxslt runs (pdo.rs connection pattern).
        let Some(sheet) = self.xslt_sheets.remove(&id) else {
            put(&mut out, "out", Zval::Bool(false));
            put(&mut out, "errs", errs_zval(Vec::new()));
            return Ok(Zval::Array(std::rc::Rc::new(out)));
        };
        let opts = TransformOpts {
            params: &params,
            max_depth,
            max_vars,
            security_prefs: secprefs,
            php_functions: true,
        };
        XSL_CB_ERROR.with(|e| *e.borrow_mut() = None);
        let vm_ptr = self as *mut Vm<'m> as *mut ();
        let prev_ptr = XSL_ACTIVE_VM.replace(vm_ptr);
        let cb: XslPhpCallback = Box::new(move |string_mode, xargs| {
            let p = XSL_ACTIVE_VM.get();
            if p.is_null() {
                return XslRet::Err;
            }
            // SAFETY: single-threaded VM; the outer &mut self that installed
            // the pointer is suspended inside libxslt while this runs.
            let vm: &mut Vm<'static> = unsafe { &mut *(p as *mut Vm<'static>) };
            vm.xsl_dispatch(string_mode, xargs, &mode)
        });
        let prev_cb = xsltio::set_php_callback(Some(cb));
        let (result, errs) = xsltio::transform(&sheet, &doc_xml, &doc_url, &opts);
        xsltio::set_php_callback(prev_cb);
        XSL_ACTIVE_VM.set(prev_ptr);
        self.xslt_sheets.insert(id, sheet);
        if let Some(err) = XSL_CB_ERROR.with(|e| e.borrow_mut().take()) {
            return Err(err);
        }
        match result {
            Ok(bytes) => put(&mut out, "out", Zval::Str(PhpStr::new(bytes))),
            Err(()) => put(&mut out, "out", Zval::Bool(false)),
        }
        put(&mut out, "errs", errs_zval(errs));
        Ok(Zval::Array(std::rc::Rc::new(out)))
    }

    /// Rewrite a failed string-callable resolution into PHP's
    /// zend_make_callable diagnostics ("Invalid callback {name}, {reason}").
    /// Anything that is not a resolution failure (an exception thrown by an
    /// autoloader or by the handler body) passes through untouched.
    fn xsl_map_callable_error(&self, e: PhpError, fname: &str) -> PhpError {
        let msg = match &e {
            PhpError::Error(m) => m.clone(),
            _ => return e,
        };
        if msg == format!("Call to undefined function {fname}()") {
            return PhpError::Error(format!(
                "Invalid callback {fname}, function \"{fname}\" not found or invalid function name"
            ));
        }
        if let Some((cls, _)) = fname.split_once("::") {
            if msg == format!("Class \"{cls}\" not found") {
                return PhpError::Error(format!(
                    "Invalid callback {fname}, class \"{cls}\" not found"
                ));
            }
        }
        e
    }

    /// One php:function / php:functionString invocation from inside a
    /// transform (xsl_ext_function_php): resolve the handler against the
    /// registration mode, convert the XPath arguments, call it, convert the
    /// return. Errors are PARKED (first wins) and an inert empty string is
    /// pushed — the transform runs to completion, as with EG(exception).
    fn xsl_dispatch(&mut self, _string_mode: bool, xargs: Vec<XslArg>, mode: &XslPhpMode) -> XslRet {
        if XSL_CB_ERROR.with(|e| e.borrow().is_some()) {
            return XslRet::Err;
        }
        let park = |err: PhpError| {
            XSL_CB_ERROR.with(|e| {
                let mut e = e.borrow_mut();
                if e.is_none() {
                    *e = Some(err);
                }
            });
            XslRet::Err
        };
        let mut it = xargs.into_iter();
        let fname = match it.next() {
            Some(XslArg::Str(s)) => s,
            None => {
                return park(PhpError::Error(
                    "Function name must be passed as the first argument".into(),
                ))
            }
            _ => return park(PhpError::TypeError("Handler name must be a string".into())),
        };
        let fname_str = String::from_utf8_lossy(&fname).into_owned();
        let callable = match mode {
            XslPhpMode::Never => {
                return park(PhpError::Error("No callbacks were registered".into()))
            }
            XslPhpMode::All => Zval::Str(PhpStr::new(fname.clone())),
            XslPhpMode::Set(map) => match map.get(&Key::from_bytes(&fname)) {
                Some(v) => v.deref_clone(),
                None => {
                    return park(PhpError::Error(format!(
                        "No callback handler \"{fname_str}\" registered"
                    )))
                }
            },
        };
        let mut argv: Vec<Zval> = Vec::new();
        for a in it {
            argv.push(match a {
                XslArg::Str(s) => Zval::Str(PhpStr::new(s)),
                XslArg::Num(n) => Zval::Double(n),
                XslArg::Bool(b) => Zval::Bool(b),
                XslArg::NodeSet(nodes) => {
                    // The object-mode DOM handoff: each node re-materialises
                    // as a detached prelude DOM object (content-preserving
                    // copy — see __xsl_node_wrap; identity/liveness is NOT
                    // preserved, documented divergence).
                    let mut arr = PhpArray::new();
                    for n in nodes {
                        let wrapped = self.call_callable(
                            Zval::Str(PhpStr::from_str("__xsl_node_wrap")),
                            vec![
                                Zval::Long(n.kind as i64),
                                Zval::Str(PhpStr::new(n.name)),
                                Zval::Str(PhpStr::new(n.value)),
                                Zval::Str(PhpStr::new(n.xml)),
                            ],
                        );
                        match wrapped {
                            Ok(v) => {
                                let _ = arr.append(v);
                            }
                            Err(e) => return park(e),
                        }
                    }
                    Zval::Array(std::rc::Rc::new(arr))
                }
            });
        }
        // The call goes through the prelude __xsl_call shim: it resolves first
        // (firing the autoloader — bug33853) so a THROWING autoloader still
        // produces zend_make_callable's Error with the exception chained as
        // previous (throw_in_autoload), then invokes the handler — whose own
        // exception propagates untouched.
        let mut cargs = PhpArray::new();
        for a in argv {
            let _ = cargs.append(a);
        }
        let ret = match self.call_callable(
            Zval::Str(PhpStr::from_str("__xsl_call")),
            vec![
                Zval::Str(PhpStr::new(fname.clone())),
                callable,
                Zval::Array(std::rc::Rc::new(cargs)),
            ],
        ) {
            Ok(v) => v,
            // An unresolvable string callable gets zend_make_callable's shape
            // ("Invalid callback f, function \"f\" not found…").
            Err(e) => return park(self.xsl_map_callable_error(e, &fname_str)),
        };
        // Return conversion (the tail of xsl_ext_function_php), delegated to
        // the prelude helper: [0,string] | [1,float] | [2,bool] | [3,_] (a
        // non-DOM object → TypeError).
        match self.call_callable(Zval::Str(PhpStr::from_str("__xsl_ret_convert")), vec![ret]) {
            Ok(Zval::Array(a)) => {
                let tag = a.get(&Key::Int(0)).map(|v| v.deref_clone()).unwrap_or(Zval::Long(0));
                let payload = a.get(&Key::Int(1)).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
                match convert::to_long_cast(&tag, &mut self.diags) {
                    1 => XslRet::Num(convert::to_double(&payload)),
                    2 => XslRet::Bool(matches!(payload, Zval::Bool(true))),
                    4 => XslRet::Node(
                        convert::to_zstr_cast(&payload, &mut self.diags).as_bytes().to_vec(),
                    ),
                    3 => {
                        // A non-DOM object return throws (zend_type_error in
                        // xsl_ext_function_php), parked like every callback error.
                        return park(PhpError::TypeError(
                            "Only objects that are instances of DOM nodes can be converted to an XPath expression"
                                .to_string(),
                        ));
                    }
                    _ => XslRet::Str(
                        convert::to_zstr_cast(&payload, &mut self.diags).as_bytes().to_vec(),
                    ),
                }
            }
            Ok(_) => XslRet::Str(Vec::new()),
            Err(e) => park(e),
        }
    }
}
