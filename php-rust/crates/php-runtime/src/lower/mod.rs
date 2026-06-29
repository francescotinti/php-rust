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
use mago_span::Span;
use mago_syntax::ast::{
    AssignmentOperator, BinaryOperator, ClassLikeMemberSelector,
    Expression, Extends, Hint,
    Identifier, Literal, LiteralInteger, Modifier, Node, Statement,
    Use, UseItems, UseType, Variable,
};
use mago_syntax::parser::parse_file;

use crate::hir::{
    BinOp, Capture, ClassDecl, ExprKind, FnDecl, HintKind, Line, LoweredTrait, MethodDecl, Param,
    Program, PropDecl, ScalarType, Slot, StaticAssignOp, Stmt, StmtKind, TypeHint,
    Visibility,
};

mod stmt;
mod class;
mod expr;

/// Why a script could not be lowered to HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    /// mago reported one or more parse errors.
    Parse(String),
    /// A construct that is valid PHP but outside the current Tier 1 scope.
    Unsupported { what: &'static str, line: Line },
    /// A class declaration `extends`/`implements` a class/interface not yet known
    /// (step 57, Phase 3). When lowering an `eval`/`include` unit the VM resolves
    /// `name` via autoload and retries; at top level it is the usual fatal.
    UndefinedClass { name: Box<[u8]>, line: Line },
    /// A program that PHP *compiles* but rejects with a `Fatal error:` at link
    /// time — e.g. an unresolved trait-method collision (step 21, D-21.5). Unlike
    /// `Unsupported`, this is faithful PHP behaviour: `run_source` turns it into
    /// an [`Outcome`](crate::Outcome) whose `rendered` stream carries the fatal.
    Fatal { message: String, line: Line },
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LowerError::Parse(msg) => write!(f, "parse error: {msg}"),
            LowerError::Unsupported { what, line } => {
                write!(f, "unsupported construct ({what}) on line {line}")
            }
            LowerError::UndefinedClass { name, line } => {
                write!(f, "undefined class {} on line {line}", String::from_utf8_lossy(name))
            }
            LowerError::Fatal { message, line } => write!(f, "{message} on line {line}"),
        }
    }
}

impl std::error::Error for LowerError {}

/// Parse `source` (named `name` for diagnostics) and lower it to HIR, seeding the
/// class/function tables with the built-in prelude only.
pub fn lower_source(name: &[u8], source: &[u8]) -> Result<Program, LowerError> {
    lower_source_impl(name, source, None)
}

/// Like [`lower_source`] but seeds the class table (and the static-id counter)
/// from `seed_classes`/`seed_static` instead of just the built-in prelude —
/// "compile against image" (step 57, Phase 1c-2c/3). An `eval`/`include` unit
/// lowered this way resolves and **inherits** from classes already loaded
/// (`eval("class Bar extends Foo {}")` sees `Foo`). `seed_classes` must already
/// embed the prelude classes (the VM's accumulating image does), so the prelude
/// is not re-seeded; the unit's own classes hoist contiguously after the seeded
/// ones, keeping their ids aligned with the VM's global table. The prelude
/// *functions* are always (re-)seeded.
pub fn lower_source_seeded(
    name: &[u8],
    source: &[u8],
    seed_classes: &[crate::hir::ClassDecl],
    seed_static: usize,
    seed_traits: &[(Vec<u8>, LoweredTrait)],
) -> Result<Program, LowerError> {
    lower_source_impl(name, source, Some((seed_classes, seed_static, seed_traits)))
}

fn lower_source_impl(
    name: &[u8],
    source: &[u8],
    seed: Option<(&[crate::hir::ClassDecl], usize, &[(Vec<u8>, LoweredTrait)])>,
) -> Result<Program, LowerError> {
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
    match seed {
        // Seed the built-in exception hierarchy (Throwable/Exception/Error + the
        // SPL subclasses) at the front of the class table (ids 0..N), before any
        // user class is hoisted (step 20). This makes `extends Exception`,
        // `instanceof`, `new RuntimeException(...)`, property init and
        // `parent::__construct` reuse the whole step-19 class machinery with no
        // special-casing. The prelude's global functions (step 35: the procedural
        // date API) are seeded ahead of the user's too, so user functions get ids
        // contiguous after them. Call sites resolve by name, so no fix-up needed.
        None => {
            let (pclasses, pindex, pfunctions, pfn_index) = lower_prelude();
            low.classes = pclasses;
            low.class_index = pindex;
            low.functions = pfunctions;
            low.fn_index = pfn_index;
        }
        // Compile-against-image: seed the caller's *class* image (which already
        // embeds the prelude's classes), so an eval class can `extend`/`implement`
        // a caller user class and flatten its inherited layout. Ids equal the
        // caller's, so the eval's own classes hoist after and the VM's class-id
        // relocation stays an identity for the shared ones. `static_count` is
        // carried so the eval's new `static $x` cells get ids past the caller's
        // range. The prelude's *functions* are still seeded (the eval needs the
        // date API etc.), but the caller's user functions are deliberately NOT
        // seeded: re-emitting them into the eval unit would make a call like
        // `eval("foo();")` run the recompiled copy and mis-attribute its
        // `__FILE__`/backtrace to "eval()'d code". Calling a caller user function
        // from eval therefore remains unsupported here (a later phase resolves it
        // against the caller module instead of re-emitting).
        Some((sclasses, sstatic, straits)) => {
            low.classes = sclasses.to_vec();
            let mut ci: HashMap<Vec<u8>, usize> = HashMap::new();
            for (i, cd) in sclasses.iter().enumerate() {
                ci.entry(cd.name.to_ascii_lowercase()).or_insert(i);
            }
            low.class_index = ci;
            // Seed already-loaded traits so a `use T` here resolves against a trait
            // declared in an earlier (e.g. autoloaded) unit (step 21, trait analogue
            // of seed_classes). The keys are recorded so only this unit's *new*
            // traits are re-emitted in `Program::traits`.
            low.traits = straits
                .iter()
                .map(|(k, t)| {
                    let mut t = t.clone();
                    // Seeded from another unit: its closures aren't in this unit's
                    // table, so flatten must re-append and shift them.
                    t.external = true;
                    (k.clone(), t)
                })
                .collect();
            let (_pclasses, _pindex, pfunctions, pfn_index) = lower_prelude();
            low.functions = pfunctions;
            low.fn_index = pfn_index;
            low.static_count = sstatic;
        }
    }
    let seeded_trait_keys: HashSet<Vec<u8>> = low.traits.keys().cloned().collect();
    // Hoist function declarations first, so a call may textually precede its
    // definition (PHP's function hoisting). Bodies are lowered here; the main
    // pass below skips the declaration statements (they are no-ops). Each hoist
    // pass descends into `namespace` blocks (step 50) via `for_blocks`, so names
    // are registered fully-qualified and bodies resolve against the right imports.
    let stmts = program.statements.as_slice();
    low.for_blocks(stmts, |lo, body| {
        for s in body {
            if let Statement::Function(func) = s {
                lo.hoist_function(func)?;
            }
        }
        Ok(())
    })?;
    // Lower traits before classes, so a class's `use T` finds T fully resolved
    // (step 21). Traits stay in the Lowerer; they never enter the class table.
    low.for_blocks(stmts, |lo, body| lo.lower_traits(body))?;
    low.hoist_classes(stmts)?;
    // Seed global-namespace imports for the main pass (a no-`namespace` file may
    // still carry top-level `use`s); each `namespace` arm re-scopes its own.
    low.collect_uses(stmts);
    let body = low.lower_stmts(stmts)?;
    // Anonymous classes (`new class {…}`) collected during lowering get ids past
    // every named class; register them so `new` resolves each by its synthetic
    // name at compile time (step 51).
    for decl in std::mem::take(&mut low.anon_classes) {
        let key = decl.name.to_ascii_lowercase();
        low.class_index.insert(key, low.classes.len());
        low.classes.push(decl);
    }
    // `goto`/label validation (step 45): the top-level script body is its own
    // function scope. Each user function / method / closure validates its own
    // body where it is lowered (`lower_function`/`lower_method`/`lower_closure`).
    validate_goto(&body)?;
    Ok(Program {
        body,
        file: name.into(),
        slots: low.globals.slots,
        functions: low.functions,
        conditional_fns: low.conditional_fns,
        conditional_classes: low.conditional_classes,
        closures: low.closures,
        static_count: low.static_count,
        strict: low.strict,
        classes: low.classes,
        // Carry only the traits *this* unit declared (not the seeded ones), so the
        // VM can accumulate them into its cross-unit trait image.
        traits: low
            .traits
            .into_iter()
            .filter(|(k, _)| !seeded_trait_keys.contains(k))
            .collect(),
        const_attributes: low.const_attributes,
    })
}

/// The stack of loop/`switch`/`finally` barriers enclosing a `Label` or `Goto`,
/// innermost last; each entry pairs a unique barrier id with its kind (step 45).
type BarrierStack = Vec<(u32, BarrierKind)>;
/// Map of label name → the barriers enclosing its definition (step 45).
type LabelMap<'a> = HashMap<&'a [u8], BarrierStack>;
/// One `goto`: its label, the barriers enclosing it, and its source line.
type GotoSite<'a> = (&'a [u8], BarrierStack, Line);

/// Enforce PHP's compile-time `goto` rules over one function scope's statement
/// tree (step 45). `goto` is function-scoped, so the top-level script body and
/// every function / method / closure body is validated independently.
///
/// PHP rejects three situations at *compile* time — before any output — which we
/// surface as [`LowerError::Fatal`] (rendered with no partial output, exactly
/// like the oracle):
///   * `goto` to a label not defined in the scope → `'goto' to undefined label 'X'`;
///   * a label defined twice → `Label 'X' already defined`;
///   * `goto` that jumps *into* a loop or `switch` → `'goto' into loop or switch
///     statement is disallowed`. Jumping *out of* a loop/switch is allowed, and
///     `if` / `try` / plain blocks are transparent (verified against the oracle:
///     a `goto` into a `try` body runs and its `finally` still fires).
///
/// Legality of an "into a loop/switch/finally" jump is decided by container
/// stacks: each loop / `switch` / `finally` block gets a unique barrier id as
/// the tree is walked, and every `Label`/`Goto` records the stack of barrier
/// ids enclosing it. A `goto` may reach a label iff every barrier around the
/// label also surrounds the goto — i.e. the label's barrier stack is a prefix of
/// the goto's. `if` / `try` body / `catch` / plain blocks are *transparent*
/// (no barrier), matching PHP: a `goto` into one of those is allowed.
///
/// The barrier *kind* (loop/switch vs finally) is recorded alongside the id so
/// the right oracle message is produced: PHP distinguishes `'goto' into loop or
/// switch statement is disallowed` from `jump into a finally block is
/// disallowed`. When the label sits inside several barriers the goto is outside
/// of, the innermost such barrier (the first mismatching stack entry) picks the
/// message — the same one PHP reports.
fn validate_goto(body: &[Stmt]) -> Result<(), LowerError> {
    let mut labels: LabelMap = HashMap::new();
    let mut gotos: Vec<GotoSite> = Vec::new();
    let mut counter: u32 = 0;
    collect_goto(body, &mut Vec::new(), &mut counter, &mut labels, &mut gotos)?;
    for (name, gstack, line) in gotos {
        match labels.get(name) {
            None => {
                return Err(LowerError::Fatal {
                    message: format!(
                        "'goto' to undefined label '{}'",
                        String::from_utf8_lossy(name)
                    ),
                    line,
                });
            }
            Some(lstack) => {
                // Find the first barrier enclosing the label that does not also
                // enclose the goto: that is the construct being jumped *into*.
                let mismatch = lstack
                    .iter()
                    .enumerate()
                    .find(|(i, (id, _))| gstack.get(*i).map(|(g, _)| g) != Some(id));
                if let Some((_, (_, kind))) = mismatch {
                    return Err(LowerError::Fatal {
                        message: kind.message().to_string(),
                        line,
                    });
                }
            }
        }
    }
    Ok(())
}

