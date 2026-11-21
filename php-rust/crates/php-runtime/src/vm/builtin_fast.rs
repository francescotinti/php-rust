//! S-194 package D — builtins resolved once. A `CallBuiltin` op carries the
//! resolved function pointer and a byte of special-path flags computed from the
//! name at compile time, so the dispatcher neither hashes the name nor compares
//! it against the special cases on every call. The predicates stay where they
//! were (single owners); this file only folds their answers into bits.
use super::Vm;

/// A builtin's function pointer inside a bytecode op. Compared by address;
/// printed without the address so bytecode dumps stay identical across runs.
#[derive(Clone, Copy)]
pub struct BuiltinPtr(pub crate::builtin::BuiltinFn);

impl PartialEq for BuiltinPtr {
    fn eq(&self, o: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, o.0)
    }
}

impl std::fmt::Debug for BuiltinPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn")
    }
}

/// `var_export` / `print_r`: lazy arguments are realized first.
pub(super) const BF_EXPORT: u8 = 1;
/// `var_dump`: `__debugInfo()` results are precomputed.
pub(super) const BF_VAR_DUMP: u8 = 2;
/// `count` / `sizeof`: a Countable object dispatches its `count()`.
pub(super) const BF_COUNT: u8 = 4;
/// A stream builtin a userland wrapper stream services itself.
pub(super) const BF_USER_STREAM: u8 = 8;
/// `file_get_contents`: a registered wrapper URL goes through the wrapper.
pub(super) const BF_FGC: u8 = 16;
/// Stat / image builtins on a wrapper URL.
pub(super) const BF_WRAPPER_PATH: u8 = 32;
/// Unconditionally string-coercing builtin (`__toString` precompute).
pub(super) const BF_STR_COERCE: u8 = 64;
/// Candidate for the deep (array elements) precompute; the argument-dependent
/// half of the decision stays in `value_builtin_string_coerces_deep`.
pub(super) const BF_STR_COERCE_DEEP: u8 = 128;

/// The special-path flags of a value builtin, from its name alone.
pub(crate) fn value_builtin_flags(name: &[u8]) -> u8 {
    let mut f = 0;
    if matches!(name, b"var_export" | b"print_r") {
        f |= BF_EXPORT;
    }
    if name == b"var_dump" {
        f |= BF_VAR_DUMP;
    }
    if matches!(name, b"count" | b"sizeof") {
        f |= BF_COUNT;
    }
    if super::run::is_user_stream_op(name) {
        f |= BF_USER_STREAM;
    }
    if name == b"file_get_contents" {
        f |= BF_FGC;
    }
    if super::run::is_user_wrapper_path_op(name) {
        f |= BF_WRAPPER_PATH;
    }
    if super::calls::value_builtin_string_coerces(name) {
        f |= BF_STR_COERCE;
    } else if matches!(name, b"str_replace" | b"str_ireplace" | b"array_unique") {
        f |= BF_STR_COERCE_DEEP;
    }
    f
}

/// The op of a by-value builtin call: the dedicated `Strlen` / `Count` for
/// the one-argument `strlen` and `count`/`sizeof` (S-194, user waiver),
/// `CallBuiltin` for everything else.
pub(crate) fn value_builtin_op(name: &[u8], argc: u32, bf: crate::builtin::BuiltinFn) -> crate::bytecode::Op {
    use crate::bytecode::Op;
    let f = BuiltinPtr(bf);
    match (name, argc) {
        (b"strlen", 1) => Op::Strlen { name: name.into(), f },
        (b"count" | b"sizeof", 1) => Op::Count { name: name.into(), f },
        _ => Op::CallBuiltin { name: name.into(), argc, f, fl: value_builtin_flags(name) },
    }
}

/// S-194 package N: the per-site binding of an unqualified builtin call inside
/// a namespace. Holds the IC epoch of the run in which the site was verified
/// to reach the global builtin (0 = never). Modules are shared across runs,
/// so the cell is keyed on the epoch exactly like the property / method
/// caches, and — like them — it is invisible to structural equality.
#[derive(Clone, Default)]
pub struct NsBind(std::cell::Cell<u64>);

