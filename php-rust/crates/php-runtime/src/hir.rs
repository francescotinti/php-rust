//! High-level Intermediate Representation.
//!
//! The HIR is the resolved, *owned* tree the evaluator walks. It is produced by
//! lowering mago's borrowed-from-arena AST (see [`crate::lower`]). Two things the
//! HIR resolves that the raw AST leaves implicit:
//!
//! - **variable slots**: every distinct direct variable `$name` in a script is
//!   assigned a stable [`Slot`] index (encounter order). The evaluator keeps a
//!   `Vec<Zval>` indexed by slot instead of a name→value map.
//! - **line numbers**: every node carries its 1-based source [`Line`] so the
//!   evaluator can emit PHP-faithful diagnostics (`... on line N`) without
//!   keeping the source around.
//!
//! The HIR owns all its data (no `'arena` lifetime), so it outlives the parser
//! arena and can be cached by a resident process (D-G10).

/// 1-based source line number, for diagnostics.
pub type Line = u32;

/// Index into the per-script local variable slot table ([`Program::slots`]).
pub type Slot = u32;

/// A lowered PHP script.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
    /// The script's file name, as passed to [`crate::lower_source`]. Reproduced
    /// verbatim in rendered diagnostics (`... in <file> on line N`, step 9).
    pub file: Box<[u8]>,
    /// `slots[i]` is the name (without leading `$`) of the variable in slot `i`.
    /// Used for `Undefined variable $name` diagnostics (D-G13).
    pub slots: Vec<Box<[u8]>>,
    /// Top-level user-defined functions, hoisted at lowering time so a call may
    /// precede the declaration (PHP's function hoisting). Resolved by the
    /// evaluator's call path before the builtin registry (step 8).
    pub functions: Vec<FnDecl>,
    /// Indices into `functions` that are **conditional** declarations (a `function`
    /// inside a branch/block, possibly nested in another function/method body):
    /// they are not resolvable by name until their [`StmtKind::DeclareFn`] runs, so
    /// name resolution (compile-time and runtime) skips them. A simple prefix count
    /// can't express this because such declarations interleave with body lowering.
    pub conditional_fns: std::collections::HashSet<usize>,
    /// Indices into `classes` that are **conditional** declarations (a `class` /
    /// `interface` / `enum` inside a branch/block or a function/method body): the
    /// class body is compiled eagerly (so it has a stable `ClassId`), but its name
    /// is not registered until its [`StmtKind::DeclareClass`] runs. Name resolution
    /// (compile-time `class_index` and the runtime clone) skips these, so a `new X`
    /// / static reference before the declaration resolves dynamically (autoload or
    /// "Class not found"), exactly as PHP defers a conditional class to run time.
    pub conditional_classes: std::collections::HashSet<usize>,
    /// Anonymous functions and arrow functions, lowered into one flat table
    /// (step 18, D-18.2). A [`ExprKind::Closure`] selects its body by index;
    /// closures nest by appending to this same vector.
    pub closures: Vec<FnDecl>,
    /// Total number of `static` variable declarations across the whole program
    /// (each gets a unique id). The evaluator sizes its persistent static store
    /// to this (step 15, D-15.2).
    pub static_count: usize,
    /// `declare(strict_types=1)` is in effect — scalar type hints are enforced
    /// without coercion (step 16, D-16.1).
    pub strict: bool,
    /// User-defined classes, hoisted at lowering time so a `new`/method call may
    /// precede the declaration (PHP's class hoisting for unconditional decls).
    /// An [`ExprKind::New`] / method dispatch resolves a class by name against
    /// this table (step 19, D-19.3).
    pub classes: Vec<ClassDecl>,
    /// Traits defined by this unit, keyed by their bare lowercase name (step 21).
    /// Traits are flattened into using classes, so they never enter `classes`;
    /// they are carried here so a later unit (an autoloaded file) can resolve a
    /// `use T` against a trait declared in an earlier unit (the trait analogue of
    /// `seed_classes`). Holds only this unit's *new* traits, not seeded ones.
    pub traits: Vec<(Vec<u8>, LoweredTrait)>,
    /// `#[Attr]` attributes on top-level `const` declarations, keyed by the
    /// constant's fully-qualified name — retained for
    /// `ReflectionConstant::getAttributes()`. Empty for the common case.
    pub const_attributes: Vec<(Box<[u8]>, Vec<HirAttribute>)>,
}

/// A trait lowered to its flattened members (step 21). Stored owned so it can be
/// copied into each consuming class and carried across units via [`Program::traits`].
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredTrait {
    /// The trait's name as written (original case, namespace-qualified). Kept so
    /// runtime trait metadata (`get_declared_traits`, `trait_exists`,
    /// `ReflectionClass::getTraitNames`) can report the real name even though the
    /// trait is flattened into its consumers and keyed by bare lowercase name.
    pub name: Box<[u8]>,
    pub methods: Vec<MethodDecl>,
    pub props: Vec<PropDecl>,
    pub static_props: Vec<StaticPropDecl>,
    pub consts: Vec<ClassConstDecl>,
    /// Names of `abstract` methods the trait requires the consumer to implement.
    pub abstract_methods: Vec<Box<[u8]>>,
    /// The closures (and arrow functions) referenced by this trait's own methods,
    /// in their original unit's index order, so a consumer in another unit can
    /// re-append them and shift the method bodies' closure indices. Empty for a
    /// trait with no closures.
    pub closures: Vec<FnDecl>,
    /// The index of `closures[0]` in the trait's original unit closure table (the
    /// base used to compute the per-consumer shift).
    pub closure_base: u32,
    /// True when this trait was seeded from another unit (so its closures are NOT
    /// in the current unit's table and must be re-appended on flatten).
    pub external: bool,
}

/// Index into [`Program::classes`] (step 19, D-19.3).
pub type ClassId = usize;