/// The kind of construct a `goto` is forbidden from jumping *into* (step 45).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BarrierKind {
    /// A `for`/`foreach`/`while`/`do-while`/`switch` body.
    LoopOrSwitch,
    /// A `finally { … }` block (rejected with its own oracle message).
    Finally,
}

impl BarrierKind {
    fn message(self) -> &'static str {
        match self {
            BarrierKind::LoopOrSwitch => "'goto' into loop or switch statement is disallowed",
            BarrierKind::Finally => "jump into a finally block is disallowed",
        }
    }
}

/// Single-pass walk for [`validate_goto`]: collect every `Label` (keyed by name,
/// rejecting duplicates) and every `Goto`, each tagged with the stack of
/// enclosing barriers. Loops, `switch` and `finally` push a barrier; `if`,
/// `try` body, `catch` and plain blocks are walked transparently.
fn collect_goto<'a>(
    stmts: &'a [Stmt],
    stack: &mut BarrierStack,
    counter: &mut u32,
    labels: &mut LabelMap<'a>,
    gotos: &mut Vec<GotoSite<'a>>,
) -> Result<(), LowerError> {
    for s in stmts {
        match &s.kind {
            StmtKind::Label(name) => {
                if labels.insert(name, stack.clone()).is_some() {
                    return Err(LowerError::Fatal {
                        message: format!(
                            "Label '{}' already defined",
                            String::from_utf8_lossy(name)
                        ),
                        line: s.line,
                    });
                }
            }
            StmtKind::Goto(name) => gotos.push((name, stack.clone(), s.line)),
            StmtKind::Block(b) => collect_goto(b, stack, counter, labels, gotos)?,
            StmtKind::If {
                then,
                elseifs,
                otherwise,
                ..
            } => {
                collect_goto(then, stack, counter, labels, gotos)?;
                for (_, b) in elseifs {
                    collect_goto(b, stack, counter, labels, gotos)?;
                }
                collect_goto(otherwise, stack, counter, labels, gotos)?;
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. } => {
                *counter += 1;
                stack.push((*counter, BarrierKind::LoopOrSwitch));
                collect_goto(body, stack, counter, labels, gotos)?;
                stack.pop();
            }
            StmtKind::Switch { cases, .. } => {
                *counter += 1;
                stack.push((*counter, BarrierKind::LoopOrSwitch));
                for c in cases {
                    collect_goto(&c.body, stack, counter, labels, gotos)?;
                }
                stack.pop();
            }
            StmtKind::Try {
                body,
                catches,
                finally,
            } => {
                // The `try` body and `catch` blocks are transparent; only the
                // `finally` block is a barrier (PHP forbids jumping into it).
                collect_goto(body, stack, counter, labels, gotos)?;
                for c in catches {
                    collect_goto(&c.body, stack, counter, labels, gotos)?;
                }
                *counter += 1;
                stack.push((*counter, BarrierKind::Finally));
                collect_goto(finally, stack, counter, labels, gotos)?;
                stack.pop();
            }
            _ => {}
        }
    }
    Ok(())
}

/// The mago `Expression` variant name, for a precise `Unsupported` category
/// (step 48 micro-step): the generic `"expression"` bucket told us nothing about
/// *which* construct was missing, so the catch-all now reports the node kind
/// (e.g. `expr:Instantiation`), making the phpt-runner skip detail actionable.
fn expr_variant_name(e: &Expression) -> &'static str {
    match e {
        Expression::Binary(_) => "expr:Binary",
        Expression::UnaryPrefix(_) => "expr:UnaryPrefix",
        Expression::UnaryPostfix(_) => "expr:UnaryPostfix",
        Expression::Parenthesized(_) => "expr:Parenthesized",
        Expression::Literal(_) => "expr:Literal",
        Expression::CompositeString(_) => "expr:CompositeString",
        Expression::Assignment(_) => "expr:Assignment",
        Expression::Conditional(_) => "expr:Conditional",
        Expression::Array(_) => "expr:Array",
        Expression::LegacyArray(_) => "expr:LegacyArray",
        Expression::List(_) => "expr:List",
        Expression::ArrayAccess(_) => "expr:ArrayAccess",
        Expression::ArrayAppend(_) => "expr:ArrayAppend",
        Expression::AnonymousClass(_) => "expr:AnonymousClass",
        Expression::Closure(_) => "expr:Closure",
        Expression::ArrowFunction(_) => "expr:ArrowFunction",
        Expression::Variable(_) => "expr:Variable",
        Expression::ConstantAccess(_) => "expr:ConstantAccess",
        Expression::Identifier(_) => "expr:Identifier",
        Expression::Match(_) => "expr:Match",
        Expression::Yield(_) => "expr:Yield",
        Expression::Construct(_) => "expr:Construct",
        Expression::Throw(_) => "expr:Throw",
        Expression::Clone(_) => "expr:Clone",
        Expression::Call(_) => "expr:Call",
        Expression::PartialApplication(_) => "expr:PartialApplication",
        Expression::Access(_) => "expr:Access",
        Expression::Parent(_) => "expr:Parent",
        Expression::Static(_) => "expr:Static",
        Expression::Instantiation(_) => "expr:Instantiation",
        Expression::MagicConstant(_) => "expr:MagicConstant",
        Expression::Pipe(_) => "expr:Pipe",
        Expression::Error(_) => "expr:Error",
        _ => "expr:other",
    }
}

/// The mago `Statement` variant name, for a precise `Unsupported` category
/// (step 48 micro-step). See [`expr_variant_name`].
fn stmt_variant_name(s: &Statement) -> &'static str {
    match s {
        Statement::OpeningTag(_) => "stmt:OpeningTag",
        Statement::ClosingTag(_) => "stmt:ClosingTag",
        Statement::Inline(_) => "stmt:Inline",
        Statement::Namespace(_) => "stmt:Namespace",
        Statement::Use(_) => "stmt:Use",
        Statement::Class(_) => "stmt:Class",
        Statement::Interface(_) => "stmt:Interface",
        Statement::Trait(_) => "stmt:Trait",
        Statement::Enum(_) => "stmt:Enum",
        Statement::Block(_) => "stmt:Block",
        Statement::Constant(_) => "stmt:Constant",
        Statement::Function(_) => "stmt:Function",
        Statement::Declare(_) => "stmt:Declare",
        Statement::Goto(_) => "stmt:Goto",
        Statement::Label(_) => "stmt:Label",
        Statement::Try(_) => "stmt:Try",
        Statement::Foreach(_) => "stmt:Foreach",
        Statement::For(_) => "stmt:For",
        Statement::While(_) => "stmt:While",
        Statement::DoWhile(_) => "stmt:DoWhile",
        Statement::Continue(_) => "stmt:Continue",
        Statement::Break(_) => "stmt:Break",
        Statement::Switch(_) => "stmt:Switch",
        Statement::If(_) => "stmt:If",
        Statement::Return(_) => "stmt:Return",
        Statement::Expression(_) => "stmt:Expression",
        Statement::Echo(_) => "stmt:Echo",
        Statement::EchoTag(_) => "stmt:EchoTag",
        Statement::Global(_) => "stmt:Global",
        Statement::Static(_) => "stmt:Static",
        Statement::HaltCompiler(_) => "stmt:HaltCompiler",
        Statement::Unset(_) => "stmt:Unset",
        Statement::Noop(_) => "stmt:Noop",
        _ => "stmt:other",
    }
}

