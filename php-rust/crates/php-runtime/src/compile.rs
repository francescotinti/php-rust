//! HIR → bytecode compiler (VM-migration Fase 3, vertical proof slice).
//!
//! Where the original tree-walker `match`ed on an [`crate::hir::ExprKind`] to
//! *execute* it, this module `match`es to *emit* [`crate::bytecode::Op`]s, sharing
//! the same source of truth (the HIR) and the same value semantics
//! (`php_types::ops` / `convert`, invoked by the VM, not re-implemented here).
//!
//! # Status: Tier-1 proof slice
//!
//! This first cut compiles exactly the subset needed to prove the
//! compile→VM spine end-to-end: echo/print, scalar literals, local
//! read/write (incl. compound and inc/dec on a bare slot), the binary/unary
//! and Int/String/Bool casts, structured control flow (`if`/`while`/`do-while`/
//! `for`/ternary/short-circuit `&&` `||`, `break N` / `continue N`), and
//! `return`. Anything else returns [`CompileError::Unsupported`] with the HIR
//! variant name — the same "name the gap" discipline `lower` uses — so widening
//! coverage is a matter of turning `Unsupported` arms into emit arms.
//!
//! Calls, arrays, references, OOP and generators are deliberately out of slice;
//! `Module::functions` / `closures` are left empty until the call opcode lands.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use php_types::{ObjectInfo, PhpStr, PropVis};

use crate::builtin::{Builtin, Registry};
use crate::bytecode::{
    Addr, BuiltinIface, ClassTarget, CompiledClass, CompiledConst, CompiledEnumCase, CompiledMethod, CompiledStaticProp, Const,
    ConstIdx, DimBase, ExcRegion, FieldBase, FieldStep, Func, Instantiable, Module, Op, PropHooks, StaticInit,
};
use crate::hir::{
    BinOp, Case, CatchClause, ClassDecl, ClassId, ClassRef, Expr, ExprKind, FnDecl, HintKind, Line,
    MatchArm, Param, Place, PlaceBase, PlaceStep, Program, StaticAssignOp, Stmt, StmtKind, TypeHint,
    Visibility,
};

/// A construct the proof-slice compiler does not yet lower. Carries the HIR
/// variant name so the coverage gap is legible (mirrors `lower::LowerError`).
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    Unsupported(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Unsupported(what) => write!(f, "VM compile: unsupported {what} (not yet ported)"),
        }
    }
}

type R<T> = Result<T, CompileError>;

/// The program-wide context every body compiles against: the function table (for
/// call resolution), the builtin registry (for classifying call names), the class
/// table (for compile-time method/constructor resolution and walking parents),
/// and a case-insensitive name→[`ClassId`] index (for resolving
/// [`ClassRef::Named`]). Bundled so a body's compiler can borrow it whole.
struct ProgramCtx<'a> {
    funcs: &'a [FnDecl],
    /// Indices into `funcs` that are conditional declarations: they do NOT resolve
    /// a call by name at compile time (dispatched dynamically, callable only once
    /// their `DeclareFn` runs).
    conditional_fns: &'a HashSet<usize>,
    registry: &'a Registry,
    classes: &'a [ClassDecl],
    class_index: &'a HashMap<Vec<u8>, ClassId>,
}

/// Compile a lowered [`Program`] into an executable [`Module`].
///
/// Functions, classes and methods are compiled in the same index spaces as the
/// source [`Program`], so a call/`new`/method resolved to index `i` in the HIR
/// maps to index `i` here. Closures are still out of slice.
pub fn compile_program(program: &Program, registry: &Registry) -> R<Module> {
    // Case-insensitive name→id index for resolving `ClassRef::Named`; the first
    // declaration of a name wins (PHP forbids redeclaration).
    let mut class_index: HashMap<Vec<u8>, ClassId> = HashMap::new();
    for (i, cd) in program.classes.iter().enumerate() {
        class_index.entry(cd.name.to_ascii_lowercase()).or_insert(i);
    }
    let ctx = ProgramCtx {
        funcs: &program.functions,
        conditional_fns: &program.conditional_fns,
        registry,
        classes: &program.classes,
        class_index: &class_index,
    };

    let mut functions = Vec::with_capacity(program.functions.len());
    for fd in &program.functions {
        // Function bodies compile *tolerantly*: the always-injected PHP prelude
        // (exception classes, date API) uses not-yet-ported constructs, so a
        // failure becomes a stub that fatals only if the function is called —
        // rather than making every script uncompilable. `main`, below, is not
        // tolerant: if the script body itself is unsupported, the VM can't run it.
        match compile_fndecl(fd, &ctx) {
            Ok(f) => functions.push(f),
            Err(e) => functions.push(stub_func(fd, &e)),
        }
    }
    // Closure bodies compile tolerantly (like functions): an unsupported body
    // becomes a stub that fatals only if the closure is actually invoked. Same
    // index space as `program.closures`, so `MakeClosure { fn_idx }` lines up.
    let closures = program
        .closures
        .iter()
        .map(|fd| compile_fndecl(fd, &ctx).unwrap_or_else(|e| stub_func(fd, &e)))
        .collect();
    let main = compile_body(b"", &program.body, program.slots.len() as u32, &[], &program.slots, false, false, None, 0, &ctx, None, true)?;
    // Classes are compiled tolerantly too (see `compile_class`).
    let classes = program
        .classes
        .iter()
        .enumerate()
        .map(|(cid, cd)| compile_class(cid, cd, &ctx))
        .collect();

    Ok(Module {
        main,
        functions,
        conditional_fns: program.conditional_fns.clone(),
        closures,
        classes,
        file: program.file.clone(),
        class_index,
        static_count: program.static_count,
        strict: program.strict,
    })
}

/// Map a class name that does not resolve to a `ClassId` onto a built-in
/// interface, if it is one. These (`Generator`/`Iterator`/`Traversable`) are not
/// registered in the prelude, so `instanceof` against them is decided by the
/// operand's runtime type instead of the class table. The namespace prefix is
/// stripped and the comparison is case-insensitive, matching PHP name resolution.
fn builtin_iface_for(name: &[u8]) -> Option<BuiltinIface> {
    let bare = match name.iter().rposition(|&b| b == b'\\') {
        Some(i) => &name[i + 1..],
        None => name,
    };
    if bare.eq_ignore_ascii_case(b"Generator") {
        Some(BuiltinIface::Generator)
    } else if bare.eq_ignore_ascii_case(b"Iterator") {
        Some(BuiltinIface::Iterator)
    } else if bare.eq_ignore_ascii_case(b"Traversable") {
        Some(BuiltinIface::Traversable)
    } else {
        None
    }
}

/// Compile a user [`FnDecl`] into a [`Func`], resolving calls in its body against
/// the program context (for forward references and recursion). A free function
/// has no enclosing class (`cur_class = None`).
fn compile_fndecl(fd: &FnDecl, ctx: &ProgramCtx) -> R<Func> {
    compile_body(
        &fd.name,
        &fd.body,
        fd.slots.len() as u32,
        &fd.params,
        &fd.slots,
        fd.by_ref,
        fd.is_generator,
        fd.ret_hint.clone(),
        fd.line,
        ctx,
        None,
        false,
    )
}

/// Compile one body (the script's, a function's, or a method's) into a [`Func`].
/// `cur_class` is the enclosing class id for a method body (so `self`/`parent`
/// resolve at compile time), `None` for the script body and free functions.
#[allow(clippy::too_many_arguments)]
fn compile_body(
    name: &[u8],
    body: &[Stmt],
    n_locals: u32,
    params: &[Param],
    slot_names: &[Box<[u8]>],
    by_ref: bool,
    is_generator: bool,
    ret_hint: Option<TypeHint>,
    def_line: Line,
    ctx: &ProgramCtx,
    cur_class: Option<ClassId>,
    is_main: bool,
) -> R<Func> {
    let n_params = params.len() as u32;
    let mut c = FnCompiler::new(ctx, n_locals, cur_class, is_main, slot_names);
    c.returns_ref = by_ref;
    // Default-parameter prologue (PAR): fill any omitted optional parameter with
    // its default before the body runs. Runs in the callee frame, so a default
    // may reference earlier parameters.
    c.param_prologue(params)?;
    c.block(body)?;
    c.resolve_gotos()?;
    // A body that runs off the end returns NULL (PHP's implicit return).
    let null = c.konst(Const::Null);
    c.emit(Op::PushConst(null));
    c.emit(Op::Ret);
    Ok(Func {
        name: name.into(),
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        // Named locals plus the high-water mark of compiler temporaries.
        n_slots: n_locals + c.n_temps_max,
        n_params,
        // Parameter names / required-ness for run-time named-argument binding (A):
        // a param's name is its slot's name (`params[i].slot == i`).
        param_names: params
            .iter()
            .map(|p| slot_names[p.slot as usize].clone())
            .collect(),
        param_required: params
            .iter()
            .map(|p| p.default.is_none() && !p.variadic)
            .collect(),
        param_by_ref: params.iter().map(|p| p.by_ref).collect(),
        param_hints: params.iter().map(|p| p.hint.clone()).collect(),
        ret_hint,
        variadic_slot: params.iter().find(|p| p.variadic).map(|p| p.slot),
        by_ref,
        is_generator,
        line: def_line,
        exc_table: c.exc_regions,
    })
}

/// A placeholder body for a function that could not be compiled yet: it raises
/// a fatal naming the gap if (and only if) the function is actually called. Its
/// slot/param counts mirror the real declaration so the call ABI stays valid.
fn stub_func(fd: &FnDecl, err: &CompileError) -> Func {
    let msg = format!(
        "VM: call to `{}` — {}",
        String::from_utf8_lossy(&fd.name),
        err
    );
    Func {
        name: fd.name.clone(),
        ops: vec![Op::Fatal(0)],
        lines: vec![fd.line],
        consts: vec![Const::Str(msg.into_bytes().into())],
        n_slots: fd.slots.len() as u32,
        n_params: fd.params.len() as u32,
        param_names: fd
            .params
            .iter()
            .map(|p| fd.slots[p.slot as usize].clone())
            .collect(),
        param_required: fd
            .params
            .iter()
            .map(|p| p.default.is_none() && !p.variadic)
            .collect(),
        param_by_ref: fd.params.iter().map(|p| p.by_ref).collect(),
        param_hints: fd.params.iter().map(|p| p.hint.clone()).collect(),
        ret_hint: fd.ret_hint.clone(),
        variadic_slot: fd.params.iter().find(|p| p.variadic).map(|p| p.slot),
        by_ref: fd.by_ref,
        is_generator: fd.is_generator,
        line: fd.line,
        exc_table: Vec::new(),
    }
}

/// Compile one HIR [`ClassDecl`] into a [`CompiledClass`] (OOP-1). Tolerant, like
/// functions: a method that doesn't compile becomes a [`stub_func`]; a
/// non-constant property default marks the class `ok = false` so [`Op::Alloc`]
/// fatals rather than producing a wrong instance.
///
/// Properties and visibility are flattened **parent-first** (root→leaf), so a
/// redeclared property keeps its inherited position and takes the most-derived
/// default / visibility — matching the tree-walker's `collect_props` /
/// `class_shape` (D-19.10/D-19.20).
fn compile_class(cid: ClassId, cd: &ClassDecl, ctx: &ProgramCtx) -> CompiledClass {
    let instantiable = if cd.is_enum {
        Instantiable::Enum
    } else if cd.is_interface {
        Instantiable::Interface
    } else if cd.is_abstract {
        Instantiable::Abstract
    } else {
        Instantiable::Yes
    };

    let chain = class_chain(ctx.classes, cid); // root → leaf

    // Flatten the property layout parent-first, a redeclared property keeping its
    // inherited position but taking the most-derived default / visibility. Build
    // the visibility shape here; resolve defaults (const vs init-thunk) below.
    let mut ok = true;
    let mut flat_defaults: Vec<(Box<[u8]>, Option<&Expr>)> = Vec::new();
    let mut vis_entries: Vec<(Box<[u8]>, PropVis)> = Vec::new();
    // Property hooks (step 50), flattened parent-first like the layout: a
    // most-derived `get`/`set` overrides the inherited one. A *virtual* hooked
    // property (no backing) is excluded from the object layout below.
    let mut prop_hooks: HashMap<Box<[u8]>, PropHooks> = HashMap::new();
    for &x in &chain {
        let cname = PhpStr::new(ctx.classes[x].name.to_vec());
        for p in &ctx.classes[x].props {
            if p.get_hook.is_some() || p.set_hook.is_some() {
                let entry = prop_hooks.entry(p.name.clone()).or_insert(PropHooks {
                    get: None,
                    set: None,
                    backed: p.backed,
                });
                entry.backed = p.backed;
                // Compile each hook in its *declaring* class's context (so `self::`
                // resolves there), even though it is stored in the leaf's table.
                if let Some(g) = &p.get_hook {
                    entry.get = Some(compile_hook(g, ctx, x));
                }
                if let Some(s) = &p.set_hook {
                    entry.set = Some(compile_hook(s, ctx, x));
                }
            } else {
                // A plain (re)declaration shadows any inherited hook and is backed.
                prop_hooks.remove(p.name.as_ref());
            }
            // A virtual hooked property has no backing storage: keep it out of the
            // instance layout (not allocated, not dumped). Backed ones stay.
            let virtual_prop = (p.get_hook.is_some() || p.set_hook.is_some()) && !p.backed;
            if !virtual_prop {
                match flat_defaults.iter_mut().find(|(k, _)| k.as_ref() == p.name.as_ref()) {
                    Some(e) => e.1 = p.default.as_ref(),
                    None => flat_defaults.push((p.name.clone(), p.default.as_ref())),
                }
                let vis = match p.visibility {
                    Visibility::Public => PropVis::Public,
                    Visibility::Protected => PropVis::Protected,
                    Visibility::Private => PropVis::Private(Rc::clone(&cname)),
                };
                match vis_entries.iter_mut().find(|(k, _)| k.as_ref() == p.name.as_ref()) {
                    Some(e) => e.1 = vis,
                    None => vis_entries.push((p.name.clone(), vis)),
                }
            } else {
                // A virtual property that shadowed an inherited backed one must
                // also drop the inherited storage entry.
                flat_defaults.retain(|(k, _)| k.as_ref() != p.name.as_ref());
                vis_entries.retain(|(k, _)| k.as_ref() != p.name.as_ref());
            }
        }
    }

    // A constant default is materialised directly; a non-constant one gets a NULL
    // placeholder (so the property exists, in order) and is set at `new` time by
    // the prop-init thunk.
    let mut prop_defaults: Vec<(Box<[u8]>, Const)> = Vec::new();
    let mut init_items: Vec<(Box<[u8]>, &Expr)> = Vec::new();
    for (name, default) in &flat_defaults {
        match default {
            None => prop_defaults.push((name.clone(), Const::Null)),
            Some(e) => match const_eval(e) {
                Some(c) => prop_defaults.push((name.clone(), c)),
                None => {
                    prop_defaults.push((name.clone(), Const::Null));
                    init_items.push((name.clone(), e));
                }
            },
        }
    }
    let prop_init = if init_items.is_empty() {
        None
    } else {
        match compile_prop_init(&init_items, ctx, cid) {
            Ok(f) => Some(f),
            // A default that doesn't compile makes the class uninstantiable in the
            // VM (Alloc fatals) rather than producing a wrong instance.
            Err(_) => {
                ok = false;
                None
            }
        }
    };

    // Methods declared on this class (same order as the HIR, so a compile-time
    // `InvokeMethod` index lines up). Each compiles tolerantly to a stub.
    let methods = cd
        .methods
        .iter()
        .map(|m| {
            let func = match compile_body(
                &m.decl.name,
                &m.decl.body,
                m.decl.slots.len() as u32,
                &m.decl.params,
                &m.decl.slots,
                m.decl.by_ref,
                m.decl.is_generator,
                m.decl.ret_hint.clone(),
                m.decl.line,
                ctx,
                Some(cid),
                false,
            ) {
                Ok(f) => f,
                Err(e) => stub_func(&m.decl, &e),
            };
            CompiledMethod { name: m.decl.name.clone(), visibility: m.visibility, func }
        })
        .collect();

    // Each class constant compiles to a value *thunk* (`<expr>; Ret`) run in this
    // class's context. A thunk that doesn't compile becomes a fatal stub, so it
    // only fails if the constant is actually read (like a method stub).
    let consts = cd
        .consts
        .iter()
        .map(|k| {
            let func = compile_const_thunk(&k.name, &k.value, ctx, cid)
                .unwrap_or_else(|e| const_stub(&k.name, &e));
            CompiledConst { name: k.name.clone(), func }
        })
        .collect();

    // Static properties: a constant default is folded; a non-constant one becomes
    // an init thunk run in this class's context on first access (a thunk that
    // doesn't compile becomes a fatal stub).
    let static_props = cd
        .static_props
        .iter()
        .map(|sp| {
            let init = match &sp.default {
                None => StaticInit::Const(Const::Null),
                Some(e) => match const_eval(e) {
                    Some(c) => StaticInit::Const(c),
                    None => StaticInit::Thunk(
                        compile_const_thunk(&sp.name, e, ctx, cid)
                            .unwrap_or_else(|err| const_stub(&sp.name, &err)),
                    ),
                },
            };
            CompiledStaticProp { name: sp.name.clone(), visibility: sp.visibility, init }
        })
        .collect();

    let own_prop_vis = cd.props.iter().map(|p| (p.name.clone(), p.visibility)).collect();

    // Names of readonly instance properties declared on this class (readonly
    // enforcement). Walked parent-first at run time to find the declaring class.
    let readonly_props = cd
        .props
        .iter()
        .filter(|p| p.readonly)
        .map(|p| p.name.clone())
        .collect();

    // Enum cases, 1:1 with the source order (so `Op::EnumCase`'s index lines up).
    // `value` is the folded backing value, or `None` for a pure case *and* for a
    // backed case whose value did not const-fold — the latter never reaches the VM
    // because `class_const` only emits `Op::EnumCase` for a materialisable case.
    let enum_cases = cd
        .enum_cases
        .iter()
        .map(|c| CompiledEnumCase {
            name: c.name.clone(),
            value: c.value.as_ref().and_then(|e| const_eval_in_class(e, cid, ctx, 0)),
        })
        .collect();

    CompiledClass {
        name: cd.name.clone(),
        class_name: PhpStr::new(cd.name.to_vec()),
        parent: cd.parent,
        interfaces: cd.interfaces.clone(),
        instantiable,
        prop_defaults,
        info: Rc::new(ObjectInfo::from_entries(vis_entries)),
        methods,
        own_prop_vis,
        static_props,
        readonly_props,
        prop_init,
        consts,
        enum_cases,
        ok,
        prop_hooks,
    }
}