/// Which file-loading construct produced an [`ExprKind::Include`] (step 57,
/// Phase 2). `*_once` variants skip a file already loaded; `require*` fatals on a
/// load failure where `include*` only warns and yields `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludeMode {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}

impl IncludeMode {
    /// `include_once` / `require_once`: load at most once per resolved path.
    pub fn is_once(self) -> bool {
        matches!(self, IncludeMode::IncludeOnce | IncludeMode::RequireOnce)
    }
    /// `require` / `require_once`: a load failure is fatal (vs a warning).
    pub fn is_require(self) -> bool {
        matches!(self, IncludeMode::Require | IncludeMode::RequireOnce)
    }
    /// The construct keyword, for diagnostics (`require`, `include_once`, …).
    pub fn keyword(self) -> &'static str {
        match self {
            IncludeMode::Include => "include",
            IncludeMode::IncludeOnce => "include_once",
            IncludeMode::Require => "require",
            IncludeMode::RequireOnce => "require_once",
        }
    }
}

/// Member visibility (step 19, D-19.13). Defaults to `Public`. Used both for
/// access enforcement and for `var_dump`'s `:protected` / `:"C":private`
/// annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

/// A lowered `class Name { props; methods }` (step 19, D-19.4). Inheritance,
/// interfaces, static members and constants are layered on in later sub-steps;
/// 19-1 carries only instance properties and methods.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    /// Name as written (original case); resolved ASCII-case-insensitively.
    pub name: Box<[u8]>,
    /// Superclass (`extends`), resolved to its [`ClassId`] at lowering (step
    /// 19-3, D-19.10). `None` for a root class. Properties and methods are
    /// resolved by walking this chain at runtime, not flattened at lowering.
    pub parent: Option<ClassId>,
    /// Implemented interfaces (`implements`), or for an interface its extended
    /// interfaces — resolved to [`ClassId`]s at lowering (step 19-5, D-19.16/17).
    /// `instanceof` checks them transitively.
    pub interfaces: Vec<ClassId>,
    /// `abstract class` / `interface` — cannot be instantiated (step 19-5).
    pub is_abstract: bool,
    /// `final class` — cannot be extended (enforced at lowering; `ReflectionClass::
    /// isFinal`).
    pub is_final: bool,
    /// `interface` declaration (vs `class`), step 19-5.
    pub is_interface: bool,
    /// Instance properties *declared on this class* (not the inherited ones), in
    /// declaration order. The full instance layout is parent-first, assembled at
    /// `new` time (D-19.10).
    pub props: Vec<PropDecl>,
    /// `static` properties declared on this class (step 19-4, D-19.14). Storage
    /// is per-declaring-class and persists for the run; subclasses share the
    /// inherited cell unless they redeclare.
    pub static_props: Vec<StaticPropDecl>,
    /// Class constants declared on this class (step 19-4, D-19.15). Resolved up
    /// the chain; values are constant expressions evaluated in the declaring
    /// class's context.
    pub consts: Vec<ClassConstDecl>,
    /// Methods declared on this class, in declaration order. Resolved by name
    /// (case-insensitive), walking the parent chain for inheritance.
    pub methods: Vec<MethodDecl>,
    /// Abstract method *signatures* declared on this class — interface methods
    /// and `abstract` methods, which carry no body so are not in `methods`
    /// (step 47). Kept (always public) so `get_class_methods` can report them.
    pub abstract_methods: Vec<Box<[u8]>>,
    /// `enum` declaration (vs `class`/`interface`), step 23. When set, `parent`
    /// is `None`, `props`/`static_props` are empty, and the cases live in
    /// `enum_cases`. The whole OOP machinery (methods, consts, instanceof,
    /// static calls, `$this`) is reused.
    pub is_enum: bool,
    /// For a backed enum (`enum E: string`/`: int`), the backing scalar type.
    /// `None` for a pure enum or a non-enum class (step 23, D-23.10).
    pub enum_backing: Option<EnumBacking>,
    /// The `case` members of an enum, in declaration order (step 23, D-23.1).
    /// Empty for non-enums.
    pub enum_cases: Vec<EnumCaseDecl>,
    /// Attributes declared on this class (`#[Foo(args)]`), in source order. Each
    /// is retained as a constructible `new`-expression so `ReflectionClass::
    /// getAttributes()` + `ReflectionAttribute::newInstance()` work (the path
    /// Symfony Console uses to read `#[AsCommand]`). Empty for the common case.
    pub attributes: Vec<HirAttribute>,
    /// Names (original case, namespace-qualified) of the traits this class uses
    /// *directly* (`use T;`), in source order. Traits are flattened into the class,
    /// but the identities are kept so `class_uses()` / `ReflectionClass::
    /// getTraitNames()` can report them. Empty when the class uses no traits.
    pub uses_traits: Vec<Box<[u8]>>,
    pub line: Line,
}

/// One class attribute `#[Name(args...)]` retained for runtime reflection. The
/// arguments are kept as lowered expressions (evaluated lazily by the VM), so a
/// named/spread argument list reflects exactly as written.
#[derive(Debug, Clone, PartialEq)]
pub struct HirAttribute {
    /// Resolved fully-qualified attribute class name (for `getName()` and the
    /// `getAttributes($name)` filter).
    pub name: Box<[u8]>,
    /// `new Name(args...)` — run by `newInstance()` to build the attribute object.
    pub new_expr: Expr,
    /// An array literal of the arguments (positional → int key, named → string
    /// key) — run by `getArguments()`.
    pub args_expr: Expr,
}

/// Backing scalar type of a backed enum (step 23, D-23.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumBacking {
    Int,
    Str,
}

/// One `case Name;` (pure) or `case Name = expr;` (backed) of an enum
/// (step 23, D-23.1/D-23.4).
#[derive(Debug, Clone, PartialEq)]
pub struct EnumCaseDecl {
    /// Case name (becomes the synthetic `->name` property and the `::Name`
    /// constant-like accessor).
    pub name: Box<[u8]>,
    /// Backing value expression for a backed enum (`None` for a pure case).
    /// Evaluated once when the case singleton is first materialised.
    pub value: Option<Expr>,
}

