//! Bridge: mago AST (borrowed from a bumpalo arena) → owned [`crate::hir`].
//!
//! mago is reused as the PHP front-end (D-G8): it gives us a lossless, error
//! recovering parser for PHP 8.x, eliminating the ~25K LOC of re2c lexer + Bison
//! grammar. Its AST, however, borrows from an arena and stores text inline as
//! `&[u8]`. This module walks that tree once and produces the *owned* HIR the
//! evaluator consumes, doing the two resolutions described in [`crate::hir`]:
//! variable→slot and span→line.
//!
//! Scope is Tier 1 procedural control flow (plan step 3/4). Constructs outside
//! that scope (OOP, foreach/switch/match, functions, references, includes,
//! variable-variables, array element targets) lower to
//! [`LowerError::Unsupported`] rather than being silently dropped — the
//! phpt-runner's capability scan (step 6) turns these into motivated SKIPs.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use bumpalo::Bump;
use mago_database::file::File;
use mago_span::{HasSpan, Span};
use mago_syntax::ast::{
    Access, Argument, ArrayElement, ArrowFunction, AssignmentOperator, BinaryOperator, Call,
    Class, ClassLikeConstantSelector, ClassLikeMember, ClassLikeMemberSelector, Closure, Construct,
    DeclareBody, Expression, Extends, ForeachTarget, Function, FunctionLikeParameterList, Hint,
    Identifier, Instantiation, Interface, Literal, LiteralInteger, MatchArm as AstMatchArm, Method,
    MethodBody, Modifier, Node, PartialApplication, Property, PropertyItem, Statement, StaticItem,
    Trait, TraitUse, UnaryPostfixOperator, UnaryPrefixOperator, Variable,
};
use mago_syntax::parser::parse_file;

use crate::hir::{
    ArrayElem, BinOp, Capture, Case, CastKind, ClassDecl, ClassRef, Expr, ExprKind, FnDecl,
    GlobalBinding, Line, MatchArm, MethodDecl, Param, Place, PlaceBase, PlaceStep, Program,
    PropDecl, ScalarType, Slot, StaticAssignOp, StaticBinding, Stmt, StmtKind, TypeHint, UnOp,
    Visibility,
};

/// Why a script could not be lowered to HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    /// mago reported one or more parse errors.
    Parse(String),
    /// A construct that is valid PHP but outside the current Tier 1 scope.
    Unsupported { what: &'static str, line: Line },
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LowerError::Parse(msg) => write!(f, "parse error: {msg}"),
            LowerError::Unsupported { what, line } => {
                write!(f, "unsupported construct ({what}) on line {line}")
            }
        }
    }
}

impl std::error::Error for LowerError {}

/// Parse `source` (named `name` for diagnostics) and lower it to HIR.
pub fn lower_source(name: &[u8], source: &[u8]) -> Result<Program, LowerError> {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Owned(name.to_vec()), Cow::Owned(source.to_vec()));
    let program = parse_file(&arena, &file);

    if program.has_errors() {
        let msg = program
            .errors
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(LowerError::Parse(msg));
    }

    let mut low = Lowerer::new(&file, name);
    // Seed the built-in exception hierarchy (Throwable/Exception/Error + the SPL
    // subclasses) at the front of the class table (ids 0..N), before any user
    // class is hoisted (step 20). This makes `extends Exception`, `instanceof`,
    // `new RuntimeException(...)`, property init and `parent::__construct` reuse
    // the whole step-19 class machinery with no special-casing.
    let (pclasses, pindex) = lower_prelude();
    low.classes = pclasses;
    low.class_index = pindex;
    // Hoist top-level function declarations first, so a call may textually
    // precede its definition (PHP's function hoisting). Bodies are lowered here;
    // the main pass below skips the declaration statements (they are no-ops).
    // Classes are hoisted in two passes (names first, then bodies) so a method
    // body / `extends` may reference a class declared later (step 19, D-19.3).
    for s in program.statements.as_slice() {
        if let Statement::Function(func) = s {
            low.hoist_function(func)?;
        }
    }
    // Lower traits before classes, so a class's `use T` finds T fully resolved
    // (step 21). Traits stay in the Lowerer; they never enter the class table.
    low.lower_traits(program.statements.as_slice())?;
    low.hoist_classes(program.statements.as_slice())?;
    let body = low.lower_stmts(program.statements.as_slice())?;
    Ok(Program {
        body,
        file: name.into(),
        slots: low.globals.slots,
        functions: low.functions,
        closures: low.closures,
        static_count: low.static_count,
        strict: low.strict,
        classes: low.classes,
    })
}

/// The built-in classes, authored in PHP and lowered once into the front of
/// every program's class table (step 20): `stdClass` plus the throwable
/// hierarchy. Mirrors PHP's core/SPL classes closely enough for catch-matching,
/// the accessors, and `instanceof`.
/// `getTrace`/`getTraceAsString` are stubs (no real stack trace is modelled);
/// `file`/`line` are filled in by the evaluator at `new` time, not here.
const PRELUDE_SRC: &[u8] = br##"<?php
class stdClass {}
interface Throwable {}
class Exception implements Throwable {
    protected $message = "";
    protected $code = 0;
    protected $file = "";
    protected $line = 0;
    private $previous = null;
    public function __construct($message = "", $code = 0, $previous = null) {
        $this->message = $message;
        $this->code = $code;
        $this->previous = $previous;
    }
    public function getMessage() { return $this->message; }
    public function getCode() { return $this->code; }
    public function getPrevious() { return $this->previous; }
    public function getLine() { return $this->line; }
    public function getFile() { return $this->file; }
    public function getTrace() { return []; }
    public function getTraceAsString() { return "#0 {main}"; }
    public function __toString() { return $this->message; }
}
class Error implements Throwable {
    protected $message = "";
    protected $code = 0;
    protected $file = "";
    protected $line = 0;
    private $previous = null;
    public function __construct($message = "", $code = 0, $previous = null) {
        $this->message = $message;
        $this->code = $code;
        $this->previous = $previous;
    }
    public function getMessage() { return $this->message; }
    public function getCode() { return $this->code; }
    public function getPrevious() { return $this->previous; }
    public function getLine() { return $this->line; }
    public function getFile() { return $this->file; }
    public function getTrace() { return []; }
    public function getTraceAsString() { return "#0 {main}"; }
    public function __toString() { return $this->message; }
}
class ErrorException extends Exception {}
class LogicException extends Exception {}
class BadFunctionCallException extends LogicException {}
class BadMethodCallException extends BadFunctionCallException {}
class DomainException extends LogicException {}
class InvalidArgumentException extends LogicException {}
class LengthException extends LogicException {}
class OutOfRangeException extends LogicException {}
class RuntimeException extends Exception {}
class OutOfBoundsException extends RuntimeException {}
class OverflowException extends RuntimeException {}
class RangeException extends RuntimeException {}
class UnderflowException extends RuntimeException {}
class UnexpectedValueException extends RuntimeException {}
class JsonException extends Exception {}
class TypeError extends Error {}
class ArgumentCountError extends TypeError {}
class ValueError extends Error {}
class ArithmeticError extends Error {}
class DivisionByZeroError extends ArithmeticError {}
class UnhandledMatchError extends Error {}
"##;

/// Lower [`PRELUDE_SRC`] with a throwaway [`Lowerer`] and return its owned class
/// table + name→id index (step 20). The prelude has no functions/closures/
/// statics, so only the class table needs to be carried over.
fn lower_prelude() -> (Vec<ClassDecl>, HashMap<Vec<u8>, usize>) {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed(b"prelude".as_slice()), Cow::Borrowed(PRELUDE_SRC));
    let program = parse_file(&arena, &file);
    debug_assert!(
        !program.has_errors(),
        "exception prelude failed to parse: {:?}",
        program.errors
    );
    let mut low = Lowerer::new(&file, b"prelude");
    low.hoist_classes(program.statements.as_slice())
        .expect("exception prelude must lower");
    (low.classes, low.class_index)
}

/// A name→slot scope: the script globals, or one function's locals. Holds the
/// slot *names* (positional, reproduced into `Program`/`FnDecl.slots`) and the
/// reverse index for stable resolution.
#[derive(Default)]
struct Scope {
    slots: Vec<Box<[u8]>>,
    index: HashMap<Vec<u8>, Slot>,
}

impl Scope {
    /// Resolve `$name` (without the leading `$`) to a stable slot in this scope,
    /// allocating one on first sight.
    fn slot_for(&mut self, name: &[u8]) -> Slot {
        if let Some(&s) = self.index.get(name) {
            return s;
        }
        let s = self.slots.len() as Slot;
        self.slots.push(name.into());
        self.index.insert(name.to_vec(), s);
        s
    }

    /// Resolve `$name` only if it already has a slot (no allocation). Used by
    /// arrow-function capture analysis to decide whether a free variable refers
    /// to an enclosing-scope variable (step 18, D-18.4).
    fn get(&self, name: &[u8]) -> Option<Slot> {
        self.index.get(name).copied()
    }
}

/// The three products of lowering a closure body: parameters, lexical captures,
/// and the lowered statement list (step 18).
type LoweredClosure = (Vec<Param>, Vec<Capture>, Vec<Stmt>);