/// Compile one property-hook body (step 50) to a [`Func`], in its class's context
/// (so `self::`/`$this` resolve). Mirrors the method-body path; a hook that does
/// not compile becomes a fatal stub, like a method.
fn compile_hook(fd: &crate::hir::FnDecl, ctx: &ProgramCtx, cid: ClassId) -> Func {
    compile_body(
        &fd.name,
        &fd.body,
        fd.slots.len() as u32,
        &fd.params,
        &fd.slots,
        fd.by_ref,
        fd.is_generator,
        fd.ret_hint.clone(),
        fd.line,
        ctx,
        Some(cid),
        false,
    )
    .unwrap_or_else(|e| stub_func(fd, &e))
}

/// Compile the prop-init thunk: for each non-constant property default, `This;
/// <expr>; PropSet{name}; Pop`, ending `PushConst(null); Ret`. Run with `$this` =
/// the new object (see [`Op::InitProps`]); compiled in the class's own context so
/// a `self::CONST` default resolves.
fn compile_prop_init(items: &[(Box<[u8]>, &Expr)], ctx: &ProgramCtx, cid: ClassId) -> R<Func> {
    let mut c = FnCompiler::new(ctx, 0, Some(cid), false, &[]);
    for (name, expr) in items {
        c.emit(Op::This);
        c.expr(expr)?;
        c.emit(Op::PropSet { name: name.clone() });
        c.emit(Op::Pop); // PropSet leaves the assigned value; discard it
    }
    let null = c.konst(Const::Null);
    c.emit(Op::PushConst(null));
    c.emit(Op::Ret);
    Ok(Func {
        name: Box::from(&b"{prop-init}"[..]),
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        n_slots: c.n_temps_max,
        n_params: 0,
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        exc_table: c.exc_regions,
    })
}

/// Compile a class-constant value expression into a thunk [`Func`] (`<expr>; Ret`)
/// evaluated in `decl_class`'s context (so a `self::OTHER` inside resolves).
fn compile_const_thunk(name: &[u8], value: &Expr, ctx: &ProgramCtx, decl_class: ClassId) -> R<Func> {
    let mut c = FnCompiler::new(ctx, 0, Some(decl_class), false, &[]);
    c.expr(value)?;
    c.emit(Op::Ret);
    Ok(Func {
        name: name.into(),
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        n_slots: c.n_temps_max,
        n_params: 0,
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        exc_table: c.exc_regions,
    })
}

/// A placeholder thunk for a constant whose value couldn't be compiled: fatals
/// (naming the gap) only if the constant is read.
fn const_stub(name: &[u8], err: &CompileError) -> Func {
    let msg = format!("VM: constant `{}` — {}", String::from_utf8_lossy(name), err);
    Func {
        name: name.into(),
        ops: vec![Op::Fatal(0)],
        lines: vec![0],
        consts: vec![Const::Str(msg.into_bytes().into())],
        n_slots: 0,
        n_params: 0,
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        exc_table: Vec::new(),
    }
}

/// The class ancestry root→leaf (parent-first), for flattening properties.
fn class_chain(classes: &[ClassDecl], cid: ClassId) -> Vec<ClassId> {
    let mut chain = Vec::new();
    let mut c = Some(cid);
    while let Some(x) = c {
        chain.push(x);
        c = classes[x].parent;
    }
    chain.reverse();
    chain
}

/// Constant-fold a property-default expression to a [`Const`]. OOP-1 only handles
/// scalar literals; anything else (array, constant ref, arithmetic) yields `None`
/// and makes its class an uninstantiable stub.
fn const_eval(e: &Expr) -> Option<Const> {
    match &e.kind {
        ExprKind::Null => Some(Const::Null),
        ExprKind::Bool(b) => Some(Const::Bool(*b)),
        ExprKind::Int(i) => Some(Const::Int(*i)),
        ExprKind::Float(f) => Some(Const::Float(*f)),
        ExprKind::Str(s) => Some(Const::Str(s.clone())),
        _ => None,
    }
}


/// Like [`const_eval`] but able to resolve a class-constant reference
/// (`Self::C`, `Iface::C`, `Parent::C`) against the HIR class table, evaluated in
/// the *declaring* class's context. Used to const-fold enum case backing values
/// that name an inherited interface constant (e.g. `case C = I::A;`), which a
/// context-free fold cannot reach. Walks the parent chain then interfaces
/// (transitively), mirroring `find_class_const`. `depth` guards against a cyclic
/// constant definition (a PHP error in its own right).
fn const_eval_in_class(e: &Expr, cur: ClassId, ctx: &ProgramCtx, depth: u32) -> Option<Const> {
    if depth > 32 {
        return None;
    }
    if let ExprKind::ClassConst { class, name } = &e.kind {
        if name.eq_ignore_ascii_case(b"class") {
            return None; // `E::class` is the class name string; not folded here.
        }
        let target = match class {
            ClassRef::Named(n) => *ctx.class_index.get(&n.to_ascii_lowercase())?,
            ClassRef::SelfClass => cur,
            ClassRef::Parent => ctx.classes[cur].parent?,
            ClassRef::Static | ClassRef::Dynamic(_) => return None,
        };
        let (decl_cid, value) = find_const_decl(target, name, ctx)?;
        return const_eval_in_class(value, decl_cid, ctx, depth + 1);
    }
    const_eval(e)
}

/// Find the constant `name` (case-sensitive, like PHP) reachable from class
/// `cid`: own constants and the parent chain first, then implemented interfaces
/// transitively. Returns the *declaring* class id and the constant's value
/// expression, so it can be folded in that class's context.
fn find_const_decl<'a>(cid: ClassId, name: &[u8], ctx: &'a ProgramCtx<'a>) -> Option<(ClassId, &'a Expr)> {
    let mut c = Some(cid);
    while let Some(x) = c {
        if let Some(k) = ctx.classes[x].consts.iter().find(|k| k.name.as_ref() == name) {
            return Some((x, &k.value));
        }
        c = ctx.classes[x].parent;
    }
    let mut c = Some(cid);
    while let Some(x) = c {
        for &i in &ctx.classes[x].interfaces {
            if let Some(r) = find_const_decl(i, name, ctx) {
                return Some(r);
            }
        }
        c = ctx.classes[x].parent;
    }
    None
}

/// Per-function emit state: the growing instruction stream, the constant pool,
/// the stack of enclosing loops (for `break N` / `continue N`), and the
/// program-wide [`ProgramCtx`] for resolving call / class targets.
struct FnCompiler<'a> {
    ops: Vec<Op>,
    /// Source line of each emitted op, parallel to `ops` (EXC-3b). `emit` pushes
    /// the current line; `patch` overwrites an op in place and leaves this alone.
    lines: Vec<Line>,
    /// The line of the statement/expression currently being compiled; stamped
    /// onto every op `emit` appends. Updated at the top of `stmt`/`expr`.
    cur_line: Line,
    consts: Vec<Const>,
    loops: Vec<LoopCtx>,
    ctx: &'a ProgramCtx<'a>,
    /// The class whose method (or constant thunk) is being compiled, for
    /// resolving `self::` / `parent::` at compile time; `None` for the script
    /// body and free functions.
    cur_class: Option<ClassId>,
    /// True only for the top-level script body: a destruction sweep
    /// ([`Op::Sweep`]) is emitted after each of its statements, mirroring the
    /// tree-walker's global-scope sweep (OOP-3d). Never set for functions/methods.
    is_main: bool,
    /// True when compiling a `function &f()` body — a plain `return <expr>;` (or
    /// bare `return;`) then raises the "Only variable references should be returned
    /// by reference" notice (the operand is a non-lvalue). Set by `compile_body`.
    returns_ref: bool,
    /// Names of the HIR-allocated local slots (without the leading `$`), used to
    /// label the "Undefined variable" warning at a source-level `$x` read. Empty
    /// for bodies with no named locals (const / prop-init thunks).
    slot_names: &'a [Box<[u8]>],
    /// Number of named locals (HIR slots); compiler temporaries are allocated
    /// above this, so the frame's slot array is `n_locals + n_temps_max` wide.
    n_locals: u32,
    n_temps_cur: u32,
    n_temps_max: u32,
    /// Protected `try` regions accumulated while compiling this body (EXC); each
    /// is appended when its `try` finishes, so inner regions precede outer ones.
    exc_regions: Vec<ExcRegion>,
    /// Active `finally` scopes (EXC-2b), innermost last. A `return`/`break`/
    /// `continue` compiled while one is active is routed through the finally
    /// (parked, then a jump to the finally entry recorded for patching).
    finally_scopes: Vec<FinallyScope>,
    /// Resolved `label:` positions in this function body (step 45), by name: the
    /// op address and the block-scope path (innermost last) where the label sits,
    /// used to detect a `goto` *into* a transparent block (D-45.1).
    labels: HashMap<Vec<u8>, (Addr, Vec<u32>)>,
    /// `goto` jump sites awaiting their label, patched once the whole body is
    /// compiled (labels may be forward references). Each records the two-op site
    /// (a placeholder + a trailing slot for finally routing) and the goto's
    /// block-scope path.
    pending_gotos: Vec<(Box<[u8]>, Addr, Vec<u32>)>,
    /// Block-nesting path of the statement currently being compiled (a fresh id per
    /// `block()`), innermost last. A `goto` may only target a label whose scope is a
    /// prefix of the goto's (same block or an enclosing one); a deeper/divergent
    /// target is a jump *into* a transparent block (D-45.1).
    scope_path: Vec<u32>,
    /// Monotonic block-scope id source for `scope_path`.
    next_scope: u32,
    /// One entry per compiled `try` with a `finally`: the protected op range, the
    /// finally entry address, and the scope depth *outside* the try (its own level).
    /// `resolve_gotos` uses it to route a `goto` that leaves the protected region
    /// through the finally — the target is "outside" iff its scope depth is `<=` the
    /// outer depth (an address test breaks on a label marker at the try's boundary).
    goto_finally_meta: Vec<(std::ops::Range<Addr>, Addr, usize)>,
}

/// One active `finally` block, while its protected body/catches are compiled
/// (EXC-2b). Collects the jump sites of control transfers that must run this
/// finally first; they are patched to the finally entry once it is known.
#[derive(Default)]
struct FinallyScope {
    /// `Op::Jump` sites that should land at the finally entry.
    sites: Vec<Addr>,
    /// `self.loops.len()` when this finally was entered: a `break`/`continue`
    /// whose target loop index is `< loop_depth` exits past this finally (so it
    /// is routed through it); a deeper target stays inside the protected body.
    loop_depth: usize,
}

/// One enclosing loop's unresolved jump sites. `break` jumps land at the loop
/// exit; `continue` jumps land at the loop's step/condition re-entry. Both are
/// patched once those addresses are known.
#[derive(Default)]
struct LoopCtx {
    break_sites: Vec<Addr>,
    continue_sites: Vec<Addr>,
    /// `Op::ParkJump` sites (break/continue that cross a finally): patched to the
    /// loop target like the plain sites, but with `ParkJump` so the jump runs
    /// after the finally (EXC-2b).
    parked_break_sites: Vec<Addr>,
    parked_continue_sites: Vec<Addr>,
    /// `true` for a `foreach`: a `break`/`continue` that leaves this loop must
    /// first emit an [`Op::IterPop`] to free the iterator (Zend's `FE_FREE`).
    has_iter: bool,
}

impl<'a> FnCompiler<'a> {
    fn new(
        ctx: &'a ProgramCtx<'a>,
        n_locals: u32,
        cur_class: Option<ClassId>,
        is_main: bool,
        slot_names: &'a [Box<[u8]>],
    ) -> Self {
        FnCompiler {
            ops: Vec::new(),
            lines: Vec::new(),
            cur_line: 0,
            consts: Vec::new(),
            loops: Vec::new(),
            ctx,
            cur_class,
            is_main,
            returns_ref: false,
            slot_names,
            n_locals,
            n_temps_cur: 0,
            n_temps_max: 0,
            exc_regions: Vec::new(),
            finally_scopes: Vec::new(),
            labels: HashMap::new(),
            pending_gotos: Vec::new(),
            scope_path: Vec::new(),
            next_scope: 0,
            goto_finally_meta: Vec::new(),
        }
    }

    /// Reserve a scratch slot above the named locals (for `switch`/`match`
    /// subjects). Freed with [`Self::free_temp`] so siblings reuse the space.
    fn alloc_temp(&mut self) -> crate::hir::Slot {
        let s = self.n_locals + self.n_temps_cur;
        self.n_temps_cur += 1;
        self.n_temps_max = self.n_temps_max.max(self.n_temps_cur);
        s
    }

    fn free_temp(&mut self) {
        self.n_temps_cur -= 1;
    }

    /// Append `op`, returning its address. Records the current source line in the
    /// parallel `lines` table (EXC-3b).
    fn emit(&mut self, op: Op) -> Addr {
        let at = self.ops.len() as Addr;
        self.ops.push(op);
        self.lines.push(self.cur_line);
        at
    }

    /// The address the next emitted op will occupy.
    fn here(&self) -> Addr {
        self.ops.len() as Addr
    }

    /// Overwrite the op at `at` (used to back-patch a jump once its target is known).
    fn patch(&mut self, at: Addr, op: Op) {
        self.ops[at as usize] = op;
    }

    /// Intern a literal into the constant pool, returning its index.
    fn konst(&mut self, c: Const) -> ConstIdx {
        if let Some(i) = self.consts.iter().position(|e| *e == c) {
            return i as ConstIdx;
        }
        let i = self.consts.len() as ConstIdx;
        self.consts.push(c);
        i
    }

    /// Emit the default-parameter prologue (PAR): for each parameter with a
    /// default, `FillDefault` skips the default when the argument was supplied,
    /// else the default expression is evaluated and stored into the slot.
    /// Variadic / required parameters have no default and emit nothing.
    fn param_prologue(&mut self, params: &[Param]) -> R<()> {
        // Arity guard (PAR): a required param is a non-variadic one with no
        // default. "exactly" when there are no optional/variadic params at all.
        let required = params.iter().filter(|p| p.default.is_none() && !p.variadic).count() as u32;
        if required > 0 {
            let exactly = !params.iter().any(|p| p.default.is_some() || p.variadic);
            self.emit(Op::CheckArity { required, exactly });
        }
        for p in params {
            let Some(default) = &p.default else { continue };
            let fill = self.emit(Op::FillDefault { slot: p.slot, skip: Addr::MAX });
            self.expr(default)?;
            self.emit(Op::StoreSlot(p.slot));
            // A scalar-hinted default is coerced to its type (`float $n = 0` →
            // 0.0, D-NEW-6); an unhinted / non-scalar param keeps the value.
            // Only a scalar-hinted default needs coercion; a non-scalar hint is a
            // run-time check on passed args, not on a (constant) default value.
            if let Some(hint @ TypeHint { kind: HintKind::Scalar(_), .. }) = &p.hint {
                self.emit(Op::CoerceParam { slot: p.slot, hint: hint.clone() });
            }
            let here = self.here();
            self.patch(fill, Op::FillDefault { slot: p.slot, skip: here });
        }
        Ok(())
    }

    fn block(&mut self, stmts: &[Stmt]) -> R<()> {
        // Open a fresh block scope so a `goto` *into* this block (from a shallower
        // scope) can be detected at `resolve_gotos` (D-45.1). Every compound body
        // (if/else, try/catch/finally, loops, plain `{ }`) and the function root
        // funnel through here, so sibling statements share one scope.
        let id = self.next_scope;
        self.next_scope += 1;
        self.scope_path.push(id);
        for s in stmts {
            self.stmt(s)?;
            // At global scope, sweep unreachable objects after each statement
            // (OOP-3d); inside functions/methods the tree-walker does not.
            if self.is_main {
                self.emit(Op::Sweep);
            }
        }
        self.scope_path.pop();
        Ok(())
    }