/// One `static $p = default;` declaration (step 19-4, D-19.14).
#[derive(Debug, Clone, PartialEq)]
pub struct StaticPropDecl {
    /// Name without the leading `$`.
    pub name: Box<[u8]>,
    pub visibility: Visibility,
    pub default: Option<Expr>,
}

/// One `const NAME = expr;` declaration (step 19-4, D-19.15).
#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstDecl {
    pub name: Box<[u8]>,
    pub value: Expr,
    /// Declared visibility (`public`/`protected`/`private`; default public) —
    /// read by `ReflectionClassConstant::isPublic` etc. and the `getConstants`
    /// visibility filter.
    pub visibility: Visibility,
    /// `final const` (PHP 8.1) — `ReflectionClassConstant::isFinal`.
    pub is_final: bool,
    /// Attributes declared on the constant (`#[Foo] const X = …`, PHP 8.3) —
    /// `ReflectionClassConstant::getAttributes`. Empty for the common case.
    pub attributes: Vec<HirAttribute>,
}

/// How a `::`-qualified reference (call, constant, static property, `new`) names
/// its class (step 19-3/19-4).
#[derive(Debug, Clone, PartialEq)]
pub enum ClassRef {
    /// A named class (`Foo::`, `new Foo`), resolved against the class table at
    /// runtime so an unknown class is a runtime fatal (matching PHP).
    Named(Box<[u8]>),
    /// `self::` — the class that *defines* the running method.
    SelfClass,
    /// `parent::` — the parent of the class that defines the running method.
    Parent,
    /// `static::` / `new static` — the late-static-binding class (the runtime
    /// "called" class), step 19-4, D-19.12.
    Static,
    /// A dynamic class reference (step 48): `new $cls`, `$cls::m()`,
    /// `$cls::CONST`, `$obj::m()`. The expression is evaluated at runtime to a
    /// class name (string, leading `\` stripped) or an object (its class). Like
    /// `Named`, a dynamic static call is *non-forwarding* for late static binding.
    Dynamic(Box<Expr>),
}

/// One declared instance property (step 19). `default` is evaluated per instance
/// at `new` time (D-19.6); `None` means the property initialises to NULL.
#[derive(Debug, Clone, PartialEq)]
pub struct PropDecl {
    /// Property name, without the leading `$`.
    pub name: Box<[u8]>,
    pub visibility: Visibility,
    pub default: Option<Expr>,
    /// Declared type (`public int $x`), enforced on write (coerced under weak
    /// typing, `TypeError` otherwise). `None` for an untyped property. A literal
    /// `null` default makes a non-nullable type implicitly nullable (PHP 8.0).
    pub hint: Option<TypeHint>,
    /// PHP 8.4 property hooks (step 50). A `get`/`set` hook is a method-like body
    /// dispatched on read/write; `None` for a plain property. The `set` hook's
    /// `FnDecl` has one parameter (`$value` or the explicit `set($x)` param).
    pub get_hook: Option<FnDecl>,
    pub set_hook: Option<FnDecl>,
    /// Abstract property hooks declared as a contract only (`public abstract $p {
    /// get; set; }`, PHP 8.4) — the hook names (`get`/`set`) with no body. A
    /// concrete subclass must implement them; a non-abstract class that declares
    /// one is itself a fatal (treated like an abstract method). Empty for an
    /// ordinary or fully-concrete property.
    pub abstract_hooks: Vec<Box<[u8]>>,
    /// Whether the property has backing storage. A plain property is backed; a
    /// hooked property is *backed* only if a hook body reads/writes its own
    /// `$this->name` (else it is *virtual*: no slot, omitted from `var_dump`).
    pub backed: bool,
    /// `readonly` (PHP 8.1): the property may be written *once*, from within the
    /// declaring class scope, and never modified again (step: readonly). A second
    /// write — or an out-of-scope write — is a fatal `Error`. Also set for every
    /// property of a `readonly class` (8.2) and for promoted `readonly` params.
    pub readonly: bool,
    /// `#[Attr(args)]` attributes declared on the property, retained for
    /// `ReflectionProperty::getAttributes()` (empty for an unattributed property).
    pub attributes: Vec<HirAttribute>,
}

/// One method (step 19, D-19.5). Wraps an ordinary [`FnDecl`] (so method calls
/// reuse the whole function-frame machinery) plus the slot its body binds `$this`
/// to and the OOP modifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub visibility: Visibility,
    pub is_static: bool,
    /// `final` method — cannot be overridden by a subclass (enforced at lowering;
    /// `ReflectionMethod::isFinal`).
    pub is_final: bool,
    /// Name / parameters / body / local slots — exactly as a free function.
    /// `$this` is *not* a slot: it lowers to [`ExprKind::This`] and is read from
    /// the evaluator's current-object context (D-19.5).
    pub decl: FnDecl,
}