/// The built-in classes, authored in PHP and lowered once into the front of
/// every program's class table (step 20): `stdClass` plus the throwable
/// hierarchy. Mirrors PHP's core/SPL classes closely enough for catch-matching,
/// the accessors, and `instanceof`.
/// `getTrace`/`getTraceAsString` are stubs (no real stack trace is modelled);
/// `file`/`line` are filled in by the evaluator at `new` time, not here.
const PRELUDE_SRC: &[u8] = br##"<?php
class stdClass {}
class Attribute {
    const TARGET_CLASS = 1;
    const TARGET_FUNCTION = 2;
    const TARGET_METHOD = 4;
    const TARGET_PROPERTY = 8;
    const TARGET_CLASS_CONSTANT = 16;
    const TARGET_PARAMETER = 32;
    const TARGET_CONSTANT = 64;
    const TARGET_ALL = 127;
    const IS_REPEATABLE = 128;
    public int $flags;
    public function __construct(int $flags = self::TARGET_ALL) { $this->flags = $flags; }
}
interface UnitEnum {}
interface BackedEnum extends UnitEnum {}
interface Stringable {}
interface Throwable {}
interface Traversable {}
interface Iterator extends Traversable {}
interface IteratorAggregate extends Traversable {}
interface ArrayAccess {}
interface Countable {}
interface JsonSerializable {}
class Exception implements Throwable {
    protected $message = "";
    protected $code = 0;
    protected $file = "";
    protected $line = 0;
    private $previous = null;
    private $trace = [];
    private $traceString = "#0 {main}";
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
    public function getTrace() { return $this->trace; }
    public function getTraceAsString() { return $this->traceString; }
    public function __toString() {
        $r = "";
        if ($this->previous !== null) {
            $r = $this->previous->__toString() . "\n\nNext ";
        }
        $msg = $this->message === "" ? "" : ": " . $this->message;
        $sep = (strpos($this->message, ", called in ") !== false) ? " and defined in " : " in ";
        $r .= get_class($this) . $msg . $sep . $this->file . ":" . $this->line . "\nStack trace:\n" . $this->traceString;
        return $r;
    }
}
class Error implements Throwable {
    protected $message = "";
    protected $code = 0;
    protected $file = "";
    protected $line = 0;
    private $previous = null;
    private $trace = [];
    private $traceString = "#0 {main}";
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
    public function getTrace() { return $this->trace; }
    public function getTraceAsString() { return $this->traceString; }
    public function __toString() {
        $r = "";
        if ($this->previous !== null) {
            $r = $this->previous->__toString() . "\n\nNext ";
        }
        $msg = $this->message === "" ? "" : ": " . $this->message;
        $sep = (strpos($this->message, ", called in ") !== false) ? " and defined in " : " in ";
        $r .= get_class($this) . $msg . $sep . $this->file . ":" . $this->line . "\nStack trace:\n" . $this->traceString;
        return $r;
    }
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
class Fiber {
    private $callable;
    public function __construct($callable) { $this->callable = $callable; }
}
final class WeakReference {
    // `__h` is an internal weak handle (see __weak_create/__weak_get): it does
    // NOT keep the referent alive, so get() returns the object while a strong
    // reference exists elsewhere and null once it is collected (true weakness).
    private $__h;
    private function __construct() {}
    public static function create($object) {
        if (!is_object($object)) {
            $t = gettype($object);
            $t = ["integer" => "int", "double" => "float", "boolean" => "bool", "NULL" => "null"][$t] ?? $t;
            throw new TypeError("WeakReference::create(): Argument #1 (\$object) must be of type object, $t given");
        }
        $ref = new self();
        $ref->__h = __weak_create($object);
        return $ref;
    }
    public function get() {
        return __weak_get($this->__h);
    }
}
class WeakMap implements ArrayAccess, Countable, IteratorAggregate {
    // id => [weak-handle, value]. Keys are held *weakly* (via __weak_create): an
    // entry whose key has been collected is pruned lazily on access (__prune /
    // __live), giving true weakness without a tracing GC. Keyed by spl_object_id.
    private $__entries = [];
    private function __live($id) {
        // The live key object for $id, pruning the entry if it has been collected.
        if (!isset($this->__entries[$id])) {
            return null;
        }
        $o = __weak_get($this->__entries[$id][0]);
        if ($o === null) {
            unset($this->__entries[$id]);
        }
        return $o;
    }
    private function __prune() {
        foreach ($this->__entries as $id => $entry) {
            if (__weak_get($entry[0]) === null) {
                unset($this->__entries[$id]);
            }
        }
    }
    public function offsetExists($object) {
        // isset()/empty() on an ArrayAccess element use offsetExists as the
        // backend; PHP reports a null-valued (or collected) key as not set.
        $id = spl_object_id($object);
        return $this->__live($id) !== null && $this->__entries[$id][1] !== null;
    }
    public function offsetGet($object) {
        if (!is_object($object)) {
            throw new TypeError("WeakMap key must be an object");
        }
        $id = spl_object_id($object);
        if ($this->__live($id) === null) {
            throw new Error("Object " . get_class($object) . "#" . $id . " not contained in WeakMap");
        }
        return $this->__entries[$id][1];
    }
    public function offsetSet($object, $value) {
        if (!is_object($object)) {
            throw new TypeError("WeakMap key must be an object");
        }
        $this->__entries[spl_object_id($object)] = [__weak_create($object), $value];
    }
    public function offsetUnset($object) {
        unset($this->__entries[spl_object_id($object)]);
    }
    public function count() {
        $this->__prune();
        return count($this->__entries);
    }
    public function getIterator() {
        $this->__prune();
        foreach ($this->__entries as $entry) {
            $o = __weak_get($entry[0]);
            if ($o !== null) {
                yield $o => $entry[1];
            }
        }
    }
}
interface DateTimeInterface {}
class DateTime implements DateTimeInterface {
    private $__ts = 0;
    public function __construct($datetime = "now") {
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $this->__ts = time();
        } else {
            $r = strtotime($datetime);
            if ($r === false) {
                throw new Exception("DateTime::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r;
        }
    }
    public function format($format) { return date($format, $this->__ts); }
    public function getTimestamp() { return $this->__ts; }
    public function setTimestamp($timestamp) { $this->__ts = $timestamp; return $this; }
    public function setDate($year, $month, $day) {
        $this->__ts = mktime((int)date('G', $this->__ts), (int)date('i', $this->__ts), (int)date('s', $this->__ts), $month, $day, $year);
        return $this;
    }
    public function setTime($hour, $minute, $second = 0) {
        $this->__ts = mktime($hour, $minute, $second, (int)date('n', $this->__ts), (int)date('j', $this->__ts), (int)date('Y', $this->__ts));
        return $this;
    }
    public static function createFromFormat($format, $datetime) {
        $ts = __date_from_format($format, $datetime);
        if ($ts === false) { return false; }
        return new DateTime("@$ts");
    }
    public function modify($modifier) { $this->__ts = strtotime($modifier, $this->__ts); return $this; }
    public function add($interval) { $this->__ts = $this->__apply($interval, 1); return $this; }
    public function sub($interval) { $this->__ts = $this->__apply($interval, -1); return $this; }
    private function __apply($iv, $dir) {
        $sign = $dir * ($iv->invert ? -1 : 1);
        return mktime(
            (int)date('G', $this->__ts) + $sign * $iv->h,
            (int)date('i', $this->__ts) + $sign * $iv->i,
            (int)date('s', $this->__ts) + $sign * $iv->s,
            (int)date('n', $this->__ts) + $sign * $iv->m,
            (int)date('j', $this->__ts) + $sign * $iv->d,
            (int)date('Y', $this->__ts) + $sign * $iv->y);
    }
    public function diff($other) {
        $info = __date_diff($this->__ts, $other->getTimestamp());
        $iv = new DateInterval("PT0S");
        $iv->y = $info['y']; $iv->m = $info['m']; $iv->d = $info['d'];
        $iv->h = $info['h']; $iv->i = $info['i']; $iv->s = $info['s'];
        $iv->invert = $info['invert']; $iv->days = $info['days'];
        return $iv;
    }
}
class DateInterval {
    public $y = 0;
    public $m = 0;
    public $d = 0;
    public $h = 0;
    public $i = 0;
    public $s = 0;
    public $f = 0;
    public $invert = 0;
    public $days = false;
    public function __construct($duration) {
        $p = __interval_parse($duration);
        if ($p === false) {
            throw new Exception("DateInterval::__construct(): Unknown or bad format ($duration)");
        }
        $this->y = $p['y']; $this->m = $p['m']; $this->d = $p['d'];
        $this->h = $p['h']; $this->i = $p['i']; $this->s = $p['s'];
    }
    public function format($format) { return __interval_format($this, $format); }
}
class DateTimeImmutable implements DateTimeInterface {
    private $__ts = 0;
    public function __construct($datetime = "now") {
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $this->__ts = time();
        } else {
            $r = strtotime($datetime);
            if ($r === false) {
                throw new Exception("DateTimeImmutable::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r;
        }
    }
    public function format($format) { return date($format, $this->__ts); }
    public function getTimestamp() { return $this->__ts; }
    public function setTimestamp($timestamp) { return new DateTimeImmutable("@$timestamp"); }
    public function setDate($year, $month, $day) {
        $ts = mktime((int)date('G', $this->__ts), (int)date('i', $this->__ts), (int)date('s', $this->__ts), $month, $day, $year);
        return new DateTimeImmutable("@$ts");
    }
    public function setTime($hour, $minute, $second = 0) {
        $ts = mktime($hour, $minute, $second, (int)date('n', $this->__ts), (int)date('j', $this->__ts), (int)date('Y', $this->__ts));
        return new DateTimeImmutable("@$ts");
    }
    public static function createFromFormat($format, $datetime) {
        $ts = __date_from_format($format, $datetime);
        if ($ts === false) { return false; }
        return new DateTimeImmutable("@$ts");
    }
    public function modify($modifier) { $ts = strtotime($modifier, $this->__ts); return new DateTimeImmutable("@$ts"); }
    public function add($interval) { $ts = $this->__apply($interval, 1); return new DateTimeImmutable("@$ts"); }
    public function sub($interval) { $ts = $this->__apply($interval, -1); return new DateTimeImmutable("@$ts"); }
    private function __apply($iv, $dir) {
        $sign = $dir * ($iv->invert ? -1 : 1);
        return mktime(
            (int)date('G', $this->__ts) + $sign * $iv->h,
            (int)date('i', $this->__ts) + $sign * $iv->i,
            (int)date('s', $this->__ts) + $sign * $iv->s,
            (int)date('n', $this->__ts) + $sign * $iv->m,
            (int)date('j', $this->__ts) + $sign * $iv->d,
            (int)date('Y', $this->__ts) + $sign * $iv->y);
    }
    public function diff($other) {
        $info = __date_diff($this->__ts, $other->getTimestamp());
        $iv = new DateInterval("PT0S");
        $iv->y = $info['y']; $iv->m = $info['m']; $iv->d = $info['d'];
        $iv->h = $info['h']; $iv->i = $info['i']; $iv->s = $info['s'];
        $iv->invert = $info['invert']; $iv->days = $info['days'];
        return $iv;
    }
}

// --- Procedural date API (step 35): thin global-function wrappers over the OOP
// API above. PHP exposes both styles; these delegate so the two stay identical.
function date_create($datetime = "now") { return new DateTime($datetime); }
function date_create_immutable($datetime = "now") { return new DateTimeImmutable($datetime); }
function date_format($object, $format) { return $object->format($format); }
function date_timestamp_get($object) { return $object->getTimestamp(); }
function date_diff($base, $target, $absolute = false) {
    $r = $base->diff($target);
    if ($absolute) { $r->invert = 0; }
    return $r;
}
function date_add($object, $interval) { return $object->add($interval); }
function date_sub($object, $interval) { return $object->sub($interval); }
function date_modify($object, $modifier) { return $object->modify($modifier); }
function date_date_set($object, $year, $month, $day) { return $object->setDate($year, $month, $day); }
function date_time_set($object, $hour, $minute, $second = 0) { return $object->setTime($hour, $minute, $second); }
function date_timestamp_set($object, $timestamp) { return $object->setTimestamp($timestamp); }
function date_create_from_format($format, $datetime, $timezone = null) { return DateTime::createFromFormat($format, $datetime); }
function date_create_immutable_from_format($format, $datetime, $timezone = null) { return DateTimeImmutable::createFromFormat($format, $datetime); }
function date_interval_format($object, $format) { return $object->format($format); }
function date_interval_create_from_date_string($datetime) {
    $p = __interval_from_date_string($datetime);
    if ($p === false) { return false; }
    $iv = new DateInterval("PT0S");
    $iv->y = $p['y']; $iv->m = $p['m']; $iv->d = $p['d'];
    $iv->h = $p['h']; $iv->i = $p['i']; $iv->s = $p['s'];
    return $iv;
}

// --- SPL iterator classes (step 56): the two by-far most-demanded SPL types in
// the Zend/tests corpus (ArrayIterator 32 files, ArrayObject 28). Implemented
// entirely in PHP, backed by a plain `array $__storage`, reusing the working
// Iterator + ArrayAccess protocols + the array builtins. Zero VM changes.
// `__keys` is a key snapshot taken at rewind() so the integer `__pos` cursor is
// order-preserving and survives mutation, matching SPL semantics.
class ArrayIterator implements Iterator, ArrayAccess, Countable {
    private $__storage = [];
    private $__keys = [];
    private $__pos = 0;
    public function __construct($array = []) {
        $this->__storage = (array)$array;
        $this->__keys = array_keys($this->__storage);
    }
    public function rewind() { $this->__keys = array_keys($this->__storage); $this->__pos = 0; }
    public function valid() { return $this->__pos < count($this->__keys); }
    public function current() { return $this->__storage[$this->__keys[$this->__pos]]; }
    public function key() { return $this->__keys[$this->__pos]; }
    public function next() { $this->__pos++; }
    public function offsetExists($key) { return isset($this->__storage[$key]); }
    public function offsetGet($key) { return $this->__storage[$key] ?? null; }
    public function offsetSet($key, $value) {
        if ($key === null) { $this->__storage[] = $value; }
        else { $this->__storage[$key] = $value; }
    }
    public function offsetUnset($key) { unset($this->__storage[$key]); }
    public function count() { return count($this->__storage); }
    public function getArrayCopy() { return $this->__storage; }
    public function append($value) { $this->__storage[] = $value; }
}
class ArrayObject implements IteratorAggregate, ArrayAccess, Countable {
    private $__storage = [];
    public function __construct($array = []) { $this->__storage = (array)$array; }
    public function getIterator() { return new ArrayIterator($this->__storage); }
    public function offsetExists($key) { return isset($this->__storage[$key]); }
    public function offsetGet($key) { return $this->__storage[$key] ?? null; }
    public function offsetSet($key, $value) {
        if ($key === null) { $this->__storage[] = $value; }
        else { $this->__storage[$key] = $value; }
    }
    public function offsetUnset($key) { unset($this->__storage[$key]); }
    public function count() { return count($this->__storage); }
    public function getArrayCopy() { return $this->__storage; }
    public function append($value) { $this->__storage[] = $value; }
}
// `IteratorIterator` wraps any Traversable as a concrete `Iterator`, resolving an
// `IteratorAggregate` to its inner iterator once at construction; protocol calls
// delegate to the inner. `getInnerIterator()` returns the wrapped iterator.
class IteratorIterator implements Iterator {
    private $__it;
    public function __construct($iterator) {
        if ($iterator instanceof IteratorAggregate) { $iterator = $iterator->getIterator(); }
        $this->__it = $iterator;
    }
    public function getInnerIterator() { return $this->__it; }
    public function rewind() { return $this->__it->rewind(); }
    public function valid() { return $this->__it->valid(); }
    public function current() { return $this->__it->current(); }
    public function key() { return $this->__it->key(); }
    public function next() { return $this->__it->next(); }
}
// `SplFixedArray`: a fixed-size, integer-indexed array. Backed by `$__storage`
// filled with nulls to `$__size`; out-of-range offsets throw RuntimeException.
class SplFixedArray implements ArrayAccess, Countable, Iterator {
    private $__storage = [];
    private $__size = 0;
    private $__pos = 0;
    public function __construct($size = 0) {
        $this->__size = $size;
        for ($i = 0; $i < $size; $i++) { $this->__storage[$i] = null; }
    }
    public function getSize() { return $this->__size; }
    public function setSize($size) {
        if ($size < $this->__size) {
            for ($i = $size; $i < $this->__size; $i++) { unset($this->__storage[$i]); }
        } else {
            for ($i = $this->__size; $i < $size; $i++) { $this->__storage[$i] = null; }
        }
        $this->__size = $size;
        return true;
    }
    public function count() { return $this->__size; }
    public function toArray() { return $this->__storage; }
    public function offsetExists($i) { return $i >= 0 && $i < $this->__size; }
    public function offsetGet($i) {
        if ($i < 0 || $i >= $this->__size) { throw new RuntimeException("Index invalid or out of range"); }
        return $this->__storage[$i];
    }
    public function offsetSet($i, $v) {
        if ($i < 0 || $i >= $this->__size) { throw new RuntimeException("Index invalid or out of range"); }
        $this->__storage[$i] = $v;
    }
    public function offsetUnset($i) {
        if ($i >= 0 && $i < $this->__size) { $this->__storage[$i] = null; }
    }
    public function rewind() { $this->__pos = 0; }
    public function valid() { return $this->__pos < $this->__size; }
    public function current() { return $this->__storage[$this->__pos]; }
    public function key() { return $this->__pos; }
    public function next() { $this->__pos++; }
    public static function fromArray($array) {
        $a = new SplFixedArray(count($array));
        $i = 0;
        foreach ($array as $v) { $a[$i] = $v; $i++; }
        return $a;
    }
}
class ReflectionException extends Exception {}
class ReflectionAttribute {
    public $name;
    // Private handle to the owning class + the attribute's position in it, used by
    // the host builtins to materialise the attribute lazily. `__prop` is set for an
    // attribute that decorates a property (vs the class itself).
    public $__class;
    public $__index;
    public $__prop;
    public $__func;
    public $__method;
    public $__const;
    public function getName() { return $this->name; }
    public function getArguments() {
        if (isset($this->__prop)) {
            return __reflect_prop_attr_args($this->__class, $this->__prop, $this->__index);
        }
        if (isset($this->__func)) {
            return __reflect_func_attr_args($this->__func, $this->__index);
        }
        if (isset($this->__method)) {
            return __reflect_method_attr_args($this->__class, $this->__method, $this->__index);
        }
        if (isset($this->__const)) {
            return __reflect_const_attr_args($this->__const, $this->__index);
        }
        return __reflect_attr_arguments($this->__class, $this->__index);
    }
    public function newInstance() {
        if (isset($this->__prop)) {
            return __reflect_prop_attr_new($this->__class, $this->__prop, $this->__index);
        }
        if (isset($this->__func)) {
            return __reflect_func_attr_new($this->__func, $this->__index);
        }
        if (isset($this->__method)) {
            return __reflect_method_attr_new($this->__class, $this->__method, $this->__index);
        }
        if (isset($this->__const)) {
            return __reflect_const_attr_new($this->__const, $this->__index);
        }
        return __reflect_attr_newinstance($this->__class, $this->__index);
    }
}
class ReflectionClass {
    public $name;
    public function __construct($objectOrClass) {
        $this->name = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        if (!class_exists($this->name) && !interface_exists($this->name)) {
            throw new ReflectionException(sprintf('Class "%s" does not exist', $this->name));
        }
    }
    public function getName() { return $this->name; }
    public function getShortName() {
        $p = strrpos($this->name, '\\');
        return $p === false ? $this->name : substr($this->name, $p + 1);
    }
    // Attributes are retained at lowering; the host builds one ReflectionAttribute
    // per attribute declared on the class (optionally filtered by name).
    public function getAttributes($name = null, $flags = 0) {
        return __reflect_class_attributes($this->name, $name);
    }
    public function newInstance(...$args) { return new $this->name(...$args); }
    public function newInstanceArgs($args = []) { return new $this->name(...$args); }
    public function newInstanceWithoutConstructor() { return __reflect_new_no_ctor($this->name); }
    public function newLazyGhost(callable $initializer, int $options = 0) { return __reflect_new_lazy_ghost($this->name, $initializer); }
    public function newLazyProxy(callable $factory, int $options = 0) { return __reflect_new_lazy_proxy($this->name, $factory); }
    public function resetAsLazyGhost($object, callable $initializer, int $options = 0) {
        if (__lazy_is_initializing($object)) { throw new Error('Can not reset an object while it is being initialized'); }
        if (__lazy_is_uninitialized($object)) { throw new ReflectionException('Object is already lazy'); }
        return __reflect_reset_lazy($this->name, $object, false, $initializer);
    }
    public function resetAsLazyProxy($object, callable $factory, int $options = 0) {
        if (__lazy_is_initializing($object)) { throw new Error('Can not reset an object while it is being initialized'); }
        if (__lazy_is_uninitialized($object)) { throw new ReflectionException('Object is already lazy'); }
        return __reflect_reset_lazy($this->name, $object, true, $factory);
    }
    public function isUninitializedLazyObject($object) { return __lazy_is_uninitialized($object); }
    public function initializeLazyObject($object) { return __lazy_initialize($object); }
    public function isInstantiable() { return class_exists($this->name); }
    public function isInterface() { return interface_exists($this->name); }
    public function isFinal() { return __reflect_class_modifiers($this->name)['final']; }
    public function isAbstract() { return __reflect_class_modifiers($this->name)['abstract']; }
    public function hasMethod($name) { return method_exists($this->name, $name); }
    public function hasProperty($name) { return property_exists($this->name, $name); }
    public function getProperty($name) { return new ReflectionProperty($this->name, $name); }
    public function getProperties($filter = null) {
        $out = [];
        foreach (__reflect_prop_names($this->name) as $n) { $out[] = new ReflectionProperty($this->name, $n); }
        return $out;
    }
    public function hasConstant($name) { return defined($this->name . '::' . $name); }
    public function getConstant($name) { return constant($this->name . '::' . $name); }
    public function implementsInterface($interface) {
        return in_array($interface, class_implements($this->name), true);
    }
    public function isSubclassOf($class) {
        return in_array($class, class_parents($this->name), true)
            || in_array($class, class_implements($this->name), true);
    }
    public function getParentClass() {
        $p = get_parent_class($this->name);
        return $p === false ? false : new ReflectionClass($p);
    }
    public function getInterfaceNames() { return array_values(class_implements($this->name)); }
    public function getInterfaces() {
        $out = [];
        foreach (class_implements($this->name) as $i) { $out[$i] = new ReflectionClass($i); }
        return $out;
    }
    public function getTraitNames() { return array_values(class_uses($this->name)); }
    public function getTraits() {
        $out = [];
        foreach (class_uses($this->name) as $t) { $out[$t] = new ReflectionClass($t); }
        return $out;
    }
    public function getTraitAliases() { return []; }
    public function getMethod($name) { return new ReflectionMethod($this->name, $name); }
    public function getConstructor() {
        return method_exists($this->name, '__construct')
            ? new ReflectionMethod($this->name, '__construct') : null;
    }
    public function getMethods() {
        $out = [];
        foreach (get_class_methods($this->name) as $m) { $out[] = new ReflectionMethod($this->name, $m); }
        return $out;
    }
    public function hasMethod($name) { return method_exists($this->name, $name); }
}
class ReflectionNamedType {
    public $__name; public $__builtin; public $__nullable;
    public function getName() { return $this->__name; }
    public function allowsNull() { return $this->__nullable; }
    public function isBuiltin() { return $this->__builtin; }
    public function __toString() {
        $q = ($this->__nullable && $this->__name !== 'mixed' && $this->__name !== 'null') ? '?' : '';
        return $q . $this->__name;
    }
    public static function __fromInfo($t) {
        if ($t === false) { return null; }
        $r = new ReflectionNamedType();
        $r->__name = $t['name']; $r->__builtin = $t['builtin']; $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionParameter {
    public $name;
    public $__pos; public $__optional; public $__variadic; public $__byref;
    public $__type; public $__hasDefault; public $__default;
    public function __construct($function = null, $param = null) {
        if ($function === null) { return; } // internal factory path (__fromInfo)
        $info = is_array($function)
            ? __reflect_method_info($function[0], $function[1])
            : __reflect_func_info($function);
        if ($info === false) { throw new ReflectionException('The function does not exist'); }
        foreach ($info['params'] as $p) {
            if ((is_int($param) && $p['position'] === $param) || $p['name'] === $param) {
                $this->__init($p); return;
            }
        }
        throw new ReflectionException('The parameter specified does not exist');
    }
    public function __init($p) {
        $this->name = $p['name']; $this->__pos = $p['position'];
        $this->__optional = $p['optional']; $this->__variadic = $p['variadic'];
        $this->__byref = $p['byref']; $this->__type = $p['type'];
        $this->__hasDefault = $p['hasDefault']; $this->__default = $p['default'];
    }
    public static function __fromInfo($p) { $r = new ReflectionParameter(); $r->__init($p); return $r; }
    public function getName() { return $this->name; }
    public function getPosition() { return $this->__pos; }
    public function isOptional() { return $this->__optional; }
    public function isVariadic() { return $this->__variadic; }
    public function isPassedByReference() { return $this->__byref; }
    public function canBePassedByValue() { return !$this->__byref; }
    public function hasType() { return $this->__type !== false; }
    public function getType() { return ReflectionNamedType::__fromInfo($this->__type); }
    public function allowsNull() { return $this->__type === false ? true : $this->__type['nullable']; }
    public function isDefaultValueAvailable() { return $this->__hasDefault; }
    public function getDefaultValue() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        return $this->__default;
    }
}
class ReflectionObject extends ReflectionClass {
}
class ReflectionConstant {
    public $name;
    public function __construct($name) {
        if (!defined($name)) {
            throw new ReflectionException(sprintf('Constant "%s" does not exist', $name));
        }
        $this->name = $name;
    }
    public function getName() { return $this->name; }
    public function getValue() { return constant($this->name); }
    public function getAttributes($name = null, $flags = 0) { return __reflect_const_attributes($this->name, $name); }
    public function __toString() { return sprintf("Constant [ %s ]\n", $this->name); }
}
class ReflectionFunction {
    public $name;
    public $__info;
    public function __construct($name) {
        $this->name = is_string($name) ? $name : '{closure}';
        $this->__info = __reflect_func_info($this->name);
        if ($this->__info === false) {
            throw new ReflectionException(sprintf('Function %s() does not exist', $this->name));
        }
    }
    public function getName() { return $this->name; }
    public function getParameters() {
        $out = [];
        foreach ($this->__info['params'] as $p) { $out[] = ReflectionParameter::__fromInfo($p); }
        return $out;
    }
    public function getNumberOfParameters() { return count($this->__info['params']); }
    public function getNumberOfRequiredParameters() {
        $n = 0;
        foreach ($this->__info['params'] as $p) { if (!$p['optional']) { $n++; } }
        return $n;
    }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    public function invoke(...$args) { return call_user_func_array($this->name, $args); }
    public function invokeArgs($args) { return call_user_func_array($this->name, $args); }
    public function getAttributes($name = null, $flags = 0) { return __reflect_func_attributes($this->name, $name); }
}
class ReflectionMethod {
    public $name;
    public $class;
    public $__info;
    public function getName() { return $this->name; }
    public function getParameters() {
        $out = [];
        foreach ($this->__info['params'] as $p) { $out[] = ReflectionParameter::__fromInfo($p); }
        return $out;
    }
    public function getNumberOfParameters() { return count($this->__info['params']); }
    public function getNumberOfRequiredParameters() {
        $n = 0;
        foreach ($this->__info['params'] as $p) { if (!$p['optional']) { $n++; } }
        return $n;
    }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    public function __construct($objectOrClass, $method = null) {
        if ($method === null && is_string($objectOrClass) && strpos($objectOrClass, '::') !== false) {
            $parts = explode('::', $objectOrClass, 2);
            $objectOrClass = $parts[0]; $method = $parts[1];
        }
        $this->class = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        $this->name = $method;
        $this->__info = __reflect_method_info($this->class, $method);
        if ($this->__info === false) {
            throw new ReflectionException(sprintf('Method %s::%s() does not exist', $this->class, $method));
        }
    }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function isStatic() { return $this->__info['static']; }
    public function isFinal() { return $this->__info['final']; }
    public function isAbstract() { return $this->__info['abstract']; }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function setAccessible($accessible) {}
    public function invoke($object, ...$args) {
        return call_user_func_array([$object === null ? $this->class : $object, $this->name], $args);
    }
    public function invokeArgs($object, $args) {
        return call_user_func_array([$object === null ? $this->class : $object, $this->name], $args);
    }
    public function getAttributes($name = null, $flags = 0) { return __reflect_method_attributes($this->class, $this->name, $name); }
}
class ReflectionProperty {
    public $name;
    public $class;
    public function __construct($class, $property) {
        $cls = is_object($class) ? get_class($class) : $class;
        if (!property_exists($cls, $property)) {
            throw new ReflectionException(sprintf('Property %s::$%s does not exist', $cls, $property));
        }
        // The declaring class is the most-derived class that declares the property
        // itself (a child redeclaration shadows the parent's); mirrors
        // ReflectionProperty::$class. The host resolves it from the per-class
        // declared-property lists, which `property_exists` (inherited too) can't.
        $this->name = $property;
        $decl = __reflect_prop_declaring_class($cls, $property);
        $this->class = $decl === false ? $cls : $decl;
    }
    public function getName() { return $this->name; }
    public function getValue($object = null) { return __reflect_prop_get($this->class, $this->name, $object); }
    public function setValue($object, $value = null) { __reflect_prop_set($this->class, $this->name, $object, $value); }
    public function getAttributes($name = null, $flags = 0) { return __reflect_prop_attributes($this->class, $this->name, $name); }
    public function isStatic() { return __reflect_prop_is_static($this->class, $this->name); }
    public function skipLazyInitialization($object) {
        $msg = __lazy_skip_init($object, $this->class, $this->name);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
    public function setRawValueWithoutLazyInitialization($object, $value) {
        $msg = __lazy_set_raw($object, $this->class, $this->name, $value);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
}
"##;

/// The four owned products of lowering [`PRELUDE_SRC`]: the class table + its
/// name→id index (step 20), and the global-function table + its name→index
/// (step 35). Both are seeded into every real program before user declarations
/// are hoisted, so user classes/functions get contiguous ids after them.
type LoweredPrelude = (
    Vec<ClassDecl>,
    HashMap<Vec<u8>, usize>,
    Vec<FnDecl>,
    HashMap<Vec<u8>, usize>,
);

/// Lower [`PRELUDE_SRC`] with a throwaway [`Lowerer`] and return its owned class
/// table + name→id index (step 20) plus the global functions it declares (step
/// 35: the procedural date API). Function/`new` call sites resolve by *name*
/// (the evaluator rebuilds its `fn_index`/class table from `Program`), so the
/// prelude bodies need no index fix-up after being merged in.
fn lower_prelude() -> LoweredPrelude {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed(b"prelude".as_slice()), Cow::Borrowed(PRELUDE_SRC));
    let program = parse_file(&arena, &file);
    debug_assert!(
        !program.has_errors(),
        "exception prelude failed to parse: {:?}",
        program.errors
    );
    let mut low = Lowerer::new(&file, b"prelude");
    // Hoist classes first (a prelude function may `new` a prelude class), then
    // the global functions, mirroring the order in `lower`.
    low.hoist_classes(program.statements.as_slice())
        .expect("exception prelude must lower");
    for s in program.statements.as_slice() {
        if let Statement::Function(func) = s {
            low.hoist_function(func).expect("prelude function must lower");
        }
    }
    (low.classes, low.class_index, low.functions, low.fn_index)
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
    /// Indices into `functions` that are *conditional* declarations (registered at
    /// run time by `DeclareFn`, not resolvable by name eagerly).
    conditional_fns: HashSet<usize>,
    /// Indices into `classes` that are *conditional* declarations (registered at
    /// run time by `DeclareClass`, not resolvable by name eagerly).
    conditional_classes: HashSet<usize>,
    /// Anonymous/arrow function bodies, in one flat table (step 18, D-18.2). An
    /// [`ExprKind::Closure`] indexes into this by position.
    closures: Vec<FnDecl>,
    /// The program file name, used to synthesize the `{closure:file:line}` name
    /// PHP gives anonymous functions (step 18).
    prog_name: Box<[u8]>,
    /// True while lowering the body of a `function &f()`: a `return <lvalue>`
    /// then lowers to [`StmtKind::ReturnRef`] (step 13, D-13.3).
    fn_by_ref: bool,
    /// Set when a `yield` / `yield from` is lowered in the *current* function
    /// body, marking it a generator (step 39, [`FnDecl::is_generator`]). Saved
    /// and restored around each function/closure body so a `yield` in a nested
    /// closure does not leak to the enclosing function (the boundary PHP uses to
    /// decide what a generator is).
    fn_saw_yield: bool,
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
    /// Lexical scope for the magic constants `__CLASS__` / `__FUNCTION__` /
    /// `__METHOD__` / `__TRAIT__` (step 49). PHP resolves these at compile time
    /// from the *defining* scope, so we substitute them to string literals while
    /// lowering. Each is `Some` only while the corresponding body is lowered and
    /// is saved/restored around nested bodies, exactly like `fn_by_ref`.
    cur_class: Option<Box<[u8]>>,
    cur_function: Option<Box<[u8]>>,
    cur_trait: Option<Box<[u8]>>,
    /// The namespace currently being lowered, e.g. `b"Foo\\Bar"` (empty = global),
    /// step 50. Names are resolved against this at compile time: PHP namespaces are
    /// a pure compile-time name-resolution feature, so once every declaration and
    /// reference speaks fully-qualified names the existing by-name lookups in
    /// `compile.rs` / `vm` keep working unchanged. Saved/restored around each
    /// `namespace` block (PHP forbids nested namespaces, so one level deep).
    cur_namespace: Vec<u8>,
    /// Active `use` imports for the current namespace block, keyed by ASCII-lowercased
    /// alias → fully-qualified target (no leading `\`). PHP keeps three independent
    /// import tables (class/namespace, function, const); all three reset per block.
    use_classes: HashMap<Vec<u8>, Vec<u8>>,
    use_functions: HashMap<Vec<u8>, Vec<u8>>,
    use_consts: HashMap<Vec<u8>, Vec<u8>>,
    /// Constructor-promoted parameters collected by the most recent `lower_params`
    /// call (PHP 8 property promotion). The owning method (`__construct`) drains
    /// this immediately after the param list is lowered — before the body, which
    /// may itself contain nested param lists that overwrite it — to both declare
    /// the instance properties and prepend `$this->p = $p` assignments.
    promoted: Vec<PromotedParam>,
    /// While lowering a property hook body (step 50): the hooked property's name.
    /// A `$this-><name>` access inside the body marks the property *backed* (it
    /// needs real storage and appears in `var_dump`); see `hook_backed`.
    hook_prop: Option<Box<[u8]>>,
    /// Set when the hook body currently being lowered accesses its own backing
    /// (`$this-><hook_prop>`), making the property backed rather than virtual.
    hook_backed: bool,
    /// Monotonic counter for the synthetic temp slots that array destructuring
    /// (`[$a,$b] = …`) stashes its right-hand side into (step 51). Names use a `@`
    /// prefix, which no PHP variable can have, so they never collide with locals.
    list_temp: u32,
    /// Anonymous classes (`new class {…}`, step 51) discovered while lowering
    /// expressions, with their synthetic `class@anonymous…` names. Appended to
    /// `classes` (and registered in `class_index`) after the main pass, so they get
    /// final ids past every named class; `new` resolves them by name at compile.
    anon_classes: Vec<ClassDecl>,
    /// Monotonic counter making each anonymous class's synthetic name unique.
    anon_count: u32,
    /// `#[Attr]` attributes on top-level `const` declarations (FQN → attrs),
    /// accumulated while lowering and surfaced in [`Program::const_attributes`].
    const_attributes: Vec<(Box<[u8]>, Vec<crate::hir::HirAttribute>)>,
}

/// One constructor-promoted parameter: its property name, declared visibility, the
/// parameter slot the prologue assignment reads from, and any property hooks (a
/// promoted property may itself be hooked, PHP 8.4).
struct PromotedParam {
    name: Box<[u8]>,
    visibility: Visibility,
    slot: Slot,
    get_hook: Option<FnDecl>,
    set_hook: Option<FnDecl>,
    backed: bool,
    readonly: bool,
    attributes: Vec<crate::hir::HirAttribute>,
}

/// A trait whose members have been lowered and whose own `use` clauses have been
/// flattened in (step 21). Copied member-by-member into each consuming class so
/// the step-19 runtime machinery is reused with no evaluator changes.
impl<'f> Lowerer<'f> {
    fn new(file: &'f File, prog_name: &[u8]) -> Self {
        Lowerer {
            file,
            globals: Scope::default(),
            locals: None,
            after_closing_tag: false,
            functions: Vec::new(),
            fn_index: HashMap::new(),
            conditional_fns: HashSet::new(),
            conditional_classes: HashSet::new(),
            closures: Vec::new(),
            prog_name: prog_name.into(),
            fn_by_ref: false,
            fn_saw_yield: false,
            static_count: 0,
            strict: false,
            classes: Vec::new(),
            class_index: HashMap::new(),
            traits: HashMap::new(),
            cur_class: None,
            cur_function: None,
            cur_trait: None,
            cur_namespace: Vec::new(),
            use_classes: HashMap::new(),
            use_functions: HashMap::new(),
            use_consts: HashMap::new(),
            promoted: Vec::new(),
            hook_prop: None,
            hook_backed: false,
            list_temp: 0,
            anon_classes: Vec::new(),
            anon_count: 0,
            const_attributes: Vec::new(),
        }
    }