    fn stmt(&mut self, s: &Stmt) -> R<()> {
        self.cur_line = s.line;
        match &s.kind {
            StmtKind::Nop => {}
            StmtKind::DeclareFn(idx) => {
                self.emit(Op::DeclareFn { func: *idx as u32 });
            }
            StmtKind::Echo(values) => {
                for e in values {
                    self.expr(e)?;
                    self.emit(Op::Stringify); // honour __toString (OOP-3c)
                    self.emit(Op::Echo);
                }
            }
            StmtKind::ConstDecl(items) => {
                for (name, value) in items {
                    self.expr(value)?;
                    self.emit(Op::DefineConst { name: name.clone() });
                }
            }
            StmtKind::InlineHtml(bytes) => {
                // Raw text outside `<?php … ?>` (and the newline after a closing
                // tag): emitted verbatim, like `eval`'s `emit(bytes)`. Reuses the
                // string-constant + `Echo` path (a `Str` stringifies to itself).
                let k = self.konst(Const::Str(bytes.clone()));
                self.emit(Op::PushConst(k));
                self.emit(Op::Echo);
            }
            StmtKind::Expr(e) => {
                // Every expression leaves exactly one value; a statement must
                // restore the stack depth, so discard it.
                self.expr(e)?;
                self.emit(Op::Pop);
            }
            StmtKind::Block(body) => self.block(body)?,
            StmtKind::If { cond, then, elseifs, otherwise } => {
                // Collect the (cond, body) arms; the final `else` has no cond.
                let mut end_jumps: Vec<Addr> = Vec::new();
                self.cond_chain(cond, then, &mut end_jumps)?;
                for (c, b) in elseifs {
                    self.cond_chain(c, b, &mut end_jumps)?;
                }
                self.block(otherwise)?;
                let end = self.here();
                for j in end_jumps {
                    self.patch(j, Op::Jump(end));
                }
            }
            StmtKind::While { cond, body } => {
                let top = self.here();
                self.expr(cond)?;
                let exit = self.emit(Op::JumpIfFalse(Addr::MAX));
                self.loops.push(LoopCtx::default());
                self.block(body)?;
                self.emit(Op::Jump(top));
                let end = self.here();
                self.patch(exit, Op::JumpIfFalse(end));
                self.close_loop(top, end);
            }
            StmtKind::DoWhile { body, cond } => {
                let top = self.here();
                self.loops.push(LoopCtx::default());
                self.block(body)?;
                let cont = self.here();
                self.expr(cond)?;
                self.emit(Op::JumpIfTrue(top));
                let end = self.here();
                // `continue` in a do-while re-tests the condition.
                self.close_loop(cont, end);
            }
            StmtKind::For { init, cond, step, body } => {
                for e in init {
                    self.expr(e)?;
                    self.emit(Op::Pop);
                }
                let top = self.here();
                let exit = self.cond_list(cond)?;
                self.loops.push(LoopCtx::default());
                self.block(body)?;
                let cont = self.here();
                for e in step {
                    self.expr(e)?;
                    self.emit(Op::Pop);
                }
                self.emit(Op::Jump(top));
                let end = self.here();
                if let Some(exit) = exit {
                    self.patch(exit, Op::JumpIfFalse(end));
                }
                self.close_loop(cont, end);
            }
            StmtKind::Foreach { iter, key, value, by_ref, body } => {
                if *by_ref {
                    // REF-3: by-ref iteration needs an lvalue source to write back
                    // to. Over a plain variable we rebind each element live. Over
                    // any other (non-lvalue) source PHP does NOT error: it degrades
                    // to by-value iteration, where writes to `$value` land nowhere
                    // observable. So only the `Var` source takes the by-ref path;
                    // everything else falls through to the by-value loop below.
                    if let ExprKind::Var(slot) = iter.kind {
                        self.emit(Op::IterInitRef(slot));
                        let cont = self.here();
                        let fetch = self.emit(Op::IterNextRef { value: *value, key: *key, end: Addr::MAX });
                        self.loops.push(LoopCtx { has_iter: true, ..LoopCtx::default() });
                        self.block(body)?;
                        self.emit(Op::Jump(cont));
                        let exhaust = self.here();
                        self.patch(fetch, Op::IterNextRef { value: *value, key: *key, end: exhaust });
                        self.emit(Op::IterPop);
                        let after = self.here();
                        self.close_loop(cont, after);
                        return Ok(());
                    }
                }
                self.expr(iter)?;
                self.emit(Op::IterInit);
                let cont = self.here(); // `continue` re-fetches
                let fetch = self.emit(Op::IterNext { value: *value, key: *key, end: Addr::MAX });
                self.loops.push(LoopCtx { has_iter: true, ..LoopCtx::default() });
                self.block(body)?;
                self.emit(Op::Jump(cont));
                let exhaust = self.here();
                self.patch(fetch, Op::IterNext { value: *value, key: *key, end: exhaust });
                self.emit(Op::IterPop); // normal exhaustion frees the iterator
                let after = self.here(); // `break` lands here (after its own IterPop)
                self.close_loop(cont, after);
            }
            StmtKind::Break(n) => self.loop_jump(*n, true)?,
            StmtKind::Continue(n) => self.loop_jump(*n, false)?,
            StmtKind::Label(name) => {
                // `label:` (step 45) marks a jump target; the lowerer already
                // validated that no `goto` jumps *into* a loop/switch. Record the
                // scope so `resolve_gotos` can reject a jump into a transparent block.
                let here = self.here();
                self.labels.insert(name.to_vec(), (here, self.scope_path.clone()));
            }
            StmtKind::Goto(name) => {
                // Unconditional jump to a (possibly forward) label, patched once the
                // whole body is compiled. Two op slots are reserved: a `goto` that
                // crosses a `finally` is patched to `ParkJump(target)` + a jump to the
                // finally entry (the finally runs, then `EndFinally` performs the
                // parked jump, EXC-2b); a plain or into-block goto only uses the first.
                let site = self.emit(Op::Jump(Addr::MAX));
                self.emit(Op::Jump(Addr::MAX));
                self.pending_gotos.push((name.clone(), site, self.scope_path.clone()));
            }
            StmtKind::Return(opt) => {
                // A plain `return <expr>;` (or bare `return;`) inside a `function
                // &f()` means the operand is a non-lvalue: PHP raises a notice and
                // returns by value (D-13.4). The condition is known at compile time.
                if self.returns_ref {
                    let k = self.konst(Const::Str(
                        b"Only variable references should be returned by reference"[..].into(),
                    ));
                    self.emit(Op::EmitNotice(k));
                }
                match opt {
                    Some(e) => self.expr(e)?,
                    None => {
                        let null = self.konst(Const::Null);
                        self.emit(Op::PushConst(null));
                    }
                }
                // A `return` inside a try-with-finally runs the (innermost) finally
                // first: park the value, jump to the finally entry; `EndFinally`
                // then performs the return (EXC-2b).
                if !self.finally_scopes.is_empty() {
                    self.emit(Op::ParkReturn);
                    let jmp = self.emit(Op::Jump(Addr::MAX));
                    self.finally_scopes.last_mut().unwrap().sites.push(jmp);
                } else {
                    self.emit(Op::Ret);
                }
            }
            StmtKind::ReturnRef(place) => {
                // `function &f()` returning an lvalue (REF-4b): push a reference
                // to the place's cell and return it raw, so `$y = &f()` aliases
                // it. `field_path`/`MakeRef` handle index, property and `[]` steps.
                let (base, steps) = self.field_path(place)?;
                self.emit(Op::MakeRef { base, steps: steps.into() });
                self.emit(Op::Ret);
            }
            StmtKind::Unset(places) => {
                for place in places {
                    self.unset_place(place)?;
                }
            }
            StmtKind::Switch { subject, cases } => self.switch(subject, cases)?,
            StmtKind::Try { body, catches, finally } => self.try_stmt(body, catches, finally)?,
            StmtKind::Global(bindings) => {
                // REF-1. At script scope the named variable *is* the global
                // (main's frame is the global frame), so `global` is a no-op —
                // matching the tree-walker (D-12.2). Inside a function, alias each
                // local slot to its global-frame cell via a shared reference.
                if !self.is_main {
                    for b in bindings {
                        self.emit(Op::BindRef {
                            target: DimBase::Local(b.local),
                            source: DimBase::Global(b.global),
                        });
                        self.emit(Op::Pop); // statement: drop the BindRef value
                    }
                }
            }
            StmtKind::StaticVar(bindings) => {
                // `static $a = init, …` (step 15 VM port): per binding, guard the
                // one-time initialiser behind the persistent cell's existence, then
                // alias the local slot to that cell on every call. `id` is the
                // program-global static index; sharing it across calls (and across
                // recursion) gives `static` its persistence.
                for b in bindings {
                    let id = b.id as u32;
                    let guard = self.emit(Op::StaticGuard { id, skip: Addr::MAX });
                    match &b.init {
                        Some(e) => self.expr(e)?,
                        None => {
                            let null = self.konst(Const::Null);
                            self.emit(Op::PushConst(null));
                        }
                    }
                    self.emit(Op::StaticStore { id });
                    let alias = self.here();
                    self.patch(guard, Op::StaticGuard { id, skip: alias });
                    self.emit(Op::StaticAlias { slot: b.slot, id });
                }
            }
        }
        Ok(())
    }

    /// Emit one `if`/`elseif` arm: `cond`, a `JumpIfFalse` past the body, the
    /// body, and a `Jump` to the chain end (recorded for back-patching).
    fn cond_chain(&mut self, cond: &Expr, body: &[Stmt], end_jumps: &mut Vec<Addr>) -> R<()> {
        self.expr(cond)?;
        let skip = self.emit(Op::JumpIfFalse(Addr::MAX));
        self.block(body)?;
        end_jumps.push(self.emit(Op::Jump(Addr::MAX)));
        let after = self.here();
        self.patch(skip, Op::JumpIfFalse(after));
        Ok(())
    }

    /// Compile a `for`'s comma-separated condition list: all but the last are
    /// evaluated for side effects; the last drives the loop. Returns the address
    /// of the `JumpIfFalse` to back-patch, or `None` for an empty (infinite) list.
    fn cond_list(&mut self, conds: &[Expr]) -> R<Option<Addr>> {
        if conds.is_empty() {
            return Ok(None);
        }
        let (last, rest) = conds.split_last().unwrap();
        for e in rest {
            self.expr(e)?;
            self.emit(Op::Pop);
        }
        self.expr(last)?;
        Ok(Some(self.emit(Op::JumpIfFalse(Addr::MAX))))
    }

    /// Patch every `goto` jump site to its label position, once the whole body is
    /// compiled (labels may be forward references). An unknown label should have
    /// been caught at lowering, but is surfaced defensively (step 45).
    fn resolve_gotos(&mut self) -> R<()> {
        let gotos = std::mem::take(&mut self.pending_gotos);
        for (name, site, goto_scope) in gotos {
            let Some((target, label_scope)) = self.labels.get(name.as_ref()).cloned() else {
                return Err(CompileError::Unsupported(format!(
                    "goto to undefined label '{}'",
                    String::from_utf8_lossy(&name)
                )));
            };
            // A `goto` may only target a label in its own block or an enclosing one
            // (the label scope must be a prefix of the goto scope). A deeper or
            // divergent target is a jump *into* a transparent block, which the
            // tree-walker scopes out — match it with the same run-time fatal (D-45.1),
            // raised in place so output before the goto still flushes.
            let into_block = label_scope.len() > goto_scope.len()
                || goto_scope[..label_scope.len()] != label_scope[..];
            if into_block {
                let k = self.konst(Const::Str(
                    format!(
                        "'goto' into a block is not supported (label '{}', D-45.1)",
                        String::from_utf8_lossy(&name)
                    )
                    .into_bytes()
                    .into(),
                ));
                self.patch(site, Op::Fatal(k));
                continue;
            }
            // If the goto leaves a `finally`'s protected region (the target sits
            // outside it), route it through that finally like break/continue: park
            // the target, jump to the finally entry; `EndFinally` performs the parked
            // jump afterwards (EXC-2b). The innermost crossed finally runs.
            // "goto inside the protected region" is decided by address (a real op);
            // "label outside the region" by scope, not address — a `label:` marker
            // just before the try shares the try's start address (it emits no op), so
            // an address test would wrongly read it as inside. The label is outside
            // the body iff its scope is at or above the try's own level.
            let crossed = self
                .goto_finally_meta
                .iter()
                .filter(|(protected, _entry, outer_len)| {
                    protected.contains(&site) && label_scope.len() <= *outer_len
                })
                .min_by_key(|(protected, _, _)| protected.end - protected.start)
                .map(|(_, entry, _)| *entry);
            match crossed {
                Some(finally_entry) => {
                    self.patch(site, Op::ParkJump(target));
                    self.patch(site + 1, Op::Jump(finally_entry));
                }
                None => self.patch(site, Op::Jump(target)),
            }
        }
        Ok(())
    }

    /// Pop the just-compiled loop and resolve its `break`/`continue` jump sites.
    fn close_loop(&mut self, continue_target: Addr, break_target: Addr) {
        let ctx = self.loops.pop().expect("close_loop without an open loop");
        for at in ctx.break_sites {
            self.patch(at, Op::Jump(break_target));
        }
        for at in ctx.continue_sites {
            self.patch(at, Op::Jump(continue_target));
        }
        // Sites that cross a `finally` resume at the loop target via `ParkJump`,
        // run by `EndFinally` after the finally completes (EXC-2b).
        for at in ctx.parked_break_sites {
            self.patch(at, Op::ParkJump(break_target));
        }
        for at in ctx.parked_continue_sites {
            self.patch(at, Op::ParkJump(continue_target));
        }
    }

    /// Emit a `break N` / `continue N` as a placeholder `Jump`, registered with
    /// the N-th enclosing loop for back-patching. `level` is >= 1.
    fn loop_jump(&mut self, level: u32, is_break: bool) -> R<()> {
        let depth = self.loops.len();
        if level == 0 || (level as usize) > depth {
            // PHP rejects this at compile time; surface it the same way.
            let kw = if is_break { "break" } else { "continue" };
            return Err(CompileError::Unsupported(format!(
                "{kw} {level} with {depth} enclosing loop(s)"
            )));
        }
        let idx = depth - level as usize;
        // Free the iterator of every `foreach` this jump leaves: for `break`,
        // the target loop itself (idx..depth); for `continue`, only the inner
        // loops fully exited (idx+1..depth) — we stay inside the target.
        let first = if is_break { idx } else { idx + 1 };
        let pops = self.loops[first..depth].iter().filter(|l| l.has_iter).count();
        for _ in 0..pops {
            self.emit(Op::IterPop);
        }
        // If this jump exits past an enclosing `finally` (the target loop sits
        // outside it), route it through that finally: park the loop target, then
        // jump to the finally entry; `EndFinally` performs the jump afterwards
        // (EXC-2b). Single-finally crossing; the innermost crossed finally runs.
        if let Some(scope_i) = self.finally_scopes.iter().rposition(|s| idx < s.loop_depth) {
            let park = self.emit(Op::ParkJump(Addr::MAX));
            if is_break {
                self.loops[idx].parked_break_sites.push(park);
            } else {
                self.loops[idx].parked_continue_sites.push(park);
            }
            let jmp = self.emit(Op::Jump(Addr::MAX));
            self.finally_scopes[scope_i].sites.push(jmp);
            return Ok(());
        }
        let at = self.emit(Op::Jump(Addr::MAX));
        if is_break {
            self.loops[idx].break_sites.push(at);
        } else {
            self.loops[idx].continue_sites.push(at);
        }
        Ok(())
    }

