//! ext/tidy host side (`__tidy_*`), backing the prelude `tidy`/`tidyNode`
//! classes and the tidy_* procedural functions in lower/prelude/tidy.php.
//! Thin wrappers over the **system libtidy** FFI in `php_types::tidyio` (the
//! same Homebrew keg the PHP oracle links, so parse trees, diagnostics text
//! and pretty-printed output bytes are identical).
//!
//! Model (pattern `__xslt_*`): each live document is a `tidyio::TidyDocH` in
//! `Vm.tidy_docs`, addressed by an int handle held in a prelude "keeper"
//! object shared by the doc object and every node created from it (mirrors
//! PHPTidyDoc::ref_count); the keeper's `__destruct` calls `__tidy_free`.
//! Node identity is the raw `TidyNode` pointer (valid for the doc's
//! lifetime, exactly as in ext/tidy). Diagnostics are rendered PHP-side.

use php_types::tidyio::{self, OptSetResult, OptValue};
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

fn put(arr: &mut PhpArray, key: &str, v: Zval) {
    arr.insert(Key::from_bytes(key.as_bytes()), v);
}

fn opt_zval(v: OptValue) -> Zval {
    match v {
        OptValue::Str(s) => Zval::Str(PhpStr::new(s)),
        OptValue::Int(i) => Zval::Long(i),
        OptValue::Bool(b) => Zval::Bool(b),
    }
}

impl<'m> Vm<'m> {
    fn tidy_arg_str(&mut self, args: &[Zval], idx: usize) -> Vec<u8> {
        convert::to_zstr_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec()
    }
    fn tidy_arg_long(&mut self, args: &[Zval], idx: usize) -> i64 {
        convert::to_long_cast(args.get(idx).unwrap_or(&Zval::Null), &mut self.diags)
    }

    /// `__tidy_new()` → int handle. Fresh doc with the forced options; the
    /// `tidy.default_config` file (INI, PHP_INI_SYSTEM) is applied here like
    /// TIDY_SET_DEFAULT_CONFIG (phpr never sets it: kept for completeness).
    pub(super) fn ho_tidy_new(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut doc = tidyio::TidyDocH::new();
        if let Some(e) = self.ini.0.get(b"tidy.default_config".as_slice()) {
            if !e.local.is_empty() {
                let path = e.local.clone();
                doc.load_config(&path);
            }
        }
        let id = self.next_tidy;
        self.next_tidy += 1;
        self.tidy_docs.insert(id, doc);
        Ok(Zval::Long(id as i64))
    }

    pub(super) fn ho_tidy_free(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.tidy_arg_long(&args, 0) as u32;
        Ok(Zval::Bool(self.tidy_docs.remove(&id).is_some()))
    }

    /// `__tidy_conf_file($h, $path)` → tidyLoadConfig's int (0 ok, <0 load
    /// failure, >0 parse errors); the prelude renders Warning / Notice.
    pub(super) fn ho_tidy_conf_file(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let path = self.tidy_arg_str(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get_mut(&id) else { return Ok(Zval::Long(-1)) };
        Ok(Zval::Long(doc.load_config(&path) as i64))
    }

    /// `__tidy_opt_set($h, $name, $value)` → 0 ok · 1 unknown option ·
    /// 2 read-only · 3 bad value (TypeError) · 4 refused (plain false).
    /// The prelude passes the value pre-coerced: bool|int|string.
    pub(super) fn ho_tidy_opt_set(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.tidy_arg_str(&args, 1);
        let value = match args.get(2).map(|v| v.deref_clone()).unwrap_or(Zval::Null) {
            Zval::Bool(b) => OptValue::Bool(b),
            Zval::Long(i) => OptValue::Int(i),
            v => OptValue::Str(
                convert::to_zstr_cast(&v, &mut self.diags).as_bytes().to_vec(),
            ),
        };
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get_mut(&id) else { return Ok(Zval::Long(4)) };
        Ok(Zval::Long(match doc.opt_set(&name, &value) {
            OptSetResult::Ok => 0,
            OptSetResult::UnknownOption => 1,
            OptSetResult::ReadOnly => 2,
            OptSetResult::BadValue => 3,
            OptSetResult::Failed => 4,
        }))
    }

    pub(super) fn ho_tidy_set_enc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let enc = self.tidy_arg_str(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Bool(doc.set_encoding(&enc)))
    }

    pub(super) fn ho_tidy_parse(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let data = self.tidy_arg_str(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Bool(doc.parse(&data)))
    }