    /// Allocate a fresh synthetic local slot for a destructuring temp (step 51).
    /// The `@`-prefixed name is unique and unreachable from PHP source.
    fn fresh_list_temp(&mut self) -> Slot {
        let n = self.list_temp;
        self.list_temp += 1;
        let name = format!("@list{n}");
        self.slot_for(name.as_bytes())
    }

    /// Note a `$this-><name>` access seen while lowering a property-hook body: if
    /// it targets the hooked property itself, the property is backed (step 50).
    fn note_this_prop(&mut self, name: &[u8]) {
        if self.hook_prop.as_deref() == Some(name) {
            self.hook_backed = true;
        }
    }

    // --- namespace name resolution (step 50) ---

    /// Resolve a qualified name (`A\B\c`) by substituting an imported first
    /// segment if `A` was `use`d as a namespace/class alias, else prefixing the
    /// current namespace. Shared by class, function and const qualified forms.
    fn resolve_qualified(&self, raw: &[u8]) -> Box<[u8]> {
        let first = first_segment(raw);
        let rest = &raw[first.len()..]; // includes the leading `\` of the remainder
        match self.use_classes.get(&first.to_ascii_lowercase()) {
            Some(fqn) => {
                let mut v = fqn.clone();
                v.extend_from_slice(rest);
                v.into()
            }
            None => join_ns(&self.cur_namespace, raw),
        }
    }