    fn expr(&mut self, e: &Expr) -> R<()> {
        self.cur_line = e.line;
        match &e.kind {
            ExprKind::Null => {
                let k = self.konst(Const::Null);
                self.emit(Op::PushConst(k));
            }
            ExprKind::Bool(b) => {
                let k = self.konst(Const::Bool(*b));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Int(i) => {
                let k = self.konst(Const::Int(*i));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Float(f) => {
                let k = self.konst(Const::Float(*f));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Str(s) => {
                let k = self.konst(Const::Str(s.clone()));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Const { name, fallback } => {
                // A *user* constant (engine constants are folded at lowering): read
                // it from the VM's constant table at run time (B3). `fallback` is
                // the global name a namespaced unqualified constant falls back to.
                self.emit(Op::ConstFetch {
                    name: name.clone(),
                    fallback: fallback.clone(),
                });
            }
            ExprKind::Var(slot) => {
                // A source-level read warns on an undefined slot (PHP 8). The bare
                // name (no `$`) is taken from the slot table; a slot without a name
                // (shouldn't happen for a source var) degrades to a silent load.
                match self.slot_names.get(*slot as usize) {
                    Some(name) => {
                        let k = self.konst(Const::Str(name.clone()));
                        self.emit(Op::LoadVar { slot: *slot, name: k });
                    }
                    None => {
                        self.emit(Op::LoadSlot(*slot));
                    }
                }
            }
            ExprKind::GlobalVar(slot) => {
                // `$GLOBALS['x']` read — a resolved script-frame slot (step 12-3),
                // reachable from inside a function.
                self.emit(Op::LoadGlobal(*slot));
            }
            ExprKind::Assign(slot, rhs) => {
                self.expr(rhs)?;
                self.emit(Op::Dup); // assignment is an expression valued by the RHS
                self.emit(Op::StoreSlot(*slot));
            }
            ExprKind::ListAssign { temp, rhs, assigns } => {
                // `[$a,$b] = rhs`: stash rhs once, run each sub-assignment (which
                // reads `temp[key]`), then leave the stored rhs as the value.
                self.expr(rhs)?;
                self.emit(Op::StoreSlot(*temp));
                for a in assigns {
                    self.expr(a)?;
                    self.emit(Op::Pop); // each sub-assignment's value is discarded
                }
                self.emit(Op::LoadSlot(*temp));
            }
            ExprKind::AssignOp(op, slot, rhs) => {
                self.emit(Op::LoadSlot(*slot));
                self.expr(rhs)?;
                self.emit(Op::Binary(*op));
                self.emit(Op::Dup);
                self.emit(Op::StoreSlot(*slot));
            }
            ExprKind::IncDec { slot, inc, pre } => {
                self.emit(Op::IncDecSlot { slot: *slot, inc: *inc, pre: *pre });
            }
            ExprKind::Binary(op, a, b) => {
                // String concatenation stringifies each operand, honouring
                // `__toString` on object operands (OOP-3c).
                self.expr(a)?;
                if *op == BinOp::Concat {
                    self.emit(Op::Stringify);
                }
                self.expr(b)?;
                if *op == BinOp::Concat {
                    self.emit(Op::Stringify);
                }
                self.emit(Op::Binary(*op));
            }
            ExprKind::Unary(op, a) => {
                self.expr(a)?;
                self.emit(Op::Unary(*op));
            }
            ExprKind::Cast(kind, a) => {
                use crate::hir::CastKind;
                match kind {
                    // `(string)` honours `__toString` (OOP-3c). (The exotic
                    // `(string)NAN` coercion warning is not reproduced here.)
                    CastKind::String => {
                        self.expr(a)?;
                        self.emit(Op::Stringify);
                    }
                    CastKind::Int
                    | CastKind::Bool
                    | CastKind::Float
                    | CastKind::Array
                    | CastKind::Object => {
                        self.expr(a)?;
                        self.emit(Op::Cast(*kind));
                    }
                }
            }
            ExprKind::Suppress(e) => {
                // `@expr` (step 48): suppress diagnostics raised while evaluating
                // `expr`, leaving its value on the stack. Engine errors / thrown
                // objects still propagate (the unwind clears suppression).
                self.emit(Op::SuppressBegin);
                self.expr(e)?;
                self.emit(Op::SuppressEnd);
            }
            ExprKind::Exit(arg) => {
                // `exit` / `die` (step 46): evaluate the optional status, then
                // diverge via `Op::Exit` (raises `PhpError::Exit`, bypassing finally).
                match arg {
                    Some(e) => {
                        self.expr(e)?;
                        self.emit(Op::Exit { has_arg: true });
                    }
                    None => {
                        self.emit(Op::Exit { has_arg: false });
                    }
                }
            }
            ExprKind::And(a, b) => self.short_circuit(a, b, false)?,
            ExprKind::Or(a, b) => self.short_circuit(a, b, true)?,
            ExprKind::Xor(a, b) => {
                // `a xor b` evaluates both operands (no short-circuit) and yields a
                // bool: `(bool)a != (bool)b`. `!!x` coerces to bool, then loose
                // `!=` on the two bools is logical xor (eval: `Bool(a ^ b)`).
                self.expr(a)?;
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.expr(b)?;
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.emit(Op::Binary(BinOp::NotEq));
            }
            ExprKind::Ternary { cond, then, otherwise } => {
                match then {
                    Some(then) => {
                        // cond ? then : otherwise
                        self.expr(cond)?;
                        let to_else = self.emit(Op::JumpIfFalse(Addr::MAX));
                        self.expr(then)?;
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let else_at = self.here();
                        self.patch(to_else, Op::JumpIfFalse(else_at));
                        self.expr(otherwise)?;
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                    }
                    None => {
                        // cond ?: otherwise — evaluate cond once, reuse if truthy.
                        self.expr(cond)?;
                        self.emit(Op::Dup);
                        let to_else = self.emit(Op::JumpIfFalse(Addr::MAX));
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let else_at = self.here();
                        self.patch(to_else, Op::JumpIfFalse(else_at));
                        self.emit(Op::Pop); // discard the falsy cond copy
                        self.expr(otherwise)?;
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                    }
                }
            }
            ExprKind::Print(a) => {
                self.expr(a)?;
                self.emit(Op::Stringify); // honour __toString (OOP-3c)
                self.emit(Op::Print);
            }
            ExprKind::Call { name, args, named } => self.call(name, args, named)?,
            ExprKind::Array(elems) => {
                self.emit(Op::ArrayInit);
                for el in elems {
                    // `[...$src]` array spread (PHP 8.1): merge the source's elements
                    // (int keys renumbered, string keys preserved; Traversables too).
                    if let ExprKind::Spread(src) = &el.value.kind {
                        self.expr(src)?;
                        self.emit(Op::ArrayAppendSpread);
                        continue;
                    }
                    match &el.key {
                        Some(k) => {
                            self.expr(k)?;
                            self.expr(&el.value)?;
                            self.emit(Op::ArrayInsert);
                        }
                        None => {
                            self.expr(&el.value)?;
                            self.emit(Op::ArrayPush);
                        }
                    }
                }
            }
            ExprKind::Index { base, index } => {
                self.expr(base)?;
                self.expr(index)?;
                self.emit(Op::FetchDim);
            }
            ExprKind::AssignPlace(place, rhs) => self.assign_place(place, rhs)?,
            ExprKind::AssignCoalescePlace(place, rhs) => self.assign_coalesce_place(place, rhs)?,
            ExprKind::AssignRef { target, source } => self.assign_ref(target, source)?,
            ExprKind::AssignRefCall { target, call } => self.assign_ref_call(target, call)?,
            ExprKind::Closure { fn_idx, captures, bind_this } => {
                self.emit(Op::MakeClosure {
                    fn_idx: *fn_idx as u32,
                    captures: captures.clone().into_boxed_slice(),
                    bind_this: *bind_this,
                });
            }
            ExprKind::FirstClassCallable(name) => {
                self.emit(Op::MakeFcc { name: name.clone() });
            }
            ExprKind::Throw(e) => {
                // PHP 8 `throw` is an expression that diverges; evaluate the
                // operand and raise. Any value the surrounding context expected is
                // never produced (the following ops are unreachable).
                self.expr(e)?;
                self.emit(Op::Throw);
            }
            ExprKind::CallDynamic { callee, args } => {
                // Push the callee, then the arguments; `CallValue` dispatches on the
                // callee at run time. With argument unpacking (`$f(...$a)`) the
                // arguments are built into a runtime array and expanded by
                // `CallValueArgs` (the value-callee analogue of `MethodCallDynamicArgs`).
                self.expr(callee)?;
                if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    self.build_args_array(args)?;
                    self.emit(Op::CallValueArgs);
                } else {
                    for a in args {
                        self.expr(a)?;
                    }
                    self.emit(Op::CallValue { argc: args.len() as u32 });
                }
            }
            ExprKind::Pipe { input, callable } => {
                // `input |> callable` == `callable(input)`, but the operands evaluate
                // left-to-right: push the input first, then the callable, then swap
                // so the stack is [callable, input] as `CallValue` expects.
                self.expr(input)?;
                self.expr(callable)?;
                self.emit(Op::Swap);
                self.emit(Op::CallValue { argc: 1 });
            }
            ExprKind::AssignOpPlace(op, place, rhs) => self.assign_op_place(*op, place, rhs)?,
            ExprKind::IncDecPlace { place, inc, pre } => self.incdec_place(place, *inc, *pre)?,
            ExprKind::Isset(places) => self.isset(places)?,
            ExprKind::Empty(place) => self.empty(place)?,
            ExprKind::Coalesce(a, b) => {
                // `$o->p ?? d`: a property is read isset-aware — `__isset` decides
                // and `__get` runs only when set, so an unset magic property never
                // warns or calls `__get`. Other operands (Var/Index) read silently.
                if let ExprKind::PropGet { object, name, nullsafe } = &a.kind {
                    if !nullsafe {
                        self.expr(object)?; // [obj]
                        self.emit(Op::Dup); // [obj, obj]
                        self.emit(Op::PropIsset { name: name.clone() }); // [obj, isset]
                        let to_default = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → default; [obj]
                        self.emit(Op::PropGet { name: name.clone() }); // set → [value]
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let default_at = self.here();
                        self.patch(to_default, Op::JumpIfFalse(default_at));
                        self.emit(Op::Pop); // drop the kept object
                        self.expr(b)?; // [default]
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                        return Ok(());
                    }
                }
                // `$o->$n ?? d`: read the dynamic property *silently* (a missing or
                // null property both take the default, exactly as `??` requires) so
                // no "Undefined property" warning leaks.
                if let ExprKind::PropGetDyn { object, name, nullsafe } = &a.kind {
                    if !nullsafe {
                        self.expr(object)?; // [obj]
                        self.expr(name)?; // [obj, name]
                        self.emit(Op::PropGetDynamicSilent); // [value-or-null]
                        let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                        self.expr(b)?; // [default]
                        let end = self.here();
                        self.patch(to_end, Op::JumpIfNotNull(end));
                        return Ok(());
                    }
                }
                // `$x[k] ?? d`: read the element isset-aware so a missing array key
                // OR an out-of-range/non-integer string offset takes the default
                // (a plain read of a string offset yields "", not null).
                if let ExprKind::Index { base, index } = &a.kind {
                    self.expr(base)?;
                    self.expr(index)?;
                    self.emit(Op::CoalesceFetchDim);
                    let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                    self.expr(b)?;
                    let end = self.here();
                    self.patch(to_end, Op::JumpIfNotNull(end));
                    return Ok(());
                }
                // Left read silently — `??` suppresses the undefined-variable
                // warning, so a plain `$x` uses the silent LoadSlot, not LoadVar.
                if let ExprKind::Var(slot) = &a.kind {
                    self.emit(Op::LoadSlot(*slot));
                } else {
                    self.expr(a)?;
                }
                let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                self.expr(b)?;
                let end = self.here();
                self.patch(to_end, Op::JumpIfNotNull(end));
            }
            ExprKind::AssignCoalesce(slot, rhs) => {
                self.emit(Op::LoadSlot(*slot));
                let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                self.expr(rhs)?;
                self.emit(Op::Dup); // the assignment yields the stored value
                self.emit(Op::StoreSlot(*slot));
                let end = self.here();
                self.patch(to_end, Op::JumpIfNotNull(end));
            }
            ExprKind::Match { subject, arms } => self.match_expr(subject, arms)?,
            ExprKind::New { class, args, named } => self.new_obj(class, args, named)?,
            ExprKind::This => {
                self.emit(Op::This);
            }
            ExprKind::Clone(e) => {
                self.expr(e)?;
                self.emit(Op::Clone);
            }
            ExprKind::Eval(e) => {
                self.expr(e)?;
                self.emit(Op::Eval);
            }
            ExprKind::Include { mode, path } => {
                self.expr(path)?;
                self.emit(Op::Include { mode: *mode });
            }
            ExprKind::PropGet { object, name, nullsafe } => {
                self.expr(object)?;
                if *nullsafe {
                    // `$o?->p`: a null receiver keeps null and skips the read.
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.emit(Op::PropGet { name: name.clone() });
                    let end = self.here();
                    self.patch(skip, Op::JumpIfNull(end));
                } else {
                    self.emit(Op::PropGet { name: name.clone() });
                }
            }
            ExprKind::MethodCall { object, method, args, named, nullsafe } => {
                self.expr(object)?;
                if *nullsafe {
                    // `$o?->m(...)`: a null receiver keeps null and skips the call
                    // (its arguments are not evaluated either).
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.emit_method_call(method, args, named)?;
                    let end = self.here();
                    self.patch(skip, Op::JumpIfNull(end));
                } else {
                    self.emit_method_call(method, args, named)?;
                }
            }
            ExprKind::PropGetDyn { object, name, nullsafe } => {
                self.expr(object)?;
                if *nullsafe {
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.expr(name)?; // [obj, name]
                    self.emit(Op::PropGetDynamic);
                    let end = self.here();
                    self.patch(skip, Op::JumpIfNull(end));
                } else {
                    self.expr(name)?; // [obj, name]
                    self.emit(Op::PropGetDynamic);
                }
            }
            ExprKind::MethodCallDyn { object, method, args, named, nullsafe } => {
                self.expr(object)?;
                if *nullsafe {
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.emit_method_call_dyn(method, args, named)?;
                    let end = self.here();
                    self.patch(skip, Op::JumpIfNull(end));
                } else {
                    self.emit_method_call_dyn(method, args, named)?;
                }
            }
            ExprKind::InstanceOf { expr, class } => self.instance_of(expr, class)?,
            ExprKind::StaticCall { class, method, args, named } => {
                // `Closure::bind`/`fromCallable` are built-in statics with no compiled
                // class — must be handled before the runtime-class routing (a missing
                // `Closure` class would otherwise look "unknown").
                let closure_static =
                    matches!(class, ClassRef::Named(n) if n.eq_ignore_ascii_case(b"Closure"));
                if !closure_static && self.is_runtime_class(class) {
                    if !named.is_empty() {
                        return Err(CompileError::Unsupported("named arguments on `$cls::m()`".into()));
                    }
                    // `$cls::m()` / an unknown named class (PAR): the class reference
                    // is pushed beneath the arguments and resolved at run time.
                    self.push_class_value(class)?;
                    if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                        // Spread `$cls::m(...$a)` (Session A): args from a runtime array.
                        self.build_args_array(args)?;
                        self.emit(Op::StaticCallDynamicArgs { method: method.clone() });
                    } else {
                        self.push_value_args(args)?;
                        self.emit(Op::StaticCallDynamic { method: method.clone(), argc: args.len() as u32 });
                    }
                } else if matches!(class, ClassRef::Named(n) if n.eq_ignore_ascii_case(b"Closure")) {
                    // `Closure::bind` / `Closure::fromCallable` are built-in statics
                    // — there is no compiled `Closure` class to resolve against.
                    if !named.is_empty()
                        || args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)))
                    {
                        return Err(CompileError::Unsupported(
                            "named/spread arguments on `Closure::m()`".into(),
                        ));
                    }
                    self.push_value_args(args)?;
                    self.emit(Op::ClosureStatic { method: method.clone(), argc: args.len() as u32 });
                } else {
                    let (target, forwarding) = self.resolve_target(class)?;
                    if named.is_empty() {
                        if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                            // Spread `C::m(...$a)` (Session A): args from a runtime array.
                            self.build_args_array(args)?;
                            self.emit(Op::StaticCallArgs { target, method: method.clone(), forwarding });
                        } else {
                            self.push_value_args(args)?;
                            self.emit(Op::StaticCall { target, method: method.clone(), forwarding, argc: args.len() as u32 });
                        }
                    } else {
                        // Named args: resolve the method at compile time against the
                        // known class's parameters (PAR). `static::` is run-time only.
                        let cid = match target {
                            ClassTarget::Class(c) => c,
                            ClassTarget::Static => {
                                return Err(CompileError::Unsupported(
                                    "named arguments on `static::m()`".into(),
                                ))
                            }
                        };
                        let (defc, midx) = self.resolve_method_compile(cid, method).ok_or_else(|| {
                            CompileError::Unsupported("named call to an unresolved static method".into())
                        })?;
                        let method_fd = &self.ctx.classes[defc].methods[midx].decl;
                        let n = method_fd.params.len() as u32;
                        self.emit_named_layout(method_fd, args, named)?;
                        self.emit(Op::StaticCall { target, method: method.clone(), forwarding, argc: n });
                    }
                }
            }
            ExprKind::ClassConst { class, name } => self.class_const(class, name)?,
            ExprKind::StaticProp { class, name } => {
                if self.is_runtime_class(class) {
                    self.push_class_value(class)?;
                    self.emit(Op::StaticPropGetDynamic { name: name.clone() });
                } else {
                    let (target, _) = self.resolve_target(class)?;
                    self.emit(Op::StaticPropGet { target, name: name.clone() });
                }
            }
            ExprKind::StaticPropAssign { class, name, op, rhs } => {
                // `$cls::$p` (PAR): resolve the class at run time; the rhs is
                // pushed first so the class reference ends up on top.
                if self.is_runtime_class(class) {
                    match op {
                        StaticAssignOp::Plain => {
                            self.expr(rhs)?;
                            self.push_class_value(class)?;
                            self.emit(Op::StaticPropSetDynamic { name: name.clone() });
                        }
                        StaticAssignOp::Op(b) => {
                            self.expr(rhs)?;
                            self.push_class_value(class)?;
                            self.emit(Op::StaticPropOpSetDynamic { name: name.clone(), op: *b });
                        }
                        StaticAssignOp::Coalesce => {
                            // `$cls::$p ??= rhs`: the class reference is evaluated
                            // *once* into a temp and reused for the read and the
                            // conditional write (the rhs is evaluated only when the
                            // property is null).
                            let t = self.alloc_temp();
                            self.push_class_value(class)?;
                            self.emit(Op::StoreSlot(t));
                            self.emit(Op::LoadSlot(t));
                            self.emit(Op::StaticPropGetDynamic { name: name.clone() });
                            let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                            self.expr(rhs)?;
                            self.emit(Op::LoadSlot(t)); // class ref on top for the set
                            self.emit(Op::StaticPropSetDynamic { name: name.clone() });
                            let end = self.here();
                            self.patch(to_end, Op::JumpIfNotNull(end));
                            self.free_temp();
                        }
                    }
                    return Ok(());
                }
                let (target, _) = self.resolve_target(class)?;
                match op {
                    StaticAssignOp::Plain => {
                        self.expr(rhs)?;
                        self.emit(Op::StaticPropSet { target, name: name.clone() });
                    }
                    StaticAssignOp::Op(b) => {
                        self.expr(rhs)?;
                        self.emit(Op::StaticPropOpSet { target, name: name.clone(), op: *b });
                    }
                    StaticAssignOp::Coalesce => {
                        // `C::$p ??= rhs`: read, keep if non-null, else assign.
                        self.emit(Op::StaticPropGet { target, name: name.clone() });
                        let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                        self.expr(rhs)?;
                        self.emit(Op::StaticPropSet { target, name: name.clone() });
                        let end = self.here();
                        self.patch(to_end, Op::JumpIfNotNull(end));
                    }
                }
            }
            ExprKind::StaticPropIncDec { class, name, inc, pre } => {
                if self.is_runtime_class(class) {
                    // `$cls::$p++` (PAR): the class reference is resolved at run time.
                    self.push_class_value(class)?;
                    self.emit(Op::StaticPropIncDecDynamic { name: name.clone(), inc: *inc, pre: *pre });
                } else {
                    let (target, _) = self.resolve_target(class)?;
                    self.emit(Op::StaticPropIncDec { target, name: name.clone(), inc: *inc, pre: *pre });
                }
            }
            ExprKind::Yield { key, value } => {
                // `yield`, `yield $v`, `yield $k => $v` (GEN). Push the value (NULL
                // for a bare `yield`) and, if present, the key beneath it, then
                // suspend. `Op::Yield` leaves the `send()` value on the stack, so
                // the `yield` expression yields it (and `StmtKind::Expr` pops it).
                if let Some(k) = key {
                    self.expr(k)?;
                }
                match value {
                    Some(v) => self.expr(v)?,
                    None => {
                        let null = self.konst(Const::Null);
                        self.emit(Op::PushConst(null));
                    }
                }
                self.emit(Op::Yield { has_key: key.is_some() });
            }
            ExprKind::YieldFrom(delegate) => {
                // `yield from $x` (GEN-3): push the delegate, then the re-entrant
                // `YieldFrom` op drives the delegation and leaves its return value.
                self.expr(delegate)?;
                self.emit(Op::YieldFrom);
            }
            other => return Err(CompileError::Unsupported(expr_name(other))),
        }
        Ok(())
    }

    /// Compile `&&` (`want_true == false`) / `||` (`want_true == true`) to a
    /// boolean result via short-circuit jumps. Leaves `true`/`false` on the stack.
    fn short_circuit(&mut self, a: &Expr, b: &Expr, want_true: bool) -> R<()> {
        // For `&&`: if either operand is falsy, result is false (jump to L_short).
        // For `||`: if either operand is truthy, result is true.
        let short = |s: &mut Self| {
            if want_true {
                s.emit(Op::JumpIfTrue(Addr::MAX))
            } else {
                s.emit(Op::JumpIfFalse(Addr::MAX))
            }
        };
        self.expr(a)?;
        let j1 = short(self);
        self.expr(b)?;
        let j2 = short(self);
        // Fell through: `&&` → true, `||` → false.
        let fallthrough = self.konst(Const::Bool(!want_true));
        self.emit(Op::PushConst(fallthrough));
        let to_end = self.emit(Op::Jump(Addr::MAX));
        let short_at = self.here();
        self.patch(j1, if want_true { Op::JumpIfTrue(short_at) } else { Op::JumpIfFalse(short_at) });
        self.patch(j2, if want_true { Op::JumpIfTrue(short_at) } else { Op::JumpIfFalse(short_at) });
        let shorted = self.konst(Const::Bool(want_true));
        self.emit(Op::PushConst(shorted));
        let end = self.here();
        self.patch(to_end, Op::Jump(end));
        Ok(())
    }

    /// Compile a named function call `name(args...)`.
    ///
    /// Resolution mirrors the evaluator: a *user* function (matched
    /// ASCII-case-insensitively) shadows builtins; otherwise the name is looked
    /// up in the registry — a by-value builtin emits [`Op::CallBuiltin`], a
    /// by-reference-first builtin (`sort`, …) emits [`Op::CallBuiltinRef`]. A name
    /// absent from the registry (higher-order / class-introspection /
    /// `define`-family / undefined) is out of slice, so the script falls back to
    /// the tree-walker. Named/spread arguments and user by-ref/variadic params are
    /// likewise deferred.
    fn call(&mut self, name: &[u8], args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        // User functions shadow builtins — but only *hoisted* ones bind statically;
        // a conditional declaration is dispatched dynamically (callable only after
        // its `DeclareFn` runs).
        if let Some(idx) = self
            .ctx
            .funcs
            .iter()
            .enumerate()
            .find(|(i, f)| !self.ctx.conditional_fns.contains(i) && ascii_eq_ignore_case(&f.name, name))
            .map(|(i, _)| i)
        {
            // Named arguments are resolved to parameter slots at compile time
            // (the callee is known), PAR.
            // A spread anywhere (`f(...$src)`, with or without named args) needs
            // run-time binding: integer keys become positional, string keys named.
            // By-value callees only (by-ref + spread is out of slice).
            if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                let callee = &self.ctx.funcs[idx];
                if callee.by_ref || callee.params.iter().any(|p| p.by_ref) {
                    return Err(CompileError::Unsupported(
                        "spread call to a by-reference function".into(),
                    ));
                }
                return self.emit_call_spread(idx, args, named);
            }
            if !named.is_empty() {
                return self.call_user_named(idx, args, named);
            }
            let callee = &self.ctx.funcs[idx];
            // Omitted optional args are filled by the callee's default prologue
            // (PAR); extra args are dropped by the binder.
            // Snapshot the by-ref mask so the immutable borrow of `callee` ends
            // before `push_call_args` borrows `self` mutably (REF-2).
            let by_ref: Vec<bool> = callee.params.iter().map(|p| p.by_ref).collect();
            let pnames: Vec<Box<[u8]>> =
                callee.params.iter().map(|p| callee.slots[p.slot as usize].clone()).collect();
            let returns_ref = callee.by_ref;
            self.push_call_args(args, &by_ref, name, &pnames)?;
            self.emit(Op::Call { func: idx as u32, argc: args.len() as u32 });
            // A `function &f()` used in value context yields a copy, not an alias
            // (REF-4b). `$y = &f()` takes the raw ref via `AssignRefCall` instead.
            if returns_ref {
                self.emit(Op::DerefTop);
            }
            return Ok(());
        }
        if !named.is_empty() {
            return Err(CompileError::Unsupported("builtin call with named arguments".into()));
        }
        // Evaluator-only *host* builtins (higher-order / class-introspection /
        // define-family, Sessions B–D) need the VM itself, so they are dispatched
        // VM-side via `Op::CallHostBuiltin` rather than the stateless registry.
        if let Some(canon) = crate::vm::host_builtin_canonical(name) {
            self.push_value_args(args)?; // rejects spread (out of slice here)
            self.emit(Op::CallHostBuiltin { name: canon.into(), argc: args.len() as u32 });
            return Ok(());
        }
        // Host builtins with a by-reference *output* parameter (`preg_match`'s
        // `&$matches` at index 2): push every argument by value, capture the
        // out-param's slot, and let the VM write the produced value back.
        if let Some((canon, out_idx)) = crate::vm::host_builtin_out_param(name) {
            let out_slot = match args.get(out_idx) {
                None => None, // out-param omitted (e.g. `preg_match($p, $s)`)
                Some(e) => match &e.kind {
                    ExprKind::Var(slot) => Some(*slot),
                    _ => {
                        return Err(CompileError::Unsupported(
                            "host builtin out-param is not a plain variable".into(),
                        ))
                    }
                },
            };
            self.push_value_args(args)?;
            self.emit(Op::CallHostBuiltinOut {
                name: canon.into(),
                out_slot,
                out_index: out_idx as u32,
                argc: args.len() as u32,
            });
            return Ok(());
        }
        // Host builtins with variadic by-reference output parameters (`sscanf`/
        // `fscanf`'s `...&$vars` from index 2): push the two fixed arguments by
        // value and capture each trailing out argument's slot (a non-variable
        // target becomes `None`, silently skipped at run time, D-54.1).
        if let Some(canon) = crate::vm::host_builtin_scanf(name) {
            let fixed = args.len().min(2);
            self.push_value_args(&args[..fixed])?;
            let out_slots = args[fixed..]
                .iter()
                .map(|e| match &e.kind {
                    ExprKind::Var(slot) => Some(*slot),
                    _ => None,
                })
                .collect::<Vec<_>>();
            self.emit(Op::CallHostBuiltinScanf {
                name: canon.into(),
                argc: fixed as u32,
                out_slots: out_slots.into(),
            });
            return Ok(());
        }
        // By-reference-first host builtins (`usort`, …): first argument is an array
        // variable taken by reference, the rest by value (Session C).
        if let Some(canon) = crate::vm::host_builtin_ref_first(name) {
            let Some((first, rest)) = args.split_first() else {
                return Err(CompileError::Unsupported(
                    "by-reference host builtin called with no arguments".into(),
                ));
            };
            match &first.kind {
                ExprKind::Var(slot) => {
                    let slot = *slot;
                    self.push_value_args(rest)?;
                    self.emit(Op::CallHostBuiltinRef {
                        name: canon.into(),
                        slot,
                        argc: rest.len() as u32,
                    });
                    return Ok(());
                }
                // Bare static property (`usort(self::$items, $cb)`): RMW through a
                // temp, mirroring the registry by-ref path.
                ExprKind::StaticProp { class, name: prop } => {
                    let canon: Box<[u8]> = canon.into();
                    return self.static_prop_rmw(class, prop, &[], true, |c, place| {
                        let slot = local_slot(place);
                        c.push_value_args(rest)?;
                        c.emit(Op::CallHostBuiltinRef {
                            name: canon,
                            slot,
                            argc: rest.len() as u32,
                        });
                        Ok(())
                    });
                }
                _ => {
                    return Err(CompileError::Unsupported(
                        "by-reference host builtin whose first argument is not a plain variable"
                            .into(),
                    ))
                }
            }
        }
        // Builtins: classify by-value vs by-reference-first via the registry.
        match self.ctx.registry.get(name) {
            Some(Builtin::Value(_)) => {
                if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    return self.emit_builtin_spread(name, args);
                }
                self.push_value_args(args)?;
                self.emit(Op::CallBuiltin { name: name.into(), argc: args.len() as u32 });
                Ok(())
            }
            Some(Builtin::RefFirst(_)) => self.call_ref_builtin(name, args),
            None => {
                // An unknown name is NOT a compile error: PHP defers "Call to
                // undefined function" to run time (the function may be declared
                // conditionally, or be defined after this point). Push the name as a
                // string callable and dispatch dynamically — `invoke_named` raises
                // the catchable `Error` at the actual call site, after any output /
                // argument side effects, matching the tree-walker.
                if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
                }
                let k = self.konst(Const::Str(name.into()));
                self.emit(Op::PushConst(k));
                for a in args {
                    self.expr(a)?;
                }
                self.emit(Op::CallValue { argc: args.len() as u32 });
                Ok(())
            }
        }
    }

    /// Compile a call to known user function `idx` that has named arguments
    /// (PAR): lay positional then named args into the parameter slots at compile
    /// time, pushing `Undef` for any skipped optional (the callee's default
    /// prologue then fills it), and emit a normal positional `Op::Call`. Falls
    /// back to the tree-walker for what the compile-time layout can't express:
    /// variadic / by-ref parameters, an unknown or duplicate name, a missing
    /// required argument, or a spread.
    fn call_user_named(&mut self, idx: usize, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        let fd = &self.ctx.funcs[idx];
        let n = fd.params.len() as u32;
        let returns_ref = fd.by_ref;
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        // Fast path: lay the arguments into slots at compile time when expressible.
        if !has_spread && self.can_emit_named_layout(fd, args, named) {
            self.emit_named_layout(fd, args, named)?;
            self.emit(Op::Call { func: idx as u32, argc: n });
            if returns_ref {
                self.emit(Op::DerefTop);
            }
            return Ok(());
        }
        if has_spread {
            // Spread + named is the run-time spread path (handled elsewhere).
            return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
        }
        // Run-time named binding: a variadic / by-ref parameter, or a name that is
        // unknown / collides / routes into `...$rest` — all of which PHP resolves
        // (or errors) at run time rather than at compile time. Push positional args
        // (honouring by-ref), then one value per named arg (by-ref when its target
        // parameter is), and let `build_named_frame` bind them.
        let by_ref: Vec<bool> = fd.params.iter().map(|p| p.by_ref).collect();
        let fname: Box<[u8]> = fd.name.clone();
        let pnames: Vec<Box<[u8]>> =
            fd.params.iter().map(|p| fd.slots[p.slot as usize].clone()).collect();
        let named_by_ref: Vec<bool> = named
            .iter()
            .map(|(nm, _)| {
                fd.params
                    .iter()
                    .find(|p| fd.slots[p.slot as usize][..] == nm[..])
                    .is_some_and(|p| p.by_ref)
            })
            .collect();
        self.push_call_args(args, &by_ref, &fname, &pnames)?;
        for ((_, expr), &br) in named.iter().zip(&named_by_ref) {
            match (&expr.kind, br) {
                (ExprKind::Var(slot), true) => {
                    self.emit(Op::PushRef(*slot));
                }
                _ => self.expr(expr)?,
            }
        }
        self.emit(Op::CallNamed {
            func: idx as u32,
            positional: args.len() as u32,
            names: named.iter().map(|(nm, _)| nm.clone()).collect(),
        });
        if returns_ref {
            self.emit(Op::DerefTop);
        }
        Ok(())
    }

    /// Whether [`emit_named_layout`] can lay this named call into parameter slots
    /// at compile time. Mirrors its reject conditions without emitting: a variadic
    /// or by-reference parameter, a spread, too many positionals, an unknown or
    /// colliding name, or a missing required argument all force the run-time binder.
    fn can_emit_named_layout(&self, fd: &FnDecl, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> bool {
        if fd.params.iter().any(|p| p.variadic || p.by_ref) {
            return false;
        }
        let n = fd.params.len();
        if args.len() > n || args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
            return false;
        }
        let mut filled = vec![false; n];
        for slot in filled.iter_mut().take(args.len()) {
            *slot = true;
        }
        for (nm, _) in named {
            match fd
                .params
                .iter()
                .position(|p| fd.slots[p.slot as usize][..] == nm[..])
            {
                Some(pi) if !filled[pi] => filled[pi] = true,
                _ => return false, // unknown name or overwrite
            }
        }
        for p in &fd.params {
            if p.default.is_none() && !filled[p.slot as usize] {
                return false;
            }
        }
        true
    }

    /// Compile a spread function call `f(comp…, name: v, …)` (PAR): push one value
    /// per leading component — a positional value or a spread *source* (marked in
    /// `spreads`) — then the explicit named values, and let `Op::CallSpread` expand
    /// and bind them at run time.
    fn emit_call_spread(
        &mut self,
        idx: usize,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        let returns_ref = self.ctx.funcs[idx].by_ref;
        let mut spreads = Vec::with_capacity(args.len());
        for a in args {
            if let ExprKind::Spread(src) = &a.kind {
                self.expr(src)?;
                spreads.push(true);
            } else {
                self.expr(a)?;
                spreads.push(false);
            }
        }
        for (_, expr) in named {
            self.expr(expr)?;
        }
        self.emit(Op::CallSpread {
            func: idx as u32,
            spreads: spreads.into(),
            names: named.iter().map(|(n, _)| n.clone()).collect(),
        });
        if returns_ref {
            self.emit(Op::DerefTop);
        }
        Ok(())
    }

    /// Compile a spread call into a by-value builtin `b(comp…)` (step 56b): push
    /// one value per leading component (a positional value, or a spread *source*
    /// marked in `spreads`), then let `Op::CallBuiltinSpread` flatten and run it.
    fn emit_builtin_spread(&mut self, name: &[u8], args: &[Expr]) -> R<()> {
        let mut spreads = Vec::with_capacity(args.len());
        for a in args {
            if let ExprKind::Spread(src) = &a.kind {
                self.expr(src)?;
                spreads.push(true);
            } else {
                self.expr(a)?;
                spreads.push(false);
            }
        }
        self.emit(Op::CallBuiltinSpread { name: name.into(), spreads: spreads.into() });
        Ok(())
    }

    /// Lay named + positional arguments into `fd`'s parameter slots at compile
    /// time and emit them in slot order — pushing `Undef` for a skipped optional
    /// (the callee's default prologue fills it) — so a normal positional call op
    /// with `argc = fd.params.len()` can follow (PAR). Shared by named function,
    /// `new`, and static calls. Returns `Unsupported` for what the compile-time
    /// layout can't express: variadic / by-ref parameters, an unknown or
    /// duplicate name, a missing required argument, or a spread.
    fn emit_named_layout(
        &mut self,
        fd: &FnDecl,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        if fd.params.iter().any(|p| p.variadic || p.by_ref) {
            return Err(CompileError::Unsupported(
                "named arguments with a variadic or by-reference parameter".into(),
            ));
        }
        let n = fd.params.len();
        if args.len() > n {
            return Err(CompileError::Unsupported(
                "named call with too many positional arguments".into(),
            ));
        }
        // Lay each argument into its parameter slot; `None` is a skipped optional.
        let mut slots: Vec<Option<&Expr>> = vec![None; n];
        for (i, a) in args.iter().enumerate() {
            if matches!(a.kind, ExprKind::Spread(_)) {
                return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
            }
            slots[i] = Some(a);
        }
        for (nm, expr) in named {
            let pos = fd
                .params
                .iter()
                .position(|p| fd.slots[p.slot as usize][..] == nm[..]);
            match pos {
                Some(pi) if slots[pi].is_none() => slots[pi] = Some(expr),
                Some(_) => {
                    return Err(CompileError::Unsupported(
                        "named argument overwrites a positional one".into(),
                    ))
                }
                None => {
                    return Err(CompileError::Unsupported(format!(
                        "unknown named parameter ${}",
                        String::from_utf8_lossy(nm)
                    )))
                }
            }
        }
        // Every required (default-less) parameter must be supplied.
        for p in &fd.params {
            if p.default.is_none() && slots[p.slot as usize].is_none() {
                return Err(CompileError::Unsupported(
                    "named call missing a required argument".into(),
                ));
            }
        }
        // Emit in slot order; a gap pushes `Undef` for the default prologue.
        for s in slots {
            match s {
                Some(e) => self.expr(e)?,
                None => {
                    self.emit(Op::PushUndef);
                }
            }
        }
        Ok(())
    }

    /// Push each positional argument for a user call, honouring by-reference
    /// parameters (REF-2): a by-ref position whose argument is a plain variable
    /// is passed by [`Op::PushRef`] (the callee slot aliases the caller's cell);
    /// every other position is pushed by value. A by-ref position with a
    /// non-variable argument (e.g. a literal) is a *run-time* catchable `Error` in
    /// PHP — `fname(): Argument #N ($p) could not be passed by reference` — so it
    /// compiles to an [`Op::Fatal`] in place rather than a compile rejection.
    /// `fname` is the callee's display name and `pnames` its parameter names
    /// (indexed positionally) for that message.
    fn push_call_args(
        &mut self,
        args: &[Expr],
        by_ref: &[bool],
        fname: &[u8],
        pnames: &[Box<[u8]>],
    ) -> R<()> {
        for (i, a) in args.iter().enumerate() {
            if matches!(a.kind, ExprKind::Spread(_)) {
                return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
            }
            if by_ref.get(i).copied().unwrap_or(false) {
                match &a.kind {
                    ExprKind::Var(slot) => {
                        self.emit(Op::PushRef(*slot));
                    }
                    _ => {
                        let pname = pnames.get(i).map(|n| n.as_ref()).unwrap_or(b"");
                        let msg = format!(
                            "{}(): Argument #{} (${}) could not be passed by reference",
                            String::from_utf8_lossy(fname),
                            i + 1,
                            String::from_utf8_lossy(pname),
                        );
                        let k = self.konst(Const::Str(msg.into_bytes().into()));
                        self.emit(Op::Fatal(k));
                        return Ok(());
                    }
                };
            } else {
                self.expr(a)?;
            }
        }
        Ok(())
    }

    /// Build a runtime argument array on the stack from `args`, expanding spreads
    /// (`...$src` via [`Op::ArrayAppendSpread`]) and pushing positional values
    /// (via [`Op::ArrayPush`]). Mirrors the `f(...$arr)` path (PAR-13) but feeds a
    /// dynamic-dispatch call (`$obj->m(...)`, `new C(...)`, `C::m(...)`, Session A)
    /// whose callee — hence parameter count — isn't known at compile time. Leaves
    /// the array on top of the stack.
    fn build_args_array(&mut self, args: &[Expr]) -> R<()> {
        self.emit(Op::ArrayInit);
        for a in args {
            if let ExprKind::Spread(src) = &a.kind {
                self.expr(src)?;
                self.emit(Op::ArrayAppendSpread);
            } else {
                self.expr(a)?;
                self.emit(Op::ArrayPush);
            }
        }
        Ok(())
    }

    /// Emit an instance method call `$obj->m(args)` (the receiver is already on the
    /// stack). Named arguments (`name: v`) push the positional values then the named
    /// values and dispatch via [`Op::MethodCallNamed`] (the method, hence its
    /// parameters, is only known at run time); a spread (`...$a`) builds a runtime
    /// argument array for [`Op::MethodCallArgs`]; otherwise the values are pushed
    /// positionally for [`Op::MethodCall`]. Named + spread mixed falls back to the
    /// evaluator.
    fn emit_method_call(
        &mut self,
        method: &[u8],
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        if !named.is_empty() {
            if has_spread {
                return Err(CompileError::Unsupported(
                    "method call mixing argument unpacking and named arguments".into(),
                ));
            }
            // Positional values first, then each named value (its label rides in the
            // op); the run-time binder maps names against the callee's params.
            self.push_value_args(args)?;
            for (_, expr) in named {
                self.expr(expr)?;
            }
            self.emit(Op::MethodCallNamed {
                method: method.into(),
                positional: args.len() as u32,
                names: named.iter().map(|(n, _)| n.clone()).collect(),
            });
        } else if has_spread {
            self.build_args_array(args)?;
            self.emit(Op::MethodCallArgs { method: method.into() });
        } else {
            self.push_value_args(args)?;
            self.emit(Op::MethodCall { method: method.into(), argc: args.len() as u32 });
        }
        Ok(())
    }

    /// Emit a dynamic instance method call `$obj->$m(args)` / `$obj->{expr}(args)`
    /// (the receiver is already on the stack). The method-name expression is pushed
    /// last — on top of the positional args / args-array — so the VM pops the name,
    /// then dispatches on the same `[obj, args…]` layout as the static path (step
    /// 51). Named arguments on a dynamic call fall back to the evaluator.
    fn emit_method_call_dyn(
        &mut self,
        method: &Expr,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        if !named.is_empty() {
            return Err(CompileError::Unsupported(
                "named arguments on a dynamic method call".into(),
            ));
        }
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        if has_spread {
            self.build_args_array(args)?; // [obj, argsArray]
            self.expr(method)?; // [obj, argsArray, name]
            self.emit(Op::MethodCallDynamicArgs);
        } else {
            self.push_value_args(args)?; // [obj, arg0…]
            self.expr(method)?; // [obj, arg0…, name]
            self.emit(Op::MethodCallDynamic { argc: args.len() as u32 });
        }
        Ok(())
    }

    /// Push each positional argument's value (source order); reject spreads.
    fn push_value_args(&mut self, args: &[Expr]) -> R<()> {
        for a in args {
            if matches!(a.kind, ExprKind::Spread(_)) {
                return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
            }
            self.expr(a)?;
        }
        Ok(())
    }

    /// Emit a by-reference-first builtin call (`sort`, `array_push`, …). As the
    /// evaluator requires, the first argument must be a plain variable: it is
    /// passed by reference via its slot, the rest by value.
    fn call_ref_builtin(&mut self, name: &[u8], args: &[Expr]) -> R<()> {
        let Some((first, rest)) = args.split_first() else {
            return Err(CompileError::Unsupported(
                "by-reference builtin called with no arguments".into(),
            ));
        };
        match &first.kind {
            ExprKind::Var(slot) => {
                let slot = *slot;
                self.push_value_args(rest)?;
                self.emit(Op::CallBuiltinRef { name: name.into(), slot, argc: rest.len() as u32 });
                Ok(())
            }
            // A bare static property as the by-reference argument
            // (`array_pop(self::$stack)`): read-modify-write through a temp — load
            // the property, run the builtin in place on the temp slot (leaving its
            // result), then write the mutated temp back into the property.
            ExprKind::StaticProp { class, name: prop } => {
                let nm: Box<[u8]> = name.into();
                self.static_prop_rmw(class, prop, &[], true, |c, place| {
                    let slot = local_slot(place);
                    c.push_value_args(rest)?;
                    c.emit(Op::CallBuiltinRef { name: nm, slot, argc: rest.len() as u32 });
                    Ok(())
                })
            }
            _ => Err(CompileError::Unsupported(
                "by-reference builtin whose first argument is not a plain variable".into(),
            )),
        }
    }

    /// Emit the run-time constructor invocation for `new static` / `new $cls` (the
    /// receiver is already duplicated on the stack). A spread (`...$a`) builds a
    /// runtime argument array and uses [`Op::InvokeCtorArgs`]; otherwise the values
    /// are pushed positionally for [`Op::InvokeCtor`].
    fn emit_invoke_ctor(&mut self, args: &[Expr]) -> R<()> {
        if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
            self.build_args_array(args)?;
            self.emit(Op::InvokeCtorArgs);
        } else {
            self.push_value_args(args)?;
            self.emit(Op::InvokeCtor { argc: args.len() as u32 });
        }
        Ok(())
    }

    /// Compile `new ClassRef(args)` (no named / spread arguments). OOP-1 handled
    /// `Named`; OOP-2a adds `self` / `parent` (class id known at compile time) and
    /// `static` (the run-time LSB class). `Dynamic` stays out of slice.
    fn new_obj(&mut self, class: &ClassRef, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        match class {
            ClassRef::Named(name) => match self.resolve_class(name) {
                // Known at compile time: allocate + run the resolved constructor.
                Some(cid) => self.new_obj_cid(cid, args, named),
                // Unknown at compile time: PHP resolves `new X` at run time and
                // raises a catchable `Error: Class "X" not found` only if it is
                // truly undefined (it may be declared conditionally or later). Push
                // the name and allocate dynamically, exactly like `new $cls`.
                None => {
                    if !named.is_empty() {
                        return Err(CompileError::Unsupported(
                            "named arguments to `new` of an unknown class".into(),
                        ));
                    }
                    let k = self.konst(Const::Str(name.clone()));
                    self.emit(Op::PushConst(k));
                    self.emit_dynamic_new(args)
                }
            },
            ClassRef::SelfClass => {
                let cid = self
                    .cur_class
                    .ok_or_else(|| CompileError::Unsupported("`new self` outside class context".into()))?;
                self.new_obj_cid(cid, args, named)
            }
            ClassRef::Parent => {
                let cid = self
                    .cur_class
                    .and_then(|c| self.ctx.classes[c].parent)
                    .ok_or_else(|| CompileError::Unsupported("`new parent` without a parent class".into()))?;
                self.new_obj_cid(cid, args, named)
            }
            // `new static`/`new $cls` resolve the constructor at run time, where the
            // compile-time named layout can't be built.
            ClassRef::Static | ClassRef::Dynamic(_) if !named.is_empty() => {
                Err(CompileError::Unsupported("named arguments to `new static`/`new $cls`".into()))
            }
            ClassRef::Static => {
                // The actual class (hence the constructor) is only known at run
                // time, so allocate the LSB class and dispatch `__construct`
                // dynamically.
                self.emit(Op::AllocStatic);
                self.emit(Op::Dup);
                self.emit(Op::InitProps);
                self.emit(Op::Pop);
                // Fix line/file/trace on a Throwable after its defaults are set.
                self.emit(Op::StampThrowable);
                self.emit(Op::Dup);
                self.emit_invoke_ctor(args)?;
                self.emit(Op::Pop);
                Ok(())
            }
            ClassRef::Dynamic(expr) => {
                // `new $cls` (PAR): resolve the class at run time, then run the
                // constructor dynamically (like `new static`).
                self.expr(expr)?;
                self.emit_dynamic_new(args)
            }
        }
    }

    /// Emit the run-time `new` sequence for a class **value already on the stack**
    /// (a `new $cls` or a `new Name` whose class is unknown at compile time):
    /// resolve the class, init its properties, stamp a Throwable's location, and run
    /// the constructor dynamically, leaving the fresh object on the stack.
    fn emit_dynamic_new(&mut self, args: &[Expr]) -> R<()> {
        self.emit(Op::AllocDynamic);
        self.emit(Op::Dup);
        self.emit(Op::InitProps);
        self.emit(Op::Pop);
        self.emit(Op::StampThrowable);
        self.emit(Op::Dup);
        self.emit_invoke_ctor(args)?;
        self.emit(Op::Pop);
        Ok(())
    }

    /// `new` of a class whose id is known at compile time: allocate, then run the
    /// compile-time-resolved constructor (if any) with the fresh object as `$this`.
    fn new_obj_cid(&mut self, cid: ClassId, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        let ctor = self.resolve_method_compile(cid, b"__construct");
        if ctor.is_none() && !named.is_empty() {
            return Err(CompileError::Unsupported(
                "named arguments to a class with no constructor".into(),
            ));
        }
        self.emit(Op::Alloc { class: cid });
        // Materialise non-constant property defaults before the constructor runs.
        // `InitProps` is a no-op (pushes NULL) for classes with none.
        self.emit(Op::Dup);
        self.emit(Op::InitProps);
        self.emit(Op::Pop);
        // After defaults are in place, fix a Throwable's line/file/trace at the
        // `new` site (a no-op for non-Throwables), before the constructor runs.
        self.emit(Op::StampThrowable);
        // Spread `new C(...$a)` (Session A): the parameter count isn't known until
        // the array is flattened, so resolve the constructor at run time from the
        // fresh object's class (`InvokeCtorArgs`) — which also serves a ctor-less
        // class (it pushes NULL). Mixed spread + named falls back to the evaluator
        // (handled below by `emit_named_layout`, which rejects spreads).
        if named.is_empty() && args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
            self.emit(Op::Dup);
            self.build_args_array(args)?;
            self.emit(Op::InvokeCtorArgs);
            self.emit(Op::Pop);
            return Ok(());
        }
        if let Some((defc, midx)) = ctor {
            self.emit(Op::Dup); // keep the instance as the result; the dup is the receiver
            let argc = if named.is_empty() {
                self.push_value_args(args)?;
                args.len() as u32
            } else {
                // Resolve named arguments against the constructor's parameters (PAR).
                let ctor_fd = &self.ctx.classes[defc].methods[midx].decl;
                let n = ctor_fd.params.len() as u32;
                self.emit_named_layout(ctor_fd, args, named)?;
                n
            };
            self.emit(Op::InvokeMethod { class: defc, method_idx: midx as u32, argc });
            self.emit(Op::Pop); // discard the constructor's return value
        }
        Ok(())
    }

    /// Compile `expr instanceof ClassRef`. `Named`/`self`/`parent` resolve to a
    /// compile-time id; `static` tests the run-time LSB class. A named class not
    /// known at compile time is resolved by name at run time (so a class later
    /// provided by `eval`/`include`, or a conditional declaration, is honoured) —
    /// an unresolvable name still tests false, as PHP does.
    fn instance_of(&mut self, expr: &Expr, class: &ClassRef) -> R<()> {
        // Evaluate the operand first (PHP order), then test the class.
        match class {
            ClassRef::Named(name) => {
                self.expr(expr)?;
                match self.resolve_class(name) {
                    Some(cid) => self.emit(Op::InstanceOf { class: cid }),
                    None => match builtin_iface_for(name) {
                        // A built-in interface (Generator/Iterator/Traversable)
                        // has no ClassId; decide membership by runtime type.
                        Some(iface) => self.emit(Op::InstanceOfBuiltin(iface)),
                        None => {
                            // Unknown at compile time: push the name and resolve it
                            // against the live class table at run time.
                            let n = self.konst(Const::Str(name.clone()));
                            self.emit(Op::PushConst(n));
                            self.emit(Op::InstanceOfDynamic)
                        }
                    },
                };
            }
            ClassRef::SelfClass | ClassRef::Parent => {
                let (ClassTarget::Class(cid), _) = self.resolve_target(class)? else {
                    unreachable!("self/parent resolve to a class id")
                };
                self.expr(expr)?;
                self.emit(Op::InstanceOf { class: cid });
            }
            ClassRef::Static => {
                self.expr(expr)?;
                self.emit(Op::InstanceOfStatic);
            }
            ClassRef::Dynamic(cls) => {
                // `$x instanceof $cls` (PAR): evaluate the operand, then the class
                // reference, and test at run time.
                self.expr(expr)?;
                self.expr(cls)?;
                self.emit(Op::InstanceOfDynamic);
            }
        }
        Ok(())
    }

    /// Compile `ClassRef::name` — a class constant or the special `::class`.
    fn class_const(&mut self, class: &ClassRef, name: &[u8]) -> R<()> {
        // `::class` is a compile-time constant. A *named* class yields its
        // fully-qualified name as a string even when the class is undefined (PHP
        // resolves the name, not the class; the lowerer already made it the FQN).
        if name.eq_ignore_ascii_case(b"class") {
            match class {
                ClassRef::Named(n) => {
                    let k = self.konst(Const::Str(n.clone()));
                    self.emit(Op::PushConst(k));
                }
                ClassRef::Dynamic(cexpr) => {
                    self.expr(cexpr)?;
                    self.emit(Op::ClassConstFromValue { name: name.into() });
                }
                _ => match self.resolve_target(class)?.0 {
                    ClassTarget::Class(cid) => {
                        let k = self.konst(Const::Str(self.ctx.classes[cid].name.clone()));
                        self.emit(Op::PushConst(k));
                    }
                    ClassTarget::Static => {
                        self.emit(Op::ClassNameStatic);
                    }
                },
            }
            return Ok(());
        }
        // A class constant `C::NAME`: `$cls::NAME` or an unknown named class resolve
        // the class from a run-time value (PHP: `Class "X" not found` if missing).
        if self.is_runtime_class(class) {
            self.push_class_value(class)?;
            self.emit(Op::ClassConstFromValue { name: name.into() });
            return Ok(());
        }
        let (target, _forwarding) = self.resolve_target(class)?;
        match target {
            ClassTarget::Class(cid) => match self.find_class_const(cid, name) {
                Some((decl, idx)) => {
                    self.emit(Op::ClassConst { class: decl, idx: idx as u32 });
                }
                // An enum case `E::Case` (Session A): materialise its singleton at
                // run time. Cases are matched case-sensitively (like PHP); a backed
                // case whose value did not const-fold is not materialisable and
                // falls through to the evaluator.
                None => match self.enum_case_index(cid, name) {
                    Some(case) => {
                        self.emit(Op::EnumCase { class: cid, case });
                    }
                    None => {
                        return Err(CompileError::Unsupported(format!(
                            "class constant `{}` (undefined here, or an enum case)",
                            String::from_utf8_lossy(name)
                        )))
                    }
                },
            },
            ClassTarget::Static => {
                self.emit(Op::ClassConstDyn { name: name.into() });
            }
        }
        Ok(())
    }

    /// Whether `class` is resolved at run time rather than compile time: a
    /// `$expr::` dynamic reference, or a named class unknown at compile time. PHP
    /// resolves an unknown named class at the point of use (throwing `Class "X"
    /// not found` if still missing), so we defer it to the same dynamic ops as
    /// `$cls::…` instead of failing to compile (step 50 follow-up).
    fn is_runtime_class(&self, class: &ClassRef) -> bool {
        match class {
            ClassRef::Dynamic(_) => true,
            ClassRef::Named(n) => self.resolve_class(n).is_none(),
            _ => false,
        }
    }

    /// Push the run-time class value for a reference where [`Self::is_runtime_class`]
    /// holds: the evaluated `$expr`, or the (already fully-qualified) class name as
    /// a string the VM resolves via the class table.
    fn push_class_value(&mut self, class: &ClassRef) -> R<()> {
        match class {
            ClassRef::Dynamic(e) => self.expr(e),
            ClassRef::Named(n) => {
                let k = self.konst(Const::Str(n.clone()));
                self.emit(Op::PushConst(k));
                Ok(())
            }
            _ => unreachable!("push_class_value on a compile-time class ref"),
        }
    }

    /// Resolve a `ClassRef` to a [`ClassTarget`] plus whether the call is
    /// *forwarding* (`self`/`parent`/`static` keep the caller's LSB class and
    /// `$this`; a named class rebinds them). `self`/`parent` collapse to a
    /// compile-time class id; `static` stays run-time.
    fn resolve_target(&self, class: &ClassRef) -> R<(ClassTarget, bool)> {
        match class {
            ClassRef::Named(name) => {
                let cid = self.resolve_class(name).ok_or_else(|| {
                    CompileError::Unsupported(format!(
                        "reference to unknown class `{}`",
                        String::from_utf8_lossy(name)
                    ))
                })?;
                Ok((ClassTarget::Class(cid), false))
            }
            ClassRef::SelfClass => {
                let cid = self
                    .cur_class
                    .ok_or_else(|| CompileError::Unsupported("`self` outside class context".into()))?;
                Ok((ClassTarget::Class(cid), true))
            }
            ClassRef::Parent => {
                let cid = self
                    .cur_class
                    .and_then(|c| self.ctx.classes[c].parent)
                    .ok_or_else(|| CompileError::Unsupported("`parent` without a parent class".into()))?;
                Ok((ClassTarget::Class(cid), true))
            }
            ClassRef::Static => Ok((ClassTarget::Static, true)),
            ClassRef::Dynamic(_) => Err(CompileError::Unsupported("dynamic class reference".into())),
        }
    }

    /// Find a class constant by name at compile time, searching the class's own
    /// constants and parent chain, then (transitively) its interfaces. Returns the
    /// declaring class id and the constant's index in that class's `consts`
    /// (matching [`CompiledClass::consts`]). Case-sensitive, like PHP.
    fn find_class_const(&self, cid: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
        let classes = self.ctx.classes;
        let mut c = Some(cid);
        while let Some(x) = c {
            if let Some(i) = classes[x].consts.iter().position(|k| k.name.as_ref() == name) {
                return Some((x, i));
            }
            c = classes[x].parent;
        }
        let mut c = Some(cid);
        while let Some(x) = c {
            for &i in &classes[x].interfaces {
                if let Some(r) = self.find_class_const(i, name) {
                    return Some(r);
                }
            }
            c = classes[x].parent;
        }
        None
    }

    /// The index of enum `cid`'s case `name` (case-sensitive, like PHP), if `cid`
    /// is an enum, the case exists, and it is *materialisable* by the VM — a pure
    /// case, or a backed case whose value const-folds (Session A). A backed case
    /// with a non-folding value returns `None` so `E::Case` falls back to the
    /// evaluator. The index matches [`CompiledClass::enum_cases`] (1:1 with source).
    fn enum_case_index(&self, cid: ClassId, name: &[u8]) -> Option<u32> {
        let cd = &self.ctx.classes[cid];
        if !cd.is_enum {
            return None;
        }
        let i = cd.enum_cases.iter().position(|c| c.name.as_ref() == name)?;
        let case = &cd.enum_cases[i];
        let materialisable = match &case.value {
            None => true,
            Some(e) => const_eval_in_class(e, cid, self.ctx, 0).is_some(),
        };
        materialisable.then_some(i as u32)
    }

    /// Resolve a class name (case-insensitive) to its [`ClassId`].
    fn resolve_class(&self, name: &[u8]) -> Option<ClassId> {
        self.ctx.class_index.get(&name.to_ascii_lowercase()).copied()
    }

    /// Resolve a method by name at compile time, walking the parent chain
    /// child→ancestor; returns the *defining* class id and the method's index in
    /// that class's `methods` (matching [`CompiledClass::methods`]).
    fn resolve_method_compile(&self, start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
        let classes = self.ctx.classes;
        let mut cid = Some(start);
        while let Some(c) = cid {
            if let Some(i) = classes[c]
                .methods
                .iter()
                .position(|m| m.decl.name.eq_ignore_ascii_case(name))
            {
                return Some((c, i));
            }
            cid = classes[c].parent;
        }
        None
    }

    /// If `place` is a single-step property access on `$this` or a local
    /// (`$this->p` / `$o->p`), push the object onto the stack and return the
    /// property name; otherwise return `None` so the caller falls through to the
    /// mixed field path (`field_path` / `FieldAssign`) or the array path. A
    /// `$GLOBALS`-rooted property (`$GLOBALS['x']->p`) returns `None` too: the
    /// `FieldBase::Global` field path handles it (the [`Op::PropSet`] fast path
    /// only roots at `$this` / a local slot).
    fn prop_place(&mut self, place: &Place) -> R<Option<Box<[u8]>>> {
        if place.steps.len() != 1 {
            return Ok(None);
        }
        let PlaceStep::Prop(name) = &place.steps[0] else {
            return Ok(None);
        };
        match place.base {
            PlaceBase::This => {
                self.emit(Op::This);
            }
            PlaceBase::Local(s) => {
                self.emit(Op::LoadSlot(s));
            }
            // A `$GLOBALS`-rooted property write goes through the field path; an
            // indexed static-property target is rewritten before reaching here. A
            // class-constant test base is materialised into a temp before any
            // property step, so it never reaches here either.
            PlaceBase::Global(_)
            | PlaceBase::StaticProp { .. }
            | PlaceBase::ClassConst { .. } => return Ok(None),
        }
        Ok(Some(name.clone()))
    }

    /// Lower a mixed property/index place (`$o->a[$k]`, `$this->x->y`, …) into a
    /// [`FieldBase`] plus a [`FieldStep`] list, emitting each `Index` step's key
    /// expression in source order (consumed at run time beneath the value). The
    /// in-place vs copy-on-write distinction between object and array steps is the
    /// VM's job; the compiler only records the shape.
    /// Compile `try { body } catch (...) { } [finally { }]` (EXC). The body's op
    /// range becomes a *catch* region (→ a `CatchMatch`-per-clause / `Rethrow`
    /// dispatch) and, when a `finally` is present, the body+catches range also
    /// becomes a *finally* region (→ the finally body, re-raising at `EndFinally`)
    /// — so normal, caught, and propagating exits all run `finally`, and nesting
    /// works via re-raise. EXC-2 scope: a `return`/`break`/`continue`/`goto`
    /// crossing a `finally` is out of slice (falls back to the evaluator).
    fn try_stmt(&mut self, body: &[Stmt], catches: &[CatchClause], finally: &[Stmt]) -> R<()> {
        let has_finally = !finally.is_empty();
        // `return`/`break`/`continue` and now `goto` crossing a finally are handled
        // (EXC-2b): `resolve_gotos` routes a goto that leaves a finally's protected
        // region through it. A goto confined to the finally body does not cross it.
        // Snapshot the scope depth outside the try (its own level) so the goto router
        // can tell an outside-the-body target from an inside-the-body one.
        let outer_scope_len = self.scope_path.len();
        let start = self.here();
        if has_finally {
            // Route control transfers in the body/catches through this finally.
            self.finally_scopes.push(FinallyScope { sites: Vec::new(), loop_depth: self.loops.len() });
        }
        self.block(body)?;
        let body_end = self.here();
        let after_body = self.emit(Op::Jump(Addr::MAX)); // normal completion → finally / after
        let catch_addr = self.here();
        if !catches.is_empty() {
            self.exc_regions.push(ExcRegion { start, end: body_end, target: catch_addr, is_finally: false });
        }
        // Catch dispatch: one `CatchMatch` per clause (body forward-referenced),
        // then `Rethrow` if none matched.
        let mut sites: Vec<(Addr, Vec<ClassId>, Vec<Box<[u8]>>, Option<crate::hir::Slot>)> = Vec::new();
        for c in catches {
            let (cids, cnames) = self.resolve_catch_types(&c.types);
            let at = self.emit(Op::CatchMatch {
                types: cids.clone().into(),
                names: cnames.clone().into(),
                var: c.var,
                body: Addr::MAX,
            });
            sites.push((at, cids, cnames, c.var));
        }
        if !catches.is_empty() {
            self.emit(Op::Rethrow);
        }
        let mut catch_end_jumps = Vec::new();
        for (i, c) in catches.iter().enumerate() {
            let body_at = self.here();
            let (at, cids, cnames, var) = &sites[i];
            self.patch(*at, Op::CatchMatch {
                types: cids.clone().into(),
                names: cnames.clone().into(),
                var: *var,
                body: body_at,
            });
            self.block(&c.body)?;
            catch_end_jumps.push(self.emit(Op::Jump(Addr::MAX)));
        }
        let finally_entry = self.here();
        let after;
        if has_finally {
            // Every parked control transfer (return/break/continue in the body or
            // catches) lands at the finally entry now that it is known (EXC-2b).
            let scope = self.finally_scopes.pop().expect("finally scope");
            for site in scope.sites {
                self.patch(site, Op::Jump(finally_entry));
            }
            // Covers the body, the catch dispatch, and the catch bodies — an
            // exception anywhere before `finally_entry` runs `finally` then
            // re-propagates. Pushed after the catch region so catches win first.
            self.exc_regions.push(ExcRegion {
                start,
                end: finally_entry,
                target: finally_entry,
                is_finally: true,
            });
            // Record the protected range for `goto`-through-finally routing (EXC-2b).
            self.goto_finally_meta.push((start..finally_entry, finally_entry, outer_scope_len));
            self.block(finally)?;
            // On normal completion `EndFinally` jumps to `after` (skipping the
            // trailing `Ret`); for a parked return it pushes the value and falls
            // through to that `Ret`; for a parked break/continue it jumps to the
            // loop target.
            let endf = self.emit(Op::EndFinally { after: Addr::MAX });
            self.emit(Op::Ret);
            after = self.here();
            self.patch(endf, Op::EndFinally { after });
        } else {
            after = self.here();
        }
        let normal_target = if has_finally { finally_entry } else { after };
        self.patch(after_body, Op::Jump(normal_target));
        for j in catch_end_jumps {
            self.patch(j, Op::Jump(normal_target));
        }
        Ok(())
    }

    /// Split a catch clause's class names into those resolvable at compile time
    /// (returned as [`ClassId`]s) and those not yet declared — left as names for
    /// the VM to resolve against the live class table at run time (step 57, Phase
    /// 2), so a `catch (E)` where `E` is provided by an `eval`/`include` still
    /// matches.
    fn resolve_catch_types(&self, names: &[Box<[u8]>]) -> (Vec<ClassId>, Vec<Box<[u8]>>) {
        let mut ids = Vec::new();
        let mut unresolved = Vec::new();
        for n in names {
            match self.resolve_class(n) {
                Some(id) => ids.push(id),
                None => unresolved.push(n.clone()),
            }
        }
        (ids, unresolved)
    }

    fn field_path(&mut self, place: &Place) -> R<(FieldBase, Vec<FieldStep>)> {
        let base = match place.base {
            PlaceBase::Local(s) => FieldBase::Local(s),
            PlaceBase::Global(s) => FieldBase::Global(s),
            PlaceBase::This => FieldBase::This,
            // Indexed static-property targets are rewritten into a temp before
            // reaching the field-path walker (see `static_prop_rmw`).
            PlaceBase::StaticProp { .. } => {
                return Err(CompileError::Unsupported("static property field path".into()))
            }
            // A class-constant base is read-only and materialised into a temp for
            // isset/empty before any field walk, so it never reaches here.
            PlaceBase::ClassConst { .. } => {
                return Err(CompileError::Unsupported("class constant field path".into()))
            }
        };
        let mut steps = Vec::with_capacity(place.steps.len());
        for step in &place.steps {
            match step {
                PlaceStep::Index(k) => {
                    self.expr(k)?;
                    steps.push(FieldStep::Index);
                }
                PlaceStep::Prop(name) => steps.push(FieldStep::Prop(name.clone())),
                // `->$n` / `->{expr}`: the name expression is emitted here (in source
                // order, consumed at run time beneath the value, like an index key)
                // and the VM resolves it to a property name (step 51).
                PlaceStep::PropDyn(name) => {
                    self.expr(name)?;
                    steps.push(FieldStep::PropDyn);
                }
                // `[]` autovivifies a fresh array and descends into it (the VM's
                // `field_write` recurses through an appended child), so an
                // intermediate `$a[][] = …` is valid here, not just as the last step.
                PlaceStep::Append => steps.push(FieldStep::Append),
            }
        }
        Ok((base, steps))
    }

    /// Compile a `switch`: the subject is evaluated once into a temp, each `case`
    /// is compared with loose `==`, and on a match control jumps to that case's
    /// body. Bodies are laid out in source order so execution falls through to the
    /// next case until a `break` (the switch is one `break`/`continue` level, both
    /// landing past its end). `default` runs when no case matches, at its source
    /// position in the fall-through chain.
    fn switch(&mut self, subject: &Expr, cases: &[Case]) -> R<()> {
        let t = self.alloc_temp();
        self.expr(subject)?;
        self.emit(Op::StoreSlot(t));
        // Dispatch: compare against each non-default case, jump to its body.
        let mut test_jumps: Vec<(usize, Addr)> = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            if let Some(test) = &case.test {
                self.emit(Op::LoadSlot(t));
                self.expr(test)?;
                self.emit(Op::Binary(BinOp::Eq));
                test_jumps.push((i, self.emit(Op::JumpIfTrue(Addr::MAX))));
            }
        }
        // No case matched -> default (if any) or past the end.
        let no_match = self.emit(Op::Jump(Addr::MAX));
        // Bodies in source order (fall-through between consecutive cases).
        self.loops.push(LoopCtx::default());
        let mut body_addrs: Vec<Addr> = Vec::with_capacity(cases.len());
        let mut default_addr: Option<Addr> = None;
        for case in cases {
            let at = self.here();
            body_addrs.push(at);
            if case.test.is_none() {
                default_addr = Some(at);
            }
            self.block(&case.body)?;
        }
        let end = self.here();
        for (i, j) in test_jumps {
            self.patch(j, Op::JumpIfTrue(body_addrs[i]));
        }
        self.patch(no_match, Op::Jump(default_addr.unwrap_or(end)));
        self.free_temp();
        // `break` and (PHP) `continue` both leave the switch.
        self.close_loop(end, end);
        Ok(())
    }

    /// Compile a `match` expression: the subject is evaluated once into a temp,
    /// each arm condition compared with strict `===`; the first match's body is
    /// evaluated as the result (no fall-through). With no matching arm and no
    /// `default`, PHP throws `UnhandledMatchError`; lacking VM exceptions, this
    /// raises a fatal (catchable-match handling is deferred). Leaves the result.
    fn match_expr(&mut self, subject: &Expr, arms: &[MatchArm]) -> R<()> {
        let t = self.alloc_temp();
        self.expr(subject)?;
        self.emit(Op::StoreSlot(t));
        let mut to_body: Vec<(usize, Addr)> = Vec::new();
        let mut default_arm: Option<usize> = None;
        for (i, arm) in arms.iter().enumerate() {
            if arm.conditions.is_empty() {
                default_arm = Some(i);
                continue;
            }
            for cond in &arm.conditions {
                self.emit(Op::LoadSlot(t));
                self.expr(cond)?;
                self.emit(Op::Binary(BinOp::Identical));
                to_body.push((i, self.emit(Op::JumpIfTrue(Addr::MAX))));
            }
        }
        let no_match = self.emit(Op::Jump(Addr::MAX));
        // Each arm body is an expression leaving one value, then jumps to the end.
        let mut body_addrs: Vec<Addr> = vec![0; arms.len()];
        let mut to_end: Vec<Addr> = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            body_addrs[i] = self.here();
            self.expr(&arm.body)?;
            to_end.push(self.emit(Op::Jump(Addr::MAX)));
        }
        let unhandled = self.here();
        self.emit(Op::MatchError(t));
        let end = self.here();
        for (i, j) in to_body {
            self.patch(j, Op::JumpIfTrue(body_addrs[i]));
        }
        let nm_target = default_arm.map(|i| body_addrs[i]).unwrap_or(unhandled);
        self.patch(no_match, Op::Jump(nm_target));
        for j in to_end {
            self.patch(j, Op::Jump(end));
        }
        self.free_temp();
        Ok(())
    }

    /// `$target = &$source`. A step-less pair (REF-1: bare variables /
    /// `$GLOBALS['x']`) binds via a single [`Op::BindRef`]. Otherwise (REF-4:
    /// array elements) the source cell is produced with [`Op::MakeRef`] and the
    /// target bound with [`Op::BindRefTo`], evaluating the target's index
    /// expressions before the source's — the tree-walker's order. References into
    /// object properties or an appended slot fall back to the evaluator.
    fn assign_ref(&mut self, target: &Place, source: &Place) -> R<()> {
        if target.steps.is_empty() && source.steps.is_empty() {
            let t = dim_base(target)?;
            let s = dim_base(source)?;
            self.emit(Op::BindRef { target: t, source: s });
            return Ok(());
        }
        let (tbase, tsteps) = self.field_path(target)?; // pushes target keys…
        let (sbase, ssteps) = self.field_path(source)?; // …then source keys
        self.emit(Op::MakeRef { base: sbase, steps: ssteps.into() });
        self.emit(Op::BindRefTo { base: tbase, steps: tsteps.into() });
        Ok(())
    }

    /// `$target = &f(...)` (REF-4b): invoke the call *raw* (no `DerefTop`) so a
    /// by-reference return's cell can be aliased, then bind the target to it. The
    /// target's index expressions are emitted before the call (the tree-walker's
    /// order) so the returned reference lands on top of them for `BindRefTo`. Only
    /// user-function calls are in slice; anything else falls back to the evaluator.
    fn assign_ref_call(&mut self, target: &Place, call: &Expr) -> R<()> {
        let ExprKind::Call { name, args, named } = &call.kind else {
            return Err(CompileError::Unsupported("reference assignment from a non-call".into()));
        };
        if !named.is_empty() {
            return Err(CompileError::Unsupported("reference call with named arguments".into()));
        }
        let Some(idx) = self.ctx.funcs.iter().position(|f| ascii_eq_ignore_case(&f.name, name)) else {
            return Err(CompileError::Unsupported(
                "reference assignment from a builtin / undefined call".into(),
            ));
        };
        let callee = &self.ctx.funcs[idx];
        if callee.params.iter().any(|p| p.variadic) {
            return Err(CompileError::Unsupported("reference call to a variadic function".into()));
        }
        if args.len() != callee.params.len() {
            return Err(CompileError::Unsupported("reference call arity mismatch".into()));
        }
        let by_ref: Vec<bool> = callee.params.iter().map(|p| p.by_ref).collect();
        let callee_by_ref = callee.by_ref;
        let pnames: Vec<Box<[u8]>> =
            callee.params.iter().map(|p| callee.slots[p.slot as usize].clone()).collect();
        let (base, steps) = self.field_path(target)?; // target index keys first…
        self.push_call_args(args, &by_ref, name, &pnames)?; // …then the call args…
        self.emit(Op::Call { func: idx as u32, argc: args.len() as u32 }); // …leaving the raw ref on top
        // Aliasing a non-reference-returning function copies the value and raises a
        // notice (D-13.5). A by-ref callee that returned a non-place already raised
        // its own "returned by reference" notice from inside `f`, so suppress here.
        if !callee_by_ref {
            let k = self.konst(Const::Str(
                b"Only variables should be assigned by reference"[..].into(),
            ));
            self.emit(Op::EmitNotice(k));
        }
        self.emit(Op::BindRefTo { base, steps: steps.into() });
        Ok(())
    }

    /// Compile an array-element write `$a[…][k] = rhs` / `$a[…][] = rhs`, rooted
    /// at a local (or `$GLOBALS`) slot, at any nesting depth — or a single-step
    /// object-property write `$o->p = rhs` / `$this->p = rhs` (OOP-1). Mixed
    /// property+index chains (`$o->a[$k] = …`) remain out of slice.
    fn assign_place(&mut self, place: &Place, rhs: &Expr) -> R<()> {
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.assign_place(p, rhs)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            self.expr(rhs)?;
            self.emit(Op::PropSet { name });
            return Ok(());
        }
        // An object-property step, or an *intermediate* `[]` append (`$a[][] = …`),
        // routes through the general field walker — `Op::AssignPath` only models a
        // run of index keys plus an optional trailing append.
        if place_has_prop(place) || place_has_intermediate_append(place) {
            let (base, steps) = self.field_path(place)?;
            self.expr(rhs)?;
            self.emit(Op::FieldAssign { base, steps: steps.into() });
            return Ok(());
        }
        if let PlaceBase::Global(s) = place.base {
            if place.steps.is_empty() {
                // `$GLOBALS['x'] = rhs`: a direct global write (no array steps).
                self.expr(rhs)?;
                self.emit(Op::Dup);
                self.emit(Op::StoreGlobal(s));
                return Ok(());
            }
        }
        let base = dim_base(place)?;
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if nkeys == 0 && !append {
            return Err(CompileError::Unsupported("array write with no steps".into()));
        }
        self.expr(rhs)?;
        self.emit(Op::AssignPath { base, nkeys, append });
        Ok(())
    }

    /// Compile a single `unset(place)`: a `->prop` routes to `PropUnset`, a mixed
    /// object/array path to `FieldUnset`, a plain array element to `UnsetPath`, and
    /// an indexed static property is read-modify-written through a temp.
    fn unset_place(&mut self, place: &Place) -> R<()> {
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_rmw(&class, &name, &place.steps, false, |s, p| {
                s.unset_place(p)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            self.emit(Op::PropUnset { name });
        } else if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldUnset { base, steps: steps.into() });
        } else {
            let base = dim_base(place)?;
            let nkeys = self.test_path_steps(place)?;
            self.emit(Op::UnsetPath { base, nkeys });
        }
        Ok(())
    }

    /// Read-modify-write wrapper for an indexed static-property target
    /// (`self::$arr[k] = v`, `unset(self::$arr[k])`). The property value is loaded
    /// into a temp, `core` runs over a `Local`-rooted place on that temp, then the
    /// temp is written back into the static property — value-correct for PHP
    /// arrays (copy-on-write). When `leaves_value` is set, `core` leaves the
    /// expression's result on the stack and it is preserved across the write-back.
    /// Dynamic class references (`$cls::$arr[k]`) are out of scope.
    fn static_prop_rmw(
        &mut self,
        class: &ClassRef,
        name: &[u8],
        steps: &[PlaceStep],
        leaves_value: bool,
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        if self.is_runtime_class(class) {
            return Err(CompileError::Unsupported(
                "indexed write to dynamic static property".into(),
            ));
        }
        let (target, _) = self.resolve_target(class)?;
        let nm: Box<[u8]> = name.into();
        let t = self.alloc_temp();
        self.emit(Op::StaticPropGet { target, name: nm.clone() });
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        if leaves_value {
            core(self, &local)?; // [result]
            let t2 = self.alloc_temp();
            self.emit(Op::StoreSlot(t2)); // []
            self.emit(Op::LoadSlot(t));
            self.emit(Op::StaticPropSet { target, name: nm }); // [arr]
            self.emit(Op::Pop);
            self.emit(Op::LoadSlot(t2)); // [result]
            self.free_temp(); // t2
        } else {
            core(self, &local)?; // []
            self.emit(Op::LoadSlot(t));
            self.emit(Op::StaticPropSet { target, name: nm }); // [arr]
            self.emit(Op::Pop);
        }
        self.free_temp(); // t
        Ok(())
    }

    /// Materialise an indexed static property into a temp for a *read-only*
    /// operation (`isset`/`empty`): load the value, run `core` over a
    /// `Local`-rooted place on the temp, then free the temp. No write-back — the
    /// property is not modified, so this also avoids a visibility-checked
    /// `StaticPropSet` on an out-of-scope read.
    fn static_prop_read(
        &mut self,
        class: &ClassRef,
        name: &[u8],
        steps: &[PlaceStep],
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        if self.is_runtime_class(class) {
            return Err(CompileError::Unsupported(
                "indexed read of dynamic static property".into(),
            ));
        }
        let (target, _) = self.resolve_target(class)?;
        let t = self.alloc_temp();
        self.emit(Op::StaticPropGet { target, name: name.into() });
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        core(self, &local)?;
        self.free_temp();
        Ok(())
    }

    /// Materialise a class constant into a temp for a read-only `isset`/`empty`
    /// test on an index into it (`isset(self::TABLE[$k])`): evaluate the constant,
    /// run `core` over a `Local`-rooted place carrying the index steps, then free
    /// the temp. The constant is a value (never written), mirroring
    /// [`Self::static_prop_read`] but reading via [`Self::class_const`].
    fn class_const_read(
        &mut self,
        class: &ClassRef,
        name: &[u8],
        steps: &[PlaceStep],
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        let t = self.alloc_temp();
        self.class_const(class, name)?;
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        core(self, &local)?;
        self.free_temp();
        Ok(())
    }

    /// Compile `$o->p ??= rhs` on a single property (magic-aware). Read via
    /// `__isset`; if already set, the existing value (`__get`) is the result and
    /// no write happens; if unset, assign `rhs` (`__set`) and yield it. Each magic
    /// op leaves its result for the next, so the composition works for both magic
    /// and declared properties.
    fn assign_coalesce_place(&mut self, place: &Place, rhs: &Expr) -> R<()> {
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.assign_coalesce_place(p, rhs)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            // stack: [obj]
            self.emit(Op::Dup); // [obj, obj]
            self.emit(Op::PropIsset { name: name.clone() }); // [obj, isset]
            let to_set = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → set; [obj]
            self.emit(Op::PropGet { name: name.clone() }); // set: existing value → [value]
            let to_end = self.emit(Op::Jump(Addr::MAX));
            let set_at = self.here();
            self.patch(to_set, Op::JumpIfFalse(set_at));
            self.expr(rhs)?; // [obj, rhs]
            self.emit(Op::PropSet { name }); // [value]
            let end = self.here();
            self.patch(to_end, Op::Jump(end));
            return Ok(());
        }
        // `$a[k] ??= rhs` on a single-step array element rooted at a local /
        // `$GLOBALS` slot: evaluate the key once (into a temp), then assign only if
        // the element is unset, yielding the existing or newly-stored value.
        if place.steps.len() == 1 {
            if let PlaceStep::Index(k) = &place.steps[0] {
                let base = dim_base(place)?;
                let t_key = self.alloc_temp();
                self.expr(k)?; // [key]
                self.emit(Op::Dup); // [key, key]
                self.emit(Op::StoreSlot(t_key)); // [key]
                self.emit(Op::IssetPath { base, nkeys: 1 }); // [bool]
                let to_assign = self.emit(Op::JumpIfFalse(Addr::MAX));
                // Set: yield the existing element value.
                match base {
                    DimBase::Local(s) => self.emit(Op::LoadSlot(s)),
                    DimBase::Global(s) => self.emit(Op::LoadGlobal(s)),
                };
                self.emit(Op::LoadSlot(t_key)); // [baseval, key]
                self.emit(Op::FetchDim); // [value]
                let to_end = self.emit(Op::Jump(Addr::MAX));
                let assign_at = self.here();
                self.patch(to_assign, Op::JumpIfFalse(assign_at));
                self.emit(Op::LoadSlot(t_key)); // [key]
                self.expr(rhs)?; // [key, rhs]
                self.emit(Op::AssignPath { base, nkeys: 1, append: false }); // [value]
                let end = self.here();
                self.patch(to_end, Op::Jump(end));
                self.free_temp();
                return Ok(());
            }
        }
        Err(CompileError::Unsupported("`??=` on this place".into()))
    }

    /// Compile a compound element write `$a[…][k] op= rhs`.
    fn assign_op_place(&mut self, op: crate::hir::BinOp, place: &Place, rhs: &Expr) -> R<()> {
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.assign_op_place(op, p, rhs)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            // `$o->p op= rhs` as read-modify-write so a magic property routes
            // through `__get` then `__set` (each op leaves its result for the
            // next): [obj] → Dup → PropGet → rhs → Binary → PropSet → [result].
            self.emit(Op::Dup);
            self.emit(Op::PropGet { name: name.clone() });
            self.expr(rhs)?;
            self.emit(Op::Binary(op));
            self.emit(Op::PropSet { name });
            return Ok(());
        }
        if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.expr(rhs)?;
            self.emit(Op::FieldAssignOp { base, steps: steps.into(), op });
            return Ok(());
        }
        if let PlaceBase::Global(s) = place.base {
            if place.steps.is_empty() {
                // `$GLOBALS['x'] op= rhs`.
                self.emit(Op::LoadGlobal(s));
                self.expr(rhs)?;
                self.emit(Op::Binary(op));
                self.emit(Op::Dup);
                self.emit(Op::StoreGlobal(s));
                return Ok(());
            }
        }
        let base = dim_base(place)?;
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if append || nkeys == 0 {
            return Err(CompileError::Unsupported("`[]` has no value for reading".into()));
        }
        self.expr(rhs)?;
        self.emit(Op::AssignOpPath { base, nkeys, op });
        Ok(())
    }

    /// Compile `++`/`--` on an array element `$a[…][k]`.
    fn incdec_place(&mut self, place: &Place, inc: bool, pre: bool) -> R<()> {
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.incdec_place(p, inc, pre)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            self.emit(Op::PropIncDec { name, inc, pre });
            return Ok(());
        }
        if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldIncDec { base, steps: steps.into(), inc, pre });
            return Ok(());
        }
        if let PlaceBase::Global(s) = place.base {
            if place.steps.is_empty() {
                // `$GLOBALS['x']++` / `--$GLOBALS['x']`.
                self.emit(Op::IncDecGlobal { slot: s, inc, pre });
                return Ok(());
            }
        }
        let base = dim_base(place)?;
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if append || nkeys == 0 {
            return Err(CompileError::Unsupported("`[]` has no value for reading".into()));
        }
        self.emit(Op::IncDecPath { base, nkeys, inc, pre });
        Ok(())
    }

    /// Compile `isset($p0, $p1, …)` to a boolean: each place is tested in turn
    /// and the result short-circuits to `false` on the first absent one (so a
    /// later place's index expressions aren't evaluated), mirroring PHP.
    fn isset(&mut self, places: &[Place]) -> R<()> {
        let last = places.len() - 1;
        let mut to_false = Vec::new();
        for (i, place) in places.iter().enumerate() {
            self.isset_one(place)?;
            if i != last {
                // [bi]: if false, jump to the shared false-result; else discard.
                to_false.push(self.emit(Op::JumpIfFalse(Addr::MAX)));
            }
        }
        if to_false.is_empty() {
            return Ok(()); // single place: its IssetPath bool is the result
        }
        let to_end = self.emit(Op::Jump(Addr::MAX));
        let false_at = self.here();
        let f = self.konst(Const::Bool(false));
        self.emit(Op::PushConst(f));
        let end = self.here();
        self.patch(to_end, Op::Jump(end));
        for j in to_false {
            self.patch(j, Op::JumpIfFalse(false_at));
        }
        Ok(())
    }

    /// Push a single place's `isset` boolean: a `->prop` via `PropIsset`, a mixed
    /// object/array path via `FieldIsset`, a plain array element via `IssetPath`,
    /// and an indexed static property via a read-only temp.
    fn isset_one(&mut self, place: &Place) -> R<()> {
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_read(&class, &name, &place.steps, |s, p| s.isset_one(p));
        }
        if let PlaceBase::ClassConst { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.class_const_read(&class, &name, &place.steps, |s, p| s.isset_one(p));
        }
        if let Some(name) = self.prop_place(place)? {
            self.emit(Op::PropIsset { name });
        } else if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldIsset { base, steps: steps.into() });
        } else {
            let base = dim_base(place)?;
            let nkeys = self.test_path_steps(place)?;
            self.emit(Op::IssetPath { base, nkeys });
        }
        Ok(())
    }

    /// Compile `empty($place)`. A single property is `__isset`-then-silent-`__get`
    /// (`empty` is `!isset || !truthy(value)`), so an unset magic property never
    /// warns or calls `__get`; other places use the array `EmptyPath`.
    fn empty(&mut self, place: &Place) -> R<()> {
        if let PlaceBase::ClassConst { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.class_const_read(&class, &name, &place.steps, |s, p| s.empty(p));
        }
        if let PlaceBase::StaticProp { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.static_prop_read(&class, &name, &place.steps, |s, p| s.empty(p));
        }
        if let Some(name) = self.prop_place(place)? {
            // stack: [obj]
            self.emit(Op::Dup); // [obj, obj]
            self.emit(Op::PropIsset { name: name.clone() }); // [obj, isset]
            let to_true = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → empty=true; [obj]
            self.emit(Op::PropGetSilent { name }); // [value]
            self.emit(Op::Unary(crate::hir::UnOp::Not)); // [empty = !truthy(value)]
            let to_end = self.emit(Op::Jump(Addr::MAX));
            let true_at = self.here();
            self.patch(to_true, Op::JumpIfFalse(true_at));
            self.emit(Op::Pop); // drop the kept object
            let t = self.konst(Const::Bool(true));
            self.emit(Op::PushConst(t)); // [true]
            let end = self.here();
            self.patch(to_end, Op::Jump(end));
            return Ok(());
        }
        if place_has_prop(place) {
            return Err(CompileError::Unsupported("empty() on a nested property path".into()));
        }
        let base = dim_base(place)?;
        let nkeys = self.test_path_steps(place)?;
        self.emit(Op::EmptyPath { base, nkeys });
        Ok(())
    }

    /// Like [`Self::push_index_steps`] but for a read-only test target
    /// (`isset` / `empty` / `unset`): pushes the index values and returns the
    /// key count. `[]` and `->prop` steps are not valid here.
    fn test_path_steps(&mut self, place: &Place) -> R<u32> {
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if append {
            return Err(CompileError::Unsupported("`[]` is not a readable place".into()));
        }
        Ok(nkeys)
    }

    /// Push each `Index` step's value (source order) and report `(nkeys, append)`:
    /// how many index values were pushed, and whether the final step is `[]`.
    /// A `Prop` step or a non-final `Append` is out of slice.
    fn push_index_steps(&mut self, steps: &[PlaceStep]) -> R<(u32, bool)> {
        let mut nkeys = 0u32;
        let mut append = false;
        let last = steps.len().saturating_sub(1);
        for (i, step) in steps.iter().enumerate() {
            match step {
                PlaceStep::Index(k) => {
                    self.expr(k)?;
                    nkeys += 1;
                }
                PlaceStep::Append if i == last => append = true,
                PlaceStep::Append => {
                    return Err(CompileError::Unsupported("`[]` is only valid as the last step".into()))
                }
                PlaceStep::Prop(_) | PlaceStep::PropDyn(_) => {
                    return Err(CompileError::Unsupported("object property step".into()))
                }
            }
        }
        Ok((nkeys, append))
    }
}