struct Lowerer<'f> {
    file: &'f File,
    /// The global scope (always present) and the active function-local overlay
    /// (`Some` while a function body is lowered). `slot_for` resolves against the
    /// active scope; the globals stay reachable so a global slot can be
    /// pre-registered from inside a function (D-12.1).
    globals: Scope,
    locals: Option<Scope>,
    /// True when the previous statement was a `?>` closing tag, so the next
    /// inline-HTML chunk must drop one leading newline (Zend lexer rule:
    /// `?>` consumes a single trailing `\n` / `\r\n`).
    after_closing_tag: bool,
    /// Hoisted top-level user functions and a name→index map (ASCII-lowercased,
    /// since PHP function names are case-insensitive).
    functions: Vec<FnDecl>,
    fn_index: HashMap<Vec<u8>, usize>,
    /// Anonymous/arrow function bodies, in one flat table (step 18, D-18.2). An
    /// [`ExprKind::Closure`] indexes into this by position.
    closures: Vec<FnDecl>,
    /// The program file name, used to synthesize the `{closure:file:line}` name
    /// PHP gives anonymous functions (step 18).
    prog_name: Box<[u8]>,
    /// True while lowering the body of a `function &f()`: a `return <lvalue>`
    /// then lowers to [`StmtKind::ReturnRef`] (step 13, D-13.3).
    fn_by_ref: bool,
    /// Running count of `static` declarations seen; each gets a unique id into
    /// the evaluator's persistent static store (step 15, D-15.3).
    static_count: usize,
    /// Set by `declare(strict_types=1)` — copied into `Program.strict` (step 16).
    strict: bool,
    /// Hoisted user classes and a name→index map (ASCII-lowercased; PHP class
    /// names are case-insensitive), step 19.
    classes: Vec<ClassDecl>,
    class_index: HashMap<Vec<u8>, usize>,
    /// Lowered traits, keyed by ASCII-lowercased name (step 21). Held only in the
    /// Lowerer — traits are not types and never enter `Program.classes`. Each
    /// entry is fully resolved (nested `use` already flattened), so a consuming
    /// class copies the members verbatim into its own [`ClassDecl`] (D-21.1/2/8).
    traits: HashMap<Vec<u8>, LoweredTrait>,
}

/// A trait whose members have been lowered and whose own `use` clauses have been
/// flattened in (step 21). Copied member-by-member into each consuming class so
/// the step-19 runtime machinery is reused with no evaluator changes.
struct LoweredTrait {
    methods: Vec<MethodDecl>,
    props: Vec<PropDecl>,
    static_props: Vec<crate::hir::StaticPropDecl>,
    consts: Vec<crate::hir::ClassConstDecl>,
    /// Names of `abstract` methods the trait requires the consumer to implement
    /// (D-21.11; enforcement arrives in 21-4).
    abstract_methods: Vec<Box<[u8]>>,
}

impl<'f> Lowerer<'f> {
    fn new(file: &'f File, prog_name: &[u8]) -> Self {
        Lowerer {
            file,
            globals: Scope::default(),
            locals: None,
            after_closing_tag: false,
            functions: Vec::new(),
            fn_index: HashMap::new(),
            closures: Vec::new(),
            prog_name: prog_name.into(),
            fn_by_ref: false,
            static_count: 0,
            strict: false,
            classes: Vec::new(),
            class_index: HashMap::new(),
            traits: HashMap::new(),
        }
    }

    /// 1-based source line for a span's start offset (`File::line_number` is 0-based).
    fn line_of(&self, span: Span) -> Line {
        self.file.line_number(span.start.offset) + 1
    }

    /// The active scope: the function-local overlay while a body is lowered,
    /// otherwise the script globals (D-12.1).
    fn scope_mut(&mut self) -> &mut Scope {
        self.locals.as_mut().unwrap_or(&mut self.globals)
    }

    /// Resolve `$name` (name given *without* the leading `$`) to a stable slot in
    /// the active scope.
    fn slot_for(&mut self, name: &[u8]) -> Slot {
        self.scope_mut().slot_for(name)
    }

    /// Resolve `$name` in the active (enclosing) scope *without* allocating a
    /// slot — `None` if the name is not already bound there (step 18, D-18.4).
    fn enclosing_slot(&self, name: &[u8]) -> Option<Slot> {
        self.locals.as_ref().unwrap_or(&self.globals).get(name)
    }

    // --- statements ---

    fn lower_stmts(&mut self, stmts: &[Statement]) -> Result<Vec<Stmt>, LowerError> {
        let mut out = Vec::with_capacity(stmts.len());
        for s in stmts {
            if let Some(st) = self.lower_stmt(s)? {
                out.push(st);
            }
        }
        Ok(out)
    }

    /// `Ok(None)` for nodes that carry no runtime statement (tags).
    fn lower_stmt(&mut self, stmt: &Statement) -> Result<Option<Stmt>, LowerError> {
        let line = self.line_of(stmt.span());
        let kind = match stmt {
            // `?>` consumes one trailing newline of the inline chunk that follows.
            Statement::ClosingTag(_) => {
                self.after_closing_tag = true;
                return Ok(None);
            }
            // `<?php` carries no runtime behaviour.
            Statement::OpeningTag(_) => return Ok(None),

            Statement::Inline(inline) => {
                let mut bytes: &[u8] = inline.value;
                if std::mem::take(&mut self.after_closing_tag) {
                    bytes = strip_one_newline(bytes);
                }
                StmtKind::InlineHtml(bytes.into())
            }
            Statement::Noop(_) => StmtKind::Nop,

            Statement::Echo(echo) => StmtKind::Echo(self.lower_expr_list(echo.values.iter())?),
            Statement::EchoTag(echo) => StmtKind::Echo(self.lower_expr_list(echo.values.iter())?),

            Statement::Expression(es) => StmtKind::Expr(self.lower_expr(es.expression)?),
            Statement::Block(block) => StmtKind::Block(self.lower_stmts(block.statements.as_slice())?),

            Statement::If(node) => {
                let cond = self.lower_expr(node.condition)?;
                let then = self.lower_stmts(node.body.statements())?;
                let mut elseifs = Vec::new();
                for (econd, ebody) in node.body.else_if_clauses() {
                    elseifs.push((self.lower_expr(econd)?, self.lower_stmts(ebody)?));
                }
                let otherwise = match node.body.else_statements() {
                    Some(s) => self.lower_stmts(s)?,
                    None => Vec::new(),
                };
                StmtKind::If {
                    cond,
                    then,
                    elseifs,
                    otherwise,
                }
            }

            Statement::While(node) => StmtKind::While {
                cond: self.lower_expr(node.condition)?,
                body: self.lower_stmts(node.body.statements())?,
            },

            Statement::DoWhile(node) => StmtKind::DoWhile {
                body: self.lower_stmts(std::slice::from_ref(node.statement))?,
                cond: self.lower_expr(node.condition)?,
            },

            Statement::For(node) => StmtKind::For {
                init: self.lower_expr_list(node.initializations.iter())?,
                cond: self.lower_expr_list(node.conditions.iter())?,
                step: self.lower_expr_list(node.increments.iter())?,
                body: self.lower_stmts(node.body.statements())?,
            },

            Statement::Foreach(node) => {
                let iter = self.lower_expr(node.expression)?;
                let (key, (value, by_ref)) = match &node.target {
                    ForeachTarget::Value(v) => (None, self.foreach_value_slot(v.value, line)?),
                    ForeachTarget::KeyValue(kv) => (
                        Some(self.foreach_slot(kv.key, line)?),
                        self.foreach_value_slot(kv.value, line)?,
                    ),
                };
                let body = self.lower_stmts(node.body.statements())?;
                StmtKind::Foreach {
                    iter,
                    key,
                    value,
                    by_ref,
                    body,
                }
            }

            Statement::Switch(node) => {
                let subject = self.lower_expr(node.expression)?;
                let mut cases = Vec::new();
                for c in node.body.cases() {
                    let test = match c.expression() {
                        Some(e) => Some(self.lower_expr(e)?),
                        None => None,
                    };
                    let body = self.lower_stmts(c.statements())?;
                    cases.push(Case { test, body });
                }
                StmtKind::Switch { subject, cases }
            }

            Statement::Unset(node) => {
                let mut places = Vec::new();
                for v in node.values.iter() {
                    places.push(self.lower_place(v, line)?);
                }
                StmtKind::Unset(places)
            }

            Statement::Global(node) => {
                let mut bindings = Vec::new();
                for v in node.variables.iter() {
                    let name = match v {
                        Variable::Direct(d) => strip_dollar(d.name),
                        // `global $$x` (variable-variable) needs runtime name
                        // resolution — outside step 12 scope (D-12.6).
                        _ => {
                            return Err(LowerError::Unsupported {
                                what: "variable-variable in `global`",
                                line,
                            })
                        }
                    };
                    // Local-frame slot for the alias, plus a (pre-registered)
                    // global-frame slot for the cell it aliases (D-12.2/D-12.4).
                    let local = self.slot_for(name);
                    let global = self.globals.slot_for(name);
                    bindings.push(GlobalBinding { local, global });
                }
                StmtKind::Global(bindings)
            }

            Statement::Declare(node) => {
                // Pick up `strict_types=N`; other directives (ticks/encoding) have
                // no observable effect in this runtime (D-16.1).
                for item in node.items.iter() {
                    if item.name.value.eq_ignore_ascii_case(b"strict_types") {
                        if let Expression::Literal(Literal::Integer(i)) = item.value {
                            self.strict = i.value == Some(1);
                        }
                    }
                }
                // `declare(...);` carries the following statement as its body (for
                // `strict_types` that is the `;` → a no-op); lower it through.
                match &node.body {
                    DeclareBody::Statement(s) => return self.lower_stmt(s),
                    DeclareBody::ColonDelimited(_) => {
                        return Err(LowerError::Unsupported {
                            what: "declare block body",
                            line,
                        })
                    }
                }
            }

            Statement::Static(node) => {
                let mut bindings = Vec::new();
                for item in node.items.iter() {
                    let (var, init) = match item {
                        StaticItem::Abstract(a) => (&a.variable, None),
                        StaticItem::Concrete(c) => {
                            (&c.variable, Some(self.lower_expr(c.value)?))
                        }
                    };
                    let slot = self.slot_for(strip_dollar(var.name));
                    let id = self.static_count;
                    self.static_count += 1;
                    bindings.push(StaticBinding { slot, id, init });
                }
                StmtKind::StaticVar(bindings)
            }

            Statement::Break(node) => StmtKind::Break(self.lower_level(node.level, line)?),
            Statement::Continue(node) => StmtKind::Continue(self.lower_level(node.level, line)?),

            Statement::Return(node) => match node.value {
                // Inside a `function &f()`, `return <lvalue>` returns a reference
                // to the place (D-13.2/D-13.3). A non-lvalue (or bare `return;`)
                // stays a value return; the runtime emits the by-ref Notice.
                Some(e) if self.fn_by_ref && is_returnable_lvalue(e) => {
                    StmtKind::ReturnRef(self.lower_place(e, line)?)
                }
                Some(e) => StmtKind::Return(Some(self.lower_expr(e)?)),
                None => StmtKind::Return(None),
            },

            // A function declaration carries no runtime behaviour: the top-level
            // ones were already hoisted into `functions`. A declaration that was
            // *not* hoisted is nested inside a branch/block — PHP defines it
            // conditionally, which is outside step 8 scope.
            Statement::Function(func) => {
                if self.fn_index.contains_key(&func.name.value.to_ascii_lowercase()) {
                    return Ok(None);
                }
                return Err(LowerError::Unsupported {
                    what: "conditional function declaration",
                    line,
                });
            }

            // A class declaration carries no runtime behaviour: the top-level
            // ones were already hoisted into `classes` (step 19, D-19.3). A class
            // nested inside a branch/block is a conditional declaration, outside
            // Tier-1 scope.
            Statement::Class(class) => {
                if self.class_index.contains_key(&class.name.value.to_ascii_lowercase()) {
                    return Ok(None);
                }
                return Err(LowerError::Unsupported {
                    what: "conditional class declaration",
                    line,
                });
            }
            Statement::Interface(iface) => {
                if self.class_index.contains_key(&iface.name.value.to_ascii_lowercase()) {
                    return Ok(None);
                }
                return Err(LowerError::Unsupported {
                    what: "conditional interface declaration",
                    line,
                });
            }
            // A trait declaration carries no runtime behaviour: the top-level
            // ones were lowered into `self.traits` and flattened into their
            // consumers at lowering time (step 21).
            Statement::Trait(_) => return Ok(None),

            // `try { } catch (T $e) { } finally { }` (step 20). Each catch's type
            // hint is a single class or a `A | B` union (collected to names); its
            // variable is optional (`catch (T)`); finally is optional.
            Statement::Try(node) => {
                let body = self.lower_stmts(node.block.statements.as_slice())?;
                let mut catches = Vec::with_capacity(node.catch_clauses.len());
                for c in node.catch_clauses.iter() {
                    let mut types = Vec::new();
                    collect_catch_types(&c.hint, line, &mut types)?;
                    let var = c
                        .variable
                        .as_ref()
                        .map(|d| self.slot_for(strip_dollar(d.name)));
                    let cbody = self.lower_stmts(c.block.statements.as_slice())?;
                    catches.push(crate::hir::CatchClause {
                        types,
                        var,
                        body: cbody,
                    });
                }
                let finally = match &node.finally_clause {
                    Some(f) => self.lower_stmts(f.block.statements.as_slice())?,
                    None => Vec::new(),
                };
                StmtKind::Try {
                    body,
                    catches,
                    finally,
                }
            }

            _ => {
                return Err(LowerError::Unsupported {
                    what: "statement",
                    line,
                })
            }
        };
        Ok(Some(Stmt { line, kind }))
    }