    /// Resolve a class/interface/trait/enum name reference to its fully-qualified
    /// form. Unqualified names resolve against the class import table then the
    /// current namespace (PHP gives class names **no** global fallback).
    fn resolve_class(&self, id: &Identifier) -> Box<[u8]> {
        match id {
            Identifier::FullyQualified(f) => strip_leading_backslash(f.value).into(),
            Identifier::Qualified(q) => self.resolve_qualified(q.value),
            Identifier::Local(l) => match self.use_classes.get(&l.value.to_ascii_lowercase()) {
                Some(fqn) => fqn.clone().into(),
                None => join_ns(&self.cur_namespace, l.value),
            },
        }
    }

    /// Resolve a called function's name. Unlike classes, an **unqualified** name
    /// falls back to the global function: `foo()` in namespace `N` calls `N\foo`
    /// if such a user function was hoisted, otherwise global `foo` (a builtin or a
    /// runtime-resolved name). Qualified/fully-qualified forms never fall back.
    fn resolve_fn_name(&self, id: &Identifier) -> Box<[u8]> {
        match id {
            Identifier::FullyQualified(f) => strip_leading_backslash(f.value).into(),
            Identifier::Qualified(q) => self.resolve_qualified(q.value),
            Identifier::Local(l) => {
                if let Some(fqn) = self.use_functions.get(&l.value.to_ascii_lowercase()) {
                    return fqn.clone().into();
                }
                if self.cur_namespace.is_empty() {
                    return l.value.into();
                }
                let cand = join_ns(&self.cur_namespace, l.value);
                if self.fn_index.contains_key(&cand.to_ascii_lowercase()) {
                    cand
                } else {
                    l.value.into() // fall back to the global function
                }
            }
        }
    }

