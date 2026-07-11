//! Function/method/thunk compilation. Split from compile/mod.rs.

use super::*;

/// Compile a user [`FnDecl`] into a [`Func`], resolving calls in its body against
/// the program context (for forward references and recursion). A free function
/// has no enclosing class (`cur_class = None`).
pub(super) fn compile_fndecl(fd: &FnDecl, ctx: &ProgramCtx, is_closure: bool) -> R<Func> {
    // A closure/arrow inherits its lexically enclosing class (recorded by the
    // lowerer) so `new self` and visibility still know the lexical class; but a
    // closure's `self::`/`parent::` *ops* compile to run-time scope targets
    // (`is_closure`), because `Closure::bind` can rebind the scope afterwards.
    // Free functions carry no defining class → `cur_class = None`.
    let cur_class = fd
        .defining_class
        .as_ref()
        .and_then(|n| ctx.class_index.get(&n.to_ascii_lowercase()).copied());
    compile_body(
        &fd.name,
        &fd.file,
        &fd.body,
        fd.slots.len() as u32,
        &fd.params,
        &fd.slots,
        fd.by_ref,
        fd.is_generator,
        fd.ret_hint.clone(),
        fd.line,
        ctx,
        cur_class,
        is_closure,
        false,
        fd.closure_shift,
    )
    .map(|mut f| {
        f.attributes = compile_attrs(&fd.attributes, ctx, cur_class);
        f.ret_reflect_type = fd.ret_reflect_type.clone();
        f.doc = fd.doc.clone();
        f.end_line = fd.end_line;
        f
    })
}

/// Compile one body (the script's, a function's, or a method's) into a [`Func`].
/// `cur_class` is the enclosing class id for a method body (so `self`/`parent`
/// resolve at compile time), `None` for the script body and free functions.
#[allow(clippy::too_many_arguments)]
pub(super) fn compile_body(
    name: &[u8],
    file: &[u8],
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
    closure_scope: bool,
    is_main: bool,
    closure_shift: i32,
) -> R<Func> {
    let n_params = params.len() as u32;
    let mut c = FnCompiler::new(ctx, n_locals, cur_class, is_main, slot_names);
    c.returns_ref = by_ref;
    c.closure_scope = closure_scope;
    c.closure_shift = closure_shift;
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
        file: file.into(),
        // Set by `compile_fndecl` (compile_body only sees the pieces).
        doc: None,
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        static_vars: c.static_vars,
        // Named locals plus the high-water mark of compiler temporaries.
        n_slots: n_locals + c.n_temps_max,
        n_params,
        slot_names: slot_names.to_vec().into_boxed_slice(),
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
        // Default-value thunks for `ReflectionParameter::getDefaultValue()` (run in
        // this body's class context). A required/variadic param, or a default that
        // does not compile, has `None`.
        param_defaults: params
            .iter()
            .map(|p| {
                p.default
                    .as_ref()
                    .filter(|_| !p.variadic)
                    .and_then(|e| compile_default_thunk(e, ctx, cur_class))
            })
            .collect(),
        // Constant-reference name of each default (for isDefaultValueConstant).
        param_default_const: params
            .iter()
            .map(|p| p.default.as_ref().filter(|_| !p.variadic).and_then(default_const_name))
            .collect(),
        param_promoted: params.iter().map(|p| p.promoted).collect(),
        // Per-parameter `#[Attr]` thunks for `ReflectionParameter::getAttributes()`,
        // compiled in this body's class context (so a `self::C` argument resolves).
        param_attributes: params
            .iter()
            .map(|p| compile_attrs(&p.attributes, ctx, cur_class))
            .collect(),
        // Composite (union/intersection) reflection types; the return one is set by
        // `compile_fndecl` (compile_body only sees a `TypeHint` return).
        param_reflect_types: params.iter().map(|p| p.reflect_type.clone()).collect(),
        ret_reflect_type: None,
        ret_hint,
        variadic_slot: params.iter().find(|p| p.variadic).map(|p| p.slot),
        by_ref,
        is_generator,
        line: def_line,
        end_line: 0,
        attributes: Vec::new(),
        exc_table: c.exc_regions,
    })
}

