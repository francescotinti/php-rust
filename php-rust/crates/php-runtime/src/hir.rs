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
}

/// Index into [`Program::classes`] (step 19, D-19.3).
pub type ClassId = usize;

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
    pub line: Line,
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
}

/// One method (step 19, D-19.5). Wraps an ordinary [`FnDecl`] (so method calls
/// reuse the whole function-frame machinery) plus the slot its body binds `$this`
/// to and the OOP modifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub visibility: Visibility,
    pub is_static: bool,
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

/// A scalar type hint, optionally nullable (`?int`). Only these are enforced;
/// every other hint (class, union, array, mixed, …) lowers to `None` (step 14,
/// D-14.1/D-14.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeHint {
    pub kind: ScalarType,
    pub nullable: bool,
}

impl TypeHint {
    /// The hint as written, for diagnostics: `int`, `float`, `string`, `bool`,
    /// with a leading `?` when nullable.
    pub fn display_name(&self) -> String {
        let base = match self.kind {
            ScalarType::Int => "int",
            ScalarType::Float => "float",
            ScalarType::String => "string",
            ScalarType::Bool => "bool",
        };
        if self.nullable {
            format!("?{base}")
        } else {
            base.to_string()
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

    /// `$x` — read of a resolved (local-frame) variable slot.
    Var(Slot),
    /// `$GLOBALS['literal']` read — a resolved *global*-frame slot, reachable
    /// even from inside a function (step 12-3, D-12.3). `$GLOBALS['x'][k]` reads
    /// as `Index { base: GlobalVar(x), index: k }`.
    GlobalVar(Slot),

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

    /// `print expr` (step 46) — an *expression*: emits `expr` stringified, then
    /// evaluates to `int(1)` (so `$x = print "a"` and `(print "a") + 10` work).
    Print(Box<Expr>),
    /// `exit`/`die [arg]` (step 46) — terminates the script. An `int` argument
    /// is the process exit code; any other argument is stringified and emitted
    /// (exit code 0); no argument means code 0. Raised as `PhpError::Exit`.
    Exit(Option<Box<Expr>>),

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

/// The base a [`Place`] is rooted at: a slot in the active local frame, or a
/// slot in the global frame for a `$GLOBALS['literal']` target (step 12-3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaceBase {
    Local(Slot),
    Global(Slot),
    /// `$this` as the root of a property write target (`$this->x = …`), step 19.
    /// Resolved against the evaluator's current-object context, not a slot.
    This,
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