    /// Resolve a constant fetch to its primary name plus an optional global
    /// fallback. An *unqualified* constant inside a namespace primarily names
    /// `CURNS\NAME` and falls back to the global `NAME` (PHP tries the namespaced
    /// constant first, then global). Qualified / fully-qualified / imported and
    /// global-scope names resolve to a single name with no fallback.
    fn resolve_const_fetch(&self, id: &Identifier) -> (Box<[u8]>, Option<Box<[u8]>>) {
        match id {
            Identifier::FullyQualified(f) => (strip_leading_backslash(f.value).into(), None),
            Identifier::Qualified(q) => (self.resolve_qualified(q.value), None),
            Identifier::Local(l) => {
                if let Some(fqn) = self.use_consts.get(&l.value.to_ascii_lowercase()) {
                    (fqn.clone().into(), None)
                } else if self.cur_namespace.is_empty() {
                    (l.value.into(), None)
                } else {
                    (join_ns(&self.cur_namespace, l.value), Some(l.value.into()))
                }
            }
        }
    }

    /// Run `f` once per namespace scope in `stmts`, with `cur_namespace` and the
    /// three `use` tables set for that scope. PHP forbids nested namespaces, so
    /// each `namespace` block is one level deep; a file with no `namespace` at all
    /// runs `f` once over the whole list in the global namespace. Context is saved
    /// and restored around each block. Used by the hoisting passes (step 50).
    fn for_blocks<F>(&mut self, stmts: &[Statement], mut f: F) -> Result<(), LowerError>
    where
        F: FnMut(&mut Self, &[Statement]) -> Result<(), LowerError>,
    {
        let mut had_block = false;
        for s in stmts {
            if let Statement::Namespace(ns) = s {
                had_block = true;
                let body = ns.statements().as_slice();
                let saved_ns =
                    std::mem::replace(&mut self.cur_namespace, ns_name_of(ns.name.as_ref()));
                let saved_c = std::mem::take(&mut self.use_classes);
                let saved_f = std::mem::take(&mut self.use_functions);
                let saved_k = std::mem::take(&mut self.use_consts);
                self.collect_uses(body);
                let r = f(self, body);
                self.cur_namespace = saved_ns;
                self.use_classes = saved_c;
                self.use_functions = saved_f;
                self.use_consts = saved_k;
                r?;
            }
        }
        if !had_block {
            // No `namespace` blocks: the whole file is the global namespace.
            self.cur_namespace.clear();
            self.collect_uses(stmts);
            f(self, stmts)?;
        }
        Ok(())
    }

    /// Reset and repopulate the three `use` import tables from a namespace block's
    /// statements (PHP scopes imports to their namespace block). Called when a
    /// block is entered in every pass that resolves names inside it.
    fn collect_uses(&mut self, stmts: &[Statement]) {
        self.use_classes.clear();
        self.use_functions.clear();
        self.use_consts.clear();
        for s in stmts {
            if let Statement::Use(u) = s {
                self.add_use(u);
            }
        }
    }

    /// Insert one import: `kind` is `None` for a class/namespace import, or the
    /// `function`/`const` discriminator. `fqn` is the absolute target (no leading
    /// `\`), `alias` the bare local name it is reached by.
    fn insert_use(&mut self, kind: Option<&UseType>, fqn: Vec<u8>, alias: &[u8]) {
        let key = alias.to_ascii_lowercase();
        match kind {
            Some(UseType::Function(_)) => self.use_functions.insert(key, fqn),
            Some(UseType::Const(_)) => self.use_consts.insert(key, fqn),
            None => self.use_classes.insert(key, fqn),
        };
    }