    /// `break`/`continue` level: optional, must be a constant integer >= 1.
    fn lower_level(
        &self,
        level: Option<&Expression>,
        line: Line,
    ) -> Result<u32, LowerError> {
        match level {
            None => Ok(1),
            Some(Expression::Literal(Literal::Integer(i))) => match i.value {
                Some(v) if v >= 1 && v <= u32::MAX as u64 => Ok(v as u32),
                _ => Err(LowerError::Unsupported {
                    what: "break/continue level",
                    line,
                }),
            },
            Some(_) => Err(LowerError::Unsupported {
                what: "non-constant break/continue level",
                line,
            }),
        }
    }

    // --- functions ---

    /// Lower a top-level function declaration and register it in the function
    /// table. A duplicate name is a redeclaration (PHP fatal), reported as an
    /// unsupported construct so the phpt-runner skips it rather than crashing.
    fn hoist_function(&mut self, func: &Function) -> Result<(), LowerError> {
        let decl = self.lower_function(func)?;
        let key = decl.name.to_ascii_lowercase();
        if self.fn_index.contains_key(&key) {
            return Err(LowerError::Unsupported {
                what: "function redeclaration",
                line: decl.line,
            });
        }
        let idx = self.functions.len();
        self.fn_index.insert(key, idx);
        self.functions.push(decl);
        Ok(())
    }

    // --- classes (step 19) ---

