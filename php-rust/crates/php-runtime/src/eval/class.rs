//! Objects, classes and enums: instantiation (`eval_new`), class/interface
//! resolution, class constants, enum cases, property access and visibility,
//! magic methods, method/static dispatch (`call_method`/`call_static`/
//! `invoke_method`) and destructors. Split out of `eval.rs` (step 60).
#![allow(unused_imports)]
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use php_types::{
    convert, dtoa, numstr, ops, Closure, ClosureInfo, ClosureParam, ClosureRender, Diag, Diags,
    DirHandle, GenDriver, GenKey, GenState, GenStatus, GenStep, Key, Object, ObjectInfo, PhpArray,
    PhpError, PhpStr, PropVis, Props, ResKind, Resource, Stream, StreamBackend, Zval,
};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::hir::{
    BinOp, Capture, CastKind, ClassDecl, ClassId, ClassRef, Expr, ExprKind, FnDecl, Line,
    MethodDecl, Param, Place, PlaceBase, PlaceStep, Program, ScalarType, Slot, StaticAssignOp,
    Stmt, StmtKind, TypeHint, UnOp, Visibility,
};

use super::*;

impl<'p> Evaluator<'p> {
    // --- objects (step 19) ---

    /// Evaluate a call's arguments by value into a flat positional vector plus
    /// the named arguments produced by unpacking string keys (step 40). Shared
    /// by method calls and constructor dispatch. Methods take all positional
    /// arguments by value, so unpacking has no by-reference subtlety here.
    pub(super) fn eval_value_args(
        &mut self,
        args: &[Expr],
    ) -> Result<(Vec<Zval>, SpreadNamed), PhpError> {
        let mut out = Vec::with_capacity(args.len());
        let mut named: SpreadNamed = Vec::new();
        for a in args {
            if let ExprKind::Spread(inner) = &a.kind {
                self.expand_spread(inner, &mut out, &mut named)?;
            } else {
                out.push(self.eval(a)?);
            }
        }
        Ok((out, named))
    }

    /// Resolve a method by name, walking the inheritance chain child→ancestor
    /// (step 19-3, D-19.10). Returns the *defining* class id and the method.
    pub(super) fn resolve_method(&self, start: ClassId, name: &[u8]) -> Option<(ClassId, &'p MethodDecl)> {
        let classes: &'p [ClassDecl] = self.classes;
        let mut cid = Some(start);
        while let Some(c) = cid {
            if let Some(m) = classes[c]
                .methods
                .iter()
                .find(|m| m.decl.name.eq_ignore_ascii_case(name))
            {
                return Some((c, m));
            }
            cid = classes[c].parent;
        }
        None
    }