/// A lowered `function name(params) { body }`. Each declaration owns a *local*
/// slot table independent of the script's globals: PHP functions do not see the
/// enclosing scope (no implicit capture), so a call sets up a fresh frame sized
/// by [`FnDecl::slots`] with the parameters occupying its leading slots.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    /// Name as written (original case); calls match it ASCII-case-insensitively.
    pub name: Box<[u8]>,
    /// Formal parameters, in declaration order. `params[i].slot == i`.
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    /// Local variable slot names (params first, then body locals in encounter
    /// order) — the analogue of [`Program::slots`] for this function's frame.
    pub slots: Vec<Box<[u8]>>,
    /// `function &f()` — returns by reference (step 13). A `return <lvalue>` in
    /// the body lowers to [`StmtKind::ReturnRef`]; the call site decides whether
    /// to alias (`$y = &f()`) or copy.
    pub by_ref: bool,
    /// Declared scalar return type hint, enforced (weak mode) on the returned
    /// value (step 14). `None` for absent or non-scalar return types.
    pub ret_hint: Option<TypeHint>,
    /// `true` when the body contains a `yield` / `yield from` at any depth (but
    /// not inside a nested function/closure) — the function is a *generator*
    /// (step 39). Calling it does not run the body: it returns a `Generator`
    /// object whose body executes lazily. Computed by a body walk at lowering.
    pub is_generator: bool,
    /// For a closure/arrow defined lexically inside a class, the name of that
    /// class — so the compiler can set `cur_class` for the closure body and
    /// resolve `self::`/`parent::`/`new self` at compile time (a closure inherits
    /// the defining class's scope). `None` for free functions, methods (compiled
    /// with their class directly), and closures defined outside any class.
    pub defining_class: Option<Box<[u8]>>,
    /// Amount added to every `ExprKind::Closure` index when this body is compiled.
    /// Non-zero only for a trait method/closure flattened into a class from a
    /// *different* unit: the trait's closures are re-appended to the consumer
    /// unit's closure table, so the indices baked into the body must shift by the
    /// append offset (the cross-unit trait-closure fix). 0 for everything else.
    pub closure_shift: i32,
    /// `#[Attr(args)]` attributes on the `function`/method declaration, retained
    /// for `ReflectionFunction`/`ReflectionMethod::getAttributes()` (empty for
    /// closures, hooks, and unattributed functions).
    pub attributes: Vec<HirAttribute>,
    pub line: Line,
}

/// One formal parameter. By-value by default; `by_ref` marks `&$x` parameters
/// (step 11b), which bind the caller's variable cell instead of a copy.
/// Variadic params still lower to [`crate::lower::LowerError::Unsupported`].
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The local slot this parameter binds (equal to its positional index).
    pub slot: Slot,
    /// Default value expression, evaluated in the callee frame when the
    /// argument is omitted. `None` makes the parameter required.
    pub default: Option<Expr>,
    /// `true` for a `&$x` by-reference parameter: the matching argument must be
    /// a variable, whose storage cell is shared with this slot for the call.
    pub by_ref: bool,
    /// `true` for a `...$rest` variadic parameter (step 38-5): it collects every
    /// remaining positional argument (int keys) and named argument (string keys)
    /// into an array. Always the last parameter.
    pub variadic: bool,
    /// Declared scalar type hint, enforced (weak mode) on the bound argument
    /// (step 14). `None` for an absent or non-scalar hint (no enforcement).
    pub hint: Option<TypeHint>,
}

/// One captured variable of a closure (step 18, D-18.3). At closure *creation*
/// the evaluator reads `src` in the active (enclosing) frame and binds the
/// resulting value into the closure's `dst` slot when the closure is later
/// called. `by_ref` selects `use(&$x)` semantics (share the cell) over `use($x)`
/// / arrow auto-capture (snapshot the value).
#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    /// Slot in the enclosing frame to read at creation time.
    pub src: Slot,
    /// Slot in the closure's own frame to bind at call time.
    pub dst: Slot,
    pub by_ref: bool,
}

/// The four coercible scalar type hints enforced in step 14.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Int,
    Float,
    String,
    Bool,
}

/// A single (non-union) type hint, optionally nullable (`?int`, `?Foo`). Scalars
/// are *coerced* under weak typing; the rest are *checked* at the call binder.
/// Union / intersection / `mixed` / `void` still lower to `None` (unenforced).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeHint {
    pub kind: HintKind,
    pub nullable: bool,
}

/// The kind of a [`TypeHint`]. Scalars carry their coercible type; `Class` carries
/// the declared class/interface name for an `instanceof` check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintKind {
    Scalar(ScalarType),
    /// `array` — accepts any PHP array.
    Array,
    /// `callable` — accepts any value `is_callable` accepts.
    Callable,
    /// `iterable` — accepts an array or a `Traversable` instance.
    Iterable,
    /// `object` — accepts any object.
    Object,
    /// A class/interface name — accepts an instance that is `instanceof` it.
    Class(Box<[u8]>),
}