    /// Hoist top-level class declarations in two passes: first reserve every
    /// class name → index (so a method body / `new` can reference a class
    /// declared later), then lower each body now that all names resolve
    /// (D-19.3).
    fn hoist_classes(&mut self, stmts: &[Statement]) -> Result<(), LowerError> {
        // Both classes and interfaces share one table, so a class can implement
        // an interface declared later and vice versa (step 19-5).
        enum Pending<'a> {
            Class(&'a Class<'a>),
            Interface(&'a Interface<'a>),
        }
        let mut pending: Vec<Pending> = Vec::new();
        for s in stmts {
            let (name, span) = match s {
                Statement::Class(c) => (c.name.value, c.span()),
                Statement::Interface(i) => (i.name.value, i.span()),
                _ => continue,
            };
            let key = name.to_ascii_lowercase();
            if self.class_index.contains_key(&key) {
                return Err(LowerError::Unsupported {
                    what: "class/interface redeclaration",
                    line: self.line_of(span),
                });
            }
            // One entry per `pending` slot, pushed below in the same order, so the
            // index equals the eventual position in `self.classes`. Offset by the
            // current table length so user classes follow the injected built-in
            // exception prelude (step 20), keeping their ids contiguous after it.
            self.class_index
                .insert(key, self.classes.len() + pending.len());
            pending.push(match s {
                Statement::Class(c) => Pending::Class(c),
                Statement::Interface(i) => Pending::Interface(i),
                _ => unreachable!(),
            });
        }
        for p in pending {
            let decl = match p {
                Pending::Class(c) => self.lower_class(c)?,
                Pending::Interface(i) => self.lower_interface(i)?,
            };
            self.classes.push(decl);
        }
        Ok(())
    }

    /// Lower every top-level `trait T { ... }` into [`Lowerer::traits`] (step 21).
    /// Each is resolved on demand (so a trait may `use` another declared later)
    /// with a cycle guard; nested `use` clauses are flattened in (D-21.8).
    fn lower_traits(&mut self, stmts: &[Statement]) -> Result<(), LowerError> {
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
            (&own_m, &own_p, &own_s, &own_c),
            (&mut t_methods, &mut t_props, &mut t_static, &mut t_consts),
            &mut abstract_methods,
            line,
        )?;
        // Own members come last so the trait's own declarations win over inherited
        // ones; trait members keep their (declaration) order in front for layout.
        t_methods.extend(methods);
        t_props.extend(props);
        t_static.extend(static_props);
        t_consts.extend(consts);
        in_progress.remove(key);
        self.traits.insert(
            key.to_vec(),
            LoweredTrait {
                methods: t_methods,
                props: t_props,
                static_props: t_static,
                consts: t_consts,
                abstract_methods,
            },
        );
        Ok(())
    }

    /// Copy the members of every trait named in `uses` into the four `out` vecs,
    /// skipping any name the consumer already declares (`own_*`, precedence
    /// D-21.4) or that an earlier trait in this list already supplied (first
    /// wins; true conflict detection + `insteadof`/`as` arrive in 21-3). Reads
    /// `self.traits`, which the caller has ensured is fully resolved.
    #[allow(clippy::type_complexity)]
    fn flatten_into(
        &self,
        uses: &[&TraitUse],
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
        let mut seen_m = own_m.clone();
        let mut seen_p = own_p.clone();
        let mut seen_s = own_s.clone();
        let mut seen_c = own_c.clone();
        for u in uses {
            for tn in u.trait_names.iter() {
                let tkey = function_name(tn).to_ascii_lowercase();
                let lt = self.traits.get(&tkey).ok_or(LowerError::Unsupported {
                    what: "use of undefined trait",
                    line,
                })?;
                for m in &lt.methods {
                    if seen_m.insert(m.decl.name.to_ascii_lowercase()) {
                        methods.push(m.clone());
                    }
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
        Ok(())
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
    fn lower_interface(&mut self, iface: &Interface) -> Result<ClassDecl, LowerError> {
        let line = self.line_of(iface.span());
        let interfaces = match &iface.extends {
            Some(ext) => {
                let names: Vec<&[u8]> = ext.types.iter().map(function_name).collect();
                self.resolve_interfaces(&names, line)?
            }
            None => Vec::new(),
        };
        let mut consts = Vec::new();
        for member in iface.members.iter() {
            match member {
                ClassLikeMember::Constant(c) => self.lower_class_const(c, &mut consts)?,
                // Interface methods are signatures only (abstract) — no body to run.
                ClassLikeMember::Method(_) => {}
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
            line,
        })
    }

    /// Lower one `class Name { ... }` into a [`ClassDecl`] (step 19-1). Only
    /// instance properties and methods are in 19-1 scope; `extends`/`implements`,
    /// static members, constants, and other member kinds arrive in later
    /// sub-steps and lower to [`LowerError::Unsupported`] for now.
    fn lower_class(&mut self, class: &Class) -> Result<ClassDecl, LowerError> {
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
        for member in class.members.iter() {
            match member {
                ClassLikeMember::Property(p) => {
                    self.lower_property(p, &mut props, &mut static_props, line)?
                }
                // An abstract method is a signature only — no body to run, so it
                // is not materialised (a concrete subclass supplies the impl).
                ClassLikeMember::Method(m) if matches!(m.body, MethodBody::Abstract(_)) => {}
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
        // Flatten any `use TraitName;` members into this class (step 21). The
        // class's own declarations take precedence; trait members are placed in
        // front so the instance layout / dump order matches PHP's (`use` first).
        if !uses.is_empty() {
            let (own_m, own_p, own_s, own_c) =
                member_name_sets(&methods, &props, &static_props, &consts);
            let mut t_methods = Vec::new();
            let mut t_props = Vec::new();
            let mut t_static = Vec::new();
            let mut t_consts = Vec::new();
            let mut _abstract: Vec<Box<[u8]>> = Vec::new();
            self.flatten_into(
                &uses,
                (&own_m, &own_p, &own_s, &own_c),
                (&mut t_methods, &mut t_props, &mut t_static, &mut t_consts),
                &mut _abstract,
                line,
            )?;
            t_methods.extend(methods);
            methods = t_methods;
            t_props.extend(props);
            props = t_props;
            t_static.extend(static_props);
            static_props = t_static;
            t_consts.extend(consts);
            consts = t_consts;
        }
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

        let inner = (|| {
            let params = self.lower_params(&method.parameter_list, line)?;
            let body = self.lower_stmts(body.statements.as_slice())?;
            Ok::<_, LowerError>((params, body))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for method body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;

        let (params, body) = inner?;
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
    fn lower_function(&mut self, func: &Function) -> Result<FnDecl, LowerError> {
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

        let inner = self.lower_function_body(func, line);

        // Reclaim the function's local scope and restore the outer one.
        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for function body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;

        let (params, body) = inner?;
        let ret_hint = func
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(&r.hint));
        Ok(FnDecl {
            name,
            params,
            body,
            slots: local_scope.slots,
            by_ref,
            ret_hint,
            line,
        })
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
            if p.ellipsis.is_some() {
                return Err(LowerError::Unsupported {
                    what: "variadic parameter",
                    line,
                });
            }
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
    fn lower_closure(&mut self, closure: &Closure, line: Line) -> Result<ExprKind, LowerError> {
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

        let (params, captures, body) = inner?;
        let ret_hint = closure
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(&r.hint));
        let fn_idx = self.push_closure(params, body, local_scope.slots, by_ref, ret_hint, line);
        Ok(ExprKind::Closure {
            fn_idx,
            captures,
            bind_this,
        })
    }

    /// Append a lowered closure body to the flat table and return its index. The
    /// `FnDecl.name` is the PHP `{closure:file:line}` synthetic name (step 18).
    fn push_closure(
        &mut self,
        params: Vec<Param>,
        body: Vec<Stmt>,
        slots: Vec<Box<[u8]>>,
        by_ref: bool,
        ret_hint: Option<TypeHint>,
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
    fn lower_arrow_function(
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

        let (params, captures, body) = inner?;
        let ret_hint = af
            .return_type_hint
            .as_ref()
            .and_then(|r| lower_hint(&r.hint));
        let fn_idx = self.push_closure(params, body, local_scope.slots, false, ret_hint, line);
        // An arrow function is never `static` here (rejected above), so it binds
        // `$this` like an ordinary closure (step 19-6).
        Ok(ExprKind::Closure {
            fn_idx,
            captures,
            bind_this: true,
        })
    }

    // --- expressions ---

    fn lower_expr_list<'a, I>(&mut self, it: I) -> Result<Vec<Expr>, LowerError>
    where
        I: Iterator<Item = &'a &'a Expression<'a>>,
    {
        let mut out = Vec::new();
        for e in it {
            out.push(self.lower_expr(e)?);
        }
        Ok(out)
    }

    fn lower_expr(&mut self, e: &Expression) -> Result<Expr, LowerError> {
        // `( expr )` is transparent: keep the inner node (and its own line).
        if let Expression::Parenthesized(p) = e {
            return self.lower_expr(p.expression);
        }

        let line = self.line_of(e.span());
        let kind = match e {
            Expression::Literal(lit) => self.lower_literal(lit, line)?,

            Expression::Variable(Variable::Direct(d)) => {
                let name = strip_dollar(d.name);
                // `$this` is not a slot: it reads from the evaluator's current
                // object context (step 19, D-19.5).
                if name == b"this" {
                    ExprKind::This
                } else {
                    ExprKind::Var(self.slot_for(name))
                }
            }
            Expression::Variable(_) => {
                return Err(LowerError::Unsupported {
                    what: "variable variable",
                    line,
                })
            }

            Expression::Binary(b) => {
                // `instanceof`'s RHS is a *class* reference, not a value, so it is
                // handled before the operands are lowered as expressions (19-5).
                if let BinaryOperator::Instanceof(_) = b.operator {
                    let expr = Box::new(self.lower_expr(b.lhs)?);
                    let class = class_ref_of(b.rhs, line)?;
                    return Ok(Expr {
                        line,
                        kind: ExprKind::InstanceOf { expr, class },
                    });
                }
                let l = Box::new(self.lower_expr(b.lhs)?);
                let r = Box::new(self.lower_expr(b.rhs)?);
                match b.operator {
                    BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => ExprKind::And(l, r),
                    BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => ExprKind::Or(l, r),
                    BinaryOperator::LowXor(_) => ExprKind::Xor(l, r),
                    BinaryOperator::NullCoalesce(_) => ExprKind::Coalesce(l, r),
                    BinaryOperator::Instanceof(_) => unreachable!("handled above"),
                    other => ExprKind::Binary(map_binop(other), l, r),
                }
            }

            Expression::UnaryPrefix(u) => self.lower_unary_prefix(&u.operator, u.operand, line)?,
            Expression::UnaryPostfix(u) => match u.operator {
                UnaryPostfixOperator::PostIncrement(_) => self.lower_incdec(u.operand, true, false, line)?,
                UnaryPostfixOperator::PostDecrement(_) => self.lower_incdec(u.operand, false, false, line)?,
            },

            Expression::Assignment(a) => {
                // `$target = &$source`: reference binding (step 11a). Detect it
                // up front — `&$source` would otherwise reach the rejected
                // reference operator. Only bare-variable targets and sources are
                // in Tier 1 scope (`$x = &$a[0]` stays deferred).
                if let AssignmentOperator::Assign(_) = a.operator {
                    if let Expression::UnaryPrefix(u) = a.rhs {
                        if let UnaryPrefixOperator::Reference(_) = u.operator {
                            let target = self.lower_place(a.lhs, line)?;
                            // `$y = &f(...)`: alias the cell a by-reference
                            // function returns (step 13, D-13.5).
                            if let Expression::Call(_) = u.operand {
                                let call = Box::new(self.lower_expr(u.operand)?);
                                return Ok(Expr {
                                    line,
                                    kind: ExprKind::AssignRefCall { target, call },
                                });
                            }
                            // Otherwise both sides are places: a bare variable or
                            // an array element (step 11d-2). `lower_place` rejects
                            // anything that is not an lvalue.
                            let source = self.lower_place(u.operand, line)?;
                            return Ok(Expr {
                                line,
                                kind: ExprKind::AssignRef { target, source },
                            });
                        }
                    }
                }
                // `Class::$p = …` / `+= ` / `??=` — static-property assignment is
                // not a `Place` (it roots at a per-class cell, not a slot), so it
                // gets dedicated nodes (step 19-4, D-19.14).
                if let Expression::Access(Access::StaticProperty(sp)) = a.lhs {
                    let class = class_ref_of(sp.class, line)?;
                    let name: Box<[u8]> = static_prop_name(&sp.property, line)?.into();
                    let rhs = Box::new(self.lower_expr(a.rhs)?);
                    let op = static_assign_op(&a.operator);
                    return Ok(Expr {
                        line,
                        kind: ExprKind::StaticPropAssign {
                            class,
                            name,
                            op,
                            rhs,
                        },
                    });
                }
                let place = self.lower_place(a.lhs, line)?;
                let rhs = Box::new(self.lower_expr(a.rhs)?);
                // A bare variable keeps the slot-based encoding (lighter, and
                // preserves the existing diagnostics path); an array element
                // target uses the [`Place`]-based variants.
                let op = match a.operator {
                    AssignmentOperator::Assign(_) => None,
                    AssignmentOperator::Coalesce(_) => Some(AssignFlavour::Coalesce),
                    AssignmentOperator::Addition(_) => Some(AssignFlavour::Op(BinOp::Add)),
                    AssignmentOperator::Subtraction(_) => Some(AssignFlavour::Op(BinOp::Sub)),
                    AssignmentOperator::Multiplication(_) => Some(AssignFlavour::Op(BinOp::Mul)),
                    AssignmentOperator::Division(_) => Some(AssignFlavour::Op(BinOp::Div)),
                    AssignmentOperator::Modulo(_) => Some(AssignFlavour::Op(BinOp::Mod)),
                    AssignmentOperator::Exponentiation(_) => Some(AssignFlavour::Op(BinOp::Pow)),
                    AssignmentOperator::Concat(_) => Some(AssignFlavour::Op(BinOp::Concat)),
                    AssignmentOperator::BitwiseAnd(_) => Some(AssignFlavour::Op(BinOp::BitAnd)),
                    AssignmentOperator::BitwiseOr(_) => Some(AssignFlavour::Op(BinOp::BitOr)),
                    AssignmentOperator::BitwiseXor(_) => Some(AssignFlavour::Op(BinOp::BitXor)),
                    AssignmentOperator::LeftShift(_) => Some(AssignFlavour::Op(BinOp::Shl)),
                    AssignmentOperator::RightShift(_) => Some(AssignFlavour::Op(BinOp::Shr)),
                };
                // A bare *local* variable keeps the lighter slot-based encoding
                // (and the existing diagnostics path). A `$GLOBALS['x']` target
                // has empty steps too but a global base, so it must take the
                // Place-based variant to reach the global frame (D-12.3).
                if let (PlaceBase::Local(slot), true) = (place.base, place.steps.is_empty()) {
                    match op {
                        None => ExprKind::Assign(slot, rhs),
                        Some(AssignFlavour::Coalesce) => ExprKind::AssignCoalesce(slot, rhs),
                        Some(AssignFlavour::Op(b)) => ExprKind::AssignOp(b, slot, rhs),
                    }
                } else {
                    match op {
                        None => ExprKind::AssignPlace(place, rhs),
                        Some(AssignFlavour::Coalesce) => ExprKind::AssignCoalescePlace(place, rhs),
                        Some(AssignFlavour::Op(b)) => ExprKind::AssignOpPlace(b, place, rhs),
                    }
                }
            }

            Expression::Conditional(c) => ExprKind::Ternary {
                cond: Box::new(self.lower_expr(c.condition)?),
                then: match c.then {
                    Some(t) => Some(Box::new(self.lower_expr(t)?)),
                    None => None,
                },
                otherwise: Box::new(self.lower_expr(c.r#else)?),
            },

            Expression::Call(call) => self.lower_call(call, line)?,

            // `new ClassName(args)` (step 19, D-19.6). Tier-1 resolves the class
            // as a literal identifier; `new $var` / `new self` / `new static`
            // arrive in later sub-steps.
            Expression::Instantiation(inst) => self.lower_instantiation(inst, line)?,

            // `throw <expr>` (step 20). Valid as a statement or, in PHP 8, an
            // expression (`$x ?? throw new …`); both reach here.
            Expression::Throw(t) => ExprKind::Throw(Box::new(self.lower_expr(t.exception)?)),

            // `$obj->prop` (step 19, D-19.8). Static / class-constant accesses are
            // later sub-steps.
            Expression::Access(access) => self.lower_access(access, line)?,

            Expression::Closure(closure) => self.lower_closure(closure, line)?,
            Expression::ArrowFunction(af) => self.lower_arrow_function(af, line)?,

            // A first-class callable `name(...)` (step 18-6, D-18.10).
            Expression::PartialApplication(pa) => self.lower_partial_application(pa, line)?,

            // A bare `NAME` constant: only the known engine constants are
            // resolved (to a literal at lowering time, D-18.7); user-defined
            // constants stay unsupported (the script becomes a SKIP).
            Expression::ConstantAccess(ca) => match resolve_constant(function_name(&ca.name)) {
                Some(kind) => kind,
                None => {
                    return Err(LowerError::Unsupported {
                        what: "named constant",
                        line,
                    })
                }
            },

            Expression::Array(arr) => ExprKind::Array(self.lower_array_elements(arr.elements.iter(), line)?),
            Expression::LegacyArray(arr) => {
                ExprKind::Array(self.lower_array_elements(arr.elements.iter(), line)?)
            }

            Expression::ArrayAccess(aa) => {
                // `$GLOBALS['x']` reads as the global slot directly; a nested
                // `$GLOBALS['x'][k]` becomes `Index { base: GlobalVar, .. }`
                // since the inner access lowers to `GlobalVar` (D-12.3).
                if let Some(key) = globals_key(aa.array, aa.index) {
                    ExprKind::GlobalVar(self.globals.slot_for(&key))
                } else {
                    ExprKind::Index {
                        base: Box::new(self.lower_expr(aa.array)?),
                        index: Box::new(self.lower_expr(aa.index)?),
                    }
                }
            }
            // `$a[]` only has meaning as an assignment target; reading it is an error.
            Expression::ArrayAppend(_) => {
                return Err(LowerError::Unsupported {
                    what: "[] in read context",
                    line,
                })
            }

            Expression::Construct(c) => match c {
                Construct::Isset(is) => {
                    let mut places = Vec::new();
                    for v in is.values.iter() {
                        places.push(self.lower_place(v, line)?);
                    }
                    ExprKind::Isset(places)
                }
                Construct::Empty(em) => ExprKind::Empty(self.lower_place(em.value, line)?),
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "language construct",
                        line,
                    })
                }
            },

            Expression::Match(m) => {
                let subject = Box::new(self.lower_expr(m.expression)?);
                let mut arms = Vec::new();
                for arm in m.arms.iter() {
                    let (conditions, body) = match arm {
                        AstMatchArm::Expression(ea) => {
                            let mut conds = Vec::new();
                            for c in ea.conditions.iter() {
                                conds.push(self.lower_expr(c)?);
                            }
                            (conds, self.lower_expr(ea.expression)?)
                        }
                        AstMatchArm::Default(da) => (Vec::new(), self.lower_expr(da.expression)?),
                    };
                    arms.push(MatchArm { conditions, body });
                }
                ExprKind::Match { subject, arms }
            }

            _ => {
                return Err(LowerError::Unsupported {
                    what: "expression",
                    line,
                })
            }
        };
        Ok(Expr { line, kind })
    }

    fn lower_literal(&self, lit: &Literal, line: Line) -> Result<ExprKind, LowerError> {
        Ok(match lit {
            Literal::Null(_) => ExprKind::Null,
            Literal::True(_) => ExprKind::Bool(true),
            Literal::False(_) => ExprKind::Bool(false),
            Literal::Float(f) => ExprKind::Float(*f.value),
            Literal::Integer(i) => lower_int(i, line)?,
            Literal::String(s) => match s.value {
                Some(bytes) => ExprKind::Str(bytes.into()),
                // Interpolated content is `CompositeString`, not `Literal::String`,
                // so a `None` here is an unparsable literal we defer.
                None => {
                    return Err(LowerError::Unsupported {
                        what: "unparsable string literal",
                        line,
                    })
                }
            },
        })
    }

    fn lower_unary_prefix(
        &mut self,
        op: &UnaryPrefixOperator,
        operand: &Expression,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        use UnaryPrefixOperator as P;
        let cast = |k: CastKind, this: &mut Self| -> Result<ExprKind, LowerError> {
            Ok(ExprKind::Cast(k, Box::new(this.lower_expr(operand)?)))
        };
        Ok(match op {
            P::Negation(_) => ExprKind::Unary(UnOp::Neg, Box::new(self.lower_expr(operand)?)),
            P::Plus(_) => ExprKind::Unary(UnOp::Plus, Box::new(self.lower_expr(operand)?)),
            P::Not(_) => ExprKind::Unary(UnOp::Not, Box::new(self.lower_expr(operand)?)),
            P::BitwiseNot(_) => ExprKind::Unary(UnOp::BitNot, Box::new(self.lower_expr(operand)?)),
            P::PreIncrement(_) => self.lower_incdec(operand, true, true, line)?,
            P::PreDecrement(_) => self.lower_incdec(operand, false, true, line)?,
            P::IntCast(..) | P::IntegerCast(..) => cast(CastKind::Int, self)?,
            P::FloatCast(..) | P::DoubleCast(..) | P::RealCast(..) => cast(CastKind::Float, self)?,
            P::StringCast(..) | P::BinaryCast(..) => cast(CastKind::String, self)?,
            P::BoolCast(..) | P::BooleanCast(..) => cast(CastKind::Bool, self)?,
            P::ArrayCast(..) => cast(CastKind::Array, self)?,
            P::ObjectCast(..) | P::UnsetCast(..) | P::VoidCast(..) => {
                return Err(LowerError::Unsupported {
                    what: "object/unset/void cast",
                    line,
                })
            }
            P::ErrorControl(_) => {
                return Err(LowerError::Unsupported {
                    what: "@ error-control operator",
                    line,
                })
            }
            P::Reference(_) => {
                return Err(LowerError::Unsupported {
                    what: "reference operator",
                    line,
                })
            }
        })
    }

    fn lower_incdec(
        &mut self,
        operand: &Expression,
        inc: bool,
        pre: bool,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        match operand {
            // A bare local keeps the lighter slot-based encoding (and its
            // string/null increment diagnostics). `$this` is not a slot, so it
            // falls through to the place form below.
            Expression::Variable(Variable::Direct(d)) if strip_dollar(d.name) != b"this" => {
                Ok(ExprKind::IncDec {
                    slot: self.slot_for(strip_dollar(d.name)),
                    inc,
                    pre,
                })
            }
            // `Class::$p++` — static-property inc/dec (step 19-4), its own node.
            Expression::Access(Access::StaticProperty(sp)) => Ok(ExprKind::StaticPropIncDec {
                class: class_ref_of(sp.class, line)?,
                name: static_prop_name(&sp.property, line)?.into(),
                inc,
                pre,
            }),
            // An array element / object property target (step 19-2): reuse the
            // place machinery. `lower_place` rejects non-lvalues.
            _ => Ok(ExprKind::IncDecPlace {
                place: self.lower_place(operand, line)?,
                inc,
                pre,
            }),
        }
    }

    /// Lower a call. Tier 1 supports only direct calls to a named function with
    /// positional arguments (builtins); methods, static calls, dynamic callees,
    /// and named/variadic arguments are deferred.
    fn lower_call(&mut self, call: &Call, line: Line) -> Result<ExprKind, LowerError> {
        let fc = match call {
            Call::Function(fc) => fc,
            // `$obj->method(args)` instance call (step 19, D-19.7).
            Call::Method(mc) => {
                let object = Box::new(self.lower_expr(mc.object)?);
                let method = member_name(&mc.method, line)?;
                let args = self.lower_positional_args(&mc.argument_list, line)?;
                return Ok(ExprKind::MethodCall {
                    object,
                    method: method.into(),
                    args,
                    nullsafe: false,
                });
            }
            Call::NullSafeMethod(mc) => {
                let object = Box::new(self.lower_expr(mc.object)?);
                let method = member_name(&mc.method, line)?;
                let args = self.lower_positional_args(&mc.argument_list, line)?;
                return Ok(ExprKind::MethodCall {
                    object,
                    method: method.into(),
                    args,
                    nullsafe: true,
                });
            }
            // `Class::m()` / `self::m()` / `parent::m()` / `static::m()`.
            Call::StaticMethod(sm) => {
                let class = class_ref_of(sm.class, line)?;
                let method = member_name(&sm.method, line)?;
                let args = self.lower_positional_args(&sm.argument_list, line)?;
                return Ok(ExprKind::StaticCall {
                    class,
                    method: method.into(),
                    args,
                });
            }
        };
        // A non-identifier callee (`$f(...)`, `$a['k'](...)`, an IIFE) is a
        // dynamic call dispatched on the runtime callee value (step 18, D-18.5).
        let name = match fc.function {
            Expression::Identifier(id) => function_name(id),
            other => {
                let callee = Box::new(self.lower_expr(other)?);
                let args = self.lower_positional_args(&fc.argument_list, line)?;
                return Ok(ExprKind::CallDynamic { callee, args });
            }
        };
        let args = self.lower_positional_args(&fc.argument_list, line)?;
        Ok(ExprKind::Call {
            name: name.into(),
            args,
        })
    }

    /// Lower `new ClassName(args)` (step 19, D-19.6). Only a literal class name is
    /// in 19-1 scope; a dynamic / `self` / `static` class lowers to unsupported.
    fn lower_instantiation(
        &mut self,
        inst: &Instantiation,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        let class = class_ref_of(inst.class, line)?;
        let args = match &inst.argument_list {
            Some(list) => self.lower_positional_args(list, line)?,
            None => Vec::new(),
        };
        Ok(ExprKind::New { class, args })
    }

    /// Lower `$obj->prop` / `$obj?->prop` reads (step 19, D-19.8). Static-property
    /// and class-constant accesses (`::`) are later sub-steps.
    fn lower_access(&mut self, access: &Access, line: Line) -> Result<ExprKind, LowerError> {
        match access {
            Access::Property(p) => Ok(ExprKind::PropGet {
                object: Box::new(self.lower_expr(p.object)?),
                name: member_name(&p.property, line)?.into(),
                nullsafe: false,
            }),
            Access::NullSafeProperty(p) => Ok(ExprKind::PropGet {
                object: Box::new(self.lower_expr(p.object)?),
                name: member_name(&p.property, line)?.into(),
                nullsafe: true,
            }),
            // `Class::CONST` / `self::CONST` / `Class::class` (step 19-4).
            Access::ClassConstant(cc) => {
                let class = class_ref_of(cc.class, line)?;
                let name = match &cc.constant {
                    ClassLikeConstantSelector::Identifier(id) => id.value,
                    _ => {
                        return Err(LowerError::Unsupported {
                            what: "dynamic class constant name",
                            line,
                        })
                    }
                };
                Ok(ExprKind::ClassConst {
                    class,
                    name: name.into(),
                })
            }
            // `Class::$prop` static-property read (step 19-4).
            Access::StaticProperty(sp) => {
                let class = class_ref_of(sp.class, line)?;
                let name = static_prop_name(&sp.property, line)?;
                Ok(ExprKind::StaticProp {
                    class,
                    name: name.into(),
                })
            }
        }
    }

    /// Lower a first-class callable `name(...)` (step 18-6, D-18.10). Only the
    /// plain function form with the `(...)` placeholder is supported; method /
    /// static-method first-class callables and partial applications with real
    /// placeholders stay unsupported (OOP / scope-out).
    fn lower_partial_application(
        &mut self,
        pa: &PartialApplication,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        let func = match pa {
            PartialApplication::Function(f) if f.argument_list.is_first_class_callable() => f,
            _ => {
                return Err(LowerError::Unsupported {
                    what: "partial application",
                    line,
                })
            }
        };
        let name = match func.function {
            Expression::Identifier(id) => function_name(id),
            _ => {
                return Err(LowerError::Unsupported {
                    what: "dynamic first-class callable",
                    line,
                })
            }
        };
        Ok(ExprKind::FirstClassCallable(name.into()))
    }

    /// Lower a call's argument list, accepting only plain positional arguments
    /// (named / variadic-spread arguments stay out of scope).
    fn lower_positional_args(
        &mut self,
        list: &mago_syntax::ast::ArgumentList,
        line: Line,
    ) -> Result<Vec<Expr>, LowerError> {
        let mut args = Vec::new();
        for arg in list.arguments.iter() {
            match arg {
                Argument::Positional(p) if p.ellipsis.is_none() => {
                    args.push(self.lower_expr(p.value)?);
                }
                _ => {
                    return Err(LowerError::Unsupported {
                        what: "named or variadic argument",
                        line,
                    })
                }
            }
        }
        Ok(args)
    }

    /// Resolve an lvalue: a base variable plus a chain of index steps. `$x`
    /// yields an empty step list; `$a[k]`, `$a[]`, and nested forms append
    /// [`PlaceStep`]s. Property and `list()` targets stay out of Tier 1 scope.
    fn lower_place(&mut self, lhs: &Expression, line: Line) -> Result<Place, LowerError> {
        match lhs {
            Expression::Parenthesized(p) => self.lower_place(p.expression, line),
            Expression::Variable(Variable::Direct(d)) => {
                let name = strip_dollar(d.name);
                let base = if name == b"this" {
                    PlaceBase::This
                } else {
                    PlaceBase::Local(self.slot_for(name))
                };
                Ok(Place {
                    base,
                    steps: Vec::new(),
                })
            }
            Expression::ArrayAccess(aa) => {
                // `$GLOBALS['x']` is a global base with no steps; `$GLOBALS['x'][k]`
                // recurses so the global base carries the `[k]` step (D-12.3).
                if let Some(key) = globals_key(aa.array, aa.index) {
                    return Ok(Place {
                        base: PlaceBase::Global(self.globals.slot_for(&key)),
                        steps: Vec::new(),
                    });
                }
                let mut place = self.lower_place(aa.array, line)?;
                place.steps.push(PlaceStep::Index(self.lower_expr(aa.index)?));
                Ok(place)
            }
            Expression::ArrayAppend(ap) => {
                let mut place = self.lower_place(ap.array, line)?;
                place.steps.push(PlaceStep::Append);
                Ok(place)
            }
            // `$obj->prop = ...`, `$this->prop = ...` — a property write target
            // (step 19, D-19.9). The base is the object-bearing expression; a
            // `Prop` step navigates into it. Property writes whose base is not a
            // place (e.g. `(new C)->x = 1`) are rare and stay unsupported via the
            // base's own `lower_place`.
            Expression::Access(Access::Property(p)) => {
                let mut place = self.lower_place(p.object, line)?;
                place
                    .steps
                    .push(PlaceStep::Prop(member_name(&p.property, line)?.into()));
                Ok(place)
            }
            _ => Err(LowerError::Unsupported {
                what: "assignment target",
                line,
            }),
        }
    }

    /// A `foreach` key/value target: Tier 1 supports only a direct variable
    /// (`list()` destructuring is deferred).
    fn foreach_slot(&mut self, target: &Expression, line: Line) -> Result<Slot, LowerError> {
        match target {
            Expression::Variable(Variable::Direct(d)) => Ok(self.slot_for(strip_dollar(d.name))),
            _ => Err(LowerError::Unsupported {
                what: "foreach list target",
                line,
            }),
        }
    }

    /// A `foreach` *value* target, which may be by reference (`&$v`, step 11d-3).
    /// Returns the bound slot plus whether the binding is by reference.
    fn foreach_value_slot(
        &mut self,
        target: &Expression,
        line: Line,
    ) -> Result<(Slot, bool), LowerError> {
        if let Expression::UnaryPrefix(u) = target {
            if let UnaryPrefixOperator::Reference(_) = u.operator {
                return Ok((self.foreach_slot(u.operand, line)?, true));
            }
        }
        Ok((self.foreach_slot(target, line)?, false))
    }

    /// Lower the elements of an array literal. Keyed and keyless elements are
    /// supported; spread (`...$x`) and missing elements are deferred.
    fn lower_array_elements<'a, I>(&mut self, it: I, line: Line) -> Result<Vec<ArrayElem>, LowerError>
    where
        I: Iterator<Item = &'a ArrayElement<'a>>,
    {
        let mut out = Vec::new();
        for el in it {
            match el {
                ArrayElement::KeyValue(kv) => out.push(ArrayElem {
                    key: Some(self.lower_expr(kv.key)?),
                    value: self.lower_expr(kv.value)?,
                }),
                ArrayElement::Value(v) => out.push(ArrayElem {
                    key: None,
                    value: self.lower_expr(v.value)?,
                }),
                ArrayElement::Variadic(_) | ArrayElement::Missing(_) => {
                    return Err(LowerError::Unsupported {
                        what: "array spread / missing element",
                        line,
                    })
                }
            }
        }
        Ok(out)
    }
}

/// The kind of compound assignment, abstracted over the lvalue encoding.
enum AssignFlavour {
    Coalesce,
    Op(BinOp),
}

/// Unqualified function name: the segment after the last `\` (so `\strlen` and
/// `Foo\strlen` both resolve to `strlen`). Tier 1 has no namespaces, so this is
/// a faithful-enough resolution for global/builtin calls.
/// Collect the class names of a `catch` type hint (step 20): a single
/// `Identifier`, or a `A | B` union (recursively). Any other hint shape in catch
/// position is outside scope.
fn collect_catch_types(
    hint: &Hint,
    line: Line,
    out: &mut Vec<Box<[u8]>>,
) -> Result<(), LowerError> {
    match hint {
        Hint::Identifier(id) => {
            out.push(function_name(id).into());
            Ok(())
        }
        Hint::Union(u) => {
            collect_catch_types(u.left, line, out)?;
            collect_catch_types(u.right, line, out)
        }
        _ => Err(LowerError::Unsupported {
            what: "catch type",
            line,
        }),
    }
}

/// ASCII-lowercased name sets for a member group — used to give a class/trait's
/// own declarations precedence over flattened trait members (step 21, D-21.4).
#[allow(clippy::type_complexity)]
fn member_name_sets(
    methods: &[MethodDecl],
    props: &[PropDecl],
    static_props: &[crate::hir::StaticPropDecl],
    consts: &[crate::hir::ClassConstDecl],
) -> (
    HashSet<Vec<u8>>,
    HashSet<Vec<u8>>,
    HashSet<Vec<u8>>,
    HashSet<Vec<u8>>,
) {
    (
        methods
            .iter()
            .map(|m| m.decl.name.to_ascii_lowercase())
            .collect(),
        props.iter().map(|p| p.name.to_ascii_lowercase()).collect(),
        static_props
            .iter()
            .map(|p| p.name.to_ascii_lowercase())
            .collect(),
        consts.iter().map(|c| c.name.to_ascii_lowercase()).collect(),
    )
}

fn function_name<'a>(id: &Identifier<'a>) -> &'a [u8] {
    let raw = match id {
        Identifier::Local(l) => l.value,
        Identifier::Qualified(q) => q.value,
        Identifier::FullyQualified(f) => f.value,
    };
    match raw.iter().rposition(|&b| b == b'\\') {
        Some(i) => &raw[i + 1..],
        None => raw,
    }
}

/// The textual name of a member selector (`->name`, method/property). Tier-1
/// supports only the static-identifier form; a dynamic selector (`$obj->$n`,
/// `$obj->{expr}`) is out of 19-1 scope (step 19).
fn member_name<'a>(sel: &ClassLikeMemberSelector<'a>, line: Line) -> Result<&'a [u8], LowerError> {
    match sel {
        ClassLikeMemberSelector::Identifier(id) => Ok(id.value),
        _ => Err(LowerError::Unsupported {
            what: "dynamic member name",
            line,
        }),
    }
}

/// The single parent class name in an `extends` clause (PHP classes are
/// single-inheritance, so only the first type matters), step 19-3.
fn parent_name<'a>(ext: &Extends<'a>, line: Line) -> Result<&'a [u8], LowerError> {
    match ext.types.iter().next() {
        Some(id) => Ok(function_name(id)),
        None => Err(LowerError::Unsupported {
            what: "empty extends clause",
            line,
        }),
    }
}

/// Map an assignment operator to the static-property assignment flavour (19-4).
fn static_assign_op(op: &AssignmentOperator) -> StaticAssignOp {
    match op {
        AssignmentOperator::Assign(_) => StaticAssignOp::Plain,
        AssignmentOperator::Coalesce(_) => StaticAssignOp::Coalesce,
        AssignmentOperator::Addition(_) => StaticAssignOp::Op(BinOp::Add),
        AssignmentOperator::Subtraction(_) => StaticAssignOp::Op(BinOp::Sub),
        AssignmentOperator::Multiplication(_) => StaticAssignOp::Op(BinOp::Mul),
        AssignmentOperator::Division(_) => StaticAssignOp::Op(BinOp::Div),
        AssignmentOperator::Modulo(_) => StaticAssignOp::Op(BinOp::Mod),
        AssignmentOperator::Exponentiation(_) => StaticAssignOp::Op(BinOp::Pow),
        AssignmentOperator::Concat(_) => StaticAssignOp::Op(BinOp::Concat),
        AssignmentOperator::BitwiseAnd(_) => StaticAssignOp::Op(BinOp::BitAnd),
        AssignmentOperator::BitwiseOr(_) => StaticAssignOp::Op(BinOp::BitOr),
        AssignmentOperator::BitwiseXor(_) => StaticAssignOp::Op(BinOp::BitXor),
        AssignmentOperator::LeftShift(_) => StaticAssignOp::Op(BinOp::Shl),
        AssignmentOperator::RightShift(_) => StaticAssignOp::Op(BinOp::Shr),
    }
}

/// The name of a static property selector `::$name` (without the `$`), step
/// 19-4. Only a direct variable is supported (dynamic `::$$x` stays out).
fn static_prop_name<'a>(var: &Variable<'a>, line: Line) -> Result<&'a [u8], LowerError> {
    match var {
        Variable::Direct(d) => Ok(strip_dollar(d.name)),
        _ => Err(LowerError::Unsupported {
            what: "dynamic static property name",
            line,
        }),
    }
}

