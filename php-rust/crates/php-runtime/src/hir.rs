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
    /// `break N;` — level is >= 1 (defaults to 1).
    Break(u32),
    /// `continue N;` — level is >= 1 (defaults to 1).
    Continue(u32),
    /// `return [expr];`
    Return(Option<Expr>),
    /// A lone `;`.
    Nop,
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