impl PartialEq for NsBind {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for NsBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bind")
    }
}

/// The op of an unqualified call inside a namespace whose global fallback
/// `fb` is the value builtin `bname`: `CallNsBuiltin` when the namespaced
/// `name` ends in `\\` + the builtin's own name, the by-name
/// `CallNsFallback` otherwise.
pub(crate) fn ns_builtin_op(name: &[u8], fb: &[u8], bname: &[u8], argc: usize, bf: crate::builtin::BuiltinFn) -> crate::bytecode::Op {
    use crate::bytecode::Op;
    let tail = fb == bname
        && fb.len() < 256
        && argc <= u16::MAX as usize
        && name.len() > fb.len()
        && name.ends_with(fb)
        && name[name.len() - fb.len() - 1] == b'\\';
    if tail {
        Op::CallNsBuiltin { name: name.into(), f: BuiltinPtr(bf), bind: NsBind::default(), argc: argc as u16, fl: value_builtin_flags(bname), bare: fb.len() as u8 }
    } else {
        Op::CallNsFallback { name: name.into(), fallback: fb.into(), argc: argc as u32 }
    }
}

impl<'m> Vm<'m> {
    /// Whether `flush_diags` has anything to do (pending diagnostics or line
    /// marks to retire): the in-line test that lets the common builtin call
    /// skip the out-of-line flush.
    #[inline(always)]
    pub(super) fn diags_pending(&self) -> bool {
        self.diags_rendered < self.diags.len() || !self.diag_line_marks.is_empty()
    }

    /// S-194 D-bis: the call of a value builtin from the dispatcher. The common
    /// case — no special-path flag beyond string coercion / `count`, nothing
    /// pending to flush, no reference argument and (for the flagged families)
    /// no object argument — calls the builtin directly: one level instead of
    /// three, the two hand-over maps lent in place (empty by invariant), the
    /// source line computed only if the epilogue needs it. Everything else
    /// goes through `value_builtin_call`, unchanged.
    #[inline(never)]
    pub(super) fn value_builtin_fast(
        &mut self,
        top: usize,
        f: crate::builtin::BuiltinFn,
        fl: u8,
        name: &[u8],
        args: &mut [super::Zval],
    ) -> Result<super::Zval, super::PhpError> {
        use super::Zval;
        if fl & !(BF_STR_COERCE | BF_COUNT) != 0
            || self.diags_pending()
            || args.iter().any(|a| matches!(a, Zval::Ref(_)) || (fl != 0 && matches!(a, Zval::Object(_))))
        {
            return self.value_builtin_call(top, f, fl, name, args);
        }
        debug_assert!(self.var_dump_debug.is_empty() && self.stringify_args.is_empty());
        let (mut produced, mut direct) = (Vec::new(), Vec::new());
        let res = {
            let mut ctx = super::Ctx {
                out: &mut produced,
                diags: &mut self.diags,
                direct_out: &mut direct,
                debug_info: &self.var_dump_debug,
                stringify: &self.stringify_args,
            };
            f(args, &mut ctx)
        };
        if self.diags_pending() || !produced.is_empty() || !direct.is_empty() {
            return self.value_builtin_epilogue(top, &produced, &direct, res);
        }
        res
    }

    /// The observable tail of a value-builtin call that produced something:
    /// the builtin's own warnings, then its output, then its direct stream
    /// writes — the same order as `run_value_builtin`.
    #[cold]
    #[inline(never)]
    fn value_builtin_epilogue(
        &mut self,
        top: usize,
        produced: &[u8],
        direct: &[u8],
        res: Result<super::Zval, super::PhpError>,
    ) -> Result<super::Zval, super::PhpError> {
        let line = self.cur_line(top);
        let flushed = self.diags_pending();
        if flushed {
            self.flush_diags(line)?;
        }
        if flushed || !produced.is_empty() {
            self.write_output(produced)?;
        }
        if !direct.is_empty() {
            self.stdout.extend_from_slice(direct);
            self.rendered.extend_from_slice(direct);
            super::cli_stream::drain_if_on(&self.rendered);
        }
        res
    }