/// Whether a place contains an object-property step — routing it to the mixed
/// field-path opcodes (OOP-2c) rather than the array-only path opcodes.
fn place_has_prop(place: &Place) -> bool {
    place.steps.iter().any(|s| matches!(s, PlaceStep::Prop(_) | PlaceStep::PropDyn(_)))
}

/// Whether a place has an `[]` append anywhere but the last step (`$a[][] = …`):
/// the array-only `Op::AssignPath` can't express it, so it routes through the
/// general field walker (which autovivifies through the appended child).
fn place_has_intermediate_append(place: &Place) -> bool {
    let last = place.steps.len().saturating_sub(1);
    place
        .steps
        .iter()
        .enumerate()
        .any(|(i, s)| i != last && matches!(s, PlaceStep::Append))
}

/// Map a [`Place`]'s base to the VM's write-cell selector. Only a single-step
/// array write on a local / `$GLOBALS` slot is in slice; `$this` and deeper
/// chains are rejected so the VM never sees an opcode it can't honour.
/// Extract the local slot from a `Local`-rooted, step-less place. Used by the
/// static-property by-reference-builtin RMW path, which materialises the
/// property into a temp local before running the in-place builtin on it.
fn local_slot(place: &Place) -> crate::hir::Slot {
    match place.base {
        PlaceBase::Local(s) if place.steps.is_empty() => s,
        _ => unreachable!("static_prop_rmw builds a step-less Local place for the by-ref builtin"),
    }
}