impl TypeHint {
    /// The hint as written, for diagnostics (`int`, `array`, `Foo`), with a leading
    /// `?` when nullable.
    pub fn display_name(&self) -> String {
        let base: std::borrow::Cow<'_, str> = match &self.kind {
            HintKind::Scalar(ScalarType::Int) => "int".into(),
            HintKind::Scalar(ScalarType::Float) => "float".into(),
            HintKind::Scalar(ScalarType::String) => "string".into(),
            HintKind::Scalar(ScalarType::Bool) => "bool".into(),
            HintKind::Array => "array".into(),
            HintKind::Callable => "callable".into(),
            HintKind::Iterable => "iterable".into(),
            HintKind::Object => "object".into(),
            HintKind::Class(name) => String::from_utf8_lossy(name).into_owned().into(),
        };
        if self.nullable {
            format!("?{base}")
        } else {
            base.into_owned()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub line: Line,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// `echo a, b, c;` and the `<?= ... ?>` short tag.
    Echo(Vec<Expr>),

    /// `const A = 1, B = 2;` top-level / namespaced constant declaration (step 51):
    /// each name is already resolved to its fully-qualified form. Executed in order
    /// at run time (a later item may reference an earlier one); redefining warns and
    /// keeps the first value, exactly like `define()`.
    ConstDecl(Vec<(Box<[u8]>, Expr)>),
    /// Literal HTML / text outside `<?php ?>` (emitted verbatim).
    InlineHtml(Box<[u8]>),
    /// An expression evaluated for its side effects (`$x = 1;`, `f();`).
    Expr(Expr),
    /// A `{ ... }` block.
    Block(Vec<Stmt>),
    If {
        cond: Expr,
        then: Vec<Stmt>,
        /// `elseif` clauses in source order.
        elseifs: Vec<(Expr, Vec<Stmt>)>,
        /// `else` body; empty when there is no `else`.
        otherwise: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        cond: Expr,
    },
    For {
        init: Vec<Expr>,
        cond: Vec<Expr>,
        step: Vec<Expr>,
        body: Vec<Stmt>,
    },
    /// `foreach ($iter as [$key =>] $value) { body }`. By value by default; with
    /// `&$value` (step 11d-3) each element is bound by reference, so body writes
    /// land in the source array. `list()` targets stay out of Tier 1 scope.
    Foreach {
        iter: Expr,
        /// Slot bound to the key, when the source uses `$k => $v`.
        key: Option<Slot>,
        /// Slot bound to the value.
        value: Slot,
        /// `true` for `foreach (… as &$value)`: bind each element by reference.
        by_ref: bool,
        body: Vec<Stmt>,
    },
    /// `switch ($subject) { case ...: ...; default: ...; }`. Cases are kept in
    /// source order; fall-through and `default` placement are honoured by the
    /// evaluator. Loose `==` matching (contrast `match`, which is strict).
    Switch {
        subject: Expr,
        cases: Vec<Case>,
    },
    /// `unset($a, $b[k], ...);` — drops variables / array elements.
    Unset(Vec<Place>),
    /// `global $a, $b;` — alias each named global cell into the local frame
    /// (step 12-2, D-12.2). Each binding records the local slot to install the
    /// alias into and the global slot it aliases.
    Global(Vec<GlobalBinding>),
    /// `static $a = init, $b;` — alias each local slot to a persistent cell that
    /// survives across calls, initialised once on first execution (step 15).
    StaticVar(Vec<StaticBinding>),
    /// `break N;` — level is >= 1 (defaults to 1).
    Break(u32),
    /// `continue N;` — level is >= 1 (defaults to 1).
    Continue(u32),
    /// `return [expr];`
    Return(Option<Expr>),
    /// `return <lvalue>;` inside a `function &f()` — returns a *reference* to the
    /// place (its shared cell), so `$y = &f()` aliases it (step 13, D-13.2).
    ReturnRef(Place),
    /// `try { body } catch (T $e) { } ... finally { }` (step 20). On a thrown
    /// exception the first `catch` whose type matches (by `instanceof`) runs;
    /// `finally` runs unconditionally afterwards and its own control flow (a
    /// `return`/`throw`/`break` inside it) overrides the try/catch outcome. An
    /// empty `finally` means the clause was absent.
    Try {
        body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally: Vec<Stmt>,
    },
    /// `label:` — a `goto` target (step 45). A pure marker: no-op at runtime,
    /// used by `exec_stmts` to locate the jump destination within a block.
    Label(Box<[u8]>),
    /// `goto label;` — unconditional jump to the matching `Label` in the current
    /// function scope (step 45). Validity (label exists, not jumping *into* a
    /// loop/switch) is checked at lowering time, mirroring PHP's compile-time
    /// fatals; at runtime it raises `Flow::Goto` which `exec_stmts` resolves.
    Goto(Box<[u8]>),
    /// A lone `;`.
    Nop,
    /// Conditional function declaration: register `functions[idx]` in the runtime
    /// function table when this statement is reached (a `function` inside a
    /// branch/block — not hoisted). `idx` is in [`Program::functions`], at or past
    /// [`Program::hoisted_fn_count`].
    DeclareFn(usize),
    /// Conditional class/interface/enum declaration: register `classes[idx]`'s name
    /// in the runtime class index when this statement is reached (a declaration
    /// inside a branch/block — not hoisted). `idx` is in [`Program::classes`] and is
    /// listed in [`Program::conditional_classes`].
    DeclareClass(usize),
}

/// One `catch (T1 | T2 $e) { body }` clause (step 20). `types` are the caught
/// class names (a multi-catch `A | B` lists both); `var` is the bound slot, or
/// `None` for the variable-less `catch (T)` form (PHP 8).
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub types: Vec<Box<[u8]>>,
    pub var: Option<Slot>,
    pub body: Vec<Stmt>,
}

/// One `global $x;` binding: the local-frame slot the alias is installed into,
/// and the global-frame slot it aliases (step 12-2, D-12.2).
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBinding {
    pub local: Slot,
    pub global: Slot,
}

/// One `static $x = init;` binding (step 15, D-15.2): the local slot to alias,
/// a program-unique `id` into the evaluator's persistent static store, and the
/// optional one-time initializer (`None` for `static $x;`).
#[derive(Debug, Clone, PartialEq)]
pub struct StaticBinding {
    pub slot: Slot,
    pub id: usize,
    pub init: Option<Expr>,
}