    /// S-194 (user waiver of the no-new-ops rule, limited to these two):
    /// `Op::Strlen` / `Op::Count`. With nothing pending to flush and a string
    /// (resp. an array) on top of the stack, the top is replaced in place by
    /// its length. Every other case pops the argument and runs today's
    /// builtin path, so coercions, warnings and errors are the builtin's own.
    #[inline(always)]
    pub(super) fn len_op(
        &mut self,
        top: usize,
        name: &[u8],
        f: crate::builtin::BuiltinFn,
        fl: Option<u8>,
        count: bool,
    ) -> Result<(), super::PhpError> {
        use super::Zval;
        if !self.diags_pending() {
            if let Some(v) = self.frames[top].stack.last_mut() {
                let n = match (&*v, count) {
                    (Zval::Str(s), false) => Some(s.len()),
                    (Zval::Array(a), true) => Some(a.len()),
                    _ => None,
                };
                if let Some(n) = n {
                    *v = Zval::Long(n as i64);
                    return Ok(());
                }
            }
        }
        self.len_op_slow(top, name, f, fl)
    }

    #[cold]
    #[inline(never)]
    fn len_op_slow(
        &mut self,
        top: usize,
        name: &[u8],
        f: crate::builtin::BuiltinFn,
        fl: Option<u8>,
    ) -> Result<(), super::PhpError> {
        let a0 = self.frames[top].stack.pop().expect("builtin args");
        let fl = fl.unwrap_or_else(|| value_builtin_flags(name));
        let result = self.value_builtin_fast(top, f, fl, name, &mut [a0])?;
        self.frames[top].stack.push(result);
        Ok(())
    }

    /// S-194 package N: `Op::CallNsBuiltin`. PHP binds an unqualified call
    /// inside a namespace once per call site: the namespaced function if it
    /// exists when the site first runs, the global one otherwise. The first
    /// execution in a run checks that the namespaced name is not callable
    /// (and that no user function shadows the builtin) and binds the site;
    /// from then on the call goes straight to the builtin path of
    /// `CallBuiltin`. While the namespaced name is callable the by-name
    /// dispatch runs, unchanged.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn call_ns_builtin(
        &mut self,
        top: usize,
        name: &[u8],
        f: crate::builtin::BuiltinFn,
        bind: &NsBind,
        argc: u16,
        fl: u8,
        bare: u8,
    ) -> Result<(), super::PhpError> {
        let bname = &name[name.len() - bare as usize..];
        let epoch = crate::bytecode::ic_epoch();
        if bind.0.get() != epoch {
            if self.is_name_callable(name)
                || self.module.find_fn_ci(bname).is_some()
                || self.linked_functions.contains_key(super::LcKey::new(bname).as_slice())
            {
                let args = self.pop_keys(top, argc as u32);
                return self.invoke_named_fallback(name, bname, args);
            }
            bind.0.set(epoch);
        }
        if argc == 1 && (fl & BF_COUNT != 0 || (fl == BF_STR_COERCE && bname == b"strlen")) {
            return self.len_op(top, bname, f, Some(fl), fl & BF_COUNT != 0);
        }
        let result = match argc {
            0 => self.value_builtin_fast(top, f, fl, bname, &mut [])?,
            1 => {
                let a0 = self.frames[top].stack.pop().expect("builtin args");
                self.value_builtin_fast(top, f, fl, bname, &mut [a0])?
            }
            2 => {
                let a1 = self.frames[top].stack.pop().expect("builtin args");
                let a0 = self.frames[top].stack.pop().expect("builtin args");
                self.value_builtin_fast(top, f, fl, bname, &mut [a0, a1])?
            }
            3 => {
                let a2 = self.frames[top].stack.pop().expect("builtin args");
                let a1 = self.frames[top].stack.pop().expect("builtin args");
                let a0 = self.frames[top].stack.pop().expect("builtin args");
                self.value_builtin_fast(top, f, fl, bname, &mut [a0, a1, a2])?
            }
            _ => {
                let mut args = self.pop_keys(top, argc as u32);
                self.value_builtin_fast(top, f, fl, bname, &mut args)?
            }
        };
        self.frames[top].stack.push(result);
        Ok(())
    }
}
