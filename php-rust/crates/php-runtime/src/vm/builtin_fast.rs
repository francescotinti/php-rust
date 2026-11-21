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
        name: &std::rc::Rc<[u8]>,
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
}