    /// Record one `use` statement's imports into the appropriate table. Handles
    /// plain, typed (`use function`/`use const`), and grouped (`use Foo\{A, B}`)
    /// forms, including the mixed-type group `use Foo\{function f, const C}`.
    fn add_use(&mut self, u: &Use) {
        match &u.items {
            UseItems::Sequence(seq) => {
                for it in seq.items.iter() {
                    let fqn = strip_leading_backslash(it.name.value()).to_vec();
                    let alias = it.alias.as_ref().map_or_else(|| it.name.last_segment(), |a| a.identifier.value);
                    self.insert_use(None, fqn, alias);
                }
            }
            UseItems::TypedSequence(seq) => {
                for it in seq.items.iter() {
                    let fqn = strip_leading_backslash(it.name.value()).to_vec();
                    let alias = it.alias.as_ref().map_or_else(|| it.name.last_segment(), |a| a.identifier.value);
                    self.insert_use(Some(&seq.r#type), fqn, alias);
                }
            }
            UseItems::TypedList(list) => {
                let prefix = strip_leading_backslash(list.namespace.value()).to_vec();
                for it in list.items.iter() {
                    let fqn = join_ns(&prefix, it.name.value()).into_vec();
                    let alias = it.alias.as_ref().map_or_else(|| it.name.last_segment(), |a| a.identifier.value);
                    self.insert_use(Some(&list.r#type), fqn, alias);
                }
            }
            UseItems::MixedList(list) => {
                let prefix = strip_leading_backslash(list.namespace.value()).to_vec();
                for mit in list.items.iter() {
                    let it = &mit.item;
                    let fqn = join_ns(&prefix, it.name.value()).into_vec();
                    let alias = it.alias.as_ref().map_or_else(|| it.name.last_segment(), |a| a.identifier.value);
                    self.insert_use(mit.r#type.as_ref(), fqn, alias);
                }
            }
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
/// Process PHP double-quoted escape sequences in the literal segment of an
/// interpolated string (mago hands these back raw). Mirrors the lexer rules:
/// `\n \r \t \v \f \e \\ \$ \"`, `\x..` hex (1-2), `\u{..}` codepoint, and
/// `\0..\777` octal (1-3). An unknown `\X` keeps the backslash and X.
///
/// `process_quote` is true for double-quoted strings (`\"` -> `"`) and false in
/// a heredoc, where double quotes are literal so `\"` stays `\"`.
fn unescape_double_quoted(raw: &[u8], process_quote: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] != b'\\' || i + 1 >= raw.len() {
            out.push(raw[i]);
            i += 1;
            continue;
        }
        let c = raw[i + 1];
        match c {
            b'n' => { out.push(b'\n'); i += 2; }
            b'r' => { out.push(b'\r'); i += 2; }
            b't' => { out.push(b'\t'); i += 2; }
            b'v' => { out.push(0x0B); i += 2; }
            b'f' => { out.push(0x0C); i += 2; }
            b'e' => { out.push(0x1B); i += 2; }
            b'\\' => { out.push(b'\\'); i += 2; }
            b'$' => { out.push(b'$'); i += 2; }
            b'"' if process_quote => { out.push(b'"'); i += 2; }
            b'x' => {
                let mut j = i + 2;
                let mut val = 0u32;
                let mut n = 0;
                while n < 2 && j < raw.len() && raw[j].is_ascii_hexdigit() {
                    val = val * 16 + (raw[j] as char).to_digit(16).unwrap();
                    j += 1;
                    n += 1;
                }
                if n == 0 {
                    out.push(b'\\');
                    out.push(b'x');
                    i += 2;
                } else {
                    out.push(val as u8);
                    i = j;
                }
            }
            b'u' if i + 2 < raw.len() && raw[i + 2] == b'{' => {
                let mut j = i + 3;
                let mut val = 0u32;
                let mut n = 0;
                while j < raw.len() && raw[j].is_ascii_hexdigit() {
                    val = val * 16 + (raw[j] as char).to_digit(16).unwrap();
                    j += 1;
                    n += 1;
                }
                if n > 0 && j < raw.len() && raw[j] == b'}' {
                    if let Some(ch) = char::from_u32(val) {
                        let mut buf = [0u8; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                    }
                    i = j + 1;
                } else {
                    out.push(b'\\');
                    out.push(b'u');
                    i += 2;
                }
            }
            b'0'..=b'7' => {
                let mut j = i + 1;
                let mut val = 0u32;
                let mut n = 0;
                while n < 3 && j < raw.len() && (b'0'..=b'7').contains(&raw[j]) {
                    val = val * 8 + (raw[j] - b'0') as u32;
                    j += 1;
                    n += 1;
                }
                out.push(val as u8);
                i = j;
            }
            _ => {
                // Unknown escape: PHP keeps the backslash and the character.
                out.push(b'\\');
                out.push(c);
                i += 2;
            }
        }
    }
    out
}

/// Strip up to `indent` leading whitespace (space/tab) characters from each
/// line of a heredoc/nowdoc literal segment, mirroring PHP 7.3+ flexible
/// dedent. `at_line_start` says whether the segment begins a fresh line (i.e.
/// the previous byte emitted was a newline or it is the very first segment);
/// the returned flag carries that state to the next segment.
fn dedent_literal(lit: &[u8], indent: usize, mut at_line_start: bool) -> (Vec<u8>, bool) {
    if indent == 0 {
        return (lit.to_vec(), lit.last() == Some(&b'\n'));
    }
    let mut out = Vec::with_capacity(lit.len());
    let mut i = 0;
    while i < lit.len() {
        if at_line_start {
            let mut skipped = 0;
            while skipped < indent && i < lit.len() && (lit[i] == b' ' || lit[i] == b'\t') {
                i += 1;
                skipped += 1;
            }
            at_line_start = false;
            if i >= lit.len() {
                break;
            }
        }
        let b = lit[i];
        out.push(b);
        i += 1;
        if b == b'\n' {
            at_line_start = true;
        }
    }
    (out, at_line_start)
}

fn collect_catch_types(
    lo: &Lowerer,
    hint: &Hint,
    line: Line,
    out: &mut Vec<Box<[u8]>>,
) -> Result<(), LowerError> {
    match hint {
        Hint::Identifier(id) => {
            out.push(lo.resolve_class(id));
            Ok(())
        }
        Hint::Union(u) => {
            collect_catch_types(lo, u.left, line, out)?;
            collect_catch_types(lo, u.right, line, out)
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

/// The last `\`-separated segment of a name (`A\B\c` → `c`), discarding any
/// namespace prefix. Used only where a bareword is genuinely *not* a namespaced
/// name — e.g. an interpolated array-key barewords inside a double-quoted string.
fn bare_last_segment<'a>(id: &Identifier<'a>) -> &'a [u8] {
    id.last_segment()
}

/// Drop a single leading namespace separator (`\Foo\Bar` → `Foo\Bar`).
fn strip_leading_backslash(s: &[u8]) -> &[u8] {
    s.strip_prefix(b"\\").unwrap_or(s)
}

/// Join a namespace and a relative name into a fully-qualified name with no
/// leading separator (`Foo\Bar` + `Baz` → `Foo\Bar\Baz`; empty ns → `Baz`).
fn join_ns(ns: &[u8], name: &[u8]) -> Box<[u8]> {
    if ns.is_empty() {
        return name.into();
    }
    let mut v = Vec::with_capacity(ns.len() + 1 + name.len());
    v.extend_from_slice(ns);
    v.push(b'\\');
    v.extend_from_slice(name);
    v.into()
}

/// The first `\`-separated segment of a name (`A\B\c` → `A`). For an unqualified
/// name this is the whole thing.
fn first_segment(s: &[u8]) -> &[u8] {
    match s.iter().position(|&b| b == b'\\') {
        Some(i) => &s[..i],
        None => s,
    }
}

/// The namespace name a `namespace` declaration introduces (`None`/`namespace {}`
/// → the global namespace, empty bytes; a leading `\` is dropped).
fn ns_name_of(name: Option<&Identifier>) -> Vec<u8> {
    match name {
        Some(id) => strip_leading_backslash(id.value()).to_vec(),
        None => Vec::new(),
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
/// single-inheritance, so only the first type matters), step 19-3. Resolved to a
/// fully-qualified name against the current namespace + imports (step 50).
fn parent_name(lo: &Lowerer, ext: &Extends, line: Line) -> Result<Box<[u8]>, LowerError> {
    match ext.types.iter().next() {
        Some(id) => Ok(lo.resolve_class(id)),
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

/// Build the PHP link-time fatal for a concrete class that leaves abstract
/// methods unimplemented (step 21-4, D-21.11). Singular/plural and the
/// `Class::method` list match PHP's wording byte-for-byte.
fn abstract_unimplemented_fatal(class: &[u8], missing: &[&[u8]], line: Line) -> LowerError {
    let cname = String::from_utf8_lossy(class);
    let n = missing.len();
    let word = if n == 1 { "method" } else { "methods" };
    let list = missing
        .iter()
        .map(|m| format!("{}::{}", cname, String::from_utf8_lossy(m)))
        .collect::<Vec<_>>()
        .join(", ");
    LowerError::Fatal {
        message: format!(
            "Class {cname} contains {n} abstract {word} and must therefore be \
             declared abstract or implement the remaining {word} ({list})"
        ),
        line,
    }
}

/// Map a single visibility modifier (from a trait `as` adaptation) to [`Visibility`].
fn visibility_of_modifier(m: &Modifier) -> Visibility {
    match m {
        Modifier::Protected(_) => Visibility::Protected,
        Modifier::Private(_) => Visibility::Private,
        _ => Visibility::Public,
    }
}

/// Resolve a bare constant name to its literal HIR value (step 18, D-18.7).
/// Only engine constants are known; `true`/`false`/`null` are case-insensitive,
/// every other name is case-sensitive (PHP constants are). `None` for an unknown
/// (user-defined) constant — the caller turns that into an Unsupported SKIP.
pub(crate) fn resolve_constant(name: &[u8]) -> Option<ExprKind> {
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
        b"PHP_SAPI" => str_lit(b"cli"),
        // Platform identity (macOS oracle, matching `DIRECTORY_SEPARATOR`).
        b"PHP_OS" => str_lit(b"Darwin"),
        b"PHP_OS_FAMILY" => str_lit(b"Darwin"),
        b"DIRECTORY_SEPARATOR" => str_lit(b"/"),
        b"PATH_SEPARATOR" => str_lit(b":"),
        // setlocale() category selectors (macOS values, matching the oracle).
        b"LC_ALL" => ExprKind::Int(0),
        b"LC_COLLATE" => ExprKind::Int(1),
        b"LC_CTYPE" => ExprKind::Int(2),
        b"LC_MONETARY" => ExprKind::Int(3),
        b"LC_NUMERIC" => ExprKind::Int(4),
        b"LC_TIME" => ExprKind::Int(5),
        b"LC_MESSAGES" => ExprKind::Int(6),
        // ext/filter validate/sanitize selectors + flags (`filter_var`). The
        // oracle build lacks ext/filter, but Composer's symfony polyfill and
        // ErrorHandler reference these, so we define them with the canonical values.
        b"FILTER_DEFAULT" => ExprKind::Int(516),
        b"FILTER_VALIDATE_INT" => ExprKind::Int(257),
        b"FILTER_VALIDATE_BOOLEAN" | b"FILTER_VALIDATE_BOOL" => ExprKind::Int(258),
        b"FILTER_VALIDATE_FLOAT" => ExprKind::Int(259),
        b"FILTER_VALIDATE_REGEXP" => ExprKind::Int(272),
        b"FILTER_VALIDATE_DOMAIN" => ExprKind::Int(277),
        b"FILTER_VALIDATE_URL" => ExprKind::Int(273),
        b"FILTER_VALIDATE_EMAIL" => ExprKind::Int(274),
        b"FILTER_VALIDATE_IP" => ExprKind::Int(275),
        b"FILTER_NULL_ON_FAILURE" => ExprKind::Int(134217728),
        // Stream seek whence (step 51b): `fseek($f, $offset, $whence)`.
        b"SEEK_SET" => ExprKind::Int(0),
        b"SEEK_CUR" => ExprKind::Int(1),
        b"SEEK_END" => ExprKind::Int(2),
        // file_put_contents / file flags (step 51c, 55a).
        b"FILE_USE_INCLUDE_PATH" => ExprKind::Int(1),
        b"LOCK_EX" => ExprKind::Int(2),
        b"FILE_IGNORE_NEW_LINES" => ExprKind::Int(2),
        b"FILE_SKIP_EMPTY_LINES" => ExprKind::Int(4),
        b"FILE_APPEND" => ExprKind::Int(8),
        // pathinfo() component selectors (step 52a).
        b"PATHINFO_DIRNAME" => ExprKind::Int(1),
        b"PATHINFO_BASENAME" => ExprKind::Int(2),
        b"PATHINFO_EXTENSION" => ExprKind::Int(4),
        b"PATHINFO_FILENAME" => ExprKind::Int(8),
        // crypt() capability flags (step 64) — PHP bundles every algorithm, so
        // all are available; CRYPT_SALT_LENGTH is PHP_MAX_SALT_LEN.
        b"CRYPT_SALT_LENGTH" => ExprKind::Int(123),
        b"CRYPT_STD_DES" => ExprKind::Int(1),
        b"CRYPT_EXT_DES" => ExprKind::Int(1),
        b"CRYPT_MD5" => ExprKind::Int(1),
        b"CRYPT_BLOWFISH" => ExprKind::Int(1),
        b"CRYPT_SHA256" => ExprKind::Int(1),
        b"CRYPT_SHA512" => ExprKind::Int(1),
        // htmlspecialchars / htmlentities flags (step 56b).
        b"ENT_NOQUOTES" => ExprKind::Int(0),
        b"ENT_HTML401" => ExprKind::Int(0),
        b"ENT_COMPAT" => ExprKind::Int(2),
        b"ENT_QUOTES" => ExprKind::Int(3),
        b"ENT_IGNORE" => ExprKind::Int(4),
        b"ENT_SUBSTITUTE" => ExprKind::Int(8),
        b"ENT_HTML5" => ExprKind::Int(48),
        // scandir() sort order (step 52e).
        b"SCANDIR_SORT_ASCENDING" => ExprKind::Int(0),
        b"SCANDIR_SORT_DESCENDING" => ExprKind::Int(1),
        b"SCANDIR_SORT_NONE" => ExprKind::Int(2),
        // glob() flags (step 52e).
        b"GLOB_MARK" => ExprKind::Int(8),
        b"GLOB_NOSORT" => ExprKind::Int(32),
        b"GLOB_NOCHECK" => ExprKind::Int(16),
        b"GLOB_NOESCAPE" => ExprKind::Int(4096),
        b"GLOB_BRACE" => ExprKind::Int(128),
        b"GLOB_ONLYDIR" => ExprKind::Int(1_073_741_824),
        b"GLOB_ERR" => ExprKind::Int(4),
        // error_reporting / set_error_handler levels (bit flags). PHP 8.5 keeps
        // the E_STRICT slot (2048) reserved/unused; E_ALL is 32767.
        b"E_ERROR" => ExprKind::Int(1),
        b"E_WARNING" => ExprKind::Int(2),
        b"E_PARSE" => ExprKind::Int(4),
        b"E_NOTICE" => ExprKind::Int(8),
        b"E_CORE_ERROR" => ExprKind::Int(16),
        b"E_CORE_WARNING" => ExprKind::Int(32),
        b"E_COMPILE_ERROR" => ExprKind::Int(64),
        b"E_COMPILE_WARNING" => ExprKind::Int(128),
        b"E_USER_ERROR" => ExprKind::Int(256),
        b"E_USER_WARNING" => ExprKind::Int(512),
        b"E_USER_NOTICE" => ExprKind::Int(1024),
        b"E_STRICT" => ExprKind::Int(2048),
        b"E_RECOVERABLE_ERROR" => ExprKind::Int(4096),
        b"E_DEPRECATED" => ExprKind::Int(8192),
        b"E_USER_DEPRECATED" => ExprKind::Int(16384),
        // PHP 8.0 removed E_STRICT from E_ALL; PHP 8.4 made E_STRICT a no-op. The
        // current value is 30719 (E_ALL without E_STRICT=2048), matching 8.5.
        b"E_ALL" => ExprKind::Int(30719),
        // str_pad / array_filter / count flags.
        b"STR_PAD_RIGHT" => ExprKind::Int(1),
        b"STR_PAD_LEFT" => ExprKind::Int(0),
        b"STR_PAD_BOTH" => ExprKind::Int(2),
        // mb_convert_case modes (mbstring).
        b"MB_CASE_UPPER" => ExprKind::Int(0),
        b"MB_CASE_LOWER" => ExprKind::Int(1),
        b"MB_CASE_TITLE" => ExprKind::Int(2),
        b"MB_CASE_FOLD" => ExprKind::Int(3),
        b"MB_CASE_UPPER_SIMPLE" => ExprKind::Int(4),
        b"MB_CASE_LOWER_SIMPLE" => ExprKind::Int(5),
        b"MB_CASE_TITLE_SIMPLE" => ExprKind::Int(6),
        b"MB_CASE_FOLD_SIMPLE" => ExprKind::Int(7),
        b"ARRAY_FILTER_USE_KEY" => ExprKind::Int(2),
        b"ARRAY_FILTER_USE_BOTH" => ExprKind::Int(1),
        b"COUNT_NORMAL" => ExprKind::Int(0),
        b"COUNT_RECURSIVE" => ExprKind::Int(1),
        // json_encode / json_decode flags (step 26).
        b"JSON_UNESCAPED_SLASHES" => ExprKind::Int(64),
        b"JSON_PRETTY_PRINT" => ExprKind::Int(128),
        b"JSON_UNESCAPED_UNICODE" => ExprKind::Int(256),
        b"JSON_THROW_ON_ERROR" => ExprKind::Int(4_194_304),
        b"JSON_ERROR_NONE" => ExprKind::Int(0),
        // preg flags (step 31).
        b"PREG_PATTERN_ORDER" => ExprKind::Int(1),
        b"PREG_SET_ORDER" => ExprKind::Int(2),
        b"PREG_OFFSET_CAPTURE" => ExprKind::Int(256),
        b"PREG_UNMATCHED_AS_NULL" => ExprKind::Int(512),
        b"PREG_SPLIT_NO_EMPTY" => ExprKind::Int(1),
        b"PREG_SPLIT_DELIM_CAPTURE" => ExprKind::Int(2),
        b"PREG_SPLIT_OFFSET_CAPTURE" => ExprKind::Int(4),
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

/// PHP `dirname()` of the script path, for `__DIR__` (step 49). No separator →
/// `"."`; a leading-slash-only parent → `"/"`; otherwise the bytes before the
/// last `/`. Matches PHP for the POSIX paths `.phpt` runners use.
fn dirname(path: &[u8]) -> &[u8] {
    match path.iter().rposition(|&b| b == b'/') {
        None => b".",
        Some(0) => b"/",
        Some(i) => &path[..i],
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
fn lower_hint(lo: &Lowerer, hint: &Hint) -> Option<TypeHint> {
    let kind = match hint {
        Hint::Integer(_) => HintKind::Scalar(ScalarType::Int),
        Hint::Float(_) => HintKind::Scalar(ScalarType::Float),
        Hint::String(_) => HintKind::Scalar(ScalarType::String),
        Hint::Bool(_) => HintKind::Scalar(ScalarType::Bool),
        Hint::Array(_) => HintKind::Array,
        Hint::Callable(_) => HintKind::Callable,
        Hint::Iterable(_) => HintKind::Iterable,
        Hint::Object(_) => HintKind::Object,
        // A class / interface name → an `instanceof` check at the binder. A name
        // that is actually a reserved type keyword (a *qualified* `\int`, `\bool`,
        // …, which PHP rejects as "must be unqualified") is left unenforced rather
        // than mistaken for a class.
        Hint::Identifier(id) => {
            // A reserved scalar/type keyword is decided on the *bare* last segment
            // (`\int` → `int`); a real class hint resolves to its FQN (step 50).
            if is_reserved_type_name(bare_last_segment(id)) {
                return None;
            }
            HintKind::Class(lo.resolve_class(id))
        }
        Hint::Nullable(n) => {
            // `?T`: enforce only when the inner hint is itself enforced.
            let inner = lower_hint(lo, n.hint)?;
            return Some(TypeHint { nullable: true, ..inner });
        }
        // Union / intersection / `self`/`parent`/`static` / `mixed` / `void` /
        // literal types stay unenforced (lowered to `None`) for now.
        _ => return None,
    };
    Some(TypeHint { kind, nullable: false })
}

/// Whether `name` (case-insensitive) is a reserved type keyword rather than a
/// class name — so a qualified form like `\int` is not mistaken for a class.
fn is_reserved_type_name(name: &[u8]) -> bool {
    matches!(
        name.to_ascii_lowercase().as_slice(),
        b"int" | b"float" | b"string" | b"bool" | b"array" | b"object" | b"callable"
            | b"iterable" | b"void" | b"never" | b"mixed" | b"null" | b"false" | b"true"
            | b"self" | b"parent" | b"static"
    )
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
