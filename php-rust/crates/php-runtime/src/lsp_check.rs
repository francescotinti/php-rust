//! A-DS51 fase 1 — checker LSP PURO.
//!
//! Pure-function model of PHP 8.5 inheritance-compatibility checking
//! (Liskov / `zend_do_inheritance` signature checks) over hand-built
//! signature models. Produces the EXACT Zend fatal core messages as
//! `Err(String)` (no "PHP Fatal error:" prefix, no file/line suffix), and
//! tentative-return deprecations as a distinct non-fatal channel
//! (`Ok(Vec<String>)`).
//!
//! NOT WIRED: nothing in the runtime calls this module yet — fase 2 (A-DS51)
//! does the wiring. Ground truth = the committed fixtures in
//! wp89-harness/fixtures-ds35, wp90-harness/fixtures-ds35-v2,
//! wp91-harness/fixtures-ds35-v3, wp92-harness/fixtures-ds35-v4, oracled
//! against /opt/homebrew/opt/php/bin/php 8.5.7.
//!
//! Binding design points encoded here:
//! - A-DS59 (KS-DS-93-2, Stogov WP-93): the constructor exemption is keyed on
//!   the PROTOTYPE, not on the class. `__construct` is exempt from signature
//!   checks UNLESS its prototype comes from an INTERFACE or the parent ctor is
//!   declared abstract. A CONCRETE ctor in an ABSTRACT class is EXEMPT. The
//!   interface-ctor constraint propagates to grandchildren (fixture x6).
//! - A-DS60 formatting rules (each with a dedicated unit test):
//!   1. union/intersection types print in DECLARED order — zero sorting for
//!      named/class elements ("(B&A)|string" stays as declared); builtin
//!      scalar atoms print in Zend's fixed bitmask order
//!      (callable, object, array, iterable, string, int, float, bool, false,
//!      true, null) — that is why declared `int|string` prints "string|int".
//!   2. the `?T` collapse applies ONLY to the binary union with null.
//!   3. by-ref param grafia: "int &$x".
//!   4. by-ref property hooks print the "& " prefix WITH the space.
//!   5. the hook-set family does NOT use the "Declaration of" form.

use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Type model
// ---------------------------------------------------------------------------

/// A PHP type expression as DECLARED (order-preserving).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    /// A class/interface/enum name, display case preserved.
    Named(String),
    SelfTy,
    ParentTy,
    StaticTy,
    Callable,
    Object,
    Array,
    Iterable,
    Str,
    Int,
    Float,
    Bool,
    False,
    True,
    Null,
    Void,
    Never,
    Mixed,
    /// `?T` grafia as declared.
    Nullable(Box<Ty>),
    /// Union members in declared order (members: atoms or intersections).
    Union(Vec<Ty>),
    /// Intersection members in declared order (named types).
    Intersection(Vec<Ty>),
}