    /// Assemble an instance's property map: parent declarations first, then each
    /// subclass, so the layout order is root→leaf and a redeclared property keeps
    /// its inherited position with the subclass's default (step 19-3, D-19.10).
    pub(super) fn collect_props(&mut self, cid: ClassId) -> Result<Props, PhpError> {
        let classes: &'p [ClassDecl] = self.classes;
        let mut chain = Vec::new();
        let mut c = Some(cid);
        while let Some(x) = c {
            chain.push(x);
            c = classes[x].parent;
        }
        chain.reverse();
        let mut props = Props::new();
        for &x in &chain {
            for p in &classes[x].props {
                let v = match &p.default {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                props.set(&p.name, v);
            }
        }
        Ok(props)
    }

    /// Build (and cache) a class's property-visibility shape for object dumping
    /// (step 19-7, D-19.20): declared properties in root→leaf order, a redeclared
    /// property taking the most-derived visibility.
    pub(super) fn class_shape(&mut self, cid: ClassId) -> Rc<ObjectInfo> {
        if let Some(s) = self.class_shapes.get(&cid) {
            return Rc::clone(s);
        }
        let classes: &'p [ClassDecl] = self.classes;
        let mut chain = Vec::new();
        let mut c = Some(cid);
        while let Some(x) = c {
            chain.push(x);
            c = classes[x].parent;
        }
        chain.reverse();
        let mut entries: Vec<(Box<[u8]>, PropVis)> = Vec::new();
        for &x in &chain {
            let cname = PhpStr::new(classes[x].name.to_vec());
            for p in &classes[x].props {
                let vis = match p.visibility {
                    Visibility::Public => PropVis::Public,
                    Visibility::Protected => PropVis::Protected,
                    Visibility::Private => PropVis::Private(Rc::clone(&cname)),
                };
                match entries.iter_mut().find(|(k, _)| k.as_ref() == p.name.as_ref()) {
                    Some(e) => e.1 = vis,
                    None => entries.push((p.name.clone(), vis)),
                }
            }
        }
        let info = Rc::new(ObjectInfo::from_entries(entries));
        self.class_shapes.insert(cid, Rc::clone(&info));
        info
    }

    /// The visibility and *declaring* class of a declared property, found by
    /// walking the chain child→ancestor. `None` for a dynamic/undeclared property
    /// (which is effectively public), step 19-3, D-19.13.
    pub(super) fn resolve_prop_decl(&self, class_id: ClassId, name: &[u8]) -> Option<(Visibility, ClassId)> {
        let classes: &'p [ClassDecl] = self.classes;
        let mut cid = Some(class_id);
        while let Some(c) = cid {
            if let Some(p) = classes[c].props.iter().find(|p| p.name.as_ref() == name) {
                return Some((p.visibility, c));
            }
            cid = classes[c].parent;
        }
        None
    }

    /// Whether `a` is `b` or descends from it (used for protected access checks).
    fn class_is_a(&self, a: ClassId, b: ClassId) -> bool {
        let classes: &'p [ClassDecl] = self.classes;
        let mut cur = Some(a);
        while let Some(c) = cur {
            if c == b {
                return true;
            }
            cur = classes[c].parent;
        }
        false
    }

    /// Whether the given visibility, declared on `decl_class`, is accessible from
    /// the current class context (`self.cur_class`), step 19-3, D-19.13.
    pub(super) fn visible_from(&self, vis: Visibility, decl_class: ClassId) -> bool {
        match vis {
            Visibility::Public => true,
            Visibility::Private => self.cur_class == Some(decl_class),
            // Protected: accessible from anywhere in the same hierarchy.
            Visibility::Protected => matches!(
                self.cur_class,
                Some(cc) if self.class_is_a(cc, decl_class) || self.class_is_a(decl_class, cc)
            ),
        }
    }

    /// Enforce property visibility for an access on `class_id`. A dynamic /
    /// undeclared property is always accessible (public).
    fn check_prop_access(&self, class_id: ClassId, name: &[u8]) -> Result<(), PhpError> {
        let Some((vis, decl_class)) = self.resolve_prop_decl(class_id, name) else {
            return Ok(());
        };
        if self.visible_from(vis, decl_class) {
            return Ok(());
        }
        let kind = if matches!(vis, Visibility::Private) {
            "private"
        } else {
            "protected"
        };
        Err(PhpError::Error(format!(
            "Cannot access {kind} property {}::${}",
            String::from_utf8_lossy(&self.classes[decl_class].name),
            String::from_utf8_lossy(name)
        )))
    }

    /// If the first place step is a property, enforce its visibility against the
    /// object the base designates (write/unset contexts), step 19-3. Deeper
    /// properties in a chain are not checked (19-3 simplification). When a magic
    /// accessor (`__set`/`__unset`) will handle a missing-or-inaccessible
    /// property, the visibility error is suppressed so the magic call can run
    /// (step 22, D-22.2).
    pub(super) fn check_first_prop_write(
        &self,
        base: PlaceBase,
        steps: &[Step],
        kind: MagicAccess,
        magic_name: &[u8],
    ) -> Result<(), PhpError> {
        if let Some(Step::Prop(name)) = steps.first() {
            if let Zval::Object(o) = self.base_clone(base) {
                if self.magic_prop_method(&o, name, kind, magic_name).is_some() {
                    return Ok(());
                }
                let cid = o.borrow().class_id as usize;
                return self.check_prop_access(cid, name);
            }
        }
        Ok(())
    }

    /// Resolve a [`ClassRef`] to a class id in the current context (step 19-4):
    /// a named class via the class table, `self`/`parent` via the defining class,
    /// `static` via the late-static-binding class.
    pub(super) fn resolve_class_ref(&mut self, class: &ClassRef) -> Result<ClassId, PhpError> {
        match class {
            ClassRef::Named(name) => self.resolve_class_name(name),
            ClassRef::SelfClass => self
                .cur_class
                .ok_or_else(|| PhpError::Error("Cannot use \"self\" outside class context".into())),
            ClassRef::Parent => self
                .cur_class
                .and_then(|c| self.classes[c].parent)
                .ok_or_else(|| {
                    PhpError::Error(
                        "Cannot use \"parent\" when current class scope has no parent".into(),
                    )
                }),
            ClassRef::Static => self.cur_static_class.ok_or_else(|| {
                PhpError::Error("Cannot use \"static\" outside class context".into())
            }),
            // `new $cls`, `$cls::m()`, `$obj::m()` (step 48): evaluate to a class
            // name (string) or an object, then resolve to a class id.
            ClassRef::Dynamic(expr) => match self.eval(expr)?.deref_clone() {
                Zval::Str(s) => {
                    // A leading namespace separator is stripped (`\Foo` == `Foo`).
                    let name = s.as_bytes();
                    let name = name.strip_prefix(b"\\").unwrap_or(name);
                    self.resolve_class_name(name)
                }
                Zval::Object(o) => Ok(o.borrow().class_id as usize),
                other => Err(PhpError::TypeError(format!(
                    "Class name must be a valid object or a string, {} given",
                    other.error_type_name()
                ))),
            },
        }
    }

    /// Resolve a class *name* (case-insensitive) to its id, or PHP's "not found"
    /// error (step 48; shared by `Named` and `Dynamic` class refs).
    fn resolve_class_name(&self, name: &[u8]) -> Result<ClassId, PhpError> {
        self.class_index
            .get(&name.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| {
                PhpError::Error(format!("Class \"{}\" not found", String::from_utf8_lossy(name)))
            })
    }

    /// Evaluate `new ClassRef(args)` (step 19, D-19.6/D-19.12): resolve the class
    /// (including `self`/`static` late binding), build an instance with the full
    /// inherited property set, then run `__construct` (resolved up the chain).
    pub(super) fn eval_new(
        &mut self,
        class: &ClassRef,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Zval, PhpError> {
        let cid = self.resolve_class_ref(class)?;
        // An enum has no constructor and cannot be instantiated (step 23, D-23.9).
        if self.classes[cid].is_enum {
            return Err(PhpError::Error(format!(
                "Cannot instantiate enum {}",
                String::from_utf8_lossy(&self.classes[cid].name)
            )));
        }
        // An abstract class or interface cannot be instantiated (step 19-5).
        if self.classes[cid].is_abstract {
            let what = if self.classes[cid].is_interface {
                "interface"
            } else {
                "abstract class"
            };
            return Err(PhpError::Error(format!(
                "Cannot instantiate {what} {}",
                String::from_utf8_lossy(&self.classes[cid].name)
            )));
        }
        let class_name = PhpStr::new(self.classes[cid].name.to_vec());
        let props = self.collect_props(cid)?;
        let info = self.class_shape(cid);
        let id = self.next_id();
        let obj = Object {
            class_id: cid as u32,
            class_name,
            props,
            id,
            info,
        };
        let value = Zval::Object(Rc::new(RefCell::new(obj)));
        // Track the new instance for `__destruct` (step 24-2/24-3): an extra
        // strong ref whose presence is later used to detect unreachability.
        if let Zval::Object(o) = &value {
            self.created.push(o.clone());
        }
        // A Throwable records its creation site (`getLine`/`getFile`) at `new`
        // time, before the constructor runs (step 20). PHP sets these from the
        // engine, not from `Exception::__construct`.
        if self.is_throwable(cid) {
            // Capture the trace at construction (step 28), before the constructor
            // runs — PHP snapshots the stack at `new`, not at `throw`.
            let (trace, trace_string) = self.capture_trace();
            if let Zval::Object(o) = &value {
                let create_line = self.cur_line as i64;
                let mut b = o.borrow_mut();
                b.props.set(b"line", Zval::Long(create_line));
                b.props.set(b"file", Zval::Str(PhpStr::new(self.file.to_vec())));
                b.props.set(b"trace", trace);
                b.props.set(b"traceString", Zval::Str(PhpStr::new(trace_string)));
            }
        }
        // Run the constructor (inherited if not overridden); its mutations write
        // through the shared `Rc`, so they show in the returned value. The new
        // instance's class is its own LSB class.
        if let Some((defc, m)) = self.resolve_method(cid, b"__construct") {
            // Positional args (incl. unpacked, step 40), then named placed by
            // parameter name (step 38-2).
            let (vals, spread_named) = self.eval_value_args(args)?;
            let argv: Vec<Arg> = vals.into_iter().map(Arg::Val).collect();
            let argv = self.apply_named_args(&m.decl, argv, spread_named, named)?;
            self.invoke_method_args(Some(value.clone()), defc, cid, m, b"__construct", argv)?;
        } else if !named.is_empty() || !args.is_empty() {
            // No constructor: named args would have nowhere to bind. PHP ignores
            // extra args to a default constructor, but a named arg is an Error.
            // Keep parity with the no-ctor positional path (args ignored); a named
            // arg to a constructor-less class is rare — treat as unknown param.
            if let Some((name, _)) = named.first() {
                return Err(PhpError::Error(format!(
                    "Unknown named parameter ${}",
                    String::from_utf8_lossy(name)
                )));
            }
        }
        Ok(value)
    }

    /// Run `__destruct` on object `o` exactly once, if it declares one. The
    /// caller is responsible for having removed `o` from `created` already.
    fn run_one_destructor(&mut self, o: &Rc<RefCell<Object>>) {
        let (cid, id) = {
            let b = o.borrow();
            (b.class_id as usize, b.id)
        };
        if self.destructed.contains(&id) {
            return;
        }
        if let Some((defc, m)) = self.resolve_method(cid, b"__destruct") {
            self.destructed.insert(id);
            let value = Zval::Object(o.clone());
            // A destructor that throws is swallowed: its unwinding would otherwise
            // abort the remaining destructors. PHP turns it into a shutdown fatal;
            // refining that is future work.
            let _ = self.invoke_method(Some(value), defc, cid, m, b"__destruct", Vec::new());
            self.flush_diags();
        }
    }

    /// Mid-script destruction sweep (step 24-3): release every tracked object the
    /// program can no longer reach (`Rc::strong_count == 1`, i.e. only the
    /// tracking ref remains), most-recently-created first. Running one destructor
    /// or dropping a destructor-less object may make another unreachable
    /// (transitively, e.g. an object held only by a now-freed array), so the scan
    /// repeats until a fixpoint. Called at global-scope statement boundaries;
    /// destructor bodies run with a local frame, so the `locals.is_none()` gate at
    /// the call site keeps this from re-entering.
    pub(super) fn sweep_destructors(&mut self) {
        loop {
            let idx = self
                .created
                .iter()
                .rposition(|o| Rc::strong_count(o) == 1);
            let Some(i) = idx else { break };
            let o = self.created.remove(i);
            self.run_one_destructor(&o);
            // `o` drops here, possibly releasing another tracked object.
        }
    }

    /// End-of-script shutdown (step 24-2): invoke `__destruct` on every object
    /// still tracked at the end of the run, in reverse creation order (PHP
    /// shutdown is LIFO). These are the objects still reachable when the script
    /// ends (e.g. held by globals); mid-script releases were already handled by
    /// [`Evaluator::sweep_destructors`].
    pub(super) fn run_destructors(&mut self) {
        let survivors: Vec<Rc<RefCell<Object>>> = std::mem::take(&mut self.created);
        for o in survivors.into_iter().rev() {
            self.run_one_destructor(&o);
        }
    }

    /// Resolve and evaluate a class constant `Class::NAME` (step 19-4, D-19.15),
    /// or the special `Class::class` (the class name string). The constant's
    /// value expression is evaluated in its *declaring* class's context.
    pub(super) fn eval_class_const(&mut self, class: &ClassRef, name: &[u8]) -> Result<Zval, PhpError> {
        let cid = self.resolve_class_ref(class)?;
        if name.eq_ignore_ascii_case(b"class") {
            return Ok(Zval::Str(PhpStr::new(self.classes[cid].name.to_vec())));
        }
        // An enum case (`E::Case`) is matched case-sensitively before user
        // constants, and resolves to the interned singleton (step 23, D-23.2).
        if self.classes[cid].is_enum
            && self.classes[cid].enum_cases.iter().any(|c| c.name.as_ref() == name)
        {
            return self.eval_enum_case(cid, name);
        }
        // Resolve the constant through the parent chain *and* implemented
        // interfaces (interface constants are inherited too — gh7821, also a
        // general class gap surfaced by enums in step 23).
        let (decl_class, expr) = self.find_class_const(cid, name).ok_or_else(|| {
            PhpError::Error(format!(
                "Undefined constant {}::{}",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(name)
            ))
        })?;
        // Evaluate in the declaring class's context so a `self::OTHER` inside the
        // constant resolves correctly.
        let saved_class = self.cur_class.replace(decl_class);
        let result = self.eval(expr);
        self.cur_class = saved_class;
        result
    }

    /// Find a class constant by name, searching the class's own constants, then
    /// its parent chain, then (transitively) its implemented interfaces. Returns
    /// the declaring class id and the value expression (step 23 / gh7821).
    fn find_class_const(&self, cid: ClassId, name: &[u8]) -> Option<(ClassId, &'p Expr)> {
        let classes: &'p [ClassDecl] = self.classes;
        // Own constants + parent chain take precedence.
        let mut c = Some(cid);
        while let Some(x) = c {
            if let Some(k) = classes[x].consts.iter().find(|k| k.name.as_ref() == name) {
                return Some((x, &k.value));
            }
            c = classes[x].parent;
        }
        // Then interfaces of the class and its ancestors (transitively).
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

    /// Return the interned singleton object for enum case `E::name`, creating it
    /// on first access (step 23, D-23.2/D-23.4). The case is guaranteed to exist
    /// (the caller checked). Synthesises the read-only `name` (and, for a backed
    /// enum, `value`) properties; the object carries the enum's class id so the
    /// whole OOP machinery (`instanceof`, method dispatch, `$this`) applies.
    fn eval_enum_case(&mut self, cid: ClassId, name: &[u8]) -> Result<Zval, PhpError> {
        let key = (cid, name.to_vec());
        if let Some(o) = self.enum_cache.get(&key) {
            return Ok(Zval::Object(Rc::clone(o)));
        }
        let classes: &'p [ClassDecl] = self.classes;
        let case = classes[cid]
            .enum_cases
            .iter()
            .find(|c| c.name.as_ref() == name)
            .expect("caller verified the case exists");
        let mut props = Props::new();
        let mut entries: Vec<(Box<[u8]>, PropVis)> =
            vec![(Box::from(&b"name"[..]), PropVis::Public)];
        props.set(b"name", Zval::Str(PhpStr::new(name.to_vec())));
        if let Some(expr) = &case.value {
            // The backing value is a compile-time literal of the declared type
            // (PHP rejects a mismatch at link time), so it is stored as-is once,
            // when the singleton is first materialised (step 23, D-23.4/D-23.10).
            let saved = self.cur_class.replace(cid);
            let value = self.eval(expr);
            self.cur_class = saved;
            props.set(b"value", value?);
            entries.push((Box::from(&b"value"[..]), PropVis::Public));
        }
        let id = self.next_id();
        let obj = Object {
            class_id: cid as u32,
            class_name: PhpStr::new(classes[cid].name.to_vec()),
            props,
            id,
            info: Rc::new(ObjectInfo::enum_case(entries)),
        };
        let rc = Rc::new(RefCell::new(obj));
        self.enum_cache.insert(key, Rc::clone(&rc));
        Ok(Zval::Object(rc))
    }

    /// `E::cases()` (step 23, D-23.6): a sequential array of every case singleton
    /// in declaration order. Works on pure and backed enums alike.
    fn enum_cases(&mut self, cid: ClassId) -> Result<Zval, PhpError> {
        let names: Vec<Vec<u8>> = self.classes[cid]
            .enum_cases
            .iter()
            .map(|c| c.name.to_vec())
            .collect();
        let mut arr = PhpArray::new();
        for n in &names {
            let case = self.eval_enum_case(cid, n)?;
            let _ = arr.append(case);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `BackedEnum::from($v)` / `BackedEnum::tryFrom($v)` (step 23, D-23.6). Scans
    /// the cases for one whose backing `value` is identical (`===`) to `$v` and
    /// returns its singleton. `from` raises a catchable `ValueError` on no match;
    /// `tryFrom` returns `null`.
    fn enum_from(
        &mut self,
        cid: ClassId,
        arg: Option<&Zval>,
        try_from: bool,
    ) -> Result<Zval, PhpError> {
        let arg = arg.cloned().unwrap_or(Zval::Null);
        let names: Vec<Vec<u8>> = self.classes[cid]
            .enum_cases
            .iter()
            .map(|c| c.name.to_vec())
            .collect();
        for n in &names {
            let case = self.eval_enum_case(cid, n)?;
            let hit = matches!(&case, Zval::Object(o)
                if o.borrow().props.get(b"value").is_some_and(|v| ops::identical(v, &arg)));
            if hit {
                return Ok(case);
            }
        }
        if try_from {
            return Ok(Zval::Null);
        }
        // PHP quotes a string backing value but not an integer one.
        let repr = match &arg {
            Zval::Str(s) => format!("\"{}\"", String::from_utf8_lossy(s.as_bytes())),
            Zval::Long(l) => l.to_string(),
            other => {
                let z = convert::to_zstr(other, &mut self.diags);
                String::from_utf8_lossy(z.as_bytes()).into_owned()
            }
        };
        Err(PhpError::ValueError(format!(
            "{repr} is not a valid backing value for enum {}",
            String::from_utf8_lossy(&self.classes[cid].name)
        )))
    }

    /// The persistent cell backing a `static` property, resolving the declaring
    /// class up the chain and lazily initialising from the declared default on
    /// first access (step 19-4, D-19.14). Enforces visibility.
    pub(super) fn static_prop_cell(
        &mut self,
        class: &ClassRef,
        name: &[u8],
    ) -> Result<Rc<RefCell<Zval>>, PhpError> {
        let cid = self.resolve_class_ref(class)?;
        let classes: &'p [ClassDecl] = self.classes;
        let mut c = Some(cid);
        let mut decl: Option<(ClassId, &'p crate::hir::StaticPropDecl)> = None;
        while let Some(x) = c {
            if let Some(p) = classes[x].static_props.iter().find(|p| p.name.as_ref() == name) {
                decl = Some((x, p));
                break;
            }
            c = classes[x].parent;
        }
        let (decl_class, pd) = decl.ok_or_else(|| {
            PhpError::Error(format!(
                "Access to undeclared static property {}::${}",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(name)
            ))
        })?;
        // Visibility against the current class context.
        if !self.visible_from(pd.visibility, decl_class) {
            let kind = if matches!(pd.visibility, Visibility::Private) {
                "private"
            } else {
                "protected"
            };
            return Err(PhpError::Error(format!(
                "Cannot access {kind} property {}::${}",
                String::from_utf8_lossy(&self.classes[decl_class].name),
                String::from_utf8_lossy(name)
            )));
        }
        let key = (decl_class, name.to_vec());
        if let Some(cell) = self.static_props.get(&key) {
            return Ok(Rc::clone(cell));
        }
        // First access: initialise from the default (evaluated in the declaring
        // class's context), then store.
        let init = match &pd.default {
            Some(e) => {
                let saved = self.cur_class.replace(decl_class);
                let v = self.eval(e);
                self.cur_class = saved;
                v?
            }
            None => Zval::Null,
        };
        let cell = Rc::new(RefCell::new(init));
        self.static_props.insert(key, Rc::clone(&cell));
        Ok(cell)
    }

    /// Resolve the argument of `exit`/`die` to a process exit code, following
    /// PHP's `exit(string|int $status = 0)` signature (step 46). A `string` (or
    /// a `__toString` object) takes the string branch: it is emitted as a
    /// message with exit code `0`. An `int`/`float`/`bool`/`null` takes the int
    /// branch: coerced to an integer exit code (normalised to `0..=255`, nothing
    /// printed). Anything else (`array`, a non-stringable object, …) is a
    /// `TypeError`, matching the oracle (`exit(): Argument #1 ($status) must be
    /// of type string|int, X given`). The float-precision / null deprecation
    /// notices PHP emits on coercion are a declared scope-out (D-46.1).
    pub(super) fn exit_status(&mut self, v: Zval) -> Result<u8, PhpError> {
        // Collapse a reference to its referent (the invariant forbids ref-to-ref).
        let v = match v {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        };
        match &v {
            // A string is a message printed verbatim.
            Zval::Str(s) => {
                self.emit(s.as_bytes());
                Ok(0)
            }
            // Scalars with a defined integer coercion become the exit code.
            Zval::Long(_) | Zval::Double(_) | Zval::Bool(_) | Zval::Null | Zval::Undef => {
                Ok(convert::to_long_cast(&v, &mut self.diags).rem_euclid(256) as u8)
            }
            // An object joins the `string` branch only if it is stringable.
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                if self.resolve_method(cid, b"__toString").is_some() {
                    let s = self.stringify(&v)?;
                    self.emit(s.as_bytes());
                    Ok(0)
                } else {
                    Err(self.exit_type_error(&v))
                }
            }
            // array / closure / generator: no string|int coercion → TypeError.
            _ => Err(self.exit_type_error(&v)),
        }
    }

    /// The `TypeError` for `exit`/`die` given a value outside `string|int`
    /// (step 46). Objects are named by their class (`stdClass given`), other
    /// values by their PHP type name.
    fn exit_type_error(&self, v: &Zval) -> PhpError {
        let given = match v {
            Zval::Object(o) => {
                String::from_utf8_lossy(&self.classes[o.borrow().class_id as usize].name)
                    .into_owned()
            }
            other => php_type_name(other).to_string(),
        };
        PhpError::TypeError(format!(
            "exit(): Argument #1 ($status) must be of type string|int, {given} given"
        ))
    }

    /// Convert a value to a string, honouring `__toString` on objects (step 19-6,
    /// D-19.18). A non-object goes through the ordinary `to_zstr` funnel; an
    /// object without `__toString` is the fatal PHP raises (the placeholder the
    /// infallible funnel emits is thereby avoided for the contexts that route
    /// through here: echo, concat, `(string)`).
    pub(super) fn stringify(&mut self, v: &Zval) -> Result<Rc<PhpStr>, PhpError> {
        let v = v.deref_clone();
        match &v {
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                match self.resolve_method(cid, b"__toString") {
                    Some((defc, m)) => {
                        let r =
                            self.invoke_method(Some(v.clone()), defc, cid, m, b"__toString", vec![])?;
                        Ok(convert::to_zstr(&r, &mut self.diags))
                    }
                    None => {
                        let name =
                            String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned();
                        Err(PhpError::Error(format!(
                            "Object of class {name} could not be converted to string"
                        )))
                    }
                }
            }
            other => Ok(convert::to_zstr(other, &mut self.diags)),
        }
    }

    /// Decide whether a magic property accessor of `kind` (`__get`/`__set`/…)
    /// should run for `name` on object `o` instead of direct access (step 22,
    /// D-22.2/D-22.4). A magic call applies when the property is missing or not
    /// visible from the current scope, the class defines the accessor, and no
    /// same-kind guard is already active. Returns `(defining class, object class,
    /// object handle, method)` to invoke, or `None` for direct access.
    pub(super) fn magic_prop_method(
        &self,
        o: &Rc<RefCell<Object>>,
        name: &[u8],
        kind: MagicAccess,
        magic_name: &[u8],
    ) -> Option<(ClassId, ClassId, u32, &'p MethodDecl)> {
        let (obj_cid, oid, present, accessible) = {
            let obj = o.borrow();
            let cid = obj.class_id as usize;
            let accessible = match self.resolve_prop_decl(cid, name) {
                Some((vis, dc)) => self.visible_from(vis, dc),
                None => true,
            };
            (cid, obj.id, obj.props.contains(name), accessible)
        };
        if present && accessible {
            return None;
        }
        if self.magic_guard.contains(&(oid, kind, name.to_vec())) {
            return None;
        }
        let (defc, m) = self.resolve_method(obj_cid, magic_name)?;
        Some((defc, obj_cid, oid, m))
    }

    /// If `__isset` applies to `name` on `o` (property missing/inaccessible,
    /// method present, not guarded), invoke it under a guard and return its
    /// boolean result; `None` means no magic (caller does the direct check),
    /// step 22, D-22.1.
    pub(super) fn magic_isset_bool(
        &mut self,
        o: &Rc<RefCell<Object>>,
        name: &[u8],
    ) -> Option<Result<bool, PhpError>> {
        let (defc, obj_cid, oid, m) = self.magic_prop_method(o, name, MagicAccess::Isset, b"__isset")?;
        let key = (oid, MagicAccess::Isset, name.to_vec());
        self.magic_guard.insert(key.clone());
        let recv = Zval::Object(Rc::clone(o));
        let arg = Zval::Str(PhpStr::new(name.to_vec()));
        let r = self.invoke_method(Some(recv), defc, obj_cid, m, b"__isset", vec![arg]);
        self.magic_guard.remove(&key);
        Some(r.map(|v| convert::is_true_silent(&v)))
    }

    /// `isset()` truth for a resolved place (step 22): a single trailing property
    /// on an object routes to `__isset`; anything else uses the silent traversal.
    pub(super) fn place_isset(&mut self, base: PlaceBase, steps: &[Step]) -> Result<bool, PhpError> {
        if let [Step::Prop(name)] = steps {
            if let Zval::Object(o) = self.base_clone(base) {
                if let Some(r) = self.magic_isset_bool(&o, name) {
                    return r;
                }
            }
        }
        Ok(matches!(
            self.silent_get(base, steps),
            Some(v) if !matches!(v, Zval::Null | Zval::Undef)
        ))
    }

    /// The value of `name` on `o` for a *silent* context — `empty()`, `??`,
    /// `??=` after `__isset` returned true (step 22): `__get` if defined, else
    /// the present value or NULL, raising no undefined-property warning
    /// (bug #44899).
    pub(super) fn prop_value_silent(&mut self, o: &Rc<RefCell<Object>>, name: &[u8]) -> Result<Zval, PhpError> {
        if let Some((defc, obj_cid, oid, m)) =
            self.magic_prop_method(o, name, MagicAccess::Get, b"__get")
        {
            let key = (oid, MagicAccess::Get, name.to_vec());
            self.magic_guard.insert(key.clone());
            let recv = Zval::Object(Rc::clone(o));
            let arg = Zval::Str(PhpStr::new(name.to_vec()));
            let r = self.invoke_method(Some(recv), defc, obj_cid, m, b"__get", vec![arg]);
            self.magic_guard.remove(&key);
            return r;
        }
        Ok(o.borrow().props.get(name).map(Zval::deref_clone).unwrap_or(Zval::Null))
    }

    /// `empty()` truth for a resolved place (step 22): a magic property is
    /// `__isset` then (if set) `__get`, mirroring PHP.
    pub(super) fn place_empty(&mut self, base: PlaceBase, steps: &[Step]) -> Result<bool, PhpError> {
        if let [Step::Prop(name)] = steps {
            if let Zval::Object(o) = self.base_clone(base) {
                if let Some(r) = self.magic_isset_bool(&o, name) {
                    if !r? {
                        return Ok(true);
                    }
                    let v = self.prop_value_silent(&o, name)?;
                    return Ok(!convert::is_true_silent(&v));
                }
            }
        }
        Ok(match self.silent_get(base, steps) {
            Some(v) => !convert::is_true_silent(&v),
            None => true,
        })
    }

    /// Read property `name` from a value (step 19, D-19.8; step 22 `__get`).
    /// Enforces visibility on a declared property; a missing or inaccessible
    /// property routes to `__get` if defined, else warns and yields NULL.
    pub(super) fn read_property(&mut self, recv: &Zval, name: &[u8]) -> Result<Zval, PhpError> {
        match recv {
            Zval::Object(o) => {
                if let Some((defc, obj_cid, oid, m)) =
                    self.magic_prop_method(o, name, MagicAccess::Get, b"__get")
                {
                    let key = (oid, MagicAccess::Get, name.to_vec());
                    self.magic_guard.insert(key.clone());
                    let arg = Zval::Str(PhpStr::new(name.to_vec()));
                    let r = self.invoke_method(
                        Some(recv.clone()),
                        defc,
                        obj_cid,
                        m,
                        b"__get",
                        vec![arg],
                    );
                    self.magic_guard.remove(&key);
                    return r;
                }
                let cid = o.borrow().class_id as usize;
                self.check_prop_access(cid, name)?;
                let obj = o.borrow();
                if let Some(v) = obj.props.get(name) {
                    return Ok(v.deref_clone());
                }
                let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
                drop(obj);
                let prop = String::from_utf8_lossy(name).into_owned();
                self.diags.push(Diag::Warning(format!(
                    "Undefined property: {cls}::${prop}"
                )));
                Ok(Zval::Null)
            }
            Zval::Null | Zval::Undef => {
                let prop = String::from_utf8_lossy(name).into_owned();
                self.diags.push(Diag::Warning(format!(
                    "Attempt to read property \"{prop}\" on null"
                )));
                Ok(Zval::Null)
            }
            other => {
                let prop = String::from_utf8_lossy(name).into_owned();
                self.diags.push(Diag::Warning(format!(
                    "Attempt to read property \"{prop}\" on {}",
                    other.error_type_name()
                )));
                Ok(Zval::Null)
            }
        }
    }

    /// Pack call arguments into a 0-indexed list array, the second argument of
    /// `__call`/`__callStatic` (step 22, D-22.5).
    fn pack_args(&self, argv: Vec<Zval>) -> Zval {
        let mut arr = PhpArray::new();
        for v in argv {
            let _ = arr.append(v);
        }
        Zval::Array(Rc::new(arr))
    }

    /// Invoke `$obj->method(argv)` (step 19, D-19.7; step 22 `__call`): resolve
    /// the method up the chain, enforce visibility, then run it with `$this`
    /// bound to the receiver. A method missing or inaccessible from the current
    /// scope routes to `__call($method, $args)` if defined.
    pub(super) fn call_method(
        &mut self,
        recv: Zval,
        method: &[u8],
        argv: Vec<Zval>,
        spread_named: SpreadNamed,
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Zval, PhpError> {
        let cid = match &recv {
            Zval::Object(o) => o.borrow().class_id as usize,
            other => {
                return Err(PhpError::Error(format!(
                    "Call to a member function {}() on {}",
                    String::from_utf8_lossy(method),
                    other.error_type_name()
                )))
            }
        };
        match self.resolve_method(cid, method) {
            Some((defc, m)) if self.visible_from(m.visibility, defc) => {
                // An instance call's LSB class is the object's actual class.
                // Named args (and named unpacking) are placed by name (step 38-3 / 40).
                let argv: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
                let argv = self.apply_named_args(&m.decl, argv, spread_named, named)?;
                self.invoke_method_args(Some(recv), defc, cid, m, method, argv)
            }
            found => {
                // `__call` collects args into an array; named-arg placement does
                // not apply to it (step 38 / 40 scope-out).
                if let Some(e) = self.reject_named(named, &spread_named) {
                    return Err(e);
                }
                if let Some((cdefc, cm)) = self.resolve_method(cid, b"__call") {
                    let args = self.pack_args(argv);
                    let name = Zval::Str(PhpStr::new(method.to_vec()));
                    return self.invoke_method(Some(recv), cdefc, cid, cm, b"__call", vec![name, args]);
                }
                match found {
                    // Found but inaccessible and no __call: the visibility error.
                    Some((defc, m)) => {
                        self.check_method_access(defc, m, method)?;
                        unreachable!("check_method_access errors when not visible")
                    }
                    None => Err(PhpError::Error(format!(
                        "Call to undefined method {}::{}()",
                        String::from_utf8_lossy(&self.classes[cid].name),
                        String::from_utf8_lossy(method)
                    ))),
                }
            }
        }
    }

    /// Dispatch `Class::m()` / `self::m()` / `parent::m()` / `static::m()` (step
    /// 19-3/19-4). The starting class comes from the reference; `self`/`parent`/
    /// `static` are *forwarding* (keep `$this` and the caller's LSB class), while
    /// a named class sets the LSB class to itself.
    pub(super) fn call_static(
        &mut self,
        class: &ClassRef,
        method: &[u8],
        argv: Vec<Zval>,
        spread_named: SpreadNamed,
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Zval, PhpError> {
        // `Closure::bind(...)` / `Closure::fromCallable(...)` are built-in (the
        // engine `Closure` class is not in the user class table), step 19-6.
        // Named args to these / enum built-in statics are out of scope (step 38 / 40).
        if let ClassRef::Named(n) = class {
            if n.eq_ignore_ascii_case(b"Closure") {
                if let Some(e) = self.reject_named(named, &spread_named) {
                    return Err(e);
                }
                return self.closure_static(method, argv);
            }
        }
        let start = self.resolve_class_ref(class)?;
        // Enum built-in static methods (step 23, D-23.6). They are reserved names,
        // so they shadow user resolution. `cases` exists on every enum;
        // `from`/`tryFrom` only on backed ones (on a pure enum they fall through
        // to "undefined method").
        if self.classes[start].is_enum {
            if method.eq_ignore_ascii_case(b"cases") {
                return self.enum_cases(start);
            }
            if self.classes[start].enum_backing.is_some() {
                if method.eq_ignore_ascii_case(b"from") {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.enum_from(start, argv.first(), false);
                }
                if method.eq_ignore_ascii_case(b"tryFrom") {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.enum_from(start, argv.first(), true);
                }
            }
        }
        match self.resolve_method(start, method) {
            Some((defc, m)) if self.visible_from(m.visibility, defc) => {
                let forwarding = !matches!(class, ClassRef::Named(_) | ClassRef::Dynamic(_));
                // LSB class: forwarding calls preserve the caller's, a named call
                // rebinds it to the named class.
                let static_class = if forwarding {
                    self.cur_static_class.unwrap_or(start)
                } else {
                    start
                };
                // `$this` is kept for a forwarding call, or for a named call to a
                // class in the current object's hierarchy (`ParentName::m()`).
                let this = match &self.cur_this {
                    Some(t @ Zval::Object(o))
                        if forwarding
                            || self.class_is_a(o.borrow().class_id as usize, start) =>
                    {
                        Some(t.clone())
                    }
                    _ => None,
                };
                // Named args (and named unpacking) placed by name (step 38-3 / 40).
                let argv: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
                let argv = self.apply_named_args(&m.decl, argv, spread_named, named)?;
                self.invoke_method_args(this, defc, static_class, m, method, argv)
            }
            found => {
                if let Some(e) = self.reject_named(named, &spread_named) {
                    return Err(e);
                }
                // Missing or inaccessible. In object context (a usable `$this`,
                // e.g. `parent::priv()` from a method) it routes to `__call` on
                // `$this`; otherwise to `__callStatic` (step 22, D-22.3,
                // bug #53826).
                let forwarding = !matches!(class, ClassRef::Named(_) | ClassRef::Dynamic(_));
                let this_obj = match &self.cur_this {
                    Some(t @ Zval::Object(o))
                        if forwarding
                            || self.class_is_a(o.borrow().class_id as usize, start) =>
                    {
                        Some(t.clone())
                    }
                    _ => None,
                };
                if let Some(this) = this_obj {
                    let ocid = match &this {
                        Zval::Object(o) => o.borrow().class_id as usize,
                        _ => unreachable!("matched Zval::Object above"),
                    };
                    if let Some((cdefc, cm)) = self.resolve_method(ocid, b"__call") {
                        let args = self.pack_args(argv);
                        let name = Zval::Str(PhpStr::new(method.to_vec()));
                        return self.invoke_method(Some(this), cdefc, ocid, cm, b"__call", vec![name, args]);
                    }
                }
                if let Some((cdefc, cm)) = self.resolve_method(start, b"__callStatic") {
                    let args = self.pack_args(argv);
                    let name = Zval::Str(PhpStr::new(method.to_vec()));
                    return self.invoke_method(None, cdefc, start, cm, b"__callStatic", vec![name, args]);
                }
                match found {
                    Some((defc, m)) => {
                        self.check_method_access(defc, m, method)?;
                        unreachable!("check_method_access errors when not visible")
                    }
                    None => Err(PhpError::Error(format!(
                        "Call to undefined method {}::{}()",
                        String::from_utf8_lossy(&self.classes[start].name),
                        String::from_utf8_lossy(method)
                    ))),
                }
            }
        }
    }

    /// Enforce method visibility against the current class context (step 19-3).
    fn check_method_access(
        &self,
        defining_class: ClassId,
        m: &MethodDecl,
        method: &[u8],
    ) -> Result<(), PhpError> {
        if self.visible_from(m.visibility, defining_class) {
            return Ok(());
        }
        let kind = if matches!(m.visibility, Visibility::Private) {
            "private"
        } else {
            "protected"
        };
        Err(PhpError::Error(format!(
            "Call to {kind} method {}::{}() from {}",
            String::from_utf8_lossy(&self.classes[defining_class].name),
            String::from_utf8_lossy(method),
            match self.cur_class {
                Some(c) => format!(
                    "scope {}",
                    String::from_utf8_lossy(&self.classes[c].name)
                ),
                None => "global scope".to_string(),
            }
        )))
    }

    /// Shared method-frame setup (step 19): bind `$this` and the defining class,
    /// check arity, bind parameters, run the body, then restore the saved context.
    pub(super) fn invoke_method(
        &mut self,
        this: Option<Zval>,
        defining_class: ClassId,
        static_class: ClassId,
        m: &'p MethodDecl,
        method: &[u8],
        argv: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        let argv: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
        self.invoke_method_args(this, defining_class, static_class, m, method, argv)
    }

    /// Like [`Self::invoke_method`] but taking already-bound [`Arg`]s, so a call
    /// site can supply named-argument placement (step 38, `Arg::Default` gaps).
    fn invoke_method_args(
        &mut self,
        this: Option<Zval>,
        defining_class: ClassId,
        static_class: ClassId,
        m: &'p MethodDecl,
        method: &[u8],
        argv: Vec<Arg>,
    ) -> Result<Zval, PhpError> {
        self.guard_call_depth()?;
        let f: &'p FnDecl = &m.decl;
        let required = f
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .count();
        // A required parameter must have a real argument at its index (named args
        // can leave `Arg::Default` gaps — step 38).
        let missing_required = f
            .params
            .iter()
            .enumerate()
            .any(|(i, p)| {
                p.default.is_none()
                    && !p.variadic
                    && !matches!(argv.get(i), Some(Arg::Val(_) | Arg::Ref(_)))
            });
        if missing_required {
            let passed = argv
                .iter()
                .filter(|a| matches!(a, Arg::Val(_) | Arg::Ref(_)))
                .count();
            let expected = if required == f.params.len() {
                format!("exactly {required}")
            } else {
                format!("at least {required}")
            };
            return Err(PhpError::Error(format!(
                "Too few arguments to function {}::{}(), {} passed and {} expected",
                String::from_utf8_lossy(&self.classes[defining_class].name),
                String::from_utf8_lossy(method),
                passed,
                expected,
            )));
        }

        // Record a method stack frame for the body (step 28): `Class->m` for an
        // instance call, `Class::m` for a static one. Push before `this` moves.
        self.call_stack.push(CallFrame {
            class: Some(self.classes[static_class].name.to_vec()),
            function: method.to_vec(),
            is_static: this.is_none(),
            line: self.cur_line as i64,
        });

        let frame = fresh_slots(f.slots.len());
        let saved_locals = self.locals.replace(frame);
        let saved_names = self.local_names.replace(f.slots.as_slice());
        let saved_returns_ref = std::mem::replace(&mut self.fn_returns_ref, f.by_ref);
        let saved_this = std::mem::replace(&mut self.cur_this, this);
        let saved_class = self.cur_class.replace(defining_class);
        let saved_static = self.cur_static_class.replace(static_class);

        let result = self.run_user_fn_body(f, argv);

        self.locals = saved_locals;
        self.local_names = saved_names;
        self.fn_returns_ref = saved_returns_ref;
        self.cur_this = saved_this;
        self.cur_class = saved_class;
        self.cur_static_class = saved_static;
        self.call_stack.pop();
        result.map(|r| match r {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        })
    }
}