/// Classify the class side of a `::` static call. 19-3 handles `self`/`parent`;
/// named classes and `static::` (late static binding) are step 19-4.
fn class_ref_of(class: &Expression, line: Line) -> Result<ClassRef, LowerError> {
    match class {
        Expression::Self_(_) => Ok(ClassRef::SelfClass),
        Expression::Parent(_) => Ok(ClassRef::Parent),
        Expression::Static(_) => Ok(ClassRef::Static),
        Expression::Identifier(id) => Ok(ClassRef::Named(function_name(id).into())),
        // A dynamic class reference (`$cls::m()`, `new $cls`) stays out of scope.
        _ => Err(LowerError::Unsupported {
            what: "dynamic class reference",
            line,
        }),
    }
}

/// The visibility declared by a modifier list, defaulting to `Public` when none
/// is written (step 19, D-19.13).
fn visibility_of<'a>(modifiers: impl Iterator<Item = &'a Modifier<'a>>) -> Visibility {
    for m in modifiers {
        match m {
            Modifier::Public(_) => return Visibility::Public,
            Modifier::Protected(_) => return Visibility::Protected,
            Modifier::Private(_) => return Visibility::Private,
            _ => {}
        }
    }
    Visibility::Public
}

/// Resolve a bare constant name to its literal HIR value (step 18, D-18.7).
/// Only engine constants are known; `true`/`false`/`null` are case-insensitive,
/// every other name is case-sensitive (PHP constants are). `None` for an unknown
/// (user-defined) constant — the caller turns that into an Unsupported SKIP.
fn resolve_constant(name: &[u8]) -> Option<ExprKind> {
    match name.to_ascii_lowercase().as_slice() {
        b"true" => return Some(ExprKind::Bool(true)),
        b"false" => return Some(ExprKind::Bool(false)),
        b"null" => return Some(ExprKind::Null),
        _ => {}
    }
    let str_lit = |s: &[u8]| ExprKind::Str(s.to_vec().into_boxed_slice());
    Some(match name {
        // Integer limits / sizes.
        b"PHP_INT_MAX" => ExprKind::Int(i64::MAX),
        b"PHP_INT_MIN" => ExprKind::Int(i64::MIN),
        b"PHP_INT_SIZE" => ExprKind::Int(8),
        b"PHP_FLOAT_DIG" => ExprKind::Int(15),
        // Float limits.
        b"PHP_FLOAT_EPSILON" => ExprKind::Float(f64::EPSILON),
        b"PHP_FLOAT_MAX" => ExprKind::Float(f64::MAX),
        b"PHP_FLOAT_MIN" => ExprKind::Float(f64::MIN_POSITIVE),
        b"NAN" => ExprKind::Float(f64::NAN),
        b"INF" => ExprKind::Float(f64::INFINITY),
        // Versions / platform.
        b"PHP_EOL" => str_lit(b"\n"),
        b"PHP_VERSION" => str_lit(b"8.5.7"),
        b"PHP_MAJOR_VERSION" => ExprKind::Int(8),
        b"PHP_MINOR_VERSION" => ExprKind::Int(5),
        b"PHP_RELEASE_VERSION" => ExprKind::Int(7),
        b"PHP_VERSION_ID" => ExprKind::Int(80507),
        // str_pad / array_filter / count flags.
        b"STR_PAD_RIGHT" => ExprKind::Int(1),
        b"STR_PAD_LEFT" => ExprKind::Int(0),
        b"STR_PAD_BOTH" => ExprKind::Int(2),
        b"ARRAY_FILTER_USE_KEY" => ExprKind::Int(2),
        b"ARRAY_FILTER_USE_BOTH" => ExprKind::Int(1),
        b"COUNT_NORMAL" => ExprKind::Int(0),
        b"COUNT_RECURSIVE" => ExprKind::Int(1),
        // sort flags.
        b"SORT_REGULAR" => ExprKind::Int(0),
        b"SORT_NUMERIC" => ExprKind::Int(1),
        b"SORT_STRING" => ExprKind::Int(2),
        b"SORT_DESC" => ExprKind::Int(3),
        b"SORT_ASC" => ExprKind::Int(4),
        b"SORT_LOCALE_STRING" => ExprKind::Int(5),
        b"SORT_NATURAL" => ExprKind::Int(6),
        b"SORT_FLAG_CASE" => ExprKind::Int(8),
        // Math.
        b"M_PI" => ExprKind::Float(std::f64::consts::PI),
        b"M_E" => ExprKind::Float(std::f64::consts::E),
        b"M_SQRT2" => ExprKind::Float(std::f64::consts::SQRT_2),
        b"M_SQRT1_2" => ExprKind::Float(std::f64::consts::FRAC_1_SQRT_2),
        b"M_SQRT3" => ExprKind::Float(1.732_050_807_568_877_2),
        b"M_PI_2" => ExprKind::Float(std::f64::consts::FRAC_PI_2),
        b"M_PI_4" => ExprKind::Float(std::f64::consts::FRAC_PI_4),
        b"M_2_PI" => ExprKind::Float(std::f64::consts::FRAC_2_PI),
        b"M_LN2" => ExprKind::Float(std::f64::consts::LN_2),
        b"M_LN10" => ExprKind::Float(std::f64::consts::LN_10),
        b"M_LOG2E" => ExprKind::Float(std::f64::consts::LOG2_E),
        b"M_LOG10E" => ExprKind::Float(std::f64::consts::LOG10_E),
        b"M_EULER" => ExprKind::Float(0.577_215_664_901_532_9),
        _ => return None,
    })
}