    pub(super) fn ho_tidy_clean_repair(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Bool(doc.clean_repair()))
    }

    pub(super) fn ho_tidy_diagnose(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Bool(doc.diagnose()))
    }

    pub(super) fn ho_tidy_output(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Str(PhpStr::new(Vec::new()))) };
        Ok(Zval::Str(PhpStr::new(doc.output())))
    }

    /// `__tidy_errbuf($h)` → string | false (never written).
    pub(super) fn ho_tidy_errbuf(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Bool(false)) };
        Ok(match doc.error_buffer() {
            Some(b) => Zval::Str(PhpStr::new(b)),
            None => Zval::Bool(false),
        })
    }

    /// `__tidy_stat($h, $what)`: 0 status · 1 html_ver · 2 is_xhtml ·
    /// 3 is_xml · 4 errors · 5 warnings · 6 access · 7 config ·
    /// 8 initialized.
    pub(super) fn ho_tidy_stat(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let what = self.tidy_arg_long(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Long(0)) };
        Ok(match what {
            0 => Zval::Long(doc.status()),
            1 => Zval::Long(doc.html_ver()),
            2 => Zval::Bool(doc.is_xhtml()),
            3 => Zval::Bool(doc.is_xml()),
            4 => Zval::Long(doc.error_count()),
            5 => Zval::Long(doc.warning_count()),
            6 => Zval::Long(doc.access_count()),
            7 => Zval::Long(doc.config_count()),
            _ => Zval::Bool(doc.initialized),
        })
    }

    /// `__tidy_getopt($h, $name)` → value | null (unknown option).
    pub(super) fn ho_tidy_getopt(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.tidy_arg_str(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Null) };
        Ok(match doc.opt_get(&name) {
            Some(v) => opt_zval(v),
            None => Zval::Null,
        })
    }

    pub(super) fn ho_tidy_get_config(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = self.tidy_arg_long(&args, 0) as u32;
        let mut out = PhpArray::new();
        if let Some(doc) = self.tidy_docs.get(&id) {
            for (name, v) in doc.config() {
                out.insert(Key::from_bytes(&name), opt_zval(v));
            }
        }
        Ok(Zval::Array(std::rc::Rc::new(out)))
    }

    /// `__tidy_opt_doc($h, $name)` → string | false (no doc) | null (unknown).
    pub(super) fn ho_tidy_opt_doc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.tidy_arg_str(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Null) };
        Ok(match doc.opt_doc(&name) {
            None => Zval::Null,
            Some(None) => Zval::Bool(false),
            Some(Some(text)) => Zval::Str(PhpStr::new(text)),
        })
    }

    pub(super) fn ho_tidy_release(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        Ok(Zval::Str(PhpStr::new(tidyio::release_date())))
    }

    /// `__tidy_node($h, $which)` → node ptr | null (0 root · 1 html ·
    /// 2 head · 3 body).
    pub(super) fn ho_tidy_node(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let which = self.tidy_arg_long(&args, 1);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Null) };
        Ok(match doc.base_node(which) {
            Some(p) => Zval::Long(p as i64),
            None => Zval::Null,
        })
    }

    /// `__tidy_node_rel($h, $ptr, $rel)` → node ptr | null (0 parent ·
    /// 1 prev · 2 next · 3 first child).
    pub(super) fn ho_tidy_node_rel(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ptr = self.tidy_arg_long(&args, 1) as usize;
        let rel = self.tidy_arg_long(&args, 2);
        let id = self.tidy_arg_long(&args, 0) as u32;
        let Some(doc) = self.tidy_docs.get(&id) else { return Ok(Zval::Null) };
        Ok(match doc.node_rel(ptr, rel) {
            Some(p) => Zval::Long(p as i64),
            None => Zval::Null,
        })
    }

    /// `__tidy_node_info($h, $ptr)` → tidy_add_node_default_properties' data:
    /// ['v','n','t','l','c','pr','id','at','ch'] — 'at' null when no
    /// attributes, 'ch' a packed list of child node ptrs (empty → PHP NULL).
    pub(super) fn ho_tidy_node_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ptr = self.tidy_arg_long(&args, 1) as usize;
        let id = self.tidy_arg_long(&args, 0) as u32;
        let mut out = PhpArray::new();
        let Some(doc) = self.tidy_docs.get(&id) else {
            return Ok(Zval::Array(std::rc::Rc::new(out)));
        };
        let (value, name, ty, line, col, prop, node_id, attrs, children) = doc.node_info(ptr);
        put(&mut out, "v", Zval::Str(PhpStr::new(value)));
        put(&mut out, "n", Zval::Str(PhpStr::new(name)));
        put(&mut out, "t", Zval::Long(ty));
        put(&mut out, "l", Zval::Long(line));
        put(&mut out, "c", Zval::Long(col));
        put(&mut out, "pr", Zval::Bool(prop));
        put(&mut out, "id", match node_id {
            Some(i) => Zval::Long(i),
            None => Zval::Null,
        });
        put(&mut out, "at", match attrs {
            Some(list) => {
                let mut a = PhpArray::new();
                for (k, v) in list {
                    a.insert(Key::from_bytes(&k), Zval::Str(PhpStr::new(v)));
                }
                Zval::Array(std::rc::Rc::new(a))
            }
            None => Zval::Null,
        });
        let mut ch = PhpArray::new();
        for c in children {
            let _ = ch.append(Zval::Long(c as i64));
        }
        put(&mut out, "ch", Zval::Array(std::rc::Rc::new(ch)));
        Ok(Zval::Array(std::rc::Rc::new(out)))
    }
}