/// One `switch` case. `test` is `None` for the `default:` case.
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub test: Option<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub line: Line,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // --- literals ---
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Box<[u8]>),

    /// A `NAME` constant the lowerer could not fold to an engine constant
    /// (step 49c): resolved at runtime against `define()`'d constants, falling
    /// back to an "Undefined constant" `Error` like PHP 8. `fallback` is the
    /// global name an unqualified constant used inside a namespace falls back to
    /// (`Foo\BAR` → `Some(BAR)`, step 50); `None` for already-global / explicitly
    /// qualified names. The primary `name` is what an "Undefined constant" error
    /// reports, matching PHP's namespaced lookup.
    Const {
        name: Box<[u8]>,
        fallback: Option<Box<[u8]>>,
    },

    /// `$x` — read of a resolved (local-frame) variable slot.
    Var(Slot),
    /// `$GLOBALS['literal']` read — a resolved *global*-frame slot, reachable
    /// even from inside a function (step 12-3, D-12.3). `$GLOBALS['x'][k]` reads
    /// as `Index { base: GlobalVar(x), index: k }`.
    GlobalVar(Slot),
    /// `$_SERVER` (and the other data superglobals) read — addressed by name (an
    /// index into [`crate::bytecode::SUPERGLOBAL_NAMES`]) via the VM-level
    /// superglobal store, so it resolves identically in every unit/frame
    /// (including included files), unlike a unit-local `GlobalVar` slot.
    Superglobal(u8),

    /// Binary op with eager left-then-right operand evaluation, dispatched to
    /// `php_types::ops` by the evaluator. Excludes short-circuit and coalesce.
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// Short-circuit `&&` / `and`.
    And(Box<Expr>, Box<Expr>),
    /// Short-circuit `||` / `or`.
    Or(Box<Expr>, Box<Expr>),
    /// Logical `xor` (both operands always evaluated, boolean result).
    Xor(Box<Expr>, Box<Expr>),
    /// `??` — right operand evaluated only if the left is null/undefined.
    Coalesce(Box<Expr>, Box<Expr>),

    /// Prefix `-`, `+`, `!`, `~`.
    Unary(UnOp, Box<Expr>),
    /// A type cast like `(int)$x`.
    Cast(CastKind, Box<Expr>),

    /// `$x = rhs`.
    Assign(Slot, Box<Expr>),
    /// `$target = &$source` — bind `target` as a reference alias of `source`.
    /// Both sides are [`Place`]s: a bare variable (empty `steps`, step 11a) or an
    /// array element (`$x = &$a[0]`, `$a[0] = &$x`, step 11d-2). After this, a
    /// write through either side is visible through the other (D-R3/D-R4/D-R12).
    AssignRef { target: Place, source: Place },
    /// `$target = &f(...)` — bind `target` as a reference alias of the cell a
    /// by-reference function returns (step 13, D-13.5). `call` is an
    /// [`ExprKind::Call`]; it is invoked *raw* (its `Zval::Ref` result is not
    /// dereferenced) so the shared cell can be aliased.
    AssignRefCall { target: Place, call: Box<Expr> },
    /// `$x op= rhs` (compound assignment, e.g. `+=`, `.=`).
    AssignOp(BinOp, Slot, Box<Expr>),
    /// `$x ??= rhs` — assigns only if `$x` is null/undefined.
    AssignCoalesce(Slot, Box<Expr>),
    /// `++$x` / `--$x` / `$x++` / `$x--` on a variable slot.
    IncDec { slot: Slot, inc: bool, pre: bool },
    /// `++`/`--` on a place that is not a bare local: an array element
    /// (`$a[k]++`) or an object property (`$o->n++`, `$this->n++`), step 19-2.
    IncDecPlace { place: Place, inc: bool, pre: bool },

    /// `cond ? then : otherwise` (`then` is `None` for the `?:` shorthand).
    Ternary {
        cond: Box<Expr>,
        then: Option<Box<Expr>>,
        otherwise: Box<Expr>,
    },

    /// `name(args...)` — a call to a (builtin or user) function. `args` are the
    /// leading positional arguments; `named` are trailing `name: value` arguments
    /// (step 38), empty for an all-positional call. PHP forbids a positional
    /// argument after a named one, so this split is unambiguous.
    Call {
        name: Box<[u8]>,
        args: Vec<Expr>,
        named: Vec<(Box<[u8]>, Expr)>,
    },

    /// A closure / arrow-function expression (step 18, D-18.2). `fn_idx` selects
    /// the lowered body from [`Program::closures`]; `captures` are evaluated in
    /// the active frame to produce the [`php_types::Closure`] value. `bind_this`
    /// is true for an ordinary (non-`static`) closure, which captures the current
    /// `$this` at creation (step 19-6, D-19.19).
    Closure {
        fn_idx: usize,
        captures: Vec<Capture>,
        bind_this: bool,
    },

    /// A first-class callable `name(...)` (step 18-6, D-18.10): produces a
    /// closure value wrapping the function name.
    FirstClassCallable(Box<[u8]>),

    /// A dynamic call `callee(args...)` where the callee is a runtime value
    /// (step 18, D-18.5): `$f()`, `$a['k']()`, an immediately-invoked closure
    /// `(function(){})()`. Arguments are evaluated by value; the evaluator
    /// dispatches on the callee value (closure / string name).
    CallDynamic { callee: Box<Expr>, args: Vec<Expr> },

    /// The pipe operator `input |> callable` (PHP 8.5): evaluates `input`, then
    /// `callable` (a value that must resolve to a callable), then calls it with the
    /// input as the sole positional argument — i.e. `callable(input)`. Distinct
    /// from [`ExprKind::CallDynamic`] because the operands evaluate left-to-right
    /// (input before callable), the opposite of a call's callee-before-args order.
    Pipe { input: Box<Expr>, callable: Box<Expr> },

    /// Argument unpacking `...$e` (step 40). Only valid as a direct element of a
    /// call's argument list, where the evaluator expands it: int keys (and
    /// float keys, lossily) become positional arguments in iteration order, and
    /// string keys become named arguments. Lowering enforces the ordering rules
    /// (no positional after a spread; no spread after a named argument); the
    /// runtime enforces the array/Traversable type and the within-unpacking
    /// "positional after named" rule. Appearing anywhere else is a runtime error.
    Spread(Box<Expr>),

    /// An array literal `[...]` / `array(...)`. Elements keep source order;
    /// keyless elements take the next free integer index (`PhpArray::append`).
    Array(Vec<ArrayElem>),

    /// Reading `base[index]` (`$a[k]`, also a string offset read).
    Index { base: Box<Expr>, index: Box<Expr> },

    /// `[$a, $b] = rhs` / `list(...) = rhs` array destructuring (step 51).
    /// Desugared by the lowerer: `rhs` is stored once into `temp`, each `assign`
    /// reads `temp[key]` and writes one (possibly nested) sub-target, and the whole
    /// expression yields the stored `rhs` value (`$x = [$a,$b] = $arr` sets `$x` to
    /// the array).
    ///
    /// By-reference destructuring (`[&$a, &$b] = $arr`): a `&$x` sub-target lowers to
    /// an [`ExprKind::AssignRef`] in `assigns` whose source is the real source
    /// element (`$arr[0]`, navigated from `rhs` as a place), so the reference is
    /// promoted in the source array — PHP's list-reference writeback. A non-place
    /// rhs (`[&$v] = f()`) aliases the value copy in `temp` instead (no writeback).
    ListAssign {
        temp: Slot,
        rhs: Box<Expr>,
        assigns: Vec<Expr>,
    },

    /// `place = rhs` where `place` indexes into an array (`$a[k] = v`,
    /// `$a[] = v`, nested). Plain `$x = v` keeps the lighter [`ExprKind::Assign`].
    AssignPlace(Place, Box<Expr>),
    /// `place op= rhs` on an array element (e.g. `$a[k] += v`, `$a[k] .= v`).
    AssignOpPlace(BinOp, Place, Box<Expr>),
    /// `place ??= rhs` on an array element.
    AssignCoalescePlace(Place, Box<Expr>),

    /// `isset($a, $b[k], ...)` — true iff every place is set and non-null.
    Isset(Vec<Place>),
    /// `empty($place)` — true iff the place is unset or falsy (no warnings).
    Empty(Place),

    /// `@expr` (step 48) — the error-control operator: evaluate `expr` but
    /// suppress the non-fatal diagnostics (warnings/notices/deprecations) it
    /// raises. A thrown exception / engine `Error` is *not* suppressed (it rides
    /// the `Err` channel, like PHP, which only silences the error_reporting path).
    Suppress(Box<Expr>),

    /// `print expr` (step 46) — an *expression*: emits `expr` stringified, then
    /// evaluates to `int(1)` (so `$x = print "a"` and `(print "a") + 10` work).
    Print(Box<Expr>),
    /// `exit`/`die [arg]` (step 46) — terminates the script. An `int` argument
    /// is the process exit code; any other argument is stringified and emitted
    /// (exit code 0); no argument means code 0. Raised as `PhpError::Exit`.
    Exit(Option<Box<Expr>>),

    /// `clone $obj` (step 51) — a shallow copy of the object (new handle, each
    /// property copied by value; nested objects are shared, arrays copy-on-write),
    /// then `__clone` is run on the copy if the class defines it.
    Clone(Box<Expr>),

    /// `eval($code)` (step 57, Phase 1) — compile the string operand as a PHP
    /// translation unit at run time and execute it, yielding its `return` value
    /// (or `null`). The eval'd unit runs as its own module: instanceof, method
    /// resolution and `var_dump` of its objects resolve against it.
    Eval(Box<Expr>),

    /// `include`/`include_once`/`require`/`require_once <expr>` (step 57, Phase 2)
    /// — resolve the operand to a path, load and run that file as its own
    /// translation unit, and yield its top-level `return` value (or `int(1)` if it
    /// has none). Reuses the eval machinery (class/function merge, compile against
    /// the caller image). `_once` variants no-op when the file was already loaded;
    /// a missing file is a fatal for `require*`, a warning + `false` for `include*`.
    Include { mode: IncludeMode, path: Box<Expr> },

    /// `match ($subject) { conds => body, ..., default => body }`. Strict `===`
    /// matching; an arm with empty `conditions` is the `default` arm.
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// `new ClassName(args...)` / `new self` / `new static` (step 19, D-19.6).
    /// Creates an instance with its declared properties initialised to their
    /// defaults, then runs `__construct` if the class defines one.
    New {
        class: ClassRef,
        args: Vec<Expr>,
        named: Vec<(Box<[u8]>, Expr)>,
    },

    /// `$obj->method(args...)` instance method call (step 19, D-19.7). `nullsafe`
    /// marks the `?->` form: a null receiver short-circuits to NULL instead of
    /// erroring.
    MethodCall {
        object: Box<Expr>,
        method: Box<[u8]>,
        args: Vec<Expr>,
        named: Vec<(Box<[u8]>, Expr)>,
        nullsafe: bool,
    },

    /// `$obj->prop` property read (step 19, D-19.8). A missing property warns and
    /// yields NULL. `nullsafe` marks `?->`.
    PropGet {
        object: Box<Expr>,
        name: Box<[u8]>,
        nullsafe: bool,
    },

    /// `$obj->$name` / `$obj->{expr}` dynamic property read (step 51). `name` is
    /// evaluated to a string at runtime; otherwise identical to [`PropGet`].
    PropGetDyn {
        object: Box<Expr>,
        name: Box<Expr>,
        nullsafe: bool,
    },

    /// `$obj->$method(args...)` / `$obj->{expr}(args...)` dynamic method call
    /// (step 51). `method` is evaluated to a string at runtime; otherwise identical
    /// to [`MethodCall`].
    MethodCallDyn {
        object: Box<Expr>,
        method: Box<Expr>,
        args: Vec<Expr>,
        named: Vec<(Box<[u8]>, Expr)>,
        nullsafe: bool,
    },

    /// `$this` (step 19, D-19.5): the current object inside a method. Outside any
    /// method context the evaluator raises the fatal "Using $this when not in
    /// object context".
    This,

    /// `Class::method(args)` / `self::` / `parent::` / `static::` call (step
    /// 19-3/19-4). Resolved against the named/self/parent/LSB class; keeps the
    /// current `$this` for forwarding (self/parent/static) calls.
    StaticCall {
        class: ClassRef,
        method: Box<[u8]>,
        args: Vec<Expr>,
        named: Vec<(Box<[u8]>, Expr)>,
    },

    /// PHP 8.4 parent property-hook call: `parent::$prop::get()` /
    /// `parent::$prop::set($value)` (also `self::`/`static::`/`Named::`). Invokes
    /// the named class's `get`/`set` hook for `prop` on the current `$this`; when
    /// that class's property has no user hook the *implicit* hook reads/writes the
    /// backing store directly. `set` carries the new value as its single argument.
    ParentHookCall {
        class: ClassRef,
        prop: Box<[u8]>,
        set: bool,
        args: Vec<Expr>,
    },

    /// `Class::CONST` / `self::CONST` / `parent::CONST` / `static::CONST`, and the
    /// special `Class::class` (which yields the class name string), step 19-4,
    /// D-19.15.
    ClassConst { class: ClassRef, name: Box<[u8]> },

    /// `Class::$prop` static-property read (step 19-4, D-19.14).
    StaticProp { class: ClassRef, name: Box<[u8]> },

    /// `Class::$prop = rhs` / `+= ` / `??=` static-property assignment.
    StaticPropAssign {
        class: ClassRef,
        name: Box<[u8]>,
        op: StaticAssignOp,
        rhs: Box<Expr>,
    },

    /// `Class::$prop++` / `--` (pre/post) static-property inc/dec.
    StaticPropIncDec {
        class: ClassRef,
        name: Box<[u8]>,
        inc: bool,
        pre: bool,
    },

    /// `$x instanceof Class` (step 19-5, D-19.16): true if the value is an object
    /// whose class is `class`, a subclass, or an implemented interface
    /// (transitively). A non-object, or an unknown class, yields `false`.
    InstanceOf { expr: Box<Expr>, class: ClassRef },

    /// `throw <expr>` as an expression (step 20). Evaluates the operand (which
    /// must be a Throwable object) and unwinds with `PhpError::Thrown`. PHP 8
    /// allows `throw` in expression position (`$x ?? throw new …`); a statement
    /// `throw e;` lowers to an [`StmtKind::Expr`] wrapping this.
    Throw(Box<Expr>),

    /// `yield [$k =>] [$v]` inside a generator (step 39). Suspends the body,
    /// surfacing `value` (NULL for a bare `yield;`) under `key` (auto-keyed when
    /// `None`). The expression *evaluates to* the value passed to the next
    /// `send()` (NULL for `next()`). Only valid in a function flagged
    /// [`FnDecl::is_generator`].
    Yield {
        key: Option<Box<Expr>>,
        value: Option<Box<Expr>>,
    },

    /// `yield from <iterator>` (step 39-6): delegates to an array, `Traversable`,
    /// or another generator, re-yielding each of its `(key, value)` pairs
    /// *verbatim* (keys preserved, the outer auto-key counter untouched). The
    /// expression evaluates to the delegate's `return` value (NULL for a
    /// non-generator).
    YieldFrom(Box<Expr>),
}