/// A placeholder body for a function that could not be compiled yet: it raises
/// a fatal naming the gap if (and only if) the function is actually called. Its
/// slot/param counts mirror the real declaration so the call ABI stays valid.
pub(super) fn stub_func(fd: &FnDecl, err: &CompileError) -> Func {
    let msg = format!(
        "VM: call to `{}` — {}",
        String::from_utf8_lossy(&fd.name),
        err
    );
    Func {
        name: fd.name.clone(),
        file: fd.file.clone(),
        doc: fd.doc.clone(),
        ops: vec![Op::Fatal(0)],
        lines: vec![fd.line],
        consts: vec![Const::Str(msg.into_bytes().into())],
        static_vars: Vec::new(),
        slot_names: fd.slots.to_vec().into_boxed_slice(),
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
        param_defaults: fd.params.iter().map(|_| None).collect(),
        param_default_const: fd.params.iter().map(|_| None).collect(),
        param_promoted: fd.params.iter().map(|_| false).collect(),
        param_attributes: fd.params.iter().map(|_| Vec::new()).collect(),
        param_reflect_types: fd.params.iter().map(|p| p.reflect_type.clone()).collect(),
        ret_reflect_type: fd.ret_reflect_type.clone(),
        ret_hint: fd.ret_hint.clone(),
        variadic_slot: fd.params.iter().find(|p| p.variadic).map(|p| p.slot),
        by_ref: fd.by_ref,
        is_generator: fd.is_generator,
        line: fd.line,
        end_line: 0,
        attributes: Vec::new(),
        exc_table: Vec::new(),
    }
}

/// The `uninitialized(T)` display for a declared type only the reflection-side
/// record models (`mixed`, unions, intersections — the enforced [`TypeHint`]
/// covers the rest).
pub(super) fn reflect_type_display(rt: &crate::hir::ReflectType) -> Vec<u8> {
    use crate::hir::ReflectType;
    let join = |ms: &[crate::hir::ReflectNamed], sep: u8| -> Vec<u8> {
        let mut out = Vec::new();
        for (i, m) in ms.iter().enumerate() {
            if i > 0 {
                out.push(sep);
            }
            out.extend_from_slice(&m.name);
        }
        out
    };
    match rt {
        ReflectType::Single(n, _) => n.name.to_vec(),
        ReflectType::Union(ms) => join(ms, b'|'),
        ReflectType::Intersection(ms) => join(ms, b'&'),
    }
}

/// Compile one property-hook body (step 50) to a [`Func`], in its class's context
/// (so `self::`/`$this` resolve). Mirrors the method-body path; a hook that does
/// not compile becomes a fatal stub, like a method.
pub(super) fn compile_hook(fd: &crate::hir::FnDecl, ctx: &ProgramCtx, cid: ClassId) -> Func {
    compile_body(
        &fd.name,
        &fd.file,
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
        false,
        fd.closure_shift,
    )
    .unwrap_or_else(|e| stub_func(fd, &e))
}

/// Compile the prop-init thunk: for each non-constant property default, `This;
/// <expr>; PropSet{name}; Pop`, ending `PushConst(null); Ret`. Run with `$this` =
/// the new object (see [`Op::InitProps`]); compiled in the class's own context so
/// a `self::CONST` default resolves.
pub(super) fn compile_prop_init(items: &[(Box<[u8]>, &Expr)], ctx: &ProgramCtx, cid: ClassId) -> R<Func> {
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
        file: Box::default(),
        doc: None,
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        static_vars: Vec::new(),
        n_slots: c.n_temps_max,
        n_params: 0,
        slot_names: Box::default(),
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        param_defaults: Box::default(),
        param_default_const: Box::default(),
        param_promoted: Box::default(),
        param_attributes: Box::default(),
        param_reflect_types: Box::default(),
        ret_reflect_type: None,
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        end_line: 0,
        attributes: Vec::new(),
        exc_table: c.exc_regions,
    })
}

