//! S-194 package F — user functions of another unit bound once per call site.
//!
//! A call whose target the compiler could not resolve (a function defined in
//! another file; inside a namespace, PHP's two-step lookup) used to be
//! resolved BY NAME on every execution. `CallNsFallback` now carries a
//! [`FnBind`] cell: the first execution that resolves to a user function
//! records it in `Vm::bound_fns` and the site enters that function directly
//! from then on. PHP binds such a site once (the run-time cache of
//! `INIT_NS_FCALL_BY_NAME` / `INIT_FCALL_BY_NAME`), so a namespaced function
//! declared after the site has already bound the global one is not seen by
//! that site. A failed resolution is never bound.

use std::cell::Cell;
use std::rc::Rc;

use super::calls::bind_params;
use super::{LcKey, PhpError, Vm, Zval};
use crate::bytecode::{Module, Op};

/// The per-site binding: `(IC epoch of the run, index into Vm::bound_fns)`.
/// Epoch 0 = never bound. Rc-shared and invisible to structural equality,
/// exactly like the property / method inline caches (modules are shared
/// across runs and `Func` equality must not depend on what a VM cached).
#[derive(Clone, Default)]
pub struct FnBind(Rc<Cell<(u64, u32)>>);

impl PartialEq for FnBind {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for FnBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bind")
    }
}

/// The op of a call resolved by name at run time: `fb` is the global
/// fallback of an unqualified call inside a namespace, `None` for a plain
/// (global or fully qualified) name — then `fallback` is the SAME `Rc` as
/// `name`, which is how the handler tells the two cases apart.
pub(crate) fn by_name_op(name: &[u8], fb: Option<&[u8]>, argc: usize) -> Op {
    let name: Rc<[u8]> = name.into();
    let fallback = match fb {
        Some(fb) => fb.into(),
        None => Rc::clone(&name),
    };
    Op::CallNsFallback { name, fallback, argc: argc as u32, bind: FnBind::default() }
}

impl<'m> Vm<'m> {
    /// `Op::CallNsFallback`: enter the bound user function, or resolve.
    #[inline(never)]
    pub(super) fn call_by_name(
        &mut self,
        top: usize,
        name: &Rc<[u8]>,
        fallback: &Rc<[u8]>,
        argc: u32,
        bind: &FnBind,
    ) -> Result<(), PhpError> {
        let mut args = self.pop_keys(top, argc);
        let (epoch, slot) = bind.0.get();
        let target = if epoch == self.ic_epoch {
            Some(self.bound_fns[slot as usize])
        } else {
            self.bind_fn(name, fallback, bind)
        };
        let Some((module, idx)) = target else {
            return if Rc::ptr_eq(name, fallback) {
                self.invoke_named(name, args)
            } else {
                self.invoke_named_fallback(name, fallback, args)
            };
        };
        // The same entry as `invoke_named` for a user function.
        let callee = &module.functions[idx];
        if args.iter().any(|a| matches!(a, Zval::ArgPlace(_))) {
            self.materialize_arg_places(top, &mut args, Some(callee))?;
        }
        let mut frame = self.pooled_frame(callee, module);
        bind_params(&mut frame, args);
        self.enter_callee(frame)
    }

    /// First execution of a site in this run: the namespaced (or plain) name
    /// if it is a user function; else, for a namespaced site whose own name
    /// is not callable at all, the global user function of the fallback.
    #[cold]
    fn bind_fn(&mut self, name: &Rc<[u8]>, fallback: &Rc<[u8]>, bind: &FnBind) -> Option<(&'m Module, usize)> {
        let target = match self.user_fn(name) {
            Some(t) => t,
            None if !Rc::ptr_eq(name, fallback) && !self.is_name_callable(name) => self.user_fn(fallback)?,
            None => return None,
        };
        self.bound_fns.push(target);
        bind.0.set((self.ic_epoch, (self.bound_fns.len() - 1) as u32));
        Some(target)
    }

    /// A user function by name, in the order `invoke_named` resolves it: the
    /// main module's hoisted functions, then the linked units'.
    fn user_fn(&self, name: &[u8]) -> Option<(&'m Module, usize)> {
        let name = name.strip_prefix(b"\\").unwrap_or(name);
        if let Some(idx) = self.module.find_fn_ci(name) {
            return Some((self.module, idx));
        }
        self.linked_functions.get(LcKey::new(name).as_slice()).copied()
    }
}