/// Collect the names (with leading `$`) of every direct variable reachable from
/// `expr`, used to discover an arrow function's free variables (step 18, D-18.4).
///
/// The walk descends into nested closures/arrows too. Over-collecting is safe:
/// a captured-but-unused value is bound into an unused slot and never observed,
/// while a variable a nested closure references via `use` must be available in
/// the arrow's frame — so seeing it here is exactly what makes that work.
fn collect_direct_vars<'a>(expr: &'a Expression<'a>, out: &mut Vec<&'a [u8]>) {
    let mut stack: Vec<Node<'a, 'a>> = vec![Node::Expression(expr)];
    while let Some(node) = stack.pop() {
        if let Node::DirectVariable(d) = node {
            if !out.contains(&d.name) {
                out.push(d.name);
            }
        }
        stack.extend(node.children());
    }
}

/// Drop a single leading newline (`\r\n` or `\n`) — the byte `?>` swallows.
fn strip_one_newline(bytes: &[u8]) -> &[u8] {
    if let Some(rest) = bytes.strip_prefix(b"\r\n") {
        rest
    } else if let Some(rest) = bytes.strip_prefix(b"\n") {
        rest
    } else {
        bytes
    }
}

/// Strip the leading `$` from a mago direct-variable name (`b"$foo"` → `b"foo"`).
/// Recognise `$GLOBALS['constant-string']` — the superglobal indexed by a
/// literal string — and return the decoded global variable name (step 12-3,
/// D-12.3). A dynamic index or the whole `$GLOBALS` array yields `None`; the
/// caller then treats `$GLOBALS` as an ordinary variable (those forms are out of
/// step 12 scope, D-12.6).
fn globals_key(array: &Expression, index: &Expression) -> Option<Vec<u8>> {
    let Expression::Variable(Variable::Direct(d)) = array else {
        return None;
    };
    if strip_dollar(d.name) != b"GLOBALS".as_slice() {
        return None;
    }
    match index {
        Expression::Literal(Literal::String(s)) => s.value.map(|b| b.to_vec()),
        _ => None,
    }
}

