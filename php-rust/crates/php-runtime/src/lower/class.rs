//! HIR lowering of classes, interfaces, traits, enums, methods, properties, closures and arrow functions. Split out of `lower.rs` (step 61).
use std::collections::{HashMap, HashSet};

use mago_span::HasSpan;
use mago_syntax::ast::{
    AnonymousClass, ArrowFunction,
    Class, ClassLikeMember, Closure, Enum, EnumCaseItem, Extends, Function, Implements,
    FunctionLikeParameterList, Hint, Interface, MagicConstant, Method,
    MethodBody, Modifier, Property, PropertyHook, PropertyHookBody, PropertyHookConcreteBody,
    PropertyHookList, PropertyItem, Statement, Trait, TraitUse, TraitUseAdaptation,
    TraitUseMethodReference, TraitUseSpecification,
};

use crate::hir::{
    Capture, ClassDecl, ClassRef, Expr, ExprKind, FnDecl, Line, LoweredTrait, MethodDecl, Param,
    Place, PlaceBase, PlaceStep, PropDecl, Slot, Stmt, StmtKind, TypeHint,
    Visibility,
};


use super::*;

impl<'f> Lowerer<'f> {
    /// Lower every top-level `trait T { ... }` into [`Lowerer::traits`] (step 21).
    /// Each is resolved on demand (so a trait may `use` another declared later)
    /// with a cycle guard; nested `use` clauses are flattened in (D-21.8).
    pub(super) fn lower_traits(&mut self, stmts: &[Statement]) -> Result<(), LowerError> {
        let mut asts: HashMap<Vec<u8>, &Trait> = HashMap::new();
        for s in stmts {
            if let Statement::Trait(t) = s {
                let key = t.name.value.to_ascii_lowercase();
                if self.class_index.contains_key(&key) || asts.contains_key(&key) {
                    return Err(LowerError::Unsupported {
                        what: "trait redeclaration",
                        line: self.line_of(t.span()),
                    });
                }
                asts.insert(key, t);
            }
        }
        let mut in_progress: HashSet<Vec<u8>> = HashSet::new();
        let names: Vec<Vec<u8>> = asts.keys().cloned().collect();
        for n in names {
            self.resolve_trait(&n, &asts, &mut in_progress)?;
        }
        Ok(())
    }

