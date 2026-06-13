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
    /// `slots[i]` is the name (without leading `$`) of the variable in slot `i`.
    /// Used for `Undefined variable $name` diagnostics (D-G13).
    pub slots: Vec<Box<[u8]>>,
    /// Top-level user-defined functions, hoisted at lowering time so a call may
    /// precede the declaration (PHP's function hoisting). Resolved by the
    /// evaluator's call path before the builtin registry (step 8).
    pub functions: Vec<FnDecl>,
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
    pub line: Line,
}

/// One formal parameter. By-value only in step 8 (by-reference / variadic
/// params lower to [`crate::lower::LowerError::Unsupported`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The local slot this parameter binds (equal to its positional index).
    pub slot: Slot,
    /// Default value expression, evaluated in the callee frame when the
    /// argument is omitted. `None` makes the parameter required.
    pub default: Option<Expr>,
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
    /// `foreach ($iter as [$key =>] $value) { body }` (by-value; by-reference
    /// and `list()` targets are out of Tier 1 scope).
    Foreach {
        iter: Expr,
        /// Slot bound to the key, when the source uses `$k => $v`.
        key: Option<Slot>,
        /// Slot bound to the value.
        value: Slot,
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
    /// `break N;` — level is >= 1 (defaults to 1).
    Break(u32),
    /// `continue N;` — level is >= 1 (defaults to 1).
    Continue(u32),
    /// `return [expr];`
    Return(Option<Expr>),
    /// A lone `;`.
    Nop,
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

    /// `$x` — read of a resolved variable slot.
    Var(Slot),

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
    /// `$x op= rhs` (compound assignment, e.g. `+=`, `.=`).
    AssignOp(BinOp, Slot, Box<Expr>),
    /// `$x ??= rhs` — assigns only if `$x` is null/undefined.
    AssignCoalesce(Slot, Box<Expr>),
    /// `++$x` / `--$x` / `$x++` / `$x--` on a variable slot.
    IncDec { slot: Slot, inc: bool, pre: bool },

    /// `cond ? then : otherwise` (`then` is `None` for the `?:` shorthand).
    Ternary {
        cond: Box<Expr>,
        then: Option<Box<Expr>>,
        otherwise: Box<Expr>,
    },

    /// `name(args...)` — a call to a (builtin) function. The evaluator resolves
    /// `name` against its builtin registry; Tier 1 has no user functions yet.
    Call { name: Box<[u8]>, args: Vec<Expr> },

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

    /// `match ($subject) { conds => body, ..., default => body }`. Strict `===`
    /// matching; an arm with empty `conditions` is the `default` arm.
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
    },
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

/// An assignable / unsettable location: a base variable slot plus a chain of
/// index steps. `$x` is `{slot, []}`; `$a[0]["k"]` is `{slot_a, [Index(0),
/// Index("k")]}`; `$a[]` ends in [`PlaceStep::Append`] (write context only).
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub slot: Slot,
    pub steps: Vec<PlaceStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaceStep {
    /// `[expr]`
    Index(Expr),
    /// `[]` — append; only valid as the final step of a write target.
    Append,
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
}