/// Whether `e` is a place that can be returned by reference (`return <lvalue>`
/// in a `function &f()`, step 13). Only the lvalue shapes `lower_place` accepts
/// as a *readable* place: a direct variable or an array access (incl.
/// `$GLOBALS['x']`), through parentheses. `$a[]` (append) is not readable.
fn is_returnable_lvalue(e: &Expression) -> bool {
    match e {
        Expression::Variable(Variable::Direct(_)) => true,
        Expression::ArrayAccess(_) => true,
        Expression::Parenthesized(p) => is_returnable_lvalue(p.expression),
        _ => false,
    }
}

/// Map an AST type hint to an enforced [`TypeHint`], or `None` for any hint that
/// step 14 does not enforce (class, union, array, mixed, …). Only the four
/// scalar hints and their nullable forms are enforced (D-14.1/D-14.2).
fn lower_hint(hint: &Hint) -> Option<TypeHint> {
    let scalar = match hint {
        Hint::Integer(_) => ScalarType::Int,
        Hint::Float(_) => ScalarType::Float,
        Hint::String(_) => ScalarType::String,
        Hint::Bool(_) => ScalarType::Bool,
        Hint::Nullable(n) => {
            // `?int` etc.: enforce only when the inner hint is itself scalar.
            let inner = lower_hint(n.hint)?;
            return Some(TypeHint {
                nullable: true,
                ..inner
            });
        }
        _ => return None,
    };
    Some(TypeHint {
        kind: scalar,
        nullable: false,
    })
}