    /// Lower one trait into [`Lowerer::traits`], memoised. Resolves the trait's
    /// own `use` clauses first (so nested members are present), then flattens
    /// them in with the trait's own members taking precedence (step 21).
    pub(super) fn resolve_trait(
        &mut self,
        key: &[u8],
        asts: &HashMap<Vec<u8>, &Trait>,
        in_progress: &mut HashSet<Vec<u8>>,
    ) -> Result<(), LowerError> {
        if self.traits.contains_key(key) {
            return Ok(());
        }
        // Not in this unit's trait ASTs and not already seeded → surface it as an
        // undefined *class* so the include-time autoload retry (lower_unit) can
        // load the trait's file, then re-lower. The name is qualified with the
        // current namespace (best effort for a same-namespace trait).
        let t = match asts.get(key) {
            Some(t) => *t,
            None => {
                return Err(LowerError::UndefinedClass {
                    name: join_ns(&self.cur_namespace, key),
                    kind: MissingSym::Trait,
                    line: 0,
                })
            }
        };
        let line = self.line_of(t.span());
        if !in_progress.insert(key.to_vec()) {
            return Err(LowerError::Unsupported {
                what: "circular trait use",
                line,
            });
        }
        // No late binding inside trait bodies: a trait's members are copied
        // verbatim into consumers — possibly in *other* units, where a deferred
        // declaration's index into this unit's `deferred` table would dangle
        // (closures have a cross-unit shift mechanism; deferred decls do not).
        // An unresolvable supertype in a trait member therefore stays the
        // eager `UndefinedClass` (pre-late-binding behaviour). Restored below.
        let saved_defer = std::mem::replace(&mut self.defer, DeferConf::No);
        let mut methods = Vec::new();
        let mut props = Vec::new();
        let mut static_props = Vec::new();
        let mut consts = Vec::new();
        let mut abstract_methods: Vec<Box<[u8]>> = Vec::new();
        let mut uses: Vec<&TraitUse> = Vec::new();
        // Closures lowered while lowering this trait's own methods occupy a
        // contiguous range [clo_start, clo_end) in the unit closure table; captured
        // so a consumer in another unit can re-append and shift them.
        let clo_start = self.closures.len();
        // `__TRAIT__` in any method body resolves to this trait name (step 49).
        // `__CLASS__` inside a trait method is the *using* class in PHP, which is
        // unknown here (members are lowered once, then copied per consumer); it
        // resolves to "" — a documented divergence (D-49).
        let saved_trait = self.cur_trait.replace(t.name.value.into());
        for member in t.members.iter() {
            match member {
                ClassLikeMember::Property(p) => {
                    self.lower_property(p, &mut props, &mut static_props, line)?
                }
                ClassLikeMember::Method(m) if matches!(m.body, MethodBody::Abstract(_)) => {
                    abstract_methods.push(m.name.value.into())
                }
                ClassLikeMember::Method(m) => {
                    methods.push(self.lower_method(m, line, &mut props)?)
                }
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts, line)?,
                ClassLikeMember::TraitUse(u) => uses.push(u),
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "trait member",
                        line,
                    })
                }
            }
        }
        self.cur_trait = saved_trait;
        let trait_closures: Vec<FnDecl> = self.closures[clo_start..].to_vec();
        // Resolve any nested traits before flattening their members in.
        for u in &uses {
            for tn in u.trait_names.iter() {
                let nk = bare_last_segment(tn).to_ascii_lowercase();
                self.resolve_trait(&nk, asts, in_progress)?;
            }
        }
        let (own_m, own_p, own_s, own_c) = member_name_sets(&methods, &props, &static_props, &consts);
        let mut t_methods = Vec::new();
        let mut t_props = Vec::new();
        let mut t_static = Vec::new();
        let mut t_consts = Vec::new();
        self.flatten_into(
            &uses,
            t.name.value,
            (&own_m, &own_p, &own_s, &own_c),
            (&mut t_methods, &mut t_props, &mut t_static, &mut t_consts),
            &mut abstract_methods,
            line,
        )?;
        // The trait's own members come first; members pulled in from nested
        // traits follow (PHP lists own properties before inherited-via-trait
        // ones). Precedence is already enforced inside flatten_into.
        methods.extend(t_methods);
        props.extend(t_props);
        static_props.extend(t_static);
        consts.extend(t_consts);
        in_progress.remove(key);
        self.defer = saved_defer;
        self.traits.insert(
            key.to_vec(),
            LoweredTrait {
                name: join_ns(&self.cur_namespace, t.name.value),
                methods,
                props,
                static_props,
                consts,
                abstract_methods,
                closures: trait_closures,
                closure_base: clo_start as u32,
                external: false,
            },
        );
        Ok(())
    }

    /// Copy the members of every trait named in `uses` into the four `out` vecs.
    /// Honours `insteadof`/`as` adaptations (D-21.6/7), gives the consumer's own
    /// declarations precedence (`own_*`, D-21.4), and raises the PHP collision
    /// fatal when two traits supply the same method unresolved (D-21.5). Reads
    /// `self.traits`, which the caller has ensured is fully resolved.
    #[allow(clippy::type_complexity)]
    fn flatten_into(
        &mut self,
        uses: &[&TraitUse],
        consumer_name: &[u8],
        own: (
            &HashSet<Vec<u8>>,
            &HashSet<Vec<u8>>,
            &HashSet<Vec<u8>>,
            &HashSet<Vec<u8>>,
        ),
        out: (
            &mut Vec<MethodDecl>,
            &mut Vec<PropDecl>,
            &mut Vec<crate::hir::StaticPropDecl>,
            &mut Vec<crate::hir::ClassConstDecl>,
        ),
        abstract_methods: &mut Vec<Box<[u8]>>,
        line: Line,
    ) -> Result<(), LowerError> {
        let (own_m, own_p, own_s, own_c) = own;
        let (methods, props, static_props, consts) = out;

        // --- collect adaptations across all `use` clauses ---
        // (trait_lc, method_lc) excluded by an `insteadof` (the losers).
        let mut excluded: HashSet<(Vec<u8>, Vec<u8>)> = HashSet::new();
        // `T::m as [vis] alias;` / `m as [vis] alias;` requests, applied last.
        struct Alias {
            trait_lc: Option<Vec<u8>>,
            method_lc: Vec<u8>,
            alias: Option<Box<[u8]>>,
            vis: Option<Visibility>,
            /// Source-case names, for PHP's fatal messages.
            trait_orig: Option<Vec<u8>>,
            method_orig: Vec<u8>,
        }
        let mut aliases: Vec<Alias> = Vec::new();
        for u in uses {
            if let TraitUseSpecification::Concrete(spec) = &u.specification {
                for ad in spec.adaptations.iter() {
                    match ad {
                        TraitUseAdaptation::Precedence(p) => {
                            let m_lc = p.method_reference.method_name.value.to_ascii_lowercase();
                            for loser in p.trait_names.iter() {
                                excluded
                                    .insert((bare_last_segment(loser).to_ascii_lowercase(), m_lc.clone()));
                            }
                        }
                        TraitUseAdaptation::Alias(a) => {
                            let (trait_lc, method_lc, trait_orig, method_orig) =
                                match &a.method_reference {
                                    TraitUseMethodReference::Absolute(abs) => (
                                        Some(bare_last_segment(&abs.trait_name).to_ascii_lowercase()),
                                        abs.method_name.value.to_ascii_lowercase(),
                                        Some(bare_last_segment(&abs.trait_name).to_vec()),
                                        abs.method_name.value.to_vec(),
                                    ),
                                    TraitUseMethodReference::Identifier(id) => {
                                        (None, id.value.to_ascii_lowercase(), None, id.value.to_vec())
                                    }
                                };
                            aliases.push(Alias {
                                trait_lc,
                                method_lc,
                                alias: a.alias.as_ref().map(|id| id.value.into()),
                                vis: a.visibility.as_ref().map(visibility_of_modifier),
                                trait_orig,
                                method_orig,
                            });
                        }
                    }
                }
            }
        }

        // --- flatten members, applying exclusions + collision detection ---
        let mut from_trait: HashMap<Vec<u8>, (Box<[u8]>, Box<[u8]>)> = HashMap::new();
        let mut seen_p = own_p.clone();
        let mut seen_s = own_s.clone();
        let mut seen_c = own_c.clone();
        for u in uses {
            for tn in u.trait_names.iter() {
                let tkey = bare_last_segment(tn).to_ascii_lowercase();
                let torig: Box<[u8]> = bare_last_segment(tn).into();
                // Unknown trait → surface as an undefined class (with its resolved
                // FQN) so lower_unit's autoload retry can load the trait's file.
                let lt = match self.traits.get(&tkey) {
                    Some(lt) => lt.clone(),
                    None => {
                        return Err(LowerError::UndefinedClass {
                            name: self.resolve_class(tn),
                            kind: MissingSym::Trait,
                            line,
                        })
                    }
                };
                // A trait seeded from another unit carries its own closures: append
                // them to this unit's table and shift the method bodies' closure
                // indices by the append offset (cross-unit trait-closure fix).
                let mshift: i32 = if lt.external && !lt.closures.is_empty() {
                    let delta = self.closures.len() as i32 - lt.closure_base as i32;
                    for mut c in lt.closures.iter().cloned() {
                        c.closure_shift = delta;
                        self.closures.push(c);
                    }
                    delta
                } else {
                    0
                };
                for m in &lt.methods {
                    let m_lc = m.decl.name.to_ascii_lowercase();
                    // `insteadof` loser, or the consumer overrides it → drop.
                    if excluded.contains(&(tkey.clone(), m_lc.clone())) || own_m.contains(&m_lc) {
                        continue;
                    }
                    if let Some((a_trait, a_method)) = from_trait.get(&m_lc) {
                        return Err(LowerError::Fatal {
                            message: format!(
                                "Trait method {}::{} has not been applied as {}::{}, \
                                 because of collision with {}::{}",
                                String::from_utf8_lossy(&torig),
                                String::from_utf8_lossy(&m.decl.name),
                                String::from_utf8_lossy(consumer_name),
                                String::from_utf8_lossy(&m.decl.name),
                                String::from_utf8_lossy(a_trait),
                                String::from_utf8_lossy(a_method),
                            ),
                            line,
                        });
                    }
                    from_trait.insert(m_lc, (torig.clone(), m.decl.name.clone()));
                    let mut m = m.clone();
                    m.decl.closure_shift = mshift;
                    methods.push(m);
                }
                for p in &lt.props {
                    if seen_p.insert(p.name.to_ascii_lowercase()) {
                        props.push(p.clone());
                    }
                }
                for s in &lt.static_props {
                    if seen_s.insert(s.name.to_ascii_lowercase()) {
                        static_props.push(s.clone());
                    }
                }
                for c in &lt.consts {
                    if seen_c.insert(c.name.to_ascii_lowercase()) {
                        consts.push(c.clone());
                    }
                }
                abstract_methods.extend(lt.abstract_methods.iter().cloned());
            }
        }

        // --- apply `as` aliases (sourced straight from the trait table) ---
        for a in &aliases {
            let src = self.find_trait_method(uses, a.trait_lc.as_deref(), &a.method_lc);
            let mut src = match src {
                Some(s) => s,
                None => {
                    if let Some(tl) = &a.trait_lc {
                        if !self.traits.contains_key(tl.as_slice()) {
                            let t = a.trait_orig.as_deref().unwrap_or(tl);
                            let t = String::from_utf8_lossy(t);
                            // A known CLASS in `as`/`insteadof` has its own fatal.
                            let message = if self.class_index.contains_key(tl.as_slice()) {
                                format!("Class {t} is not a trait, Only traits may be used in 'as' and 'insteadof' statements")
                            } else {
                                format!("Could not find trait {t}")
                            };
                            return Err(LowerError::Fatal { message, line });
                        }
                    }
                    // An ABSTRACT trait method is aliasable: the alias adds a
                    // new abstract requirement under the alias name (bug69084);
                    // the original requirement stays.
                    let is_abstract = if let Some(tl) = &a.trait_lc {
                        self.traits.get(tl.as_slice()).is_some_and(|lt| {
                            lt.abstract_methods
                                .iter()
                                .any(|n| n.to_ascii_lowercase() == a.method_lc)
                        })
                    } else {
                        uses.iter().any(|u| {
                            u.trait_names.iter().any(|tn| {
                                self.traits
                                    .get(&bare_last_segment(tn).to_ascii_lowercase())
                                    .is_some_and(|lt| {
                                        lt.abstract_methods
                                            .iter()
                                            .any(|n| n.to_ascii_lowercase() == a.method_lc)
                                    })
                            })
                        })
                    };
                    if is_abstract {
                        if let Some(new_name) = &a.alias {
                            abstract_methods.push(new_name.clone());
                        }
                        continue;
                    }
                    // PHP's link-time fatals, verbatim (zend_inheritance.c):
                    // the explicit `T::m` form has its own wording.
                    let morig = String::from_utf8_lossy(&a.method_orig).into_owned();
                    let message = if let Some(t) = &a.trait_orig {
                        format!(
                            "An alias was defined for {}::{} but this method does not exist",
                            String::from_utf8_lossy(t),
                            morig
                        )
                    } else {
                        match &a.alias {
                            Some(new_name) => format!(
                                "An alias ({}) was defined for method {}(), but this method does not exist",
                                String::from_utf8_lossy(new_name),
                                morig
                            ),
                            None => format!(
                                "The modifiers of the trait method {morig}() are changed, but this method does not exist. Error"
                            ),
                        }
                    };
                    return Err(LowerError::Fatal { message, line });
                }
            };
            match &a.alias {
                Some(new_name) => {
                    src.decl.name = new_name.clone();
                    if let Some(v) = a.vis {
                        src.visibility = v;
                    }
                    methods.retain(|m| !m.decl.name.eq_ignore_ascii_case(new_name));
                    methods.push(src);
                }
                None => {
                    if let Some(v) = a.vis {
                        if let Some(m) = methods
                            .iter_mut()
                            .find(|m| m.decl.name.to_ascii_lowercase() == a.method_lc)
                        {
                            m.visibility = v;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Find a trait method to alias: from a named trait if `trait_lc` is given,
    /// else the first match among the `uses` traits (step 21-3, `as`).
    fn find_trait_method(
        &self,
        uses: &[&TraitUse],
        trait_lc: Option<&[u8]>,
        method_lc: &[u8],
    ) -> Option<MethodDecl> {
        let pick = |lt: &LoweredTrait| {
            lt.methods
                .iter()
                .find(|m| m.decl.name.to_ascii_lowercase() == method_lc)
                .cloned()
        };
        if let Some(tl) = trait_lc {
            return self.traits.get(tl).and_then(pick);
        }
        for u in uses {
            for tn in u.trait_names.iter() {
                let tkey = bare_last_segment(tn).to_ascii_lowercase();
                if let Some(found) = self.traits.get(&tkey).and_then(pick) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Resolve a list of interface names (`implements`/interface `extends`) to
    /// their class ids (step 19-5). Unknown interfaces are out of scope.
    fn resolve_interfaces(&self, names: &[Box<[u8]>], line: Line) -> Result<Vec<usize>, LowerError> {
        let mut out = Vec::new();
        for n in names {
            match self.class_index.get(&n.to_ascii_lowercase()) {
                Some(&i) => out.push(i),
                None => {
                    return Err(LowerError::UndefinedClass {
                        name: n.clone(),
                        kind: MissingSym::Interface,
                        line,
                    })
                }
            }
        }
        Ok(out)
    }

    /// Lower an `interface I extends A, B { const ...; function ...; }` (step
    /// 19-5). Interfaces carry constants and (abstract) method signatures; the
    /// method bodies are absent, so only constants are materialised.
    pub(super) fn lower_interface(&mut self, iface: &Interface) -> Result<ClassDecl, LowerError> {
        let line = self.line_of(iface.span());
        let interfaces = match &iface.extends {
            Some(ext) => {
                let names: Vec<Box<[u8]>> = ext.types.iter().map(|id| self.resolve_class(id)).collect();
                self.resolve_interfaces(&names, line)?
            }
            None => Vec::new(),
        };
        let mut consts = Vec::new();
        let mut abstract_methods = Vec::new();
        let mut abstract_sigs = Vec::new();
        let mut props = Vec::new();
        let mut static_props = Vec::new();
        for member in iface.members.iter() {
            match member {
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts, line)?,
                // Interface methods are signatures only (abstract) — no body to
                // run, but their names are reported by `get_class_methods` and
                // their full signatures back the Reflection surface.
                ClassLikeMember::Method(m) => {
                    abstract_methods.push(m.name.value.into());
                    abstract_sigs.push(self.lower_method(m, line, &mut props)?);
                }
                // PHP 8.4 interface properties (`public $p { get; set; }`): a hook
                // contract every implementer must satisfy. Lowered like an
                // abstract-class property — its `get;`/`set;` become abstract hooks
                // and the interface (always abstract) carries them as a contract.
                ClassLikeMember::Property(p) => {
                    self.lower_property(p, &mut props, &mut static_props, line)?
                }
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "interface member",
                        line,
                    })
                }
            }
        }
        Ok(ClassDecl {
            file: self.unit_file(),
            name: join_ns(&self.cur_namespace, iface.name.value),
            doc: self.doc_for(iface.span().start.offset),
            parent: None,
            interfaces,
            is_abstract: true,
            is_final: false,
            is_interface: true,
            props,
            static_props,
            consts,
            methods: Vec::new(),
            abstract_methods,
            abstract_sigs,
            is_enum: false,
            enum_backing: None,
            enum_cases: Vec::new(),
            attributes: Vec::new(),
            uses_traits: Vec::new(),
            line,
            end_line: self.line_of_end(iface.span()),
        })
    }

    /// Lower one `class Name { ... }` into a [`ClassDecl`] (step 19-1). Only
    /// instance properties and methods are in 19-1 scope; `extends`/`implements`,
    /// static members, constants, and other member kinds arrive in later
    /// sub-steps and lower to [`LowerError::Unsupported`] for now.
    /// Whether any ancestor (walking `parent` up the already-lowered class image)
    /// declares a *concrete* method named `name_lc` (lowercased). Used so an
    /// abstract method required by a trait counts as satisfied when a base class
    /// implements it — PHP resolves abstract requirements against the full
    /// inheritance chain, not just the declaring class.
    fn ancestor_has_concrete_method(&self, mut parent: Option<crate::hir::ClassId>, name_lc: &[u8]) -> bool {
        while let Some(cid) = parent {
            let c = &self.classes[cid];
            if c.methods.iter().any(|m| m.decl.name.to_ascii_lowercase() == name_lc) {
                return true;
            }
            parent = c.parent;
        }
        false
    }

    /// The most-derived ancestor that declares method `name_lc`, returned only when
    /// that declaration is `final` (so overriding it is the PHP fatal). A closer
    /// non-final override means the method is legitimately overridable here. `None`
    /// if no ancestor declares it.
    fn final_ancestor_method(&self, mut parent: Option<crate::hir::ClassId>, name_lc: &[u8]) -> Option<crate::hir::ClassId> {
        while let Some(cid) = parent {
            // A forward-declared ancestor may not be lowered yet — stop the walk.
            let c = self.classes.get(cid)?;
            if let Some(m) = c.methods.iter().find(|m| m.decl.name.to_ascii_lowercase() == name_lc) {
                return m.is_final.then_some(cid);
            }
            parent = c.parent;
        }
        None
    }

    pub(super) fn lower_class(&mut self, class: &Class) -> Result<ClassDecl, LowerError> {
        let line = self.line_of(class.span());
        let is_abstract = class.modifiers.iter().any(|m| m.is_abstract());
        let name: Box<[u8]> = join_ns(&self.cur_namespace, class.name.value);
        let mut decl = self.lower_class_body(
            name,
            &class.extends,
            &class.implements,
            is_abstract,
            class.members.iter(),
            line,
        )?;
        decl.doc = self.doc_for(class.span().start.offset);
        decl.end_line = self.line_of_end(class.span());
        // A `readonly class` (PHP 8.2): every (non-static) instance property is
        // readonly, including promoted and trait-supplied ones.
        if class.modifiers.iter().any(|m| m.is_readonly()) {
            for p in &mut decl.props {
                p.readonly = true;
            }
        }
        decl.is_final = class.modifiers.iter().any(|m| m.is_final());
        decl.attributes = self.lower_attributes(&class.attribute_lists, line)?;
        Ok(decl)
    }

    /// Lower the `#[Foo(args), Bar]` attribute lists declared on a class/enum into
    /// retained [`HirAttribute`]s. Each attribute becomes a `new Foo(args)`
    /// expression (run by `ReflectionAttribute::newInstance()`) plus an array
    /// literal of its arguments (run by `getArguments()`). Argument expressions are
    /// lowered in the surrounding context so `self::`/constants resolve as written.
    pub(super) fn lower_attributes(
        &mut self,
        lists: &mago_syntax::ast::Sequence<mago_syntax::ast::AttributeList>,
        line: Line,
    ) -> Result<Vec<crate::hir::HirAttribute>, LowerError> {
        let mut out = Vec::new();
        for list in lists.iter() {
            for attr in list.attributes.iter() {
                let name = self.resolve_class(&attr.name);
                let (args, named) = match &attr.argument_list {
                    Some(l) => self.lower_args(l, line)?,
                    None => (Vec::new(), Vec::new()),
                };
                let new_expr = Expr {
                    kind: ExprKind::New {
                        class: ClassRef::Named(name.clone()),
                        args: args.clone(),
                        named: named.clone(),
                    },
                    line,
                };
                // Build the `getArguments()` array: positional args keyless (int
                // keys 0..), named args keyed by their string name.
                let mut elems: Vec<crate::hir::ArrayElem> = args
                    .into_iter()
                    .map(|value| crate::hir::ArrayElem { key: None, value, by_ref: false })
                    .collect();
                for (k, value) in named {
                    elems.push(crate::hir::ArrayElem {
                        key: Some(Expr { kind: ExprKind::Str(k), line }),
                        value,
                        by_ref: false,
                    });
                }
                let args_expr = Expr { kind: ExprKind::Array(elems), line };
                out.push(crate::hir::HirAttribute { name, new_expr, args_expr });
            }
        }
        Ok(out)
    }

    /// Shared class-body lowering for both named classes and anonymous classes
    /// (`new class {…}`, step 51): resolve `extends`/`implements`, lower members,
    /// flatten traits, enforce abstract implementation, and build the [`ClassDecl`].
    /// The caller supplies the already fully-qualified `name`.
    pub(super) fn lower_class_body<'a, I>(
        &mut self,
        name: Box<[u8]>,
        extends: &Option<Extends>,
        implements: &Option<Implements>,
        is_abstract: bool,
        members: I,
        line: Line,
    ) -> Result<ClassDecl, LowerError>
    where
        I: Iterator<Item = &'a ClassLikeMember<'a>>,
    {
        // Resolve `extends ParentName` to the parent's class id (registered in
        // pass 1 of `hoist_classes`, so forward references work, D-19.10).
        let parent = match extends {
            Some(ext) => {
                let pname = parent_name(self, ext, line)?;
                match self.class_index.get(&pname.to_ascii_lowercase()) {
                    Some(&i) => Some(i),
                    None => {
                        return Err(LowerError::UndefinedClass {
                            name: pname,
                            kind: MissingSym::Class,
                            line,
                        })
                    }
                }
            }
            None => None,
        };
        // A `final` class (or an enum, which is implicitly final) cannot be
        // extended — PHP fatal at the subclass's site, with a distinct message for
        // an enum parent. A forward-declared parent may not be lowered yet (its id
        // is registered but its `ClassDecl` not built); `.get` skips it (the rarer
        // forward-final-parent case is left to runtime, as before).
        if let Some(p) = parent.and_then(|pid| self.classes.get(pid)) {
            if p.is_enum {
                return Err(LowerError::Fatal {
                    message: format!(
                        "Class {} cannot extend enum {}",
                        String::from_utf8_lossy(&name),
                        String::from_utf8_lossy(&p.name)
                    ),
                    line,
                });
            } else if p.is_final {
                return Err(LowerError::Fatal {
                    message: format!(
                        "Class {} cannot extend final class {}",
                        String::from_utf8_lossy(&name),
                        String::from_utf8_lossy(&p.name)
                    ),
                    line,
                });
            }
        }
        // Resolve `implements I, J` to interface ids (step 19-5).
        let interfaces = match implements {
            Some(imp) => {
                let names: Vec<Box<[u8]>> = imp.types.iter().map(|id| self.resolve_class(id)).collect();
                self.resolve_interfaces(&names, line)?
            }
            None => Vec::new(),
        };
        let mut props = Vec::new();
        let mut static_props = Vec::new();
        let mut consts = Vec::new();
        let mut methods = Vec::new();
        let mut uses: Vec<&TraitUse> = Vec::new();
        // Names of abstract methods this class must implement (its own, plus any
        // pulled in from traits during flattening), step 21-4 / D-21.11.
        let mut abstract_req: Vec<Box<[u8]>> = Vec::new();
        // Full signatures of the abstract methods declared *here* (Reflection).
        let mut abstract_sigs: Vec<crate::hir::MethodDecl> = Vec::new();
        // `__CLASS__`/`__METHOD__` in any method body resolve to this class name
        // (step 49). Restored after the member loop; an early error here aborts
        // the whole lowering, so the leak-on-error path is harmless.
        let saved_cls = self.cur_class.replace(name.clone());
        for member in members {
            match member {
                ClassLikeMember::Property(p) => {
                    self.lower_property(p, &mut props, &mut static_props, line)?
                }
                // An abstract method is a signature only — no body to run. A
                // concrete subclass / consumer must supply the implementation;
                // the signature itself backs the Reflection surface.
                ClassLikeMember::Method(m) if matches!(m.body, MethodBody::Abstract(_)) => {
                    abstract_req.push(m.name.value.into());
                    abstract_sigs.push(self.lower_method(m, line, &mut props)?);
                }
                ClassLikeMember::Method(m) => {
                    methods.push(self.lower_method(m, line, &mut props)?)
                }
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts, line)?,
                ClassLikeMember::TraitUse(u) => uses.push(u),
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "class member",
                        line,
                    })
                }
            }
        }
        self.cur_class = saved_cls;
        // Flatten any `use TraitName;` members into this class (step 21). The
        // class's own declarations take precedence and come first; trait members
        // follow, so the instance layout / dump order matches PHP's (own props
        // before trait props).
        if !uses.is_empty() {
            let (own_m, own_p, own_s, own_c) =
                member_name_sets(&methods, &props, &static_props, &consts);
            let mut t_methods = Vec::new();
            let mut t_props = Vec::new();
            let mut t_static = Vec::new();
            let mut t_consts = Vec::new();
            self.flatten_into(
                &uses,
                &name,
                (&own_m, &own_p, &own_s, &own_c),
                (&mut t_methods, &mut t_props, &mut t_static, &mut t_consts),
                &mut abstract_req,
                line,
            )?;
            methods.extend(t_methods);
            props.extend(t_props);
            static_props.extend(t_static);
            consts.extend(t_consts);
        }
        // By-ref property-hook validations (PHP 8.4, `zend_inheritance.c` wording
        // verbatim). 1: a *backed* property (own backing, or backing inherited
        // from an ancestor's plain/backed declaration) whose get hook returns by
        // reference must not also declare a set hook — every write is expected
        // to flow through the reference the get hook hands out.
        for p in &props {
            if p.get_hook.as_ref().is_some_and(|g| g.by_ref)
                && p.set_hook.is_some()
                && (p.backed || self.ancestor_prop_backed(parent, &p.name))
            {
                return Err(LowerError::Fatal {
                    message: format!(
                        "Get hook of backed property {}::{} with set hook may not return by reference",
                        String::from_utf8_lossy(&name),
                        String::from_utf8_lossy(&p.name),
                    ),
                    line,
                });
            }
        }
        // 2: an interface's `&get` contract must be implemented by a by-ref get
        // hook. A plain property satisfies it too (its storage is addressable),
        // and the reverse (a by-value `get;` implemented by `&get`) is fine.
        let mut ifq: Vec<usize> = interfaces.clone();
        let mut if_seen: std::collections::HashSet<usize> = ifq.iter().copied().collect();
        while let Some(i) = ifq.pop() {
            let Some(icd) = self.classes.get(i) else { continue };
            for &ii in &icd.interfaces {
                if if_seen.insert(ii) {
                    ifq.push(ii);
                }
            }
            for ip in &icd.props {
                if !ip.abstract_hooks.iter().any(|h| h.as_ref() == b"&get") {
                    continue;
                }
                let val_get = props
                    .iter()
                    .find(|p| p.name == ip.name)
                    .and_then(|p| p.get_hook.as_ref())
                    .is_some_and(|g| !g.by_ref);
                if val_get {
                    return Err(LowerError::Fatal {
                        message: format!(
                            "Declaration of {c}::${p}::get() must be compatible with & {i}::${p}::get()",
                            c = String::from_utf8_lossy(&name),
                            p = String::from_utf8_lossy(&ip.name),
                            i = String::from_utf8_lossy(&icd.name),
                        ),
                        line,
                    });
                }
            }
        }
        // A `final` method in an ancestor cannot be overridden (PHP fatal). Check
        // each method this class defines against the most-derived ancestor that
        // declares the same name.
        for m in &methods {
            let name_lc = m.decl.name.to_ascii_lowercase();
            if let Some(decl) = self.final_ancestor_method(parent, &name_lc) {
                return Err(LowerError::Fatal {
                    message: format!(
                        "Cannot override final method {}::{}()",
                        String::from_utf8_lossy(&self.classes[decl].name),
                        String::from_utf8_lossy(&m.decl.name)
                    ),
                    line,
                });
            }
        }
        // Abstract property hooks this class declares directly (`public abstract $p
        // { get; }`, PHP 8.4): each is a contract `$p::get`/`$p::set` that, like an
        // abstract method, must not appear in a non-abstract class. (Inherited
        // abstract hooks from a parent are a later sub-step.)
        let abstract_hook_req: Vec<Box<[u8]>> = props
            .iter()
            .flat_map(|p| {
                p.abstract_hooks.iter().map(move |h| {
                    let mut v = b"$".to_vec();
                    v.extend_from_slice(&p.name);
                    v.extend_from_slice(b"::");
                    // A by-ref abstract get is stored as `&get`; the contract
                    // name stays `$prop::get`.
                    v.extend_from_slice(h.strip_prefix(b"&").unwrap_or(h));
                    v.into_boxed_slice()
                })
            })
            .collect();
        // A concrete class must implement every abstract method it carries (own or
        // trait-supplied); otherwise PHP fatals at link time (D-21.11). Abstract
        // classes and interfaces legitimately leave them open.
        if !is_abstract {
            let mut missing: Vec<&[u8]> = Vec::new();
            for req in &abstract_req {
                let req_lc = req.to_ascii_lowercase();
                if methods.iter().any(|m| m.decl.name.to_ascii_lowercase() == req_lc)
                    || self.ancestor_has_concrete_method(parent, &req_lc)
                {
                    continue;
                }
                if !missing.iter().any(|m| m.eq_ignore_ascii_case(req)) {
                    missing.push(req);
                }
            }
            // An abstract hook declared here is never implemented here (it has no
            // body), so every one is reported (PHP counts it as an abstract method).
            for req in &abstract_hook_req {
                if !missing.iter().any(|m| m.eq_ignore_ascii_case(req)) {
                    missing.push(req);
                }
            }
            if !missing.is_empty() {
                return Err(abstract_unimplemented_fatal(&name, &missing, line));
            }
        }
        // Abstract methods left unimplemented (an abstract class): keep their
        // names so `get_class_methods` reports them (step 47). A concrete class
        // implements them all, so this is empty there.
        let abstract_methods: Vec<Box<[u8]>> = abstract_req
            .iter()
            .filter(|req| {
                let lc = req.to_ascii_lowercase();
                !methods.iter().any(|m| m.decl.name.to_ascii_lowercase() == lc)
                    && !self.ancestor_has_concrete_method(parent, &lc)
            })
            .cloned()
            .collect();
        // The directly-used traits' resolved names, kept for `class_uses()` /
        // `ReflectionClass::getTraitNames()` (the members are already flattened in).
        let uses_traits: Vec<Box<[u8]>> = uses
            .iter()
            .flat_map(|u| u.trait_names.iter())
            .map(|tn| self.resolve_class(tn))
            .collect();
        Ok(ClassDecl {
            file: self.unit_file(),
            name,
            // Attached by the named-declaration lowerers (`lower_class`);
            // anonymous classes keep none.
            doc: None,
            parent,
            interfaces,
            is_abstract,
            is_final: false,
            is_interface: false,
            props,
            static_props,
            consts,
            methods,
            abstract_methods,
            abstract_sigs,
            is_enum: false,
            enum_backing: None,
            enum_cases: Vec::new(),
            attributes: Vec::new(),
            uses_traits,
            line,
            // Placeholder; lower_class overrides with the real closing-brace line.
            end_line: line,
        })
    }

    /// Lower `new class(args) extends P implements I { … }` (step 51): lower the
    /// body like a named class under a unique synthetic `class@anonymous…` name,
    /// stash it for registration after the main pass, and return a `new` of that
    /// name with the constructor arguments evaluated at the use site.
    pub(super) fn lower_anonymous_class(
        &mut self,
        anon: &AnonymousClass,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        let n = self.anon_count;
        self.anon_count += 1;
        // Mirror PHP's mangled anonymous-class name `class@anonymous\0…`: the part
        // before the NUL is what `var_dump`/`print_r` show (`class@anonymous`),
        // while `get_class()`/`::class` return the whole string (EXPECTF `%s`
        // matches the `\0…` tail). `@`/NUL keep it unreachable from user names.
        // The tail embeds the *unit* (like PHP's `file:line$handle`): a bare
        // per-unit counter collides across `include`d units in the link table
        // (two units' `class@anonymous\x00` resolved to whichever linked first).
        let mut nm = b"class@anonymous\0".to_vec();
        nm.extend_from_slice(&self.prog_name);
        nm.extend_from_slice(format!(":{line}${n}").as_bytes());
        let name: Box<[u8]> = nm.into();
        let is_abstract = anon.modifiers.iter().any(|m| m.is_abstract());
        let ctx = self.save_body_ctx();
        let mut decl = match self.lower_class_body(
            name.clone(),
            &anon.extends,
            &anon.implements,
            is_abstract,
            anon.members.iter(),
            line,
        ) {
            Ok(d) => d,
            // An unresolvable (post-autoload) supertype: Zend binds an anonymous
            // class when its `new` executes — defer the whole expression (the
            // constructor arguments are part of the snippet and evaluate in the
            // caller's scope via the VM's scope bridge).
            Err(LowerError::UndefinedClass { name: missing, .. })
                if self.deferrable(&missing) =>
            {
                self.restore_body_ctx(ctx);
                let idx = self.push_deferred(anon.span(), Box::default(), "class", true);
                return Ok(ExprKind::NewAnonDeferred(idx));
            }
            Err(e) => return Err(e),
        };
        if anon.modifiers.iter().any(|m| m.is_readonly()) {
            for p in &mut decl.props {
                p.readonly = true;
            }
        }
        decl.is_final = anon.modifiers.iter().any(|m| m.is_final());
        self.anon_classes.push(decl);
        let (args, named) = match &anon.argument_list {
            Some(list) => self.lower_args(list, line)?,
            None => (Vec::new(), Vec::new()),
        };
        Ok(ExprKind::New { class: ClassRef::Named(name), args, named })
    }

    /// Lower one `enum E [: int|string] { case ...; methods; consts }` into a
    /// [`ClassDecl`] with `is_enum = true` (step 23, D-23.1). Cases go to
    /// `enum_cases`; methods/constants/trait-uses reuse the class machinery.
    /// Every enum implements `UnitEnum` (backed ones also `BackedEnum`, D-23.7).
    pub(super) fn lower_enum(&mut self, en: &Enum) -> Result<ClassDecl, LowerError> {
        let line = self.line_of(en.span());
        let name: Box<[u8]> = join_ns(&self.cur_namespace, en.name.value);
        // Backing type (`: int` / `: string`), if any (D-23.10).
        let enum_backing = match &en.backing_type_hint {
            Some(bt) => Some(match &bt.hint {
                Hint::Integer(_) => crate::hir::EnumBacking::Int,
                Hint::String(_) => crate::hir::EnumBacking::Str,
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "enum backing type (only int/string)",
                        line,
                    })
                }
            }),
            None => None,
        };
        // Marker interfaces (D-23.7) + any user `implements`. The markers are the
        // global `UnitEnum`/`BackedEnum`; user interfaces resolve in-namespace.
        let mut iface_names: Vec<Box<[u8]>> = vec![(b"UnitEnum" as &[u8]).into()];
        if enum_backing.is_some() {
            iface_names.push((b"BackedEnum" as &[u8]).into());
        }
        if let Some(imp) = &en.implements {
            iface_names.extend(imp.types.iter().map(|id| self.resolve_class(id)));
        }
        let interfaces = self.resolve_interfaces(&iface_names, line)?;

        let mut consts = Vec::new();
        let mut methods = Vec::new();
        let mut enum_cases = Vec::new();
        let mut uses: Vec<&TraitUse> = Vec::new();
        let mut abstract_req: Vec<Box<[u8]>> = Vec::new();
        for member in en.members.iter() {
            match member {
                ClassLikeMember::EnumCase(c) => {
                    let (case_name, value) = match &c.item {
                        EnumCaseItem::Unit(u) => (u.name.value, None),
                        EnumCaseItem::Backed(b) => {
                            (b.name.value, Some(self.lower_expr(b.value)?))
                        }
                    };
                    enum_cases.push(crate::hir::EnumCaseDecl {
                        name: case_name.into(),
                        value,
                    });
                }
                ClassLikeMember::Method(m) if matches!(m.body, MethodBody::Abstract(_)) => {
                    abstract_req.push(m.name.value.into())
                }
                ClassLikeMember::Method(m) => {
                    // Enums have no instance properties, so promotion cannot occur.
                    methods.push(self.lower_method(m, line, &mut Vec::new())?)
                }
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts, line)?,
                ClassLikeMember::TraitUse(u) => uses.push(u),
                // Enums may not declare properties (PHP fatal); we reject them.
                ClassLikeMember::Property(_) => {
                    return Err(LowerError::Unsupported {
                        what: "property in enum",
                        line,
                    })
                }
            }
        }
        if !uses.is_empty() {
            let props: Vec<crate::hir::PropDecl> = Vec::new();
            let static_props: Vec<crate::hir::StaticPropDecl> = Vec::new();
            let (own_m, own_p, own_s, own_c) =
                member_name_sets(&methods, &props, &static_props, &consts);
            let mut t_methods = Vec::new();
            let mut t_props = Vec::new();
            let mut t_static = Vec::new();
            let mut t_consts = Vec::new();
            self.flatten_into(
                &uses,
                &name,
                (&own_m, &own_p, &own_s, &own_c),
                (&mut t_methods, &mut t_props, &mut t_static, &mut t_consts),
                &mut abstract_req,
                line,
            )?;
            methods.extend(t_methods);
            consts.extend(t_consts);
            // Traits used by an enum cannot contribute (static) properties.
            if !t_props.is_empty() || !t_static.is_empty() {
                return Err(LowerError::Unsupported {
                    what: "trait with properties used in enum",
                    line,
                });
            }
        }
        let uses_traits: Vec<Box<[u8]>> = uses
            .iter()
            .flat_map(|u| u.trait_names.iter())
            .map(|tn| self.resolve_class(tn))
            .collect();
        Ok(ClassDecl {
            file: self.unit_file(),
            name,
            doc: self.doc_for(en.span().start.offset),
            parent: None,
            interfaces,
            is_abstract: false,
            // Enums are implicitly final (cannot be extended; ReflectionClass::isFinal).
            is_final: true,
            is_interface: false,
            props: Vec::new(),
            static_props: Vec::new(),
            consts,
            methods,
            // Enums implement any interface methods concretely, so none are left
            // abstract (step 47).
            abstract_methods: Vec::new(),
            abstract_sigs: Vec::new(),
            is_enum: true,
            enum_backing,
            enum_cases,
            attributes: self.lower_attributes(&en.attribute_lists, line)?,
            uses_traits,
            line,
            end_line: self.line_of_end(en.span()),
        })
    }

    /// Lower a `const A = 1, B = 2;` declaration into one [`ClassConstDecl`] per
    /// item (step 19-4). Visibility modifiers (7.1+) are accepted but not
    /// enforced.
    fn lower_class_const(
        &mut self,
        konst: &mago_syntax::ast::ClassLikeConstant,
        out: &mut Vec<crate::hir::ClassConstDecl>,
        line: Line,
    ) -> Result<(), LowerError> {
        let visibility = visibility_of(konst.modifiers.iter());
        let is_final = konst.modifiers.iter().any(|m| m.is_final());
        // The `#[Attr]` list precedes the whole `const A = 1, B = 2;`, so every
        // item shares it (mirrors `lower_property`).
        let attributes = self.lower_attributes(&konst.attribute_lists, line)?;
        for item in konst.items.iter() {
            out.push(crate::hir::ClassConstDecl {
                name: item.name.value.into(),
                value: self.lower_expr(item.value)?,
                visibility,
                is_final,
                attributes: attributes.clone(),
            });
        }
        Ok(())
    }

    /// Lower a property declaration into one entry per item (`public $a = 1, $b;`),
    /// routing `static` properties to `static_out` and instance properties to
    /// `out` (step 19-1/19-4). A hooked property (`public $p { get … }`, PHP 8.4,
    /// step 50) is a single item carrying `get`/`set` hook bodies.
    fn lower_property(
        &mut self,
        prop: &Property,
        out: &mut Vec<PropDecl>,
        static_out: &mut Vec<crate::hir::StaticPropDecl>,
        line: Line,
    ) -> Result<(), LowerError> {
        // The property's `/** … */` doc block (shared by every item of a grouped
        // `public $a, $b;`), for ReflectionProperty::getDocComment / the export.
        let doc = self.doc_for(prop.span().start.offset);
        let plain = match prop {
            Property::Plain(p) => p,
            Property::Hooked(h) => {
                let visibility = visibility_of(h.modifiers.iter());
                let readonly = h.modifiers.iter().any(|m| m.is_readonly());
                // PHP 8.4 hook-modifier validity, checked here (before the
                // class-level abstract enforcement) so the property-specific fatal
                // wins, matching PHP's compile order.
                let is_private = matches!(visibility, Visibility::Private);
                let is_abstract = h.modifiers.iter().any(|m| m.is_abstract());
                // `final` on the property itself vs on an individual hook
                // (`{ final get; }`) yields different diagnostics.
                let prop_final = h.modifiers.iter().any(|m| m.is_final());
                let hook_final =
                    h.hook_list.hooks.iter().any(|hk| hk.modifiers.iter().any(|m| m.is_final()));
                let fatal = |message: &str| LowerError::Fatal { message: message.into(), line };
                if h.modifiers.iter().any(|m| m.is_static()) {
                    return Err(fatal("Cannot declare hooks for static property"));
                }
                if is_abstract && prop_final {
                    return Err(fatal("Cannot use the final modifier on an abstract property"));
                }
                if is_abstract && is_private {
                    return Err(fatal("Property hook cannot be both abstract and private"));
                }
                if is_abstract && hook_final {
                    return Err(fatal("Property hook cannot be both abstract and final"));
                }
                if (prop_final || hook_final) && is_private {
                    return Err(fatal("Property hook cannot be both final and private"));
                }
                let (var, default) = match &h.item {
                    PropertyItem::Abstract(a) => (&a.variable, None),
                    PropertyItem::Concrete(c) => (&c.variable, Some(self.lower_expr(c.value)?)),
                };
                let name: Box<[u8]> = strip_dollar(var.name).into();
                let (get_hook, set_hook, hooks_backing, abstract_hooks) =
                    self.lower_hooks(&h.hook_list, &name, line)?;
                // A property is backed iff it has a default, or a hook reads/writes
                // its own `$this->name`; otherwise it is virtual (no storage).
                let backed = default.is_some() || hooks_backing;
                let hint = self.lower_prop_hint(h.hint.as_ref(), &default);
                let reflect_type = h.hint.as_ref().and_then(|hh| lower_reflect_type(self, hh));
                let attributes = self.lower_attributes(&h.attribute_lists, line)?;
                out.push(PropDecl {
                    doc,
                    name, visibility, set_visibility: set_visibility_of(h.modifiers.iter()),
                    default, get_hook, set_hook, backed, readonly, hint, abstract_hooks, attributes, reflect_type,
                });
                return Ok(());
            }
        };
        let is_static = plain.modifiers.iter().any(|m| m.is_static());
        let visibility = visibility_of(plain.modifiers.iter());
        let readonly = plain.modifiers.iter().any(|m| m.is_readonly());
        for item in plain.items.iter() {
            let (var, default) = match item {
                PropertyItem::Abstract(a) => (&a.variable, None),
                PropertyItem::Concrete(c) => (&c.variable, Some(self.lower_expr(c.value)?)),
            };
            let name: Box<[u8]> = strip_dollar(var.name).into();
            if is_static {
                static_out.push(crate::hir::StaticPropDecl {
                    name,
                    visibility,
                    default,
                });
            } else {
                let hint = self.lower_prop_hint(plain.hint.as_ref(), &default);
                let reflect_type = plain.hint.as_ref().and_then(|hh| lower_reflect_type(self, hh));
                // The `#[Attr]` list precedes the whole declaration, so every item
                // of a grouped `public $a, $b;` shares it.
                let attributes = self.lower_attributes(&plain.attribute_lists, line)?;
                out.push(PropDecl {
                    doc: doc.clone(),
                    name,
                    visibility,
                    set_visibility: set_visibility_of(plain.modifiers.iter()),
                    default,
                    get_hook: None,
                    set_hook: None,
                    backed: true,
                    readonly,
                    hint,
                    abstract_hooks: Vec::new(),
                    attributes,
                    reflect_type,
                });
            }
        }
        Ok(())
    }

    /// Lower a property's declared type to an enforceable [`TypeHint`], applying the
    /// implicit-nullable rule (`int $x = null` behaves as `?int`, PHP 8.0). `None`
    /// for an untyped property or a type phpr does not enforce (union/`self`/…).
    fn lower_prop_hint(
        &self,
        hint: Option<&Hint>,
        default: &Option<Expr>,
    ) -> Option<TypeHint> {
        let mut h = hint.and_then(|h| lower_hint(self, h))?;
        if matches!(default.as_ref().map(|e| &e.kind), Some(ExprKind::Null)) {
            h.nullable = true;
        }
        Some(h)
    }

    /// Whether an ancestor's declaration of property `name` supplies backing
    /// storage: a plain declaration, or a backed hooked one (its own hooks touch
    /// `$this->name`). A hooked redeclaration of such a property stays *backed*
    /// (PHP 8.4 inheritance) — the by-ref get-hook validation needs this before
    /// `compile_class` flattens the chain. The nearest ancestor declaration
    /// decides; a forward-declared (not yet lowered) parent reports `false`.
    fn ancestor_prop_backed(&self, parent: Option<usize>, name: &[u8]) -> bool {
        let mut c = parent;
        while let Some(ci) = c {
            let Some(cd) = self.classes.get(ci) else { return false };
            if let Some(p) = cd.props.iter().find(|p| p.name.as_ref() == name) {
                return p.backed;
            }
            c = cd.parent;
        }
        false
    }

    /// Lower a property's `{ get … set … }` hook list (PHP 8.4, step 50) into an
    /// optional `get` and `set` hook (each an [`FnDecl`]), plus whether any hook
    /// accesses the property's own backing (`$this-><name>`), which makes the
    /// property backed rather than virtual.
    fn lower_hooks(
        &mut self,
        list: &PropertyHookList,
        prop_name: &[u8],
        line: Line,
    ) -> Result<(Option<FnDecl>, Option<FnDecl>, bool, Vec<Box<[u8]>>), LowerError> {
        let mut get_hook = None;
        let mut set_hook = None;
        let mut backed = false;
        let mut abstract_hooks = Vec::new();
        for hook in list.hooks.iter() {
            // A visibility modifier on the hook itself (`public get;`) is invalid —
            // the hook inherits the property's visibility (PHP 8.4).
            if let Some(m) = hook.modifiers.iter().find(|m| m.is_visibility()) {
                let kw = match m {
                    Modifier::Protected(_) | Modifier::ProtectedSet(_) => "protected",
                    Modifier::Private(_) | Modifier::PrivateSet(_) => "private",
                    _ => "public",
                };
                return Err(LowerError::Fatal {
                    message: format!("Cannot use the {kw} modifier on a property hook"),
                    line,
                });
            }
            // `&get` returns by reference (PHP 8.4): the hook body compiles like
            // a `function &f()`. A by-ref marker on `set` is tolerated (Zend only
            // deprecates the useless by-ref return of a void hook).
            let by_ref = hook.ampersand.is_some();
            let is_set = hook.name.value.eq_ignore_ascii_case(b"set");
            // An abstract hook (`get;` / `set;`) is a contract with no body: record
            // its name for abstract-method enforcement and emit no `FnDecl`. A
            // by-ref abstract get is recorded as `&get` (the variance check reads
            // the marker; consumers building `$prop::get` names strip it).
            if matches!(hook.body, PropertyHookBody::Abstract(_)) {
                abstract_hooks.push(match (is_set, by_ref) {
                    (true, _) => (&b"set"[..]).into(),
                    (false, true) => (&b"&get"[..]).into(),
                    (false, false) => (&b"get"[..]).into(),
                });
                continue;
            }
            let (fd, hook_backed) = self.lower_one_hook(hook, prop_name, is_set, by_ref, line)?;
            backed |= hook_backed;
            if is_set {
                set_hook = Some(fd);
            } else {
                get_hook = Some(fd);
            }
        }
        Ok((get_hook, set_hook, backed, abstract_hooks))
    }

    /// Lower a single property hook body into an [`FnDecl`] in a fresh local scope
    /// (like a method). A `get` hook takes no parameter and returns a value; a
    /// `set` hook takes one parameter (the explicit `set($v)` one, else an
    /// implicit `$value`). The arrow forms desugar: `get => e` → `return e;`,
    /// `set => e` → `$this-><prop> = e;` (a backing write). Returns the hook plus
    /// whether it touched the property's own backing.
    fn lower_one_hook(
        &mut self,
        hook: &PropertyHook,
        prop_name: &[u8],
        is_set: bool,
        by_ref: bool,
        line: Line,
    ) -> Result<(FnDecl, bool), LowerError> {
        // Only a get hook meaningfully returns by reference (`&get`); the body
        // then compiles like a `function &f()` (a `return <lvalue>` yields the
        // place's shared cell). `&set` keeps by-value lowering.
        let by_ref = by_ref && !is_set;
        let body_ast = match &hook.body {
            // `abstract`/interface hooks (`{ get; }`) declare a contract only.
            PropertyHookBody::Abstract(_) => {
                return Err(LowerError::Unsupported { what: "abstract property hook", line })
            }
            PropertyHookBody::Concrete(c) => c,
        };
        // The hook's function name is `$prop::get` / `$prop::set` — the leading
        // `$` is what `__FUNCTION__` / `__METHOD__` and back traces render (PHP
        // 8.4). It is display-only; hook dispatch is keyed by the property name.
        let hook_name: Box<[u8]> = {
            let mut v = vec![b'$'];
            v.extend_from_slice(prop_name);
            v.extend_from_slice(if is_set { b"::set" } else { b"::get" });
            v.into()
        };

        // Fresh local overlay + backing-access tracking for this hook body.
        let saved_locals = self.locals.replace(Scope::default());
        let saved_tag = std::mem::replace(&mut self.after_closing_tag, false);
        let saved_by_ref = std::mem::replace(&mut self.fn_by_ref, by_ref);
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);
        let saved_fn = self.cur_function.replace(hook_name.clone());
        let saved_hook_prop = self.hook_prop.replace(prop_name.into());
        let saved_hook_backed = std::mem::replace(&mut self.hook_backed, false);

        let inner = (|| {
            let params = if is_set {
                match &hook.parameter_list {
                    Some(pl) => {
                        // A set hook's value parameter must be by-value (PHP 8.4,
                        // `zend_compile.c`).
                        if let Some(p) = pl.parameters.iter().find(|p| p.ampersand.is_some()) {
                            return Err(LowerError::Fatal {
                                message: format!(
                                    "Parameter ${} of set hook {}::${} must not be pass-by-reference",
                                    String::from_utf8_lossy(strip_dollar(p.variable.name)),
                                    String::from_utf8_lossy(self.cur_class.as_deref().unwrap_or(b"")),
                                    String::from_utf8_lossy(prop_name),
                                ),
                                line,
                            });
                        }
                        self.lower_params(pl, line)?
                    }
                    None => {
                        // Implicit `$value` parameter.
                        let slot = self.slot_for(b"value");
                        vec![Param { slot, default: None, by_ref: false, variadic: false, hint: None, attributes: Vec::new(), reflect_type: None, promoted: false }]
                    }
                }
            } else {
                // A `get` hook takes no parameters — even an empty `get()` list is
                // a compile error (PHP 8.4, `zend_compile.c`).
                if hook.parameter_list.is_some() {
                    return Err(LowerError::Fatal {
                        message: format!(
                            "get hook of property {}::${} must not have a parameter list",
                            String::from_utf8_lossy(self.cur_class.as_deref().unwrap_or(b"")),
                            String::from_utf8_lossy(prop_name),
                        ),
                        line,
                    });
                }
                Vec::new()
            };
            let body = match body_ast {
                PropertyHookConcreteBody::Block(b) => self.lower_stmts(b.statements.as_slice())?,
                // `&get => <lvalue>` desugars like `return <lvalue>;` in a
                // `function &f()`: a reference to the place (mirrors the
                // `Statement::Return` arm in `lower_stmt`).
                PropertyHookConcreteBody::Expression(e)
                    if !is_set && self.fn_by_ref && is_returnable_lvalue(e.expression) =>
                {
                    vec![Stmt { line, kind: StmtKind::ReturnRef(self.lower_place(e.expression, line)?) }]
                }
                PropertyHookConcreteBody::Expression(e) => {
                    let expr = self.lower_expr(e.expression)?;
                    if is_set {
                        // `set => e` assigns the backing field: `$this-><prop> = e`.
                        self.note_this_prop(prop_name);
                        vec![Stmt {
                            line,
                            kind: StmtKind::Expr(Expr {
                                line,
                                kind: ExprKind::AssignPlace(
                                    Place {
                                        base: PlaceBase::This,
                                        steps: vec![PlaceStep::Prop(prop_name.into())],
                                    },
                                    Box::new(expr),
                                ),
                            }),
                        }]
                    } else {
                        vec![Stmt { line, kind: StmtKind::Return(Some(expr)) }]
                    }
                }
            };
            Ok::<_, LowerError>((params, body))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for hook body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
        self.cur_function = saved_fn;
        self.hook_prop = saved_hook_prop;
        let is_generator = std::mem::replace(&mut self.fn_saw_yield, saved_saw_yield);
        let hook_backed = std::mem::replace(&mut self.hook_backed, saved_hook_backed);

        let (params, body) = inner?;
        validate_goto(&body)?;
        let file = self.unit_file();
        Ok((
            FnDecl {
                name: hook_name,
                // Synthetic (a property hook body) — no docblock of its own.
                doc: None,
                file,
                params,
                body,
                is_generator,
                slots: local_scope.slots,
                by_ref,
                ret_hint: None,
                ret_reflect_type: None,
                defining_class: None,
                closure_shift: 0,
                attributes: Vec::new(),
                line,
                end_line: 0,
            },
            hook_backed,
        ))
    }

    /// Lower one method into a [`MethodDecl`] wrapping an ordinary [`FnDecl`]
    /// (step 19-1, D-19.5). The body is lowered in a fresh local scope just like
    /// a free function; `$this` is read via [`ExprKind::This`], not a slot.
    /// Static and abstract methods are deferred to later sub-steps.
    fn lower_method(
        &mut self,
        method: &Method,
        class_line: Line,
        props: &mut Vec<PropDecl>,
    ) -> Result<MethodDecl, LowerError> {
        let line = self.line_of(method.span());
        let is_static = method.modifiers.iter().any(|m| m.is_static());
        let is_final = method.modifiers.iter().any(|m| m.is_final());
        // An abstract/interface method is a signature with no body: lowered
        // with an empty statement list so its `MethodDecl` (params, defaults,
        // visibility) can back the Reflection surface via `abstract_sigs`.
        let body: &[Statement] = match &method.body {
            MethodBody::Concrete(block) => block.statements.as_slice(),
            MethodBody::Abstract(_) => &[],
        };
        let visibility = visibility_of(method.modifiers.iter());
        let by_ref = method.ampersand.is_some();
        let name: Box<[u8]> = method.name.value.into();

        // Fresh local overlay, like `lower_function` (methods do not capture).
        let saved_locals = self.locals.replace(Scope::default());
        let saved_tag = std::mem::replace(&mut self.after_closing_tag, false);
        let saved_by_ref = std::mem::replace(&mut self.fn_by_ref, by_ref);
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);
        // `cur_class` is already set by `lower_class`; set the method name so
        // `__FUNCTION__`/`__METHOD__` resolve in the body (step 49).
        let saved_fn = self.cur_function.replace(name.clone());

        let inner = (|| {
            let params = self.lower_params(&method.parameter_list, line)?;
            // Constructor property promotion: drain the promoted parameters now
            // (before the body, whose nested param lists would overwrite them),
            // declare each as an instance property, and prepend `$this->p = $p`.
            let promoted = std::mem::take(&mut self.promoted);
            let mut body = self.lower_stmts(body)?;
            if !promoted.is_empty() {
                let mut prologue: Vec<Stmt> = Vec::with_capacity(promoted.len());
                for p in &promoted {
                    prologue.push(Stmt {
                        line,
                        kind: StmtKind::Expr(Expr {
                            line,
                            kind: ExprKind::AssignPlace(
                                Place {
                                    base: PlaceBase::This,
                                    steps: vec![PlaceStep::Prop(p.name.clone())],
                                },
                                Box::new(Expr { line, kind: ExprKind::Var(p.slot) }),
                            ),
                        }),
                    });
                }
                prologue.append(&mut body);
                body = prologue;
            }
            Ok::<_, LowerError>((params, body, promoted))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for method body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
        self.cur_function = saved_fn;
        let is_generator = std::mem::replace(&mut self.fn_saw_yield, saved_saw_yield);

        let (params, body, promoted) = inner?;
        // Surface the promoted properties to the class (declared at the
        // constructor's position among members → correct dump order).
        for p in promoted {
            // A promoted property inherits its constructor parameter's declared
            // type (the param's hint already carries any implicit nullability).
            let pr = params.iter().find(|pr| pr.slot == p.slot);
            let hint = pr.and_then(|pr| pr.hint.clone());
            let reflect_type = pr.and_then(|pr| pr.reflect_type.clone());
            props.push(PropDecl {
                doc: p.doc,
                name: p.name,
                visibility: p.visibility,
                // Promoted params do not carry asymmetric set visibility here
                // (a `private(set)` promotion is not modelled yet).
                set_visibility: None,
                default: None,
                get_hook: p.get_hook,
                set_hook: p.set_hook,
                backed: p.backed,
                readonly: p.readonly,
                hint,
                abstract_hooks: Vec::new(),
                attributes: p.attributes,
                reflect_type,
            });
        }
        validate_goto(&body)?; // step 45: function-scoped goto/label check
        let ret_hint = method
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(self, &r.hint));
        let ret_reflect_type = method
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_reflect_type(self, &r.hint));
        let _ = class_line;
        let attributes = self.lower_attributes(&method.attribute_lists, line)?;
        let file = self.unit_file();
        Ok(MethodDecl {
            visibility,
            is_static,
            is_final,
            decl: FnDecl {
                name,
                doc: self.doc_for(method.span().start.offset),
                file,
                params,
                body,
                is_generator,
                slots: local_scope.slots,
                by_ref,
                ret_hint,
                ret_reflect_type,
                defining_class: None,
                closure_shift: 0,
                attributes,
                line,
                end_line: self.line_of_end(method.span()),
            },
        })
    }

    /// Lower a function body in a *fresh* local slot scope (PHP functions do not
    /// capture the enclosing scope). The outer scope is saved and restored even
    /// on error so the caller's slot table is never corrupted.
    pub(super) fn lower_function(&mut self, func: &Function) -> Result<FnDecl, LowerError> {
        let line = self.line_of(func.span());
        let name: Box<[u8]> = join_ns(&self.cur_namespace, func.name.value);
        let by_ref = func.ampersand.is_some();

        // Install a fresh local overlay; the global scope stays reachable so a
        // global slot can be pre-registered from inside this body (D-12.1).
        // Save/restore the previous overlay so nested lowering nests correctly.
        // `fn_by_ref` steers `return <lvalue>` to ReturnRef while in this body.
        let saved_locals = self.locals.replace(Scope::default());
        let saved_tag = std::mem::replace(&mut self.after_closing_tag, false);
        let saved_by_ref = std::mem::replace(&mut self.fn_by_ref, by_ref);
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);
        // A free function is not a method: `__CLASS__`/`__METHOD__` inside it must
        // not see an enclosing class, so clear `cur_class` too (step 49).
        let saved_fn = self.cur_function.replace(name.clone());
        let saved_cls = self.cur_class.take();

        let inner = self.lower_function_body(func, line);

        // Reclaim the function's local scope and restore the outer one.
        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for function body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
        self.cur_function = saved_fn;
        self.cur_class = saved_cls;
        let is_generator = std::mem::replace(&mut self.fn_saw_yield, saved_saw_yield);

        let (params, body) = inner?;
        validate_goto(&body)?; // step 45: function-scoped goto/label check
        let ret_hint = func
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(self, &r.hint));
        let ret_reflect_type = func
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_reflect_type(self, &r.hint));
        let attributes = self.lower_attributes(&func.attribute_lists, line)?;
        let file = self.unit_file();
        Ok(FnDecl {
            name,
            doc: self.doc_for(func.span().start.offset),
            file,
            params,
            body,
            is_generator,
            slots: local_scope.slots,
            by_ref,
            ret_hint,
            ret_reflect_type,
            defining_class: None,
                closure_shift: 0,
            attributes,
            line,
            end_line: self.line_of_end(func.span()),
        })
    }

    /// Substitute a magic constant to a literal from the current lexical scope
    /// (step 49). PHP resolves these at compile time, so no runtime support is
    /// needed. `__NAMESPACE__` is always `""` (Tier 1 has no namespaces) and
    /// `__PROPERTY__` (property hooks, unsupported) is also `""`.
    pub(super) fn lower_magic_constant(&self, m: &MagicConstant, line: Line) -> ExprKind {
        let s = |b: &[u8]| ExprKind::Str(b.to_vec().into_boxed_slice());
        let cls = self.cur_class.as_deref().unwrap_or(b"");
        let func = self.cur_function.as_deref().unwrap_or(b"");
        match m {
            MagicConstant::Line(_) => ExprKind::Int(line as i64),
            MagicConstant::File(_) => s(&self.prog_name),
            MagicConstant::Directory(_) => s(dirname(&self.prog_name)),
            MagicConstant::Class(_) => s(cls),
            MagicConstant::Function(_) => s(func),
            // PHP: `Class::method` inside a method, the bare function name inside
            // a free function, and `""` at the top level.
            MagicConstant::Method(_) => match (&self.cur_function, &self.cur_class) {
                (None, _) => s(b""),
                // Inside a closure body `__METHOD__` is the synthetic
                // `{closure:…}` name itself, with no class prefix (PHP 8.4+).
                (Some(f), _) if f.starts_with(b"{") => s(func),
                (Some(_), Some(_)) => {
                    let mut v = cls.to_vec();
                    v.extend_from_slice(b"::");
                    v.extend_from_slice(func);
                    ExprKind::Str(v.into_boxed_slice())
                }
                (Some(_), None) => s(func),
            },
            MagicConstant::Trait(_) => s(self.cur_trait.as_deref().unwrap_or(b"")),
            // `__NAMESPACE__` is the current namespace name without separators
            // (`""` in the global namespace), resolved at compile time (step 50).
            MagicConstant::Namespace(_) => s(&self.cur_namespace),
            MagicConstant::Property(_) => s(b""),
        }
    }

    /// Bind parameters into the leading slots of the (already fresh) scope, then
    /// lower the body. By-reference / variadic / promoted-property params are
    /// outside step 8 scope; type hints are accepted but not enforced.
    fn lower_function_body(
        &mut self,
        func: &Function,
        line: Line,
    ) -> Result<(Vec<Param>, Vec<Stmt>), LowerError> {
        let params = self.lower_params(&func.parameter_list, line)?;
        let body = self.lower_stmts(func.body.statements.as_slice())?;
        Ok((params, body))
    }

    /// Lower a parameter list into the leading slots of the active scope. Shared
    /// by named functions and closures (step 18). By-reference / variadic /
    /// promoted-property params follow the same rules as `lower_function_body`.
    fn lower_params(
        &mut self,
        plist: &FunctionLikeParameterList,
        _line: Line,
    ) -> Result<Vec<Param>, LowerError> {
        // Reset the promoted-parameter accumulator for this parameter list; the
        // owning `__construct` drains it right after this call (property promotion).
        self.promoted.clear();
        let mut params = Vec::new();
        for p in plist.parameters.iter() {
            let by_ref = p.ampersand.is_some();
            let variadic = p.ellipsis.is_some();
            let slot = self.slot_for(strip_dollar(p.variable.name));
            // The `#[Attr]` list on the parameter is shared by the parameter itself
            // (ReflectionParameter) and, if promoted, the property it declares.
            let attributes = self.lower_attributes(&p.attribute_lists, _line)?;
            if p.is_promoted_property() {
                // `public int $x` in a constructor: still a real parameter, but it
                // also declares an instance property assigned from the param. The
                // promoted property may itself carry hooks (`public $x { get … }`).
                let pname: Box<[u8]> = strip_dollar(p.variable.name).into();
                // A promoted constructor property cannot be abstract (the class is
                // instantiable), so any abstract-hook list is ignored here.
                let (get_hook, set_hook, backed, _abstract) = match &p.hooks {
                    Some(list) => self.lower_hooks(list, &pname, _line)?,
                    None => (None, None, true, Vec::new()),
                };
                self.promoted.push(PromotedParam {
                    name: pname,
                    visibility: visibility_of(p.modifiers.iter()),
                    slot,
                    get_hook,
                    set_hook,
                    backed,
                    readonly: p.modifiers.iter().any(|m| m.is_readonly()),
                    attributes: attributes.clone(),
                    // The promoted property inherits the parameter's doc block.
                    doc: self.doc_for(p.span().start.offset),
                });
            }
            let default = match &p.default_value {
                Some(d) => Some(self.lower_expr(d.value)?),
                None => None,
            };
            // A typed parameter with a literal `null` default is implicitly
            // nullable (`T $x = null` behaves as `?T $x = null`): it accepts null
            // both at the binder and via Reflection's `allowsNull()` (PHP 8.0+).
            let mut hint = p.hint.as_ref().and_then(|h| lower_hint(self, h));
            if let Some(h) = &mut hint {
                if matches!(default.as_ref().map(|e| &e.kind), Some(ExprKind::Null)) {
                    h.nullable = true;
                }
            }
            let reflect_type = p.hint.as_ref().and_then(|h| lower_reflect_type(self, h));
            params.push(Param {
                slot,
                default,
                by_ref,
                variadic,
                hint,
                attributes,
                reflect_type,
                promoted: p.is_promoted_property(),
            });
        }
        Ok(params)
    }

    /// Lower an anonymous function `function (params) use (...) { body }` into a
    /// flat-table entry plus an [`ExprKind::Closure`] that captures the `use`
    /// variables (step 18, D-18.2/D-18.3). Captures are resolved in the enclosing
    /// scope *before* the fresh closure scope is installed; the closure body then
    /// runs in its own slot frame like a function (no implicit capture).
    pub(super) fn lower_closure(&mut self, closure: &Closure, line: Line) -> Result<ExprKind, LowerError> {
        // A `static function(){}` does not bind `$this` (step 19-6, D-19.19).
        let bind_this = closure.r#static.is_none();
        let by_ref = closure.ampersand.is_some();

        // 1. Resolve each `use` variable's slot in the *enclosing* scope (the one
        //    active now), recording whether it is captured by value or reference.
        let mut use_specs: Vec<(Box<[u8]>, Slot, bool)> = Vec::new();
        if let Some(use_clause) = &closure.use_clause {
            for u in use_clause.variables.iter() {
                let name = strip_dollar(u.variable.name);
                let src = self.slot_for(name);
                use_specs.push((name.into(), src, u.ampersand.is_some()));
            }
        }

        // 2. Install a fresh local scope and lower params + use-vars + body in it.
        let saved_locals = self.locals.replace(Scope::default());
        let saved_tag = std::mem::replace(&mut self.after_closing_tag, false);
        let saved_by_ref = std::mem::replace(&mut self.fn_by_ref, by_ref);
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);
        // `__FUNCTION__`/`__METHOD__` inside a closure are its own synthetic
        // `{closure:…}` name (PHP 8.4+); the lexical class (for `__CLASS__`) is
        // inherited from the enclosing scope (step 49).
        let cname = self.closure_name(line);
        let saved_fn = self.cur_function.replace(cname.clone());

        let inner = (|| -> Result<LoweredClosure, LowerError> {
            let params = self.lower_params(&closure.parameter_list, line)?;
            let mut captures = Vec::with_capacity(use_specs.len());
            for (name, src, cap_by_ref) in &use_specs {
                let dst = self.slot_for(name);
                captures.push(Capture {
                    src: *src,
                    dst,
                    by_ref: *cap_by_ref,
                });
            }
            let body = self.lower_stmts(closure.body.statements.as_slice())?;
            Ok((params, captures, body))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for closure body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
        self.cur_function = saved_fn;
        let is_generator = std::mem::replace(&mut self.fn_saw_yield, saved_saw_yield);

        let (params, captures, body) = inner?;
        validate_goto(&body)?; // step 45: function-scoped goto/label check
        let ret_hint = closure
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(self, &r.hint));
        let ret_reflect_type = closure
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_reflect_type(self, &r.hint));
        let attributes = self.lower_attributes(&closure.attribute_lists, line)?;
        let fn_idx =
            self.push_closure(cname, params, body, local_scope.slots, by_ref, ret_hint, ret_reflect_type, is_generator, attributes, line);
        Ok(ExprKind::Closure {
            fn_idx,
            captures,
            bind_this,
        })
    }

    /// Zend's synthetic anonymous-function name (PHP 8.4 format): `{closure:` +
    /// the enclosing scope + `:line}` (the closure's own line). The scope is
    /// `Class::method()` / `func()` inside a named callable, an enclosing
    /// closure's own synthetic name verbatim (no `()`), or the program file at
    /// top level. Computed BEFORE the body is lowered — it doubles as the
    /// body's `__FUNCTION__`/`__METHOD__` (step 49; PHP 8.4 naming).
    fn closure_name(&self, line: Line) -> Box<[u8]> {
        let scope: Vec<u8> = match &self.cur_function {
            Some(f) if f.starts_with(b"{") => f.to_vec(),
            Some(f) => {
                let mut v = match &self.cur_class {
                    Some(c) => {
                        let mut v = c.to_vec();
                        v.extend_from_slice(b"::");
                        v
                    }
                    None => Vec::new(),
                };
                v.extend_from_slice(f);
                v.extend_from_slice(b"()");
                v
            }
            None => self.prog_name.to_vec(),
        };
        format!("{{closure:{}:{}}}", String::from_utf8_lossy(&scope), line)
            .into_bytes()
            .into_boxed_slice()
    }

    /// Append a lowered closure body to the flat table and return its index. The
    /// `FnDecl.name` is the PHP `{closure:scope:line}` synthetic name built by
    /// [`Self::closure_name`] before the body was lowered (step 18).
    #[allow(clippy::too_many_arguments)]
    fn push_closure(
        &mut self,
        name: Box<[u8]>,
        params: Vec<Param>,
        body: Vec<Stmt>,
        slots: Vec<Box<[u8]>>,
        by_ref: bool,
        ret_hint: Option<TypeHint>,
        ret_reflect_type: Option<crate::hir::ReflectType>,
        is_generator: bool,
        attributes: Vec<crate::hir::HirAttribute>,
        line: Line,
    ) -> usize {
        let idx = self.closures.len();
        let file = self.unit_file();
        self.closures.push(FnDecl {
            name,
            // Closures keep no docblock (rarely reflected; "none" is honest).
            doc: None,
            file,
            params,
            body,
            is_generator,
            slots,
            by_ref,
            ret_hint,
            ret_reflect_type,
            // A closure/arrow inherits the lexically enclosing class so its body
            // can use `self::`/`parent::`/`new self` (resolved at compile time).
            defining_class: self.cur_class.clone(),
                closure_shift: 0,
            attributes,
            line,
            // A closure/arrow's end line is not tracked; fall back to the op-line
            // span in the descriptor.
            end_line: 0,
        });
        idx
    }

    /// Lower an arrow function `fn (params) => expr` (step 18, D-18.4). Unlike a
    /// `function`, an arrow *implicitly* captures by value every enclosing-scope
    /// variable used in its body. Free variables are discovered by walking the
    /// body AST and keeping those that already have a slot in the enclosing scope
    /// (excluding the arrow's own parameters); the body lowers to `return expr`.
    pub(super) fn lower_arrow_function(
        &mut self,
        af: &ArrowFunction,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        // `static fn`: the only semantic is "do not bind $this" — a well-formed
        // static closure never touches $this, so lowering it like a plain arrow
        // is observationally identical for them. Residue: PHP raises "Using
        // $this when not in object context" if a static closure DOES use $this,
        // and Closure::bind() on one fails; phpr does not enforce either yet.
        // `fn &() => expr` returns by reference. The arrow still captures by
        // value, so a ref into a captured variable points at the closure's own
        // copy (Zend does the same); a ref through `$this->prop` is a real one.
        let by_ref = af.ampersand.is_some();

        // Collect free variables of the body, then keep those that name an
        // enclosing-scope variable and are not the arrow's own parameters.
        let mut param_names: HashMap<&[u8], ()> = HashMap::new();
        for p in af.parameter_list.parameters.iter() {
            param_names.insert(strip_dollar(p.variable.name), ());
        }
        let mut used: Vec<&[u8]> = Vec::new();
        collect_direct_vars(af.expression, &mut used);
        let mut use_specs: Vec<(Box<[u8]>, Slot)> = Vec::new();
        for raw in used {
            let name = strip_dollar(raw);
            if param_names.contains_key(name) {
                continue;
            }
            if let Some(src) = self.enclosing_slot(name) {
                use_specs.push((name.into(), src));
            }
        }

        let saved_locals = self.locals.replace(Scope::default());
        let saved_tag = std::mem::replace(&mut self.after_closing_tag, false);
        let saved_by_ref = std::mem::replace(&mut self.fn_by_ref, by_ref);
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);
        // Same as a closure: `__FUNCTION__` is the synthetic name, class is
        // inherited.
        let cname = self.closure_name(line);
        let saved_fn = self.cur_function.replace(cname.clone());

        let inner = (|| -> Result<LoweredClosure, LowerError> {
            let params = self.lower_params(&af.parameter_list, line)?;
            let mut captures = Vec::with_capacity(use_specs.len());
            for (name, src) in &use_specs {
                let dst = self.slot_for(name);
                captures.push(Capture {
                    src: *src,
                    dst,
                    by_ref: false,
                });
            }
            // Inside `fn &() => <lvalue>` the body returns a reference to the
            // place, mirroring the `Statement::Return` arm in `lower_stmt`.
            let body = if self.fn_by_ref && is_returnable_lvalue(af.expression) {
                vec![Stmt {
                    line,
                    kind: StmtKind::ReturnRef(self.lower_place(af.expression, line)?),
                }]
            } else {
                vec![Stmt {
                    line,
                    kind: StmtKind::Return(Some(self.lower_expr(af.expression)?)),
                }]
            };
            Ok((params, captures, body))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for arrow body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
        self.cur_function = saved_fn;
        let is_generator = std::mem::replace(&mut self.fn_saw_yield, saved_saw_yield);

        let (params, captures, body) = inner?;
        let ret_hint = af
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(self, &r.hint));
        let ret_reflect_type = af
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_reflect_type(self, &r.hint));
        let fn_idx =
            self.push_closure(cname, params, body, local_scope.slots, by_ref, ret_hint, ret_reflect_type, is_generator, Vec::new(), line);
        // An arrow function is never `static` here (see comment above), so it
        // binds `$this` like an ordinary closure (step 19-6).
        Ok(ExprKind::Closure {
            fn_idx,
            captures,
            bind_this: true,
        })
    }
}