/// The flavour of a static-property assignment (step 19-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticAssignOp {
    /// `=`
    Plain,
    /// `op=` (e.g. `+=`, `.=`)
    Op(BinOp),
    /// `??=`
    Coalesce,
}

/// One element of an array literal: an optional key plus a value.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElem {
    pub key: Option<Expr>,
    pub value: Expr,
}

/// One `match` arm. An empty `conditions` list marks the `default` arm.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub conditions: Vec<Expr>,
    pub body: Expr,
}

/// An assignable / unsettable location: a base plus a chain of index steps.
/// `$x` is `{Local(slot), []}`; `$a[0]["k"]` is `{Local(slot_a), [Index(0),
/// Index("k")]}`; `$GLOBALS['x']` is `{Global(slot_x), []}`; `$a[]` ends in
/// [`PlaceStep::Append`] (write context only).
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub base: PlaceBase,
    pub steps: Vec<PlaceStep>,
}

/// The base a [`Place`] is rooted at: a slot in the active local frame, a slot
/// in the global frame for a `$GLOBALS['literal']` target (step 12-3), or a
/// static class property for an indexed write target (`self::$arr[k] = …`).
#[derive(Debug, Clone, PartialEq)]
pub enum PlaceBase {
    Local(Slot),
    Global(Slot),
    /// A data superglobal (`$_SERVER[$k] = …`) as a write/test target, addressed
    /// by name (index into [`crate::bytecode::SUPERGLOBAL_NAMES`]) via the
    /// VM-level superglobal store. Mirrors [`ExprKind::Superglobal`] on the read
    /// side; resolves identically across units/frames.
    Superglobal(u8),
    /// `$this` as the root of a property write target (`$this->x = …`), step 19.
    /// Resolved against the evaluator's current-object context, not a slot.
    This,
    /// `Class::$prop` as the root of an indexed write/unset target
    /// (`self::$arr[k] = v`, `unset(self::$arr[k])`). The per-class cell is read
    /// into a temp, mutated, and written back at compile time — value-correct for
    /// PHP arrays (copy-on-write).
    StaticProp { class: ClassRef, name: Box<[u8]> },
    /// `Class::CONST` as the root of an `isset()`/`empty()` *read* test
    /// (`isset(self::TABLE[$k])`). The constant value is materialised into a temp
    /// and the index path tested on it — read-only, so this base never reaches a
    /// write/unset path (only `lower_test_place` produces it).
    ClassConst { class: ClassRef, name: Box<[u8]> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaceStep {
    /// `[expr]`
    Index(Expr),
    /// `[]` — append; only valid as the final step of a write target.
    Append,
    /// `->prop` — navigate into an object's property (step 19, D-19.9). Unlike
    /// array steps, this enters the shared `Rc<RefCell<Object>>` in place (no
    /// copy-on-write write-back). A missing property is created on write.
    Prop(Box<[u8]>),
    /// `->$name` / `->{expr}` — dynamic property step (step 51): `name` is
    /// evaluated to a string at runtime, otherwise identical to [`PlaceStep::Prop`].
    PropDyn(Expr),
}

/// Binary operators whose semantics live in `php_types::ops`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    NotEq,
    Identical,
    NotIdentical,
    Lt,
    Le,
    Gt,
    Ge,
    Spaceship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Unary `-`.
    Neg,
    /// Unary `+`.
    Plus,
    /// Logical `!`.
    Not,
    /// Bitwise `~`.
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Int,
    Float,
    String,
    Bool,
    Array,
    Object,
}