impl Ty {
    pub fn named(n: &str) -> Ty {
        Ty::Named(n.to_string())
    }
    pub fn union(members: Vec<Ty>) -> Ty {
        Ty::Union(members)
    }
    pub fn inter(members: Vec<Ty>) -> Ty {
        Ty::Intersection(members)
    }
    pub fn nullable(t: Ty) -> Ty {
        Ty::Nullable(Box::new(t))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

fn vis_rank(v: Visibility) -> u8 {
    match v {
        Visibility::Public => 2,
        Visibility::Protected => 1,
        Visibility::Private => 0,
    }
}

fn vis_str(v: Visibility) -> &'static str {
    match v {
        Visibility::Public => "public",
        Visibility::Protected => "protected",
        Visibility::Private => "private",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassKind {
    Class,
    AbstractClass,
    Interface,
    Trait,
    Enum,
}

// ---------------------------------------------------------------------------
// Member models + fluent builders (hand-built in tests)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Ty>,
    pub by_ref: bool,
    pub variadic: bool,
    pub has_default: bool,
    pub default_literal: Option<String>,
}

pub fn param(name: &str) -> Param {
    Param {
        name: name.to_string(),
        ty: None,
        by_ref: false,
        variadic: false,
        has_default: false,
        default_literal: None,
    }
}

impl Param {
    pub fn of(mut self, t: Ty) -> Self {
        self.ty = Some(t);
        self
    }
    pub fn by_ref_(mut self) -> Self {
        self.by_ref = true;
        self
    }
    pub fn variadic_(mut self) -> Self {
        self.variadic = true;
        self
    }
    pub fn with_default(mut self, lit: &str) -> Self {
        self.has_default = true;
        self.default_literal = Some(lit.to_string());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodModel {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub by_ref_return: bool,
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    /// Tentative return type (interface stubs): incompatibility is a
    /// Deprecated, not a fatal (fixture p5).
    pub tentative_return: bool,
    /// #[\ReturnTypeWillChange] on the CHILD suppresses the tentative
    /// deprecation (fixture p6).
    pub rtwc: bool,
}

pub fn method(name: &str) -> MethodModel {
    MethodModel {
        name: name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_final: false,
        is_abstract: false,
        by_ref_return: false,
        params: vec![],
        ret: None,
        tentative_return: false,
        rtwc: false,
    }
}

impl MethodModel {
    pub fn returns(mut self, t: Ty) -> Self {
        self.ret = Some(t);
        self
    }
    pub fn arg(mut self, p: Param) -> Self {
        self.params.push(p);
        self
    }
    pub fn private_(mut self) -> Self {
        self.visibility = Visibility::Private;
        self
    }
    pub fn protected_(mut self) -> Self {
        self.visibility = Visibility::Protected;
        self
    }
    pub fn static_(mut self) -> Self {
        self.is_static = true;
        self
    }
    pub fn final_(mut self) -> Self {
        self.is_final = true;
        self
    }
    pub fn abstract_(mut self) -> Self {
        self.is_abstract = true;
        self
    }
    pub fn by_ref_return_(mut self) -> Self {
        self.by_ref_return = true;
        self
    }
    pub fn tentative_(mut self) -> Self {
        self.tentative_return = true;
        self
    }
    pub fn rtwc_(mut self) -> Self {
        self.rtwc = true;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookKind {
    Get,
    Set,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hook {
    pub kind: HookKind,
    pub by_ref: bool,
    /// Explicit set-hook parameter (None = implicit `$value`).
    pub param: Option<Param>,
}

pub fn get_hook() -> Hook {
    Hook { kind: HookKind::Get, by_ref: false, param: None }
}

pub fn get_hook_ref() -> Hook {
    Hook { kind: HookKind::Get, by_ref: true, param: None }
}

pub fn set_hook(p: Param) -> Hook {
    Hook { kind: HookKind::Set, by_ref: false, param: Some(p) }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropModel {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub readonly: bool,
    pub ty: Option<Ty>,
    pub hooks: Vec<Hook>,
}

pub fn prop(name: &str) -> PropModel {
    PropModel {
        name: name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        readonly: false,
        ty: None,
        hooks: vec![],
    }
}

impl PropModel {
    pub fn of(mut self, t: Ty) -> Self {
        self.ty = Some(t);
        self
    }
    pub fn readonly_(mut self) -> Self {
        self.readonly = true;
        self
    }
    pub fn hook(mut self, h: Hook) -> Self {
        self.hooks.push(h);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstModel {
    pub name: String,
    pub is_final: bool,
}

pub fn konst(name: &str) -> ConstModel {
    ConstModel { name: name.to_string(), is_final: false }
}

impl ConstModel {
    pub fn final_(mut self) -> Self {
        self.is_final = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassModel {
    pub name: String,
    pub kind: ClassKind,
    pub parent: Option<String>,
    /// implemented interfaces (classes/enums) or extended interfaces
    /// (interface kind), declared order.
    pub interfaces: Vec<String>,
    pub traits: Vec<String>,
    pub methods: Vec<MethodModel>,
    pub props: Vec<PropModel>,
    pub consts: Vec<ConstModel>,
}

fn class_of_kind(name: &str, kind: ClassKind) -> ClassModel {
    ClassModel {
        name: name.to_string(),
        kind,
        parent: None,
        interfaces: vec![],
        traits: vec![],
        methods: vec![],
        props: vec![],
        consts: vec![],
    }
}

pub fn class(name: &str) -> ClassModel {
    class_of_kind(name, ClassKind::Class)
}
pub fn abstract_class(name: &str) -> ClassModel {
    class_of_kind(name, ClassKind::AbstractClass)
}
pub fn interface(name: &str) -> ClassModel {
    class_of_kind(name, ClassKind::Interface)
}
pub fn trait_(name: &str) -> ClassModel {
    class_of_kind(name, ClassKind::Trait)
}
pub fn enum_(name: &str) -> ClassModel {
    class_of_kind(name, ClassKind::Enum)
}

impl ClassModel {
    pub fn extends(mut self, p: &str) -> Self {
        self.parent = Some(p.to_string());
        self
    }
    pub fn implements(mut self, i: &str) -> Self {
        self.interfaces.push(i.to_string());
        self
    }
    pub fn uses(mut self, t: &str) -> Self {
        self.traits.push(t.to_string());
        self
    }
    pub fn with_method(mut self, m: MethodModel) -> Self {
        self.methods.push(m);
        self
    }
    pub fn with_prop(mut self, p: PropModel) -> Self {
        self.props.push(p);
        self
    }
    pub fn with_const(mut self, k: ConstModel) -> Self {
        self.consts.push(k);
        self
    }
}

// ---------------------------------------------------------------------------
// Scope + formatting (A-DS60)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Scope {
    /// Display name of the class where the member is declared (`self`
    /// resolves to this — fixture w4 prints C::m(): C).
    pub class: String,
    /// Display name of that class's parent (for `parent` resolution).
    pub parent: Option<String>,
}

fn lc(s: &str) -> String {
    s.to_lowercase()
}

fn fmt_atom(ty: &Ty, scope: &Scope) -> String {
    match ty {
        Ty::Named(n) => n.clone(),
        Ty::SelfTy => scope.class.clone(),
        Ty::ParentTy => scope.parent.clone().unwrap_or_else(|| "parent".to_string()),
        Ty::StaticTy => "static".to_string(),
        Ty::Callable => "callable".to_string(),
        Ty::Object => "object".to_string(),
        Ty::Array => "array".to_string(),
        Ty::Iterable => "iterable".to_string(),
        Ty::Str => "string".to_string(),
        Ty::Int => "int".to_string(),
        Ty::Float => "float".to_string(),
        Ty::Bool => "bool".to_string(),
        Ty::False => "false".to_string(),
        Ty::True => "true".to_string(),
        Ty::Null => "null".to_string(),
        Ty::Void => "void".to_string(),
        Ty::Never => "never".to_string(),
        Ty::Mixed => "mixed".to_string(),
        Ty::Nullable(_) | Ty::Union(_) | Ty::Intersection(_) => unreachable!("fmt_atom on compound"),
    }
}

/// Union formatting per A-DS60 rules 1 & 2. Named/intersection elements keep
/// DECLARED order; builtin scalar atoms print in Zend's fixed bitmask order;
/// null prints last, collapsing to `?T` only for the binary union.
fn fmt_union(members: &[Ty], scope: &Scope) -> String {
    // Declared-order complex parts (named classes, self/parent, intersections).
    let mut named: Vec<String> = vec![];
    let mut has_static = false;
    let mut has_null = false;
    // Fixed Zend bitmask print order for builtin atoms.
    const BUILTIN_ORDER: [(&str, usize); 10] = [
        ("callable", 0),
        ("object", 1),
        ("array", 2),
        ("iterable", 3),
        ("string", 4),
        ("int", 5),
        ("float", 6),
        ("bool", 7),
        ("false", 8),
        ("true", 9),
    ];
    let mut builtin_flags = [false; 10];

    fn walk(
        m: &Ty,
        scope: &Scope,
        named: &mut Vec<String>,
        has_static: &mut bool,
        has_null: &mut bool,
        builtin_flags: &mut [bool; 10],
    ) {
        match m {
            Ty::Named(_) | Ty::SelfTy | Ty::ParentTy => named.push(fmt_atom(m, scope)),
            Ty::Intersection(elems) => {
                let inner: Vec<String> = elems.iter().map(|e| fmt_atom(e, scope)).collect();
                named.push(format!("({})", inner.join("&")));
            }
            Ty::StaticTy => *has_static = true,
            Ty::Null => *has_null = true,
            Ty::Nullable(inner) => {
                *has_null = true;
                walk(inner, scope, named, has_static, has_null, builtin_flags);
            }
            Ty::Union(inner) => {
                for i in inner {
                    walk(i, scope, named, has_static, has_null, builtin_flags);
                }
            }
            Ty::Callable => builtin_flags[0] = true,
            Ty::Object => builtin_flags[1] = true,
            Ty::Array => builtin_flags[2] = true,
            Ty::Iterable => builtin_flags[3] = true,
            Ty::Str => builtin_flags[4] = true,
            Ty::Int => builtin_flags[5] = true,
            Ty::Float => builtin_flags[6] = true,
            Ty::Bool => builtin_flags[7] = true,
            Ty::False => builtin_flags[8] = true,
            Ty::True => builtin_flags[9] = true,
            // Not legal inside unions; print defensively in place.
            Ty::Void | Ty::Never | Ty::Mixed => named.push(fmt_atom(m, scope)),
        }
    }

    for m in members {
        walk(m, scope, &mut named, &mut has_static, &mut has_null, &mut builtin_flags);
    }

    let mut parts = named;
    if has_static {
        parts.push("static".to_string());
    }
    for (label, idx) in BUILTIN_ORDER {
        if builtin_flags[idx] {
            parts.push(label.to_string());
        }
    }
    if has_null {
        // A-DS60 rule 2: ?T collapse ONLY for the binary union with null.
        if parts.len() == 1 && !parts[0].starts_with('(') && parts[0] != "mixed" {
            return format!("?{}", parts[0]);
        }
        parts.push("null".to_string());
    }
    parts.join("|")
}

pub fn fmt_type(ty: &Ty, scope: &Scope) -> String {
    match ty {
        Ty::Union(members) => fmt_union(members, scope),
        // `?T` — the walk inside fmt_union marks has_null and collapses.
        Ty::Nullable(_) => fmt_union(std::slice::from_ref(ty), scope),
        Ty::Intersection(elems) => {
            let inner: Vec<String> = elems.iter().map(|e| fmt_atom(e, scope)).collect();
            inner.join("&")
        }
        atom => fmt_atom(atom, scope),
    }
}

/// A-DS60 rule 3: "int &$x" — space between type and &, & attached to $name.
fn fmt_param(p: &Param, scope: &Scope) -> String {
    let mut s = String::new();
    if let Some(t) = &p.ty {
        s.push_str(&fmt_type(t, scope));
        s.push(' ');
    }
    if p.by_ref {
        s.push('&');
    }
    if p.variadic {
        s.push_str("...");
    }
    s.push('$');
    s.push_str(&p.name);
    if let Some(d) = &p.default_literal {
        s.push_str(" = ");
        s.push_str(d);
    }
    s
}

/// Zend `zend_get_function_declaration` shape: "[& ]C::m(<params>)[: <ret>]".
/// Hook pseudo-methods carry names like "$x::get", printing "C::$x::get()".
fn method_sig(class_display: &str, m: &MethodModel, scope: &Scope) -> String {
    let params: Vec<String> = m.params.iter().map(|p| fmt_param(p, scope)).collect();
    let ret = match &m.ret {
        Some(t) => format!(": {}", fmt_type(t, scope)),
        None => String::new(),
    };
    format!(
        "{}{}::{}({}){}",
        if m.by_ref_return { "& " } else { "" },
        class_display,
        m.name,
        params.join(", "),
        ret
    )
}

// ---------------------------------------------------------------------------
// Linked world (resolved tables) + class graph
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct RMethod {
    /// Display name of the class where the method body was declared
    /// (the "Declaration of X::m" / "as in class X" name).
    decl: String,
    /// Parent display of the declaring class (for `parent` type resolution).
    scope_parent: Option<String>,
    m: MethodModel,
}

#[derive(Clone, Debug)]
struct RProp {
    decl: String,
    p: PropModel,
}

#[derive(Clone, Debug)]
struct RConst {
    decl: String,
    k: ConstModel,
}

#[derive(Clone, Debug)]
struct Linked {
    /// Lowercased names of all ancestors (parents + interfaces, transitive).
    ancestors: HashSet<String>,
    methods: Vec<RMethod>,
    props: Vec<RProp>,
    consts: Vec<RConst>,
    /// A-DS59: the interface-ctor prototype constraint. Propagates to
    /// grandchildren (fixture x6). Display name of the interface + its ctor.
    ctor_proto: Option<(String, MethodModel)>,
}

struct Graph<'a> {
    map: &'a HashMap<String, Linked>,
    cur_lc: String,
    cur_ancestors: &'a HashSet<String>,
}

impl Graph<'_> {
    fn exists(&self, name: &str) -> bool {
        let l = lc(name);
        l == self.cur_lc || self.map.contains_key(&l)
    }
    /// `a` is-a `b` (a exists; b assumed named). Reflexive.
    fn is_sub(&self, a: &str, b: &str) -> bool {
        let (al, bl) = (lc(a), lc(b));
        if al == bl {
            return true;
        }
        let anc = if al == self.cur_lc {
            self.cur_ancestors
        } else if let Some(x) = self.map.get(&al) {
            &x.ancestors
        } else {
            return false;
        };
        anc.contains(&bl)
    }
}

// ---------------------------------------------------------------------------
// Subtype (variance) engine
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum SubR {
    Yes,
    No,
    /// A class name involved in the check could not be resolved
    /// ("because class X is not available", fixture v15).
    Unres(String),
}

#[derive(Clone, Debug, PartialEq)]
enum RAtom {
    Named(String),
    /// `static` declared in the given class scope.
    StaticOf(String),
    Callable,
    Object,
    Array,
    Iterable,
    Str,
    Int,
    Float,
    Bool,
    False,
    True,
    Null,
    Void,
    Never,
    Mixed,
}

#[derive(Clone, Debug, PartialEq)]
enum RMember {
    Atom(RAtom),
    Inter(Vec<RAtom>),
}

fn resolve_atom(ty: &Ty, scope: &Scope) -> RAtom {
    match ty {
        Ty::Named(n) => RAtom::Named(n.clone()),
        Ty::SelfTy => RAtom::Named(scope.class.clone()),
        Ty::ParentTy => RAtom::Named(scope.parent.clone().unwrap_or_else(|| "parent".to_string())),
        Ty::StaticTy => RAtom::StaticOf(scope.class.clone()),
        Ty::Callable => RAtom::Callable,
        Ty::Object => RAtom::Object,
        Ty::Array => RAtom::Array,
        Ty::Iterable => RAtom::Iterable,
        Ty::Str => RAtom::Str,
        Ty::Int => RAtom::Int,
        Ty::Float => RAtom::Float,
        Ty::Bool => RAtom::Bool,
        Ty::False => RAtom::False,
        Ty::True => RAtom::True,
        Ty::Null => RAtom::Null,
        Ty::Void => RAtom::Void,
        Ty::Never => RAtom::Never,
        Ty::Mixed => RAtom::Mixed,
        Ty::Nullable(_) | Ty::Union(_) | Ty::Intersection(_) => {
            unreachable!("resolve_atom on compound")
        }
    }
}

fn resolve_members(ty: &Ty, scope: &Scope, out: &mut Vec<RMember>) {
    match ty {
        Ty::Union(members) => {
            for m in members {
                resolve_members(m, scope, out);
            }
        }
        Ty::Nullable(inner) => {
            resolve_members(inner, scope, out);
            out.push(RMember::Atom(RAtom::Null));
        }
        Ty::Intersection(elems) => {
            out.push(RMember::Inter(elems.iter().map(|e| resolve_atom(e, scope)).collect()));
        }
        atom => out.push(RMember::Atom(resolve_atom(atom, scope))),
    }
}

fn atom_sub(c: &RAtom, p: &RAtom, g: &Graph) -> SubR {
    use RAtom::*;
    if matches!(c, Never) {
        return SubR::Yes;
    }
    match p {
        Mixed => {
            if matches!(c, Void) {
                SubR::No
            } else {
                SubR::Yes
            }
        }
        Bool => {
            if matches!(c, Bool | False | True) {
                SubR::Yes
            } else {
                SubR::No
            }
        }
        Iterable => match c {
            Array | Iterable => SubR::Yes,
            Named(a) => {
                if !g.exists(a) {
                    SubR::Unres(a.clone())
                } else if g.is_sub(a, "Traversable") {
                    SubR::Yes
                } else {
                    SubR::No
                }
            }
            _ => SubR::No,
        },
        Object => {
            if matches!(c, Object | Named(_) | StaticOf(_)) {
                SubR::Yes
            } else {
                SubR::No
            }
        }
        Named(b) => match c {
            Named(a) => {
                if lc(a) == lc(b) {
                    SubR::Yes
                } else if !g.exists(a) {
                    // Child-side class checked first: v15 reports B.
                    SubR::Unres(a.clone())
                } else if !g.exists(b) {
                    SubR::Unres(b.clone())
                } else if g.is_sub(a, b) {
                    SubR::Yes
                } else {
                    SubR::No
                }
            }
            StaticOf(s) => {
                if !g.exists(b) {
                    SubR::Unres(b.clone())
                } else if g.is_sub(s, b) {
                    SubR::Yes
                } else {
                    SubR::No
                }
            }
            _ => SubR::No,
        },
        // Parent `static`: only `static` (or never) on the child side.
        // Neither the class name (n3) nor resolved `self` (w4) satisfies it.
        StaticOf(_) => {
            if matches!(c, StaticOf(_)) {
                SubR::Yes
            } else {
                SubR::No
            }
        }
        _ => {
            if std::mem::discriminant(c) == std::mem::discriminant(p) {
                SubR::Yes
            } else {
                SubR::No
            }
        }
    }
}

/// ∀-combinator: all must be Yes; No dominates Unres.
fn all_sub<I: Iterator<Item = SubR>>(iter: I) -> SubR {
    let mut unres: Option<String> = None;
    for r in iter {
        match r {
            SubR::Yes => {}
            SubR::No => return SubR::No,
            SubR::Unres(n) => {
                if unres.is_none() {
                    unres = Some(n);
                }
            }
        }
    }
    match unres {
        Some(n) => SubR::Unres(n),
        None => SubR::Yes,
    }
}

/// ∃-combinator: any Yes wins; Unres dominates No.
fn any_sub<I: Iterator<Item = SubR>>(iter: I) -> SubR {
    let mut unres: Option<String> = None;
    for r in iter {
        match r {
            SubR::Yes => return SubR::Yes,
            SubR::No => {}
            SubR::Unres(n) => {
                if unres.is_none() {
                    unres = Some(n);
                }
            }
        }
    }
    match unres {
        Some(n) => SubR::Unres(n),
        None => SubR::No,
    }
}

fn member_sub_member(cm: &RMember, pm: &RMember, g: &Graph) -> SubR {
    match (cm, pm) {
        (RMember::Atom(c), RMember::Atom(p)) => atom_sub(c, p, g),
        // atom ⊆ (A&B): must be subtype of every intersection element (x3).
        (RMember::Atom(c), RMember::Inter(ps)) => all_sub(ps.iter().map(|p| atom_sub(c, p, g))),
        // (A&B) ⊆ atom: any component suffices.
        (RMember::Inter(cs), RMember::Atom(p)) => any_sub(cs.iter().map(|c| atom_sub(c, p, g))),
        // (A&B) ⊆ (X&Y): every parent element covered by some child element.
        (RMember::Inter(cs), RMember::Inter(ps)) => {
            all_sub(ps.iter().map(|p| any_sub(cs.iter().map(|c| atom_sub(c, p, g)))))
        }
    }
}

/// `child_ty` ⊆ `parent_ty` (covariance direction). For parameter
/// contravariance the caller swaps sides.
fn is_covariant(child_ty: &Ty, cs: &Scope, parent_ty: &Ty, ps: &Scope, g: &Graph) -> SubR {
    let mut cms = vec![];
    resolve_members(child_ty, cs, &mut cms);
    let mut pms = vec![];
    resolve_members(parent_ty, ps, &mut pms);
    all_sub(cms.iter().map(|cm| any_sub(pms.iter().map(|pm| member_sub_member(cm, pm, g)))))
}

// ---------------------------------------------------------------------------
// Implementation check (zend_do_perform_implementation_check model)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Compat {
    Ok,
    /// Arity / by-ref / parameter-type incompatibility.
    Mismatch,
    /// Everything but the return type is compatible (tentative candidate).
    ReturnOnly,
    Unresolved(String),
}

fn required_count(m: &MethodModel) -> usize {
    let mut n = 0;
    for p in &m.params {
        if p.has_default || p.variadic {
            break;
        }
        n += 1;
    }
    n
}

fn perform_impl_check(
    child: &MethodModel,
    cs: &Scope,
    parent: &MethodModel,
    ps: &Scope,
    g: &Graph,
) -> Compat {
    // By-ref return: parent by-ref requires child by-ref.
    if parent.by_ref_return && !child.by_ref_return {
        return Compat::Mismatch;
    }

    // Arity: required params may not grow (v5); child must accept every
    // parent arg unless variadic (v6 absorbs); parent variadic needs child
    // variadic.
    if required_count(child) > required_count(parent) {
        return Compat::Mismatch;
    }
    let p_fixed: Vec<&Param> = parent.params.iter().filter(|p| !p.variadic).collect();
    let c_fixed: Vec<&Param> = child.params.iter().filter(|p| !p.variadic).collect();
    let c_variadic = child.params.iter().find(|p| p.variadic);
    let p_variadic = parent.params.iter().find(|p| p.variadic);
    if p_fixed.len() > c_fixed.len() && c_variadic.is_none() {
        return Compat::Mismatch;
    }
    if p_variadic.is_some() && c_variadic.is_none() {
        return Compat::Mismatch;
    }

    // Per-parameter: by-ref must match exactly (v3/v4); types are
    // CONTRAVARIANT — parent type ⊆ child type; a typed child over an
    // untyped parent param narrows (n2) unless the child type is mixed.
    let check_param_pair = |pp: &Param, cp: &Param| -> Compat {
        if cp.by_ref != pp.by_ref {
            return Compat::Mismatch;
        }
        match (&pp.ty, &cp.ty) {
            (_, None) => Compat::Ok,
            (None, Some(ct)) => match is_covariant(&Ty::Mixed, ps, ct, cs, g) {
                SubR::Yes => Compat::Ok,
                SubR::No => Compat::Mismatch,
                SubR::Unres(n) => Compat::Unresolved(n),
            },
            (Some(pt), Some(ct)) => match is_covariant(pt, ps, ct, cs, g) {
                SubR::Yes => Compat::Ok,
                SubR::No => Compat::Mismatch,
                SubR::Unres(n) => Compat::Unresolved(n),
            },
        }
    };
    for (i, pp) in p_fixed.iter().enumerate() {
        let cp: &Param = match c_fixed.get(i) {
            Some(cp) => cp,
            None => match c_variadic {
                Some(v) => v,
                None => return Compat::Mismatch,
            },
        };
        match check_param_pair(pp, cp) {
            Compat::Ok => {}
            other => return other,
        }
    }
    if let (Some(pv), Some(cv)) = (p_variadic, c_variadic) {
        match check_param_pair(pv, cv) {
            Compat::Ok => {}
            other => return other,
        }
    }

    // Return type: COVARIANT — child ⊆ parent; parent typed + child untyped
    // is a return-type failure (tentative channel, p5).
    match (&parent.ret, &child.ret) {
        (None, _) => Compat::Ok,
        (Some(_), None) => Compat::ReturnOnly,
        (Some(pt), Some(ct)) => match is_covariant(ct, cs, pt, ps, g) {
            SubR::Yes => Compat::Ok,
            SubR::No => Compat::ReturnOnly,
            SubR::Unres(n) => Compat::Unresolved(n),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn sig_check(
    child_display: &str,
    child: &MethodModel,
    cs: &Scope,
    parent_display: &str,
    parent: &MethodModel,
    ps: &Scope,
    g: &Graph,
    deps: &mut Vec<String>,
) -> Result<(), String> {
    match perform_impl_check(child, cs, parent, ps, g) {
        Compat::Ok => Ok(()),
        Compat::ReturnOnly if parent.tentative_return => {
            if child.rtwc {
                return Ok(()); // p6: #[\ReturnTypeWillChange] suppresses.
            }
            deps.push(format!(
                "Return type of {}::{}() should either be compatible with {}::{}(): {}, \
                 or the #[\\ReturnTypeWillChange] attribute should be used to temporarily \
                 suppress the notice",
                child_display,
                child.name,
                parent_display,
                parent.name,
                fmt_type(parent.ret.as_ref().expect("tentative parent has ret"), ps)
            ));
            Ok(())
        }
        Compat::Mismatch | Compat::ReturnOnly => Err(format!(
            "Declaration of {} must be compatible with {}",
            method_sig(child_display, child, cs),
            method_sig(parent_display, parent, ps)
        )),
        Compat::Unresolved(n) => Err(format!(
            "Could not check compatibility between {} and {}, because class {} is not available",
            method_sig(child_display, child, cs),
            method_sig(parent_display, parent, ps),
            n
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn check_override(
    cur_class: &str,
    child: &RMethod,
    parent_decl: &str,
    parent_scope_parent: Option<&str>,
    parent: &MethodModel,
    parent_is_interface: bool,
    ctor_proto: Option<&(String, MethodModel)>,
    g: &Graph,
    deps: &mut Vec<String>,
) -> Result<(), String> {
    // Private parent methods are exempt from every check (p9).
    if parent.visibility == Visibility::Private {
        return Ok(());
    }
    if parent.is_final {
        return Err(format!("Cannot override final method {}::{}()", parent_decl, parent.name));
    }

    let cs = Scope { class: child.decl.clone(), parent: child.scope_parent.clone() };
    let ps = Scope { class: parent_decl.to_string(), parent: parent_scope_parent.map(str::to_string) };

    if lc(&child.m.name) == "__construct" {
        // A-DS59 (KS-DS-93-2): the ctor exemption is keyed on the PROTOTYPE,
        // not on the class. Check ONLY when the prototype comes from an
        // interface (directly, or propagated — x6) or the parent ctor is
        // declared abstract (v14). A concrete ctor in an abstract class is
        // EXEMPT (q4a pattern).
        if parent_is_interface {
            return sig_check(&child.decl, &child.m, &cs, parent_decl, parent, &ps, g, deps);
        }
        if let Some((ipd, ipm)) = ctor_proto {
            let ips = Scope { class: ipd.clone(), parent: None };
            return sig_check(&child.decl, &child.m, &cs, ipd, ipm, &ips, g, deps);
        }
        if parent.is_abstract {
            return sig_check(&child.decl, &child.m, &cs, parent_decl, parent, &ps, g, deps);
        }
        return Ok(()); // EXEMPT (p8, q4a).
    }

    if parent.is_static != child.m.is_static {
        return Err(if parent.is_static {
            format!(
                "Cannot make static method {}::{}() non static in class {}",
                parent_decl, parent.name, cur_class
            )
        } else {
            format!(
                "Cannot make non static method {}::{}() static in class {}",
                parent_decl, parent.name, cur_class
            )
        });
    }

    if vis_rank(child.m.visibility) < vis_rank(parent.visibility) {
        return Err(format!(
            "Access level to {}::{}() must be {} (as in class {})",
            child.decl,
            child.m.name,
            vis_str(parent.visibility),
            parent_decl
        ));
    }

    sig_check(&child.decl, &child.m, &cs, parent_decl, parent, &ps, g, deps)
}

// ---------------------------------------------------------------------------
// Property / const checks
// ---------------------------------------------------------------------------

fn hook_pseudo_method(p: &PropModel, h: &Hook) -> MethodModel {
    MethodModel {
        name: format!("${}::get", p.name),
        visibility: Visibility::Public,
        is_static: false,
        is_final: false,
        is_abstract: false,
        by_ref_return: h.by_ref,
        params: vec![],
        ret: p.ty.clone(),
        tentative_return: false,
        rtwc: false,
    }
}

fn check_prop(
    cur_class: &str,
    cur_parent: Option<&str>,
    own: &PropModel,
    parent_decl: &str,
    pp: &PropModel,
    g: &Graph,
) -> Result<(), String> {
    let cs = Scope { class: cur_class.to_string(), parent: cur_parent.map(str::to_string) };
    let ps = Scope { class: parent_decl.to_string(), parent: None };

    if !pp.hooks.is_empty() && !own.hooks.is_empty() {
        // Hooked (virtual) properties: the GET hook is checked covariantly as
        // a pseudo-method "C::$x::get(): T" (w5), with by-ref printed as a
        // "& " prefix (x4, A-DS60 rule 4).
        let pg = pp.hooks.iter().find(|h| h.kind == HookKind::Get);
        let cg = own.hooks.iter().find(|h| h.kind == HookKind::Get);
        if let (Some(pg), Some(cg)) = (pg, cg) {
            let pm = hook_pseudo_method(pp, pg);
            let cm = hook_pseudo_method(own, cg);
            match perform_impl_check(&cm, &cs, &pm, &ps, g) {
                Compat::Ok => {}
                Compat::Unresolved(n) => {
                    return Err(format!(
                        "Could not check compatibility between {} and {}, because class {} is not available",
                        method_sig(cur_class, &cm, &cs),
                        method_sig(parent_decl, &pm, &ps),
                        n
                    ));
                }
                _ => {
                    return Err(format!(
                        "Declaration of {} must be compatible with {}",
                        method_sig(cur_class, &cm, &cs),
                        method_sig(parent_decl, &pm, &ps)
                    ));
                }
            }
        }
        return Ok(());
    }

    // readonly must match (v11, w7).
    if pp.readonly && !own.readonly {
        return Err(format!(
            "Cannot redeclare readonly property {}::${} as non-readonly {}::${}",
            parent_decl, pp.name, cur_class, own.name
        ));
    }
    if !pp.readonly && own.readonly {
        return Err(format!(
            "Cannot redeclare non-readonly property {}::${} as readonly {}::${}",
            parent_decl, pp.name, cur_class, own.name
        ));
    }

    // Property types are INVARIANT (n8/v12).
    match (&pp.ty, &own.ty) {
        (None, None) => Ok(()),
        (None, Some(_)) => Err(format!(
            "Type of {}::${} must be omitted to match the parent definition in class {}",
            cur_class, own.name, parent_decl
        )),
        (Some(pt), None) => Err(format!(
            "Type of {}::${} must be {} (as in class {})",
            cur_class,
            own.name,
            fmt_type(pt, &ps),
            parent_decl
        )),
        (Some(pt), Some(ct)) => {
            let a = is_covariant(ct, &cs, pt, &ps, g);
            let b = is_covariant(pt, &ps, ct, &cs, g);
            if a == SubR::Yes && b == SubR::Yes {
                Ok(())
            } else {
                Err(format!(
                    "Type of {}::${} must be {} (as in class {})",
                    cur_class,
                    own.name,
                    fmt_type(pt, &ps),
                    parent_decl
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// World linking (the pure entry point)
// ---------------------------------------------------------------------------

/// Link the hand-built classes IN ORDER, running every inheritance check.
/// Returns `Ok(deprecations)` (empty when fully clean) or the first fatal's
/// exact Zend core message as `Err`.
pub fn link(classes: &[ClassModel]) -> Result<Vec<String>, String> {
    let mut map: HashMap<String, Linked> = HashMap::new();
    let mut deps: Vec<String> = vec![];
    for c in classes {
        let linked = link_one(c, &map, &mut deps)?;
        map.insert(lc(&c.name), linked);
    }
    Ok(deps)
}

fn link_one(
    c: &ClassModel,
    map: &HashMap<String, Linked>,
    deps: &mut Vec<String>,
) -> Result<Linked, String> {
    // Ancestor set (parents + interfaces, transitive) — computed up front so
    // subtype checks involving the class being linked resolve (p3).
    let mut ancestors: HashSet<String> = HashSet::new();
    if let Some(pn) = &c.parent {
        if let Some(pl) = map.get(&lc(pn)) {
            ancestors.insert(lc(pn));
            ancestors.extend(pl.ancestors.iter().cloned());
        }
    }
    for iname in &c.interfaces {
        if let Some(il) = map.get(&lc(iname)) {
            ancestors.insert(lc(iname));
            ancestors.extend(il.ancestors.iter().cloned());
        }
    }
    let g = Graph { map, cur_lc: lc(&c.name), cur_ancestors: &ancestors };
    let cur_scope = Scope { class: c.name.clone(), parent: c.parent.clone() };

    // 1. Enum methods must not be abstract (x1).
    if c.kind == ClassKind::Enum {
        for m in &c.methods {
            if m.is_abstract {
                return Err(format!("Enum method {}::{}() must not be abstract", c.name, m.name));
            }
        }
    }

    // 2. Intra-class hook-set parameter check (x2, A-DS60 rule 5: NOT the
    //    "Declaration of" family). The set param must be able to accept every
    //    value the property type admits: property type ⊆ set param type.
    for p in &c.props {
        if let Some(pt) = &p.ty {
            for h in &p.hooks {
                if h.kind == HookKind::Set {
                    if let Some(hp) = &h.param {
                        if let Some(hpt) = &hp.ty {
                            if is_covariant(pt, &cur_scope, hpt, &cur_scope, &g) != SubR::Yes {
                                return Err(format!(
                                    "Type of parameter ${} of hook {}::${}::set must be compatible with property type",
                                    hp.name, c.name, p.name
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Own tables.
    let mut methods: Vec<RMethod> = c
        .methods
        .iter()
        .map(|m| RMethod { decl: c.name.clone(), scope_parent: c.parent.clone(), m: m.clone() })
        .collect();
    let mut props: Vec<RProp> = c
        .props
        .iter()
        .map(|p| RProp { decl: c.name.clone(), p: p.clone() })
        .collect();
    let mut consts: Vec<RConst> = c
        .consts
        .iter()
        .map(|k| RConst { decl: c.name.clone(), k: k.clone() })
        .collect();

    // 3. Traits: an ABSTRACT trait method constrains the class's own method
    //    (n9 — message names the trait). Concrete trait methods lose to the
    //    class's own method silently.
    for tn in &c.traits {
        if let Some(tl) = map.get(&lc(tn)) {
            let mut imported: Vec<RMethod> = vec![];
            for tm in &tl.methods {
                if let Some(own) = methods.iter().find(|m| lc(&m.m.name) == lc(&tm.m.name)) {
                    if tm.m.is_abstract {
                        check_override(
                            &c.name,
                            own,
                            &tm.decl,
                            tm.scope_parent.as_deref(),
                            &tm.m,
                            false,
                            None,
                            &g,
                            deps,
                        )?;
                    }
                } else if !tm.m.is_abstract {
                    imported.push(tm.clone());
                }
            }
            methods.extend(imported);
        }
    }

    // 4. Parent inheritance.
    let mut ctor_proto: Option<(String, MethodModel)> = None;
    if let Some(pn) = &c.parent {
        if let Some(pl) = map.get(&lc(pn)) {
            ctor_proto = pl.ctor_proto.clone(); // A-DS59: propagates (x6).
            let mut inherited: Vec<RMethod> = vec![];
            for pm in &pl.methods {
                if let Some(own) = methods.iter().find(|m| lc(&m.m.name) == lc(&pm.m.name)) {
                    check_override(
                        &c.name,
                        own,
                        &pm.decl,
                        pm.scope_parent.as_deref(),
                        &pm.m,
                        false,
                        ctor_proto.as_ref(),
                        &g,
                        deps,
                    )?;
                } else {
                    inherited.push(pm.clone());
                }
            }
            methods.extend(inherited);

            let mut inh_props: Vec<RProp> = vec![];
            for pp in &pl.props {
                if let Some(own) = c.props.iter().find(|p| p.name == pp.p.name) {
                    check_prop(&c.name, c.parent.as_deref(), own, &pp.decl, &pp.p, &g)?;
                } else {
                    inh_props.push(pp.clone());
                }
            }
            props.extend(inh_props);

            let mut inh_consts: Vec<RConst> = vec![];
            for pc in &pl.consts {
                if c.consts.iter().any(|k| k.name == pc.k.name) {
                    if pc.k.is_final {
                        return Err(format!(
                            "{}::{} cannot override final constant {}::{}",
                            c.name, pc.k.name, pc.decl, pc.k.name
                        ));
                    }
                } else {
                    inh_consts.push(pc.clone());
                }
            }
            consts.extend(inh_consts);
        }
    }

    // 5. Interfaces, in DECLARED order — a multi-interface conflict names the
    //    guilty interface (w1). Also the interface-extends-interface path (w2).
    for iname in &c.interfaces {
        if let Some(il) = map.get(&lc(iname)) {
            let mut imported: Vec<RMethod> = vec![];
            for im in &il.methods {
                let is_ctor = lc(&im.m.name) == "__construct";
                if let Some(own) = methods.iter().find(|m| lc(&m.m.name) == lc(&im.m.name)) {
                    check_override(
                        &c.name,
                        own,
                        &im.decl,
                        im.scope_parent.as_deref(),
                        &im.m,
                        true,
                        None,
                        &g,
                        deps,
                    )?;
                } else if c.kind == ClassKind::Interface {
                    imported.push(im.clone());
                }
                if is_ctor && ctor_proto.is_none() {
                    // A-DS59: record the interface-ctor constraint for
                    // descendants (x6).
                    ctor_proto = Some((im.decl.clone(), im.m.clone()));
                }
            }
            methods.extend(imported);
        }
    }

    Ok(Linked { ancestors, methods, props, consts, ctor_proto })
}

// ---------------------------------------------------------------------------
// Tests — one unit per fixture (named after the fixture), oracled against
// /opt/homebrew/opt/php/bin/php 8.5.7 on the committed fixture files.
//
// t3_conditional_skipped: WIRING-ONLY, skipped by design — class C is never
// declared at runtime (the conditional is false), so there is nothing for a
// PURE checker to model; the fixture judges declaration TIMING, which is
// fase 2 (wiring). t1/t2/t4 below model only their signature-compat aspect
// (hoisting order and interleaved output are likewise fase-2 concerns).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fatal(classes: &[ClassModel], msg: &str) {
        assert_eq!(link(classes), Err(msg.to_string()));
    }

    fn assert_clean(classes: &[ClassModel]) {
        assert_eq!(link(classes), Ok(vec![]));
    }

    fn assert_deprecated(classes: &[ClassModel], msg: &str) {
        assert_eq!(link(classes), Ok(vec![msg.to_string()]));
    }

    fn scope(class: &str) -> Scope {
        Scope { class: class.to_string(), parent: None }
    }

    // ----- wp89-harness/fixtures-ds35 (n1..n9 negatives) -----

    #[test]
    fn n1_return_incompat() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::Int)),
                class("C").extends("P").with_method(method("m").returns(Ty::Str)),
            ],
            "Declaration of C::m(): string must be compatible with P::m(): int",
        );
    }

    #[test]
    fn n2_param_narrow() {
        assert_fatal(
            &[
                class("P").with_method(method("m").arg(param("x"))),
                class("C").extends("P").with_method(method("m").arg(param("x").of(Ty::Int))),
            ],
            "Declaration of C::m(int $x) must be compatible with P::m($x)",
        );
    }

    #[test]
    fn n3_static_widen() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::StaticTy)),
                class("C").extends("P").with_method(method("m").returns(Ty::named("P"))),
            ],
            "Declaration of C::m(): P must be compatible with P::m(): static",
        );
    }

    #[test]
    fn n4_union_superset() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::union(vec![Ty::Int, Ty::Str]))),
                class("C")
                    .extends("P")
                    .with_method(method("m").returns(Ty::union(vec![Ty::Int, Ty::Str, Ty::Float]))),
            ],
            "Declaration of C::m(): string|int|float must be compatible with P::m(): string|int",
        );
    }

    #[test]
    fn n5_static_mismatch() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::Int)),
                class("C").extends("P").with_method(method("m").static_().returns(Ty::Int)),
            ],
            "Cannot make non static method P::m() static in class C",
        );
    }

    #[test]
    fn n6_visibility() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::Int)),
                class("C").extends("P").with_method(method("m").protected_().returns(Ty::Int)),
            ],
            "Access level to C::m() must be public (as in class P)",
        );
    }

    #[test]
    fn n7_final() {
        assert_fatal(
            &[
                class("P").with_method(method("m").final_().returns(Ty::Int)),
                class("C").extends("P").with_method(method("m").returns(Ty::Int)),
            ],
            "Cannot override final method P::m()",
        );
    }

    #[test]
    fn n8_prop_type() {
        assert_fatal(
            &[
                class("P").with_prop(prop("x").of(Ty::Int)),
                class("C").extends("P").with_prop(prop("x").of(Ty::Str)),
            ],
            "Type of C::$x must be int (as in class P)",
        );
    }

    #[test]
    fn n9_trait_abstract() {
        assert_fatal(
            &[
                trait_("T").with_method(method("f").abstract_().returns(Ty::Str)),
                class("C").uses("T").with_method(method("f").returns(Ty::Int)),
            ],
            "Declaration of C::f(): int must be compatible with T::f(): string",
        );
    }

    // ----- wp89-harness/fixtures-ds35 (p1..p9 positives) -----

    #[test]
    fn p1_covariant_return() {
        assert_clean(&[
            class("A"),
            class("B").extends("A"),
            class("P").with_method(method("m").returns(Ty::named("A"))),
            class("C").extends("P").with_method(method("m").returns(Ty::named("B"))),
        ]);
    }

    #[test]
    fn p2_param_widen() {
        assert_clean(&[
            class("P").with_method(method("m").arg(param("x").of(Ty::Int))),
            class("C").extends("P").with_method(method("m").arg(param("x").with_default("0"))),
        ]);
    }

    #[test]
    fn p3_static_return() {
        assert_clean(&[
            class("P").with_method(method("m").returns(Ty::SelfTy)),
            class("C").extends("P").with_method(method("m").returns(Ty::StaticTy)),
        ]);
    }

    #[test]
    fn p4_union_subset() {
        assert_clean(&[
            class("P").with_method(method("m").returns(Ty::union(vec![Ty::Int, Ty::Str, Ty::Null]))),
            class("C").extends("P").with_method(method("m").returns(Ty::nullable(Ty::Int))),
        ]);
    }

    #[test]
    fn p5_tentative_return() {
        assert_deprecated(
            &[
                interface("JsonSerializable")
                    .with_method(method("jsonSerialize").returns(Ty::Mixed).tentative_()),
                class("J")
                    .implements("JsonSerializable")
                    .with_method(method("jsonSerialize")),
            ],
            "Return type of J::jsonSerialize() should either be compatible with \
             JsonSerializable::jsonSerialize(): mixed, or the #[\\ReturnTypeWillChange] \
             attribute should be used to temporarily suppress the notice",
        );
    }

    #[test]
    fn p6_rtwc_suppress() {
        assert_clean(&[
            interface("JsonSerializable")
                .with_method(method("jsonSerialize").returns(Ty::Mixed).tentative_()),
            class("J")
                .implements("JsonSerializable")
                .with_method(method("jsonSerialize").rtwc_()),
        ]);
    }

    #[test]
    fn p7_never_covariant() {
        assert_clean(&[
            class("P").with_method(method("m").returns(Ty::Str)),
            class("C").extends("P").with_method(method("m").returns(Ty::Never)),
        ]);
    }

    #[test]
    fn p8_ctor_exempt() {
        assert_clean(&[
            class("P").with_method(method("__construct").arg(param("x").of(Ty::Int))),
            class("C").extends("P").with_method(
                method("__construct")
                    .arg(param("y").of(Ty::Str))
                    .arg(param("z").with_default("null")),
            ),
        ]);
    }

    #[test]
    fn p9_private_exempt() {
        assert_clean(&[
            class("P")
                .with_method(method("m").private_().returns(Ty::Str))
                .with_method(method("call")),
            class("C").extends("P").with_method(method("m").private_().returns(Ty::Int)),
        ]);
    }

    // ----- wp90-harness/fixtures-ds35-v2 (t1..t4 timing/lowering) -----
    // Only the signature-compat aspect is modeled here; declaration timing,
    // hoisting and interleaved output are WIRING (fase 2).

    #[test]
    fn t1_parent_after_sig_aspect() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::Str)),
                class("C").extends("P").with_method(method("m").returns(Ty::Int)),
            ],
            "Declaration of C::m(): int must be compatible with P::m(): string",
        );
    }

    #[test]
    fn t2_conditional_executed_sig_aspect() {
        assert_fatal(
            &[
                class("P")
                    .with_method(method("m").arg(param("x").of(Ty::union(vec![Ty::Int, Ty::Str])))),
                class("C").extends("P").with_method(method("m").arg(param("x").of(Ty::Int))),
            ],
            "Declaration of C::m(int $x) must be compatible with P::m(string|int $x)",
        );
    }

    // t3_conditional_skipped: SKIPPED — wiring-only (see module comment).

    #[test]
    fn t4_hoisted_with_output_sig_aspect() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::Str)),
                class("C").extends("P").with_method(method("m").returns(Ty::Int)),
            ],
            "Declaration of C::m(): int must be compatible with P::m(): string",
        );
    }

    // ----- wp90-harness/fixtures-ds35-v2 (v1..v15) -----

    #[test]
    fn v1_byref_widen() {
        assert_clean(&[
            class("P").with_method(method("m").arg(param("x").of(Ty::Int).by_ref_())),
            class("C").extends("P").with_method(
                method("m").arg(param("x").of(Ty::union(vec![Ty::Int, Ty::Str])).by_ref_()),
            ),
        ]);
    }

    #[test]
    fn v2_byref_narrow() {
        assert_fatal(
            &[
                class("P").with_method(
                    method("m").arg(param("x").of(Ty::union(vec![Ty::Int, Ty::Str])).by_ref_()),
                ),
                class("C")
                    .extends("P")
                    .with_method(method("m").arg(param("x").of(Ty::Int).by_ref_())),
            ],
            "Declaration of C::m(int &$x) must be compatible with P::m(string|int &$x)",
        );
    }

    #[test]
    fn v3_ref_removed() {
        assert_fatal(
            &[
                class("P").with_method(method("m").arg(param("x").of(Ty::Int).by_ref_())),
                class("C").extends("P").with_method(method("m").arg(param("x").of(Ty::Int))),
            ],
            "Declaration of C::m(int $x) must be compatible with P::m(int &$x)",
        );
    }

    #[test]
    fn v4_ref_added() {
        assert_fatal(
            &[
                class("P").with_method(method("m").arg(param("x").of(Ty::Int))),
                class("C")
                    .extends("P")
                    .with_method(method("m").arg(param("x").of(Ty::Int).by_ref_())),
            ],
            "Declaration of C::m(int &$x) must be compatible with P::m(int $x)",
        );
    }

    #[test]
    fn v5_required_grows() {
        assert_fatal(
            &[
                class("P").with_method(method("m").arg(param("a"))),
                class("C").extends("P").with_method(method("m").arg(param("a")).arg(param("b"))),
            ],
            "Declaration of C::m($a, $b) must be compatible with P::m($a)",
        );
    }

    #[test]
    fn v6_variadic_absorbs() {
        assert_clean(&[
            class("P").with_method(method("m").arg(param("a")).arg(param("b")).arg(param("c"))),
            class("C").extends("P").with_method(method("m").arg(param("rest").variadic_())),
        ]);
    }

    #[test]
    fn v7_nullable_grafia() {
        assert_clean(&[
            class("P").with_method(
                method("m")
                    .arg(param("x").of(Ty::nullable(Ty::Int)))
                    .returns(Ty::nullable(Ty::Int)),
            ),
            class("C").extends("P").with_method(
                method("m")
                    .arg(param("x").of(Ty::union(vec![Ty::Int, Ty::Null])))
                    .returns(Ty::union(vec![Ty::Int, Ty::Null])),
            ),
        ]);
    }

    #[test]
    fn v8_mixed_narrow() {
        assert_clean(&[
            class("P").with_method(method("m").returns(Ty::Mixed)),
            class("C").extends("P").with_method(method("m").returns(Ty::Int)),
        ]);
    }

    #[test]
    fn v9_void_exact() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::Void)),
                class("C").extends("P").with_method(method("m").returns(Ty::Int)),
            ],
            "Declaration of C::m(): int must be compatible with P::m(): void",
        );
    }

    #[test]
    fn v10_static_inverse() {
        assert_fatal(
            &[
                class("P").with_method(method("m").static_()),
                class("C").extends("P").with_method(method("m")),
            ],
            "Cannot make static method P::m() non static in class C",
        );
    }

    #[test]
    fn v11_readonly() {
        assert_fatal(
            &[
                class("P").with_prop(prop("x").of(Ty::Int).readonly_()),
                class("C").extends("P").with_prop(prop("x").of(Ty::Int)),
            ],
            "Cannot redeclare readonly property P::$x as non-readonly C::$x",
        );
    }

    #[test]
    fn v12_prop_typed_on_untyped() {
        assert_fatal(
            &[
                class("P").with_prop(prop("x")),
                class("C").extends("P").with_prop(prop("x").of(Ty::Int)),
            ],
            "Type of C::$x must be omitted to match the parent definition in class P",
        );
    }

    #[test]
    fn v13_iface_ctor() {
        assert_fatal(
            &[
                interface("I").with_method(method("__construct").arg(param("x").of(Ty::Int))),
                class("C")
                    .implements("I")
                    .with_method(method("__construct").arg(param("y").of(Ty::Str))),
            ],
            "Declaration of C::__construct(string $y) must be compatible with I::__construct(int $x)",
        );
    }

    #[test]
    fn v14_abstract_ctor() {
        assert_fatal(
            &[
                abstract_class("P")
                    .with_method(method("__construct").abstract_().arg(param("x").of(Ty::Int))),
                class("C")
                    .extends("P")
                    .with_method(method("__construct").arg(param("y").of(Ty::Str))),
            ],
            "Declaration of C::__construct(string $y) must be compatible with P::__construct(int $x)",
        );
    }

    #[test]
    fn v15_unresolvable() {
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::named("A"))),
                class("C").extends("P").with_method(method("m").returns(Ty::named("B"))),
            ],
            "Could not check compatibility between C::m(): B and P::m(): A, because class B is not available",
        );
    }

    // ----- wp91-harness/fixtures-ds35-v3 (w1..w8) -----

    #[test]
    fn w1_multi_iface() {
        assert_fatal(
            &[
                interface("I1").with_method(method("m").returns(Ty::Int)),
                interface("I2").with_method(method("m").returns(Ty::Str)),
                class("C")
                    .implements("I1")
                    .implements("I2")
                    .with_method(method("m").returns(Ty::Int)),
            ],
            "Declaration of C::m(): int must be compatible with I2::m(): string",
        );
    }

    #[test]
    fn w2_iface_extends() {
        assert_fatal(
            &[
                interface("A")
                    .with_method(method("m").arg(param("x").of(Ty::Int)).returns(Ty::Int)),
                interface("B")
                    .implements("A")
                    .with_method(method("m").arg(param("x").of(Ty::Str)).returns(Ty::Int)),
            ],
            "Declaration of B::m(string $x): int must be compatible with A::m(int $x): int",
        );
    }

    #[test]
    fn w3_enum_implements() {
        assert_fatal(
            &[
                interface("I")
                    .with_method(method("m").arg(param("x").of(Ty::Int)).returns(Ty::Int)),
                enum_("E")
                    .implements("I")
                    .with_method(method("m").arg(param("x").of(Ty::Str)).returns(Ty::Int)),
            ],
            "Declaration of E::m(string $x): int must be compatible with I::m(int $x): int",
        );
    }

    #[test]
    fn w4_self_vs_static() {
        // self prints RESOLVED as the class name: C::m(): C.
        assert_fatal(
            &[
                class("P").with_method(method("m").returns(Ty::StaticTy)),
                class("C").extends("P").with_method(method("m").returns(Ty::SelfTy)),
            ],
            "Declaration of C::m(): C must be compatible with P::m(): static",
        );
    }

    #[test]
    fn w5_hook_subtype() {
        assert_fatal(
            &[
                class("P").with_prop(prop("x").of(Ty::Int).hook(get_hook())),
                class("C").extends("P").with_prop(prop("x").of(Ty::Str).hook(get_hook())),
            ],
            "Declaration of C::$x::get(): string must be compatible with P::$x::get(): int",
        );
    }

    #[test]
    fn w6_final_const() {
        assert_fatal(
            &[
                class("P").with_const(konst("X").final_()),
                class("C").extends("P").with_const(konst("X")),
            ],
            "C::X cannot override final constant P::X",
        );
    }

    #[test]
    fn w7_readonly_promoted() {
        // Promoted ctor params model as plain props; the ctor itself is
        // exempt (concrete, no interface/abstract prototype — A-DS59).
        assert_fatal(
            &[
                class("P")
                    .with_prop(prop("x").of(Ty::Int).readonly_())
                    .with_method(method("__construct").arg(param("x").of(Ty::Int))),
                class("C")
                    .extends("P")
                    .with_prop(prop("x").of(Ty::Int))
                    .with_method(method("__construct").arg(param("x").of(Ty::Int))),
            ],
            "Cannot redeclare readonly property P::$x as non-readonly C::$x",
        );
    }

    #[test]
    fn w8_dnf_positive() {
        assert_clean(&[
            interface("A"),
            interface("B"),
            class("X").implements("A").implements("B"),
            class("P").with_method(method("m").returns(Ty::union(vec![
                Ty::inter(vec![Ty::named("A"), Ty::named("B")]),
                Ty::Str,
            ]))),
            class("C").extends("P").with_method(method("m").returns(Ty::Str)),
        ]);
    }

    // ----- wp92-harness/fixtures-ds35-v4 (x1..x6) -----

    #[test]
    fn x1_enum_abstract() {
        assert_fatal(
            &[enum_("E").with_method(method("m").abstract_().returns(Ty::Int))],
            "Enum method E::m() must not be abstract",
        );
    }

    #[test]
    fn x2_hook_set_param() {
        assert_fatal(
            &[class("C")
                .with_prop(prop("x").of(Ty::Int).hook(set_hook(param("v").of(Ty::Str))))],
            "Type of parameter $v of hook C::$x::set must be compatible with property type",
        );
    }

    #[test]
    fn x3_intersection_return_widen() {
        assert_fatal(
            &[
                interface("A"),
                interface("B"),
                class("P").with_method(
                    method("m").returns(Ty::inter(vec![Ty::named("A"), Ty::named("B")])),
                ),
                class("C").extends("P").with_method(method("m").returns(Ty::named("A"))),
            ],
            "Declaration of C::m(): A must be compatible with P::m(): A&B",
        );
    }

    #[test]
    fn x4_byref_hook_get() {
        assert_fatal(
            &[
                class("P").with_prop(prop("x").of(Ty::Int).hook(get_hook_ref())),
                class("C").extends("P").with_prop(prop("x").of(Ty::Str).hook(get_hook_ref())),
            ],
            "Declaration of & C::$x::get(): string must be compatible with & P::$x::get(): int",
        );
    }

    #[test]
    fn x5_utf8_classname() {
        assert_fatal(
            &[
                class("Città").with_method(method("m").returns(Ty::Int)),
                class("Provincia").extends("Città").with_method(method("m").returns(Ty::Str)),
            ],
            "Declaration of Provincia::m(): string must be compatible with Città::m(): int",
        );
    }

    #[test]
    fn x6_iface_ctor_grandchild() {
        // A-DS59: the interface-ctor constraint PROPAGATES to grandchildren,
        // and the message names the INTERFACE (the prototype), not Mid.
        assert_fatal(
            &[
                interface("I").with_method(method("__construct").arg(param("x").of(Ty::Int))),
                class("Mid")
                    .implements("I")
                    .with_method(method("__construct").arg(param("x").of(Ty::Int))),
                class("C")
                    .extends("Mid")
                    .with_method(method("__construct").arg(param("y").of(Ty::Str))),
            ],
            "Declaration of C::__construct(string $y) must be compatible with I::__construct(int $x)",
        );
    }

    // ----- A-DS59 dedicated unit (KS-DS-93-2, q4a pattern) -----

    #[test]
    fn ctor_concrete_in_abstract_class_is_exempt() {
        // A CONCRETE ctor in an ABSTRACT class carries no interface/abstract
        // prototype — the child ctor is EXEMPT from signature checks. A
        // checker that fatals on this pattern is REJECT (Stogov WP-93).
        assert_clean(&[
            abstract_class("P").with_method(method("__construct").arg(param("x").of(Ty::Int))),
            class("C")
                .extends("P")
                .with_method(method("__construct").arg(param("y").of(Ty::Str))),
        ]);
    }

    // ----- A-DS60 formatting rules (five dedicated units) -----

    #[test]
    fn ds60_rule1_union_intersection_declared_order_no_sorting() {
        // Named/intersection elements print in DECLARED order — zero sorting:
        // "(B&A)|string" stays as declared.
        let t = Ty::union(vec![Ty::inter(vec![Ty::named("B"), Ty::named("A")]), Ty::Str]);
        assert_eq!(fmt_type(&t, &scope("C")), "(B&A)|string");
    }

    #[test]
    fn ds60_rule2_nullable_collapse_binary_union_only() {
        // ?T collapse ONLY for the binary union with null...
        assert_eq!(fmt_type(&Ty::union(vec![Ty::Int, Ty::Null]), &scope("C")), "?int");
        assert_eq!(fmt_type(&Ty::union(vec![Ty::Null, Ty::Int]), &scope("C")), "?int");
        // ...while a wider union prints EXTENDED (null last).
        assert_eq!(
            fmt_type(&Ty::union(vec![Ty::Str, Ty::Int, Ty::Null]), &scope("C")),
            "string|int|null"
        );
    }

    #[test]
    fn ds60_rule3_byref_param_grafia() {
        // "int &$x" — space between type and &, & attached to $x.
        assert_eq!(fmt_param(&param("x").of(Ty::Int).by_ref_(), &scope("C")), "int &$x");
    }

    #[test]
    fn ds60_rule4_byref_hook_prefix_with_space() {
        // By-ref property hooks print the "& " prefix WITH the space.
        let msg = link(&[
            class("P").with_prop(prop("x").of(Ty::Int).hook(get_hook_ref())),
            class("C").extends("P").with_prop(prop("x").of(Ty::Str).hook(get_hook_ref())),
        ])
        .unwrap_err();
        assert_eq!(
            msg,
            "Declaration of & C::$x::get(): string must be compatible with & P::$x::get(): int"
        );
        assert!(msg.contains("& C::$x::get()"), "prefix must keep the space: {msg}");
    }

    #[test]
    fn ds60_rule5_hook_set_family_not_declaration_form() {
        // The hook-set family does NOT use the "Declaration of" form.
        let msg = link(&[class("C")
            .with_prop(prop("x").of(Ty::Int).hook(set_hook(param("v").of(Ty::Str))))])
        .unwrap_err();
        assert_eq!(
            msg,
            "Type of parameter $v of hook C::$x::set must be compatible with property type"
        );
        assert!(!msg.starts_with("Declaration of"));
    }
}
