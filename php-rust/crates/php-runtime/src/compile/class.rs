//! Class compilation (compile_class + class helpers). Split from compile/mod.rs.

use super::*;


/// Compile one HIR [`ClassDecl`] into a [`CompiledClass`] (OOP-1). Tolerant, like
/// functions: a method that doesn't compile becomes a [`stub_func`]; a
/// non-constant property default marks the class `ok = false` so [`Op::Alloc`]
/// fatals rather than producing a wrong instance.
///
/// Properties and visibility are flattened **parent-first** (root→leaf), so a
/// redeclared property keeps its inherited position and takes the most-derived
/// default / visibility — matching the tree-walker's `collect_props` /
/// `class_shape` (D-19.10/D-19.20).
pub(super) fn compile_class(cid: ClassId, cd: &ClassDecl, ctx: &ProgramCtx) -> CompiledClass {
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
    // Keyed by *storage key*: the plain name, or the mangled `\0Class\0name` for a
    // private (Stage C) — so a parent's private and a subclass's same-name
    // redeclaration occupy two distinct slots. The source-level name rides along
    // for the prop-init thunk (whose `PropSet` ops carry names, not keys).
    let mut flat_defaults: Vec<(Box<[u8]>, Box<[u8]>, Option<&Expr>, Option<&crate::hir::TypeHint>, Option<&crate::hir::ReflectType>)> = Vec::new();
    let mut vis_entries: Vec<(Box<[u8]>, PropVis)> = Vec::new();
    // Property hooks (step 50), flattened parent-first like the layout: a
    // most-derived `get`/`set` overrides the inherited one. A *virtual* hooked
    // property (no backing) is excluded from the object layout below.
    let mut prop_hooks: HashMap<Box<[u8]>, PropHooks> = HashMap::new();
    // Property names that have backing storage somewhere in the chain (a plain
    // declaration, or a backed hooked one). A hooked property that overrides an
    // inherited *plain* property is itself backed — so it is not "virtual" and a
    // hook-less read/write reaches the backing rather than being write/read-only.
    let mut backed_seen: HashSet<Box<[u8]>> = HashSet::new();
    for &x in &chain {
        let cname = PhpStr::new(ctx.classes[x].name.to_vec());
        for p in &ctx.classes[x].props {
            // The slot a private lives under is mangled with its declaring class.
            let skey: Box<[u8]> = if p.visibility == Visibility::Private {
                php_types::mangle_prop_key(&ctx.classes[x].name, &p.name).into()
            } else {
                p.name.clone()
            };
            if p.get_hook.is_some() || p.set_hook.is_some() {
                let inherited_backed = backed_seen.contains(p.name.as_ref());
                let entry = prop_hooks.entry(p.name.clone()).or_insert(PropHooks {
                    get: None,
                    set: None,
                    backed: p.backed,
                });
                entry.backed = p.backed || inherited_backed;
                if entry.backed {
                    backed_seen.insert(p.name.clone());
                }
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
                backed_seen.insert(p.name.clone());
            }
            // A virtual hooked property has no backing storage: keep it out of the
            // instance layout (not allocated, not dumped). Backed ones stay.
            let virtual_prop = (p.get_hook.is_some() || p.set_hook.is_some()) && !p.backed;
            if !virtual_prop {
                match flat_defaults.iter_mut().find(|(k, _, _, _, _)| k.as_ref() == skey.as_ref()) {
                    Some(e) => {
                        e.2 = p.default.as_ref();
                        e.3 = p.hint.as_ref();
                        e.4 = p.reflect_type.as_ref();
                    }
                    None => flat_defaults.push((skey.clone(), p.name.clone(), p.default.as_ref(), p.hint.as_ref(), p.reflect_type.as_ref())),
                }
                let vis = match p.visibility {
                    Visibility::Public => PropVis::Public,
                    Visibility::Protected => PropVis::Protected,
                    Visibility::Private => PropVis::Private(Rc::clone(&cname)),
                };
                match vis_entries.iter_mut().find(|(k, _)| k.as_ref() == skey.as_ref()) {
                    Some(e) => e.1 = vis,
                    None => vis_entries.push((skey.clone(), vis)),
                }
            } else {
                // A virtual property that shadowed an inherited backed one must
                // also drop the inherited storage entry (the plain slot; a parent's
                // private slot is not shadowable and stays).
                flat_defaults.retain(|(k, _, _, _, _)| k.as_ref() != skey.as_ref() && k.as_ref() != p.name.as_ref());
                vis_entries.retain(|(k, _)| k.as_ref() != skey.as_ref() && k.as_ref() != p.name.as_ref());
            }
        }
    }

    // A constant default is materialised directly; a non-constant one gets a NULL
    // placeholder (so the property exists, in order) and is set at `new` time by
    // the prop-init thunk.
    let mut prop_defaults: Vec<(Box<[u8]>, Const)> = Vec::new();
    let mut init_items: Vec<(Box<[u8]>, &Expr)> = Vec::new();
    // A typed property with no default starts *uninitialized* (PHP 8.0): stored as
    // `Zval::Undef` at `new` time, reads error, `var_dump` shows `uninitialized(T)`.
    let mut uninit_props: Vec<Box<[u8]>> = Vec::new();
    // Type displays for the typed properties, for the uninitialized rendering.
    let mut prop_type_displays: Vec<(Box<[u8]>, Box<[u8]>)> = Vec::new();
    for (skey, _name, default, hint, rt) in &flat_defaults {
        // The `uninitialized(T)` display: the enforced hint's name, or — for a
        // type only the reflection record models (mixed / unions /
        // intersections) — a render of that record.
        let display: Option<Box<[u8]>> = match (hint, rt) {
            (Some(h), _) => Some(h.display_name().into_bytes().into()),
            (None, Some(r)) => Some(reflect_type_display(r).into()),
            (None, None) => None,
        };
        if let Some(d) = display {
            prop_type_displays.push((skey.clone(), d));
        }
        match default {
            None => {
                prop_defaults.push((skey.clone(), Const::Null));
                // ANY declared type without default → uninitialized (Zend:
                // `mixed`/union properties included); untyped → NULL.
                if hint.is_some() || rt.is_some() {
                    uninit_props.push(skey.clone());
                }
            }
            Some(e) => match const_eval(e) {
                Some(c) => prop_defaults.push((skey.clone(), c)),
                None => {
                    prop_defaults.push((skey.clone(), Const::Null));
                    // The thunk writes by *storage key*, not source name: for a
                    // public/protected property they coincide, while a private's
                    // mangled `\0Class\0name` bypasses PropSet's declared-slot
                    // resolution (which picks the MOST-DERIVED declaration —
                    // wrong for a parent's private shadowed by a subclass
                    // redeclaration: the parent's default landed in the child's
                    // slot and PHPUnit's TestCase::$data read null). The raw
                    // fallback writes the exact declaring slot; hooks/type
                    // checks are keyed by plain name and correctly stay out of
                    // a default's materialisation.
                    init_items.push((skey.clone(), e));
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
                &m.decl.file,
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
                false,
                m.decl.closure_shift,
            ) {
                Ok(f) => f,
                Err(e) => stub_func(&m.decl, &e),
            };
            let mut func = func;
            func.attributes = compile_attrs(&m.decl.attributes, ctx, Some(cid));
            func.ret_reflect_type = m.decl.ret_reflect_type.clone();
            func.doc = m.decl.doc.clone();
            func.end_line = m.decl.end_line;
            CompiledMethod {
                name: m.decl.name.clone(),
                visibility: m.visibility,
                is_static: m.is_static,
                is_final: m.is_final,
                func,
            }
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
            CompiledConst {
                name: k.name.clone(),
                func,
                visibility: k.visibility,
                is_final: k.is_final,
                attributes: compile_attrs(&k.attributes, ctx, Some(cid)),
            }
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

    // Abstract/interface method signatures (empty bodies): compiled with the
    // full method machinery so param defaults / attributes / types reflect
    // correctly. Never dispatched — the Reflection surface reads them.
    let abstract_sigs: Vec<CompiledMethod> = cd
        .abstract_sigs
        .iter()
        .map(|m| {
            let func = match compile_body(
                &m.decl.name,
                &m.decl.file,
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
                false,
                m.decl.closure_shift,
            ) {
                Ok(f) => f,
                Err(e) => stub_func(&m.decl, &e),
            };
            let mut func = func;
            func.attributes = compile_attrs(&m.decl.attributes, ctx, Some(cid));
            func.ret_reflect_type = m.decl.ret_reflect_type.clone();
            func.doc = m.decl.doc.clone();
            func.end_line = m.decl.end_line;
            CompiledMethod {
                name: m.decl.name.clone(),
                visibility: m.visibility,
                is_static: m.is_static,
                is_final: m.is_final,
                func,
            }
        })
        .collect();

    // Own declared properties, in declaration order — used only for the *ordered*
    // per-class enumeration in `get_object_vars` / `get_class_vars`. The readonly /
    // typed / visibility *lookups* now go through the flattened `prop_info` table.
    let own_prop_vis = cd.props.iter().map(|p| (p.name.clone(), p.visibility)).collect();

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

    // Class attributes (`#[Foo(args)]`): each compiles to two thunks run in this
    // class's context — one that constructs `new Foo(args)` (newInstance), one that
    // builds the argument array (getArguments). A thunk that doesn't compile
    // becomes a fatal stub, hit only if reflected on.
    let attributes = cd
        .attributes
        .iter()
        .map(|a| {
            let new_thunk = compile_const_thunk(&a.name, &a.new_expr, ctx, cid)
                .unwrap_or_else(|e| const_stub(&a.name, &e));
            let args_thunk = compile_const_thunk(&a.name, &a.args_expr, ctx, cid)
                .unwrap_or_else(|e| const_stub(&a.name, &e));
            CompiledAttribute { name: a.name.clone(), new_thunk, args_thunk }
        })
        .collect();

    // Per-property attributes (`#[Attr] public int $x`), same two-thunk scheme,
    // keyed by the own property name (not flattened). Empty key absent.
    let mut prop_attributes: HashMap<Box<[u8]>, Vec<CompiledAttribute>> = HashMap::new();
    for p in &cd.props {
        if p.attributes.is_empty() {
            continue;
        }
        let attrs = p
            .attributes
            .iter()
            .map(|a| {
                let new_thunk = compile_const_thunk(&a.name, &a.new_expr, ctx, cid)
                    .unwrap_or_else(|e| const_stub(&a.name, &e));
                let args_thunk = compile_const_thunk(&a.name, &a.args_expr, ctx, cid)
                    .unwrap_or_else(|e| const_stub(&a.name, &e));
                CompiledAttribute { name: a.name.clone(), new_thunk, args_thunk }
            })
            .collect();
        prop_attributes.insert(p.name.clone(), attrs);
    }

    // Unified per-property metadata, flattened parent-first so the most-derived
    // (re)declaration wins — baking in the shadowing rules the runtime `resolve_*`
    // walks used to re-derive on each access. Mirrors `own_prop_vis` / `prop_types`
    // / `readonly_props` (assigned unconditionally per class, so an untyped or
    // non-readonly redeclaration clears the inherited type / readonly), and then
    // folds in the already-flattened `prop_hooks`. Every declared property (backed
    // or virtual) gets an entry; storage stays name-keyed via `storage_key`.
    let mut prop_info: rustc_hash::FxHashMap<Box<[u8]>, PropInfo> = rustc_hash::FxHashMap::default();
    for &x in &chain {
        for p in &ctx.classes[x].props {
            prop_info.insert(
                p.name.clone(),
                PropInfo {
                    visibility: p.visibility,
                    set_visibility: p.set_visibility,
                    declaring_class: x,
                    readonly: p.readonly,
                    type_hint: p.hint.clone(),
                    reflect_type: p.reflect_type.clone(),
                    hooks: None,
                    storage_key: if p.visibility == Visibility::Private {
                        php_types::mangle_prop_key(&ctx.classes[x].name, &p.name).into()
                    } else {
                        p.name.clone()
                    },
                    doc: p.doc.clone(),
                },
            );
        }
    }
    for (name, hooks) in &prop_hooks {
        if let Some(pi) = prop_info.get_mut(name) {
            pi.hooks = Some(hooks.clone());
        }
    }

    CompiledClass {
        name: cd.name.clone(),
        file: cd.file.clone(),
        line: cd.line,
        end_line: cd.end_line,
        doc: cd.doc.clone(),
        class_name: PhpStr::new(cd.name.to_vec()),
        parent: cd.parent,
        interfaces: cd.interfaces.clone(),
        instantiable,
        is_final: cd.is_final,
        is_abstract: cd.is_abstract,
        prop_defaults,
        info: Rc::new(ObjectInfo::from_entries_typed(vis_entries, prop_type_displays)),
        methods,
        abstract_methods: cd.abstract_methods.clone(),
        abstract_sigs,
        own_prop_vis,
        static_props,
        prop_init,
        consts,
        enum_cases,
        attributes,
        prop_attributes,
        uses_traits: cd.uses_traits.clone(),
        uninit_props,
        ok,
        prop_info,
    }
}

/// The class ancestry root→leaf (parent-first), for flattening properties.
pub(super) fn class_chain(classes: &[std::rc::Rc<ClassDecl>], cid: ClassId) -> Vec<ClassId> {
    let mut chain = Vec::new();
    let mut c = Some(cid);
    while let Some(x) = c {
        chain.push(x);
        c = classes[x].parent;
    }
    chain.reverse();
    chain
}

/// Find the constant `name` (case-sensitive, like PHP) reachable from class
/// `cid`: own constants and the parent chain first, then implemented interfaces
/// transitively. Returns the *declaring* class id and the constant's value
/// expression, so it can be folded in that class's context.
pub(super) fn find_const_decl<'a>(cid: ClassId, name: &[u8], ctx: &'a ProgramCtx<'a>) -> Option<(ClassId, &'a Expr)> {
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

/// Map a class name that does not resolve to a `ClassId` onto a built-in
/// interface, if it is one. These (`Generator`/`Iterator`/`Traversable`) are not
/// registered in the prelude, so `instanceof` against them is decided by the
/// operand's runtime type instead of the class table. The namespace prefix is
/// stripped and the comparison is case-insensitive, matching PHP name resolution.
pub(super) fn builtin_iface_for(name: &[u8]) -> Option<BuiltinIface> {
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

/// An inert [`CompiledClass`] for a *seed* class the running VM already links by
/// name: `drive_unit`'s remap dedups it to the existing global id, so this
/// stub's only consumer is that name lookup. `ok: false` makes an accidental
/// `Op::Alloc` on it fatal (fail-loud) rather than silently misbehaving.
pub(super) fn stub_class(cd: &crate::hir::ClassDecl) -> CompiledClass {
    CompiledClass {
        name: cd.name.clone(),
        file: b"seed-stub".to_vec().into_boxed_slice(),
        line: 0,
        end_line: 0,
        doc: None,
        class_name: PhpStr::new(cd.name.to_vec()),
        parent: None,
        interfaces: Vec::new(),
        instantiable: Instantiable::Yes,
        is_final: false,
        is_abstract: false,
        prop_defaults: Vec::new(),
        info: Rc::new(ObjectInfo::from_entries(Vec::new())),
        methods: Vec::new(),
        abstract_methods: Vec::new(),
        abstract_sigs: Vec::new(),
        own_prop_vis: Vec::new(),
        static_props: Vec::new(),
        prop_init: None,
        consts: Vec::new(),
        enum_cases: Vec::new(),
        attributes: Vec::new(),
        prop_attributes: Default::default(),
        uses_traits: Vec::new(),
        uninit_props: Vec::new(),
        ok: false,
        prop_info: Default::default(),
    }
}