fn strip_dollar(name: &[u8]) -> &[u8] {
    if name.first() == Some(&b'$') {
        &name[1..]
    } else {
        name
    }
}

/// PHP integer literal → HIR. Values exceeding `i64::MAX` promote to float,
/// matching PHP's lexer. A literal too large even for `u64` (mago clamps its
/// `value` to `u64::MAX`) is re-parsed from its own decimal text, so a
/// several-hundred-digit literal becomes `INF` exactly as PHP does (bug #74947)
/// rather than the clamped `~1.8e19`.
fn lower_int(lit: &LiteralInteger, line: Line) -> Result<ExprKind, LowerError> {
    if let Some(v) = lit.value {
        if v <= i64::MAX as u64 {
            return Ok(ExprKind::Int(v as i64));
        }
    }
    // Overflows i64: promote to float by parsing the literal's own text (decimal
    // only — hex/oct/bin overflow falls back to mago's value).
    let raw = std::str::from_utf8(lit.raw).map_err(|_| LowerError::Unsupported {
        what: "integer literal",
        line,
    })?;
    let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
    if let Ok(f) = cleaned.parse::<f64>() {
        return Ok(ExprKind::Float(f));
    }
    match lit.value {
        Some(v) => Ok(ExprKind::Float(v as f64)),
        None => Err(LowerError::Unsupported {
            what: "integer literal overflow",
            line,
        }),
    }
}

/// Map a non-logical, non-coalesce binary operator to its HIR counterpart.
/// Logical (`&&`, `||`, `and`, `or`, `xor`), `??`, and `instanceof` are handled
/// by the caller before reaching here.
fn map_binop(op: BinaryOperator) -> BinOp {
    match op {
        BinaryOperator::Addition(_) => BinOp::Add,
        BinaryOperator::Subtraction(_) => BinOp::Sub,
        BinaryOperator::Multiplication(_) => BinOp::Mul,
        BinaryOperator::Division(_) => BinOp::Div,
        BinaryOperator::Modulo(_) => BinOp::Mod,
        BinaryOperator::Exponentiation(_) => BinOp::Pow,
        BinaryOperator::StringConcat(_) => BinOp::Concat,
        BinaryOperator::BitwiseAnd(_) => BinOp::BitAnd,
        BinaryOperator::BitwiseOr(_) => BinOp::BitOr,
        BinaryOperator::BitwiseXor(_) => BinOp::BitXor,
        BinaryOperator::LeftShift(_) => BinOp::Shl,
        BinaryOperator::RightShift(_) => BinOp::Shr,
        BinaryOperator::Equal(_) => BinOp::Eq,
        BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_) => BinOp::NotEq,
        BinaryOperator::Identical(_) => BinOp::Identical,
        BinaryOperator::NotIdentical(_) => BinOp::NotIdentical,
        BinaryOperator::LessThan(_) => BinOp::Lt,
        BinaryOperator::LessThanOrEqual(_) => BinOp::Le,
        BinaryOperator::GreaterThan(_) => BinOp::Gt,
        BinaryOperator::GreaterThanOrEqual(_) => BinOp::Ge,
        BinaryOperator::Spaceship(_) => BinOp::Spaceship,
        // Logical / coalesce / instanceof are intercepted by the caller.
        BinaryOperator::And(_)
        | BinaryOperator::Or(_)
        | BinaryOperator::LowAnd(_)
        | BinaryOperator::LowOr(_)
        | BinaryOperator::LowXor(_)
        | BinaryOperator::NullCoalesce(_)
        | BinaryOperator::Instanceof(_) => unreachable!("handled by lower_expr Binary arm"),
    }
}
