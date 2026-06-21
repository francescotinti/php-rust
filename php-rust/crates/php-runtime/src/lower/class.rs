//! HIR lowering of classes, interfaces, traits, enums, methods, properties, closures and arrow functions. Split out of `lower.rs` (step 61).
use std::collections::{HashMap, HashSet};

use mago_span::HasSpan;
use mago_syntax::ast::{
    ArrowFunction,
    Class, ClassLikeMember, Closure, Enum, EnumCaseItem, Function,
    FunctionLikeParameterList, Hint, Interface, MagicConstant, Method,
    MethodBody, Property, PropertyItem, Statement, Trait, TraitUse, TraitUseAdaptation, TraitUseMethodReference,
    TraitUseSpecification,
};

use crate::hir::{
    Capture, ClassDecl, ExprKind, FnDecl, Line, MethodDecl, Param,
    PropDecl, Slot, Stmt, StmtKind, TypeHint,
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
    fn resolve_trait(
        &mut self,
        key: &[u8],
        asts: &HashMap<Vec<u8>, &Trait>,
        in_progress: &mut HashSet<Vec<u8>>,
    ) -> Result<(), LowerError> {
        if self.traits.contains_key(key) {
            return Ok(());
        }
        let t = *asts.get(key).ok_or(LowerError::Unsupported {
            what: "use of undefined trait",
            line: 0,
        })?;
        let line = self.line_of(t.span());
        if !in_progress.insert(key.to_vec()) {
            return Err(LowerError::Unsupported {
                what: "circular trait use",
                line,
            });
        }
        let mut methods = Vec::new();
        let mut props = Vec::new();
        let mut static_props = Vec::new();
        let mut consts = Vec::new();
        let mut abstract_methods: Vec<Box<[u8]>> = Vec::new();
        let mut uses: Vec<&TraitUse> = Vec::new();
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
                ClassLikeMember::Method(m) => methods.push(self.lower_method(m, line)?),
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts)?,
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
        // Resolve any nested traits before flattening their members in.
        for u in &uses {
            for tn in u.trait_names.iter() {
                let nk = function_name(tn).to_ascii_lowercase();
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
        self.traits.insert(
            key.to_vec(),
            LoweredTrait {
                methods,
                props,
                static_props,
                consts,
                abstract_methods,
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
        &self,
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
                                    .insert((function_name(loser).to_ascii_lowercase(), m_lc.clone()));
                            }
                        }
                        TraitUseAdaptation::Alias(a) => {
                            let (trait_lc, method_lc) = match &a.method_reference {
                                TraitUseMethodReference::Absolute(abs) => (
                                    Some(function_name(&abs.trait_name).to_ascii_lowercase()),
                                    abs.method_name.value.to_ascii_lowercase(),
                                ),
                                TraitUseMethodReference::Identifier(id) => {
                                    (None, id.value.to_ascii_lowercase())
                                }
                            };
                            aliases.push(Alias {
                                trait_lc,
                                method_lc,
                                alias: a.alias.as_ref().map(|id| id.value.into()),
                                vis: a.visibility.as_ref().map(visibility_of_modifier),
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
                let tkey = function_name(tn).to_ascii_lowercase();
                let torig: Box<[u8]> = function_name(tn).into();
                let lt = self.traits.get(&tkey).ok_or(LowerError::Unsupported {
                    what: "use of undefined trait",
                    line,
                })?;
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
                    methods.push(m.clone());
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
            let mut src = src.ok_or(LowerError::Unsupported {
                what: "trait alias of unknown method",
                line,
            })?;
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
                let tkey = function_name(tn).to_ascii_lowercase();
                if let Some(found) = self.traits.get(&tkey).and_then(pick) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Resolve a list of interface names (`implements`/interface `extends`) to
    /// their class ids (step 19-5). Unknown interfaces are out of scope.
    fn resolve_interfaces(&self, names: &[&[u8]], line: Line) -> Result<Vec<usize>, LowerError> {
        let mut out = Vec::new();
        for n in names {
            match self.class_index.get(&n.to_ascii_lowercase()) {
                Some(&i) => out.push(i),
                None => {
                    return Err(LowerError::Unsupported {
                        what: "implements/extends undefined interface",
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
                let names: Vec<&[u8]> = ext.types.iter().map(function_name).collect();
                self.resolve_interfaces(&names, line)?
            }
            None => Vec::new(),
        };
        let mut consts = Vec::new();
        let mut abstract_methods = Vec::new();
        for member in iface.members.iter() {
            match member {
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts)?,
                // Interface methods are signatures only (abstract) — no body to
                // run, but their names are reported by `get_class_methods`.
                ClassLikeMember::Method(m) => abstract_methods.push(m.name.value.into()),
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "interface member",
                        line,
                    })
                }
            }
        }
        Ok(ClassDecl {
            name: iface.name.value.into(),
            parent: None,
            interfaces,
            is_abstract: true,
            is_interface: true,
            props: Vec::new(),
            static_props: Vec::new(),
            consts,
            methods: Vec::new(),
            abstract_methods,
            is_enum: false,
            enum_backing: None,
            enum_cases: Vec::new(),
            line,
        })
    }

    /// Lower one `class Name { ... }` into a [`ClassDecl`] (step 19-1). Only
    /// instance properties and methods are in 19-1 scope; `extends`/`implements`,
    /// static members, constants, and other member kinds arrive in later
    /// sub-steps and lower to [`LowerError::Unsupported`] for now.
    pub(super) fn lower_class(&mut self, class: &Class) -> Result<ClassDecl, LowerError> {
        let line = self.line_of(class.span());
        // Resolve `extends ParentName` to the parent's class id (registered in
        // pass 1 of `hoist_classes`, so forward references work, D-19.10).
        let parent = match &class.extends {
            Some(ext) => {
                let pname = parent_name(ext, line)?;
                match self.class_index.get(&pname.to_ascii_lowercase()) {
                    Some(&i) => Some(i),
                    None => {
                        return Err(LowerError::Unsupported {
                            what: "extends undefined class",
                            line,
                        })
                    }
                }
            }
            None => None,
        };
        // Resolve `implements I, J` to interface ids (step 19-5).
        let interfaces = match &class.implements {
            Some(imp) => {
                let names: Vec<&[u8]> = imp.types.iter().map(function_name).collect();
                self.resolve_interfaces(&names, line)?
            }
            None => Vec::new(),
        };
        let is_abstract = class.modifiers.iter().any(|m| m.is_abstract());
        let name: Box<[u8]> = class.name.value.into();
        let mut props = Vec::new();
        let mut static_props = Vec::new();
        let mut consts = Vec::new();
        let mut methods = Vec::new();
        let mut uses: Vec<&TraitUse> = Vec::new();
        // Names of abstract methods this class must implement (its own, plus any
        // pulled in from traits during flattening), step 21-4 / D-21.11.
        let mut abstract_req: Vec<Box<[u8]>> = Vec::new();
        // `__CLASS__`/`__METHOD__` in any method body resolve to this class name
        // (step 49). Restored after the member loop; an early error here aborts
        // the whole lowering, so the leak-on-error path is harmless.
        let saved_cls = self.cur_class.replace(name.clone());
        for member in class.members.iter() {
            match member {
                ClassLikeMember::Property(p) => {
                    self.lower_property(p, &mut props, &mut static_props, line)?
                }
                // An abstract method is a signature only — no body to run. A
                // concrete subclass / consumer must supply the implementation.
                ClassLikeMember::Method(m) if matches!(m.body, MethodBody::Abstract(_)) => {
                    abstract_req.push(m.name.value.into())
                }
                ClassLikeMember::Method(m) => methods.push(self.lower_method(m, line)?),
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts)?,
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
        // A concrete class must implement every abstract method it carries (own or
        // trait-supplied); otherwise PHP fatals at link time (D-21.11). Abstract
        // classes and interfaces legitimately leave them open.
        if !is_abstract {
            let mut missing: Vec<&[u8]> = Vec::new();
            for req in &abstract_req {
                let req_lc = req.to_ascii_lowercase();
                if methods.iter().any(|m| m.decl.name.to_ascii_lowercase() == req_lc) {
                    continue;
                }
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
            })
            .cloned()
            .collect();
        Ok(ClassDecl {
            name,
            parent,
            interfaces,
            is_abstract,
            is_interface: false,
            props,
            static_props,
            consts,
            methods,
            abstract_methods,
            is_enum: false,
            enum_backing: None,
            enum_cases: Vec::new(),
            line,
        })
    }

    /// Lower one `enum E [: int|string] { case ...; methods; consts }` into a
    /// [`ClassDecl`] with `is_enum = true` (step 23, D-23.1). Cases go to
    /// `enum_cases`; methods/constants/trait-uses reuse the class machinery.
    /// Every enum implements `UnitEnum` (backed ones also `BackedEnum`, D-23.7).
    pub(super) fn lower_enum(&mut self, en: &Enum) -> Result<ClassDecl, LowerError> {
        let line = self.line_of(en.span());
        let name: Box<[u8]> = en.name.value.into();
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
        // Marker interfaces (D-23.7) + any user `implements`.
        let mut iface_names: Vec<&[u8]> = vec![b"UnitEnum"];
        if enum_backing.is_some() {
            iface_names.push(b"BackedEnum");
        }
        if let Some(imp) = &en.implements {
            iface_names.extend(imp.types.iter().map(function_name));
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
                ClassLikeMember::Method(m) => methods.push(self.lower_method(m, line)?),
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts)?,
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
        Ok(ClassDecl {
            name,
            parent: None,
            interfaces,
            is_abstract: false,
            is_interface: false,
            props: Vec::new(),
            static_props: Vec::new(),
            consts,
            methods,
            // Enums implement any interface methods concretely, so none are left
            // abstract (step 47).
            abstract_methods: Vec::new(),
            is_enum: true,
            enum_backing,
            enum_cases,
            line,
        })
    }

    /// Lower a `const A = 1, B = 2;` declaration into one [`ClassConstDecl`] per
    /// item (step 19-4). Visibility modifiers (7.1+) are accepted but not
    /// enforced.
    fn lower_class_const(
        &mut self,
        konst: &mago_syntax::ast::ClassLikeConstant,
        out: &mut Vec<crate::hir::ClassConstDecl>,
    ) -> Result<(), LowerError> {
        for item in konst.items.iter() {
            out.push(crate::hir::ClassConstDecl {
                name: item.name.value.into(),
                value: self.lower_expr(item.value)?,
            });
        }
        Ok(())
    }

    /// Lower a property declaration into one entry per item (`public $a = 1, $b;`),
    /// routing `static` properties to `static_out` and instance properties to
    /// `out` (step 19-1/19-4). Hooked properties remain out of scope.
    fn lower_property(
        &mut self,
        prop: &Property,
        out: &mut Vec<PropDecl>,
        static_out: &mut Vec<crate::hir::StaticPropDecl>,
        line: Line,
    ) -> Result<(), LowerError> {
        let plain = match prop {
            Property::Plain(p) => p,
            Property::Hooked(_) => {
                return Err(LowerError::Unsupported {
                    what: "property hooks",
                    line,
                })
            }
        };
        let is_static = plain.modifiers.iter().any(|m| m.is_static());
        let visibility = visibility_of(plain.modifiers.iter());
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
                out.push(PropDecl {
                    name,
                    visibility,
                    default,
                });
            }
        }
        Ok(())
    }

    /// Lower one method into a [`MethodDecl`] wrapping an ordinary [`FnDecl`]
    /// (step 19-1, D-19.5). The body is lowered in a fresh local scope just like
    /// a free function; `$this` is read via [`ExprKind::This`], not a slot.
    /// Static and abstract methods are deferred to later sub-steps.
    fn lower_method(&mut self, method: &Method, class_line: Line) -> Result<MethodDecl, LowerError> {
        let line = self.line_of(method.span());
        let is_static = method.modifiers.iter().any(|m| m.is_static());
        let body = match &method.body {
            MethodBody::Concrete(block) => block,
            MethodBody::Abstract(_) => {
                return Err(LowerError::Unsupported {
                    what: "abstract method",
                    line,
                })
            }
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
            let body = self.lower_stmts(body.statements.as_slice())?;
            Ok::<_, LowerError>((params, body))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for method body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
        self.cur_function = saved_fn;
        let is_generator = std::mem::replace(&mut self.fn_saw_yield, saved_saw_yield);

        let (params, body) = inner?;
        validate_goto(&body)?; // step 45: function-scoped goto/label check
        let ret_hint = method
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(&r.hint));
        let _ = class_line;
        Ok(MethodDecl {
            visibility,
            is_static,
            decl: FnDecl {
                name,
                params,
                body,
                is_generator,
                slots: local_scope.slots,
                by_ref,
                ret_hint,
                line,
            },
        })
    }

    /// Lower a function body in a *fresh* local slot scope (PHP functions do not
    /// capture the enclosing scope). The outer scope is saved and restored even
    /// on error so the caller's slot table is never corrupted.
    pub(super) fn lower_function(&mut self, func: &Function) -> Result<FnDecl, LowerError> {
        let line = self.line_of(func.span());
        let name: Box<[u8]> = func.name.value.into();
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
            .and_then(|r| lower_hint(&r.hint));
        Ok(FnDecl {
            name,
            params,
            body,
            is_generator,
            slots: local_scope.slots,
            by_ref,
            ret_hint,
            line,
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
                (Some(_), Some(_)) => {
                    let mut v = cls.to_vec();
                    v.extend_from_slice(b"::");
                    v.extend_from_slice(func);
                    ExprKind::Str(v.into_boxed_slice())
                }
                (Some(_), None) => s(func),
            },
            MagicConstant::Trait(_) => s(self.cur_trait.as_deref().unwrap_or(b"")),
            MagicConstant::Namespace(_) => s(b""),
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
        line: Line,
    ) -> Result<Vec<Param>, LowerError> {
        let mut params = Vec::new();
        for p in plist.parameters.iter() {
            let by_ref = p.ampersand.is_some();
            let variadic = p.ellipsis.is_some();
            if p.is_promoted_property() {
                return Err(LowerError::Unsupported {
                    what: "promoted constructor property",
                    line,
                });
            }
            let slot = self.slot_for(strip_dollar(p.variable.name));
            let default = match &p.default_value {
                Some(d) => Some(self.lower_expr(d.value)?),
                None => None,
            };
            params.push(Param {
                slot,
                default,
                by_ref,
                variadic,
                hint: p.hint.as_ref().and_then(lower_hint),
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
        // `__FUNCTION__` inside a closure is PHP's `{closure}`; the lexical class
        // (for `__CLASS__`) is inherited from the enclosing scope (step 49).
        let saved_fn = self.cur_function.replace((*b"{closure}").into());

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
            .and_then(|r| lower_hint(&r.hint));
        let fn_idx =
            self.push_closure(params, body, local_scope.slots, by_ref, ret_hint, is_generator, line);
        Ok(ExprKind::Closure {
            fn_idx,
            captures,
            bind_this,
        })
    }

    /// Append a lowered closure body to the flat table and return its index. The
    /// `FnDecl.name` is the PHP `{closure:file:line}` synthetic name (step 18).
    #[allow(clippy::too_many_arguments)]
    fn push_closure(
        &mut self,
        params: Vec<Param>,
        body: Vec<Stmt>,
        slots: Vec<Box<[u8]>>,
        by_ref: bool,
        ret_hint: Option<TypeHint>,
        is_generator: bool,
        line: Line,
    ) -> usize {
        let name = format!(
            "{{closure:{}:{}}}",
            String::from_utf8_lossy(&self.prog_name),
            line
        )
        .into_bytes()
        .into_boxed_slice();
        let idx = self.closures.len();
        self.closures.push(FnDecl {
            name,
            params,
            body,
            is_generator,
            slots,
            by_ref,
            ret_hint,
            line,
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
        if af.r#static.is_some() {
            return Err(LowerError::Unsupported {
                what: "static arrow function",
                line,
            });
        }
        if af.ampersand.is_some() {
            return Err(LowerError::Unsupported {
                what: "by-reference arrow function",
                line,
            });
        }

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
        let saved_by_ref = std::mem::replace(&mut self.fn_by_ref, false);
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);
        // Same as a closure: `__FUNCTION__` is `{closure}`, class is inherited.
        let saved_fn = self.cur_function.replace((*b"{closure}").into());

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
            let body_expr = self.lower_expr(af.expression)?;
            let body = vec![Stmt {
                line,
                kind: StmtKind::Return(Some(body_expr)),
            }];
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
            .and_then(|r| lower_hint(&r.hint));
        let fn_idx =
            self.push_closure(params, body, local_scope.slots, false, ret_hint, is_generator, line);
        // An arrow function is never `static` here (rejected above), so it binds
        // `$this` like an ordinary closure (step 19-6).
        Ok(ExprKind::Closure {
            fn_idx,
            captures,
            bind_this: true,
        })
    }
}