fn dim_base(place: &Place) -> R<DimBase> {
    match place.base {
        PlaceBase::Local(s) => Ok(DimBase::Local(s)),
        PlaceBase::Global(s) => Ok(DimBase::Global(s)),
        PlaceBase::This => Err(CompileError::Unsupported("$this property write".into())),
        PlaceBase::StaticProp { .. } => {
            Err(CompileError::Unsupported("static property dim base".into()))
        }
        PlaceBase::ClassConst { .. } => {
            Err(CompileError::Unsupported("class constant dim base".into()))
        }
    }
}

/// ASCII-case-insensitive byte-string equality — PHP resolves function names
/// case-insensitively in ASCII (`STRLEN` == `strlen`).
fn ascii_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.eq_ignore_ascii_case(y))
}



/// HIR expression-variant name, for [`CompileError::Unsupported`].
fn expr_name(k: &ExprKind) -> String {
    let n = match k {
        ExprKind::Null => "Null",
        ExprKind::Bool(_) => "Bool",
        ExprKind::Int(_) => "Int",
        ExprKind::Float(_) => "Float",
        ExprKind::Str(_) => "Str",
        ExprKind::Const { .. } => "Const",
        ExprKind::Var(_) => "Var",
        ExprKind::GlobalVar(_) => "GlobalVar",
        ExprKind::Binary(..) => "Binary",
        ExprKind::And(..) => "And",
        ExprKind::Or(..) => "Or",
        ExprKind::Xor(..) => "Xor",
        ExprKind::Coalesce(..) => "Coalesce",
        ExprKind::Unary(..) => "Unary",
        ExprKind::Cast(..) => "Cast",
        ExprKind::Assign(..) => "Assign",
        ExprKind::AssignRef { .. } => "AssignRef",
        ExprKind::AssignRefCall { .. } => "AssignRefCall",
        ExprKind::AssignOp(..) => "AssignOp",
        ExprKind::AssignCoalesce(..) => "AssignCoalesce",
        ExprKind::IncDec { .. } => "IncDec",
        ExprKind::IncDecPlace { .. } => "IncDecPlace",
        ExprKind::Ternary { .. } => "Ternary",
        ExprKind::Call { .. } => "Call",
        ExprKind::Closure { .. } => "Closure",
        ExprKind::FirstClassCallable(_) => "FirstClassCallable",
        ExprKind::CallDynamic { .. } => "CallDynamic",
        ExprKind::Pipe { .. } => "Pipe",
        ExprKind::Spread(_) => "Spread",
        ExprKind::Array(_) => "Array",
        ExprKind::Index { .. } => "Index",
        ExprKind::ListAssign { .. } => "ListAssign",
        ExprKind::AssignPlace(..) => "AssignPlace",
        ExprKind::AssignOpPlace(..) => "AssignOpPlace",
        ExprKind::AssignCoalescePlace(..) => "AssignCoalescePlace",
        ExprKind::Isset(_) => "Isset",
        ExprKind::Empty(_) => "Empty",
        ExprKind::Suppress(_) => "Suppress",
        ExprKind::Print(_) => "Print",
        ExprKind::Exit(_) => "Exit",
        ExprKind::Clone(_) => "Clone",
        ExprKind::Eval(_) => "Eval",
        ExprKind::Include { .. } => "Include",
        ExprKind::Match { .. } => "Match",
        ExprKind::New { .. } => "New",
        ExprKind::MethodCall { .. } => "MethodCall",
        ExprKind::MethodCallDyn { .. } => "MethodCallDyn",
        ExprKind::PropGet { .. } => "PropGet",
        ExprKind::PropGetDyn { .. } => "PropGetDyn",
        ExprKind::This => "This",
        ExprKind::StaticCall { .. } => "StaticCall",
        ExprKind::ClassConst { .. } => "ClassConst",
        ExprKind::StaticProp { .. } => "StaticProp",
        ExprKind::StaticPropAssign { .. } => "StaticPropAssign",
        ExprKind::StaticPropIncDec { .. } => "StaticPropIncDec",
        ExprKind::InstanceOf { .. } => "InstanceOf",
        ExprKind::Throw(_) => "Throw",
        ExprKind::Yield { .. } => "Yield",
        ExprKind::YieldFrom(_) => "YieldFrom",
    };
    format!("expression {n}")
}