/// Compile a parameter default expression into a value thunk (`<expr>; Ret`) run
/// in the function's class context (`cur_class`, `None` for a free function so
/// `self::` is unavailable — as it is in PHP). Returns `None` if the expression
/// does not compile, so the parameter simply reports no available default rather
/// than failing the whole class. Used only by reflection, never the call ABI.
pub(super) fn compile_default_thunk(value: &Expr, ctx: &ProgramCtx, cur_class: Option<ClassId>) -> Option<Func> {
    let mut c = FnCompiler::new(ctx, 0, cur_class, false, &[]);
    c.expr(value).ok()?;
    c.emit(Op::Ret);
    Some(Func {
        name: Box::default(),
        file: Box::default(),
        doc: None,
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        static_vars: Vec::new(),
        n_slots: c.n_temps_max,
        n_params: 0,
        slot_names: Box::default(),
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        param_defaults: Box::default(),
        param_default_const: Box::default(),
        param_promoted: Box::default(),
        param_attributes: Box::default(),
        param_reflect_types: Box::default(),
        ret_reflect_type: None,
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        end_line: 0,
        attributes: Vec::new(),
        exc_table: Vec::new(),
    })
}

/// Compile a class-constant value expression into a thunk [`Func`] (`<expr>; Ret`)
/// evaluated in `decl_class`'s context (so a `self::OTHER` inside resolves).
pub(super) fn compile_const_thunk(name: &[u8], value: &Expr, ctx: &ProgramCtx, decl_class: ClassId) -> R<Func> {
    let mut c = FnCompiler::new(ctx, 0, Some(decl_class), false, &[]);
    c.expr(value)?;
    c.emit(Op::Ret);
    Ok(Func {
        name: name.into(),
        file: Box::default(),
        doc: None,
        ops: c.ops,
        lines: c.lines,
        consts: c.consts,
        static_vars: Vec::new(),
        n_slots: c.n_temps_max,
        n_params: 0,
        slot_names: Box::default(),
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        param_defaults: Box::default(),
        param_default_const: Box::default(),
        param_promoted: Box::default(),
        param_attributes: Box::default(),
        param_reflect_types: Box::default(),
        ret_reflect_type: None,
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        end_line: 0,
        attributes: Vec::new(),
        exc_table: c.exc_regions,
    })
}

/// Compile a slice of HIR attributes into runtime [`CompiledAttribute`]s (the
/// two-thunk `new`/`args` scheme), in `cur_class`'s context so a `self::CONST`
/// in a method-attribute argument resolves. Shared by the function / method
/// attribute reflection paths (class / property paths build them inline).
pub(super) fn compile_attrs(
    attrs: &[crate::hir::HirAttribute],
    ctx: &ProgramCtx,
    cur_class: Option<ClassId>,
) -> Vec<CompiledAttribute> {
    let decl = cur_class.unwrap_or(0);
    attrs
        .iter()
        .map(|a| {
            let new_thunk = compile_const_thunk(&a.name, &a.new_expr, ctx, decl)
                .unwrap_or_else(|e| const_stub(&a.name, &e));
            let args_thunk = compile_const_thunk(&a.name, &a.args_expr, ctx, decl)
                .unwrap_or_else(|e| const_stub(&a.name, &e));
            CompiledAttribute { name: a.name.clone(), new_thunk, args_thunk }
        })
        .collect()
}

/// A placeholder thunk for a constant whose value couldn't be compiled: fatals
/// (naming the gap) only if the constant is read.
pub(super) fn const_stub(name: &[u8], err: &CompileError) -> Func {
    let msg = format!("VM: constant `{}` — {}", String::from_utf8_lossy(name), err);
    Func {
        name: name.into(),
        file: Box::default(),
        doc: None,
        ops: vec![Op::Fatal(0)],
        lines: vec![0],
        consts: vec![Const::Str(msg.into_bytes().into())],
        static_vars: Vec::new(),
        n_slots: 0,
        n_params: 0,
        slot_names: Box::default(),
        param_names: Box::default(),
        param_required: Box::default(),
        param_by_ref: Box::default(),
        param_hints: Box::default(),
        param_defaults: Box::default(),
        param_default_const: Box::default(),
        param_promoted: Box::default(),
        param_attributes: Box::default(),
        param_reflect_types: Box::default(),
        ret_reflect_type: None,
        ret_hint: None,
        variadic_slot: None,
        by_ref: false,
        is_generator: false,
        line: 0,
        end_line: 0,
        attributes: Vec::new(),
        exc_table: Vec::new(),
    }
}
