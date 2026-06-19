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
    Class, ClassLikeConstantSelector, ClassLikeMember, ClassLikeMemberSelector, Closure,
    CompositeString, Construct,
    DeclareBody, DocumentIndentation, DocumentKind, DocumentString, Enum, EnumCaseItem,
    Expression, Extends, ForeachTarget, Function,
    FunctionLikeParameterList, Hint,
    Identifier, Instantiation, Interface, Literal, LiteralInteger, MatchArm as AstMatchArm, Method,
    MethodBody, Modifier, Node, PartialApplication, Property, PropertyItem, Statement, StaticItem,
    StringPart, Trait, TraitUse, TraitUseAdaptation, TraitUseMethodReference,
    TraitUseSpecification, UnaryPostfixOperator, UnaryPrefixOperator, Variable, Yield,
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
            LowerError::Fatal { message, line } => write!(f, "{message} on line {line}"),
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
    let (pclasses, pindex, pfunctions, pfn_index) = lower_prelude();
    low.classes = pclasses;
    low.class_index = pindex;
    // Seed the prelude's global functions (step 35: the procedural date API)
    // ahead of the user's, so user functions get ids contiguous after them. Like
    // the classes, call sites resolve by name, so no index fix-up is needed.
    low.functions = pfunctions;
    low.fn_index = pfn_index;
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
    // `goto`/label validation (step 45): the top-level script body is its own
    // function scope. Each user function / method / closure validates its own
    // body where it is lowered (`lower_function`/`lower_method`/`lower_closure`).
    validate_goto(&body)?;
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

/// The built-in classes, authored in PHP and lowered once into the front of
/// every program's class table (step 20): `stdClass` plus the throwable
/// hierarchy. Mirrors PHP's core/SPL classes closely enough for catch-matching,
/// the accessors, and `instanceof`.
/// `getTrace`/`getTraceAsString` are stubs (no real stack trace is modelled);
/// `file`/`line` are filled in by the evaluator at `new` time, not here.
const PRELUDE_SRC: &[u8] = br##"<?php
class stdClass {}
interface UnitEnum {}
interface BackedEnum extends UnitEnum {}
interface Stringable {}
interface Throwable {}
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
    public function __toString() { return $this->message; }
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
            fn_saw_yield: false,
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
            Statement::Enum(en) => {
                if self.class_index.contains_key(&en.name.value.to_ascii_lowercase()) {
                    return Ok(None);
                }
                return Err(LowerError::Unsupported {
                    what: "conditional enum declaration",
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

            // `goto label;` / `label:` (step 45). Both carry a `LocalIdentifier`
            // whose `value` is the raw label bytes. Validity (label defined, no
            // jump into a loop/switch) is checked in a later compile-time pass
            // over the lowered body; here we just record the marker / jump.
            Statement::Goto(node) => {
                StmtKind::Goto(node.label.value.to_vec().into_boxed_slice())
            }
            Statement::Label(node) => {
                StmtKind::Label(node.name.value.to_vec().into_boxed_slice())
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
            Enum(&'a Enum<'a>),
        }
        let mut pending: Vec<Pending> = Vec::new();
        for s in stmts {
            let (name, span) = match s {
                Statement::Class(c) => (c.name.value, c.span()),
                Statement::Interface(i) => (i.name.value, i.span()),
                Statement::Enum(e) => (e.name.value, e.span()),
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
                Statement::Enum(e) => Pending::Enum(e),
                _ => unreachable!(),
            });
        }
        for p in pending {
            let decl = match p {
                Pending::Class(c) => self.lower_class(c)?,
                Pending::Interface(i) => self.lower_interface(i)?,
                Pending::Enum(e) => self.lower_enum(e)?,
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
        // Names of abstract methods this class must implement (its own, plus any
        // pulled in from traits during flattening), step 21-4 / D-21.11.
        let mut abstract_req: Vec<Box<[u8]>> = Vec::new();
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
    fn lower_enum(&mut self, en: &Enum) -> Result<ClassDecl, LowerError> {
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

        let inner = (|| {
            let params = self.lower_params(&method.parameter_list, line)?;
            let body = self.lower_stmts(body.statements.as_slice())?;
            Ok::<_, LowerError>((params, body))
        })();

        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for method body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
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
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);

        let inner = self.lower_function_body(func, line);

        // Reclaim the function's local scope and restore the outer one.
        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for function body");
        self.after_closing_tag = saved_tag;
        self.fn_by_ref = saved_by_ref;
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
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);

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
        let saved_saw_yield = std::mem::replace(&mut self.fn_saw_yield, false);

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

            // `yield` / `yield $k => $v` / `yield from $it` (step 39). Marks the
            // current function a generator via `fn_saw_yield` (read when its
            // `FnDecl` is built). The nested `lower_expr` calls happen *before*
            // setting the flag, so a `yield` whose operand itself contains a
            // `yield` still flags exactly this function.
            Expression::Yield(y) => {
                let kind = match y {
                    Yield::Value(v) => ExprKind::Yield {
                        key: None,
                        value: match &v.value {
                            Some(e) => Some(Box::new(self.lower_expr(e)?)),
                            None => None,
                        },
                    },
                    Yield::Pair(p) => ExprKind::Yield {
                        key: Some(Box::new(self.lower_expr(p.key)?)),
                        value: Some(Box::new(self.lower_expr(p.value)?)),
                    },
                    Yield::From(fr) => {
                        ExprKind::YieldFrom(Box::new(self.lower_expr(fr.iterator)?))
                    }
                };
                self.fn_saw_yield = true;
                kind
            }

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
                    let index = match aa.index {
                        // A bare identifier as an array index only arises from
                        // string interpolation (`"$a[k]"`), where mago rewrites
                        // the unquoted key to an identifier; it is a string key,
                        // not a constant (step 25).
                        Expression::Identifier(id) => {
                            Expr { line, kind: ExprKind::Str(function_name(id).into()) }
                        }
                        other => self.lower_expr(other)?,
                    };
                    ExprKind::Index {
                        base: Box::new(self.lower_expr(aa.array)?),
                        index: Box::new(index),
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
                // `print expr` — an expression that emits then yields int(1).
                Construct::Print(p) => ExprKind::Print(Box::new(self.lower_expr(p.value)?)),
                // `exit`/`die [arg]` — `die` is an exact alias of `exit`. Both
                // take an optional single positional argument.
                Construct::Exit(e) => {
                    ExprKind::Exit(self.lower_exit_arg(e.arguments.as_ref(), line)?)
                }
                Construct::Die(d) => {
                    ExprKind::Exit(self.lower_exit_arg(d.arguments.as_ref(), line)?)
                }
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

            Expression::CompositeString(CompositeString::Document(doc)) => {
                self.lower_document(doc, line)?
            }
            Expression::CompositeString(cs) => self.lower_interpolation(cs, line)?,

            _ => {
                return Err(LowerError::Unsupported {
                    what: "expression",
                    line,
                })
            }
        };
        Ok(Expr { line, kind })
    }

    /// Lower a double-quoted / heredoc interpolated string (step 25) to a chain
    /// of string concatenations. Each part is a literal chunk, a simple
    /// interpolation (`$x`, `$a[k]`, `$o->p`), or a braced expression (`{$e}`).
    /// Seeding with an empty string forces the whole result to a string even
    /// when it is a single interpolated value (e.g. `"$n"` for an int `$n`):
    /// `"" . x` has the same string-coercion semantics as `(string) x`, and
    /// `Concat` already honours `__toString` on objects (step 19-6).
    fn lower_interpolation(
        &mut self,
        cs: &CompositeString,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        let mut acc = Expr { line, kind: ExprKind::Str(Default::default()) };
        for part in cs.parts().iter() {
            let piece = match part {
                StringPart::Literal(l) => Expr {
                    line,
                    // Double-quoted strings process `\"` -> `"`.
                    kind: ExprKind::Str(unescape_double_quoted(l.value, true).into()),
                },
                StringPart::Expression(e) => self.lower_expr(e)?,
                StringPart::BracedExpression(b) => self.lower_expr(b.expression)?,
            };
            acc = Expr {
                line,
                kind: ExprKind::Binary(BinOp::Concat, Box::new(acc), Box::new(piece)),
            };
        }
        Ok(acc.kind)
    }

    /// Lower a heredoc/nowdoc (`<<<EOD` / `<<<'EOD'`). mago hands the raw body
    /// back (no dedent, no trailing-newline strip), exposing the closing
    /// marker's indentation separately, so we replicate the lexer here:
    ///   1. strip the marker's indentation from the start of every body line;
    ///   2. drop the final newline before the closing marker;
    ///   3. heredoc only: interpolate parts and process escapes (but `\"` stays
    ///      literal — double quotes are not special in a heredoc); nowdoc keeps
    ///      every byte verbatim (no interpolation, no escapes).
    fn lower_document(
        &mut self,
        doc: &DocumentString,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        let indent = match doc.indentation {
            DocumentIndentation::None => 0,
            DocumentIndentation::Whitespace(n) | DocumentIndentation::Tab(n) => n,
            DocumentIndentation::Mixed(a, b) => a + b,
        };
        let heredoc = matches!(doc.kind, DocumentKind::Heredoc);

        // Dedent literal segments (tracking line starts across the sequence),
        // remembering which produced segment is the last literal so we can drop
        // its trailing newline once the full body is known.
        enum Seg<'a> {
            Lit(Vec<u8>),
            Dyn(&'a Expression<'a>),
        }
        let mut segs: Vec<Seg> = Vec::new();
        let mut at_line_start = true;
        let mut last_lit: Option<usize> = None;
        for part in doc.parts.iter() {
            match part {
                StringPart::Literal(l) => {
                    let (bytes, next_start) = dedent_literal(l.value, indent, at_line_start);
                    at_line_start = next_start;
                    last_lit = Some(segs.len());
                    segs.push(Seg::Lit(bytes));
                }
                StringPart::Expression(e) => {
                    at_line_start = false;
                    segs.push(Seg::Dyn(e));
                }
                StringPart::BracedExpression(b) => {
                    at_line_start = false;
                    segs.push(Seg::Dyn(b.expression));
                }
            }
        }
        // Drop the single trailing newline (the separator before the marker).
        if let Some(idx) = last_lit {
            if let Seg::Lit(bytes) = &mut segs[idx] {
                if bytes.last() == Some(&b'\n') {
                    bytes.pop();
                    if bytes.last() == Some(&b'\r') {
                        bytes.pop();
                    }
                }
            }
        }

        // Concatenate, seeded with "" to force a string result.
        let mut acc = Expr { line, kind: ExprKind::Str(Default::default()) };
        for seg in segs {
            let piece = match seg {
                Seg::Lit(bytes) => {
                    let value = if heredoc {
                        unescape_double_quoted(&bytes, false)
                    } else {
                        bytes
                    };
                    Expr { line, kind: ExprKind::Str(value.into()) }
                }
                Seg::Dyn(e) => self.lower_expr(e)?,
            };
            acc = Expr {
                line,
                kind: ExprKind::Binary(BinOp::Concat, Box::new(acc), Box::new(piece)),
            };
        }
        Ok(acc.kind)
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
            P::ObjectCast(..) => cast(CastKind::Object, self)?,
            P::UnsetCast(..) | P::VoidCast(..) => {
                return Err(LowerError::Unsupported {
                    what: "unset/void cast",
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
                let (args, named) = self.lower_args(&mc.argument_list, line)?;
                return Ok(ExprKind::MethodCall {
                    object,
                    method: method.into(),
                    args,
                    named,
                    nullsafe: false,
                });
            }
            Call::NullSafeMethod(mc) => {
                let object = Box::new(self.lower_expr(mc.object)?);
                let method = member_name(&mc.method, line)?;
                let (args, named) = self.lower_args(&mc.argument_list, line)?;
                return Ok(ExprKind::MethodCall {
                    object,
                    method: method.into(),
                    args,
                    named,
                    nullsafe: true,
                });
            }
            // `Class::m()` / `self::m()` / `parent::m()` / `static::m()`.
            Call::StaticMethod(sm) => {
                let class = class_ref_of(sm.class, line)?;
                let method = member_name(&sm.method, line)?;
                let (args, named) = self.lower_args(&sm.argument_list, line)?;
                return Ok(ExprKind::StaticCall {
                    class,
                    method: method.into(),
                    args,
                    named,
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
        let (args, named) = self.lower_args(&fc.argument_list, line)?;
        Ok(ExprKind::Call {
            name: name.into(),
            args,
            named,
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
        let (args, named) = match &inst.argument_list {
            Some(list) => self.lower_args(list, line)?,
            None => (Vec::new(), Vec::new()),
        };
        Ok(ExprKind::New { class, args, named })
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

    /// Lower the optional single argument of `exit`/`die` (step 46). PHP accepts
    /// zero or one positional argument; we take the first positional expression
    /// (if any) and ignore the rest. `exit`/`exit()`/`die()` → `None`.
    fn lower_exit_arg(
        &mut self,
        list: Option<&mago_syntax::ast::ArgumentList>,
        line: Line,
    ) -> Result<Option<Box<Expr>>, LowerError> {
        let Some(list) = list else { return Ok(None) };
        match list.arguments.iter().next() {
            Some(Argument::Positional(p)) => {
                Ok(Some(Box::new(self.lower_expr(p.value)?)))
            }
            Some(Argument::Named(_)) => Err(LowerError::Unsupported {
                what: "named argument to exit/die",
                line,
            }),
            None => Ok(None),
        }
    }

    /// Lower a call's arguments into leading positional + trailing named (step
    /// 38). Variadic spread (`...$a`) stays out of scope. A positional argument
    /// after a named one is a PHP compile-time `Fatal error`, surfaced here.
    #[allow(clippy::type_complexity)]
    fn lower_args(
        &mut self,
        list: &mago_syntax::ast::ArgumentList,
        line: Line,
    ) -> Result<(Vec<Expr>, Vec<(Box<[u8]>, Expr)>), LowerError> {
        let mut args = Vec::new();
        let mut named: Vec<(Box<[u8]>, Expr)> = Vec::new();
        // Track whether a spread (`...$e`) has appeared: a plain positional after
        // one is a compile-time Fatal, matching PHP (step 40).
        let mut saw_spread = false;
        for arg in list.arguments.iter() {
            match arg {
                Argument::Positional(p) if p.ellipsis.is_some() => {
                    // A spread after a named argument is a compile-time Fatal.
                    if !named.is_empty() {
                        return Err(LowerError::Fatal {
                            message: "Cannot use argument unpacking after named arguments"
                                .to_string(),
                            line,
                        });
                    }
                    saw_spread = true;
                    let inner = self.lower_expr(p.value)?;
                    args.push(Expr {
                        kind: ExprKind::Spread(Box::new(inner)),
                        line,
                    });
                }
                Argument::Positional(p) => {
                    if !named.is_empty() {
                        return Err(LowerError::Fatal {
                            message: "Cannot use positional argument after named argument"
                                .to_string(),
                            line,
                        });
                    }
                    if saw_spread {
                        return Err(LowerError::Fatal {
                            message: "Cannot use positional argument after argument unpacking"
                                .to_string(),
                            line,
                        });
                    }
                    args.push(self.lower_expr(p.value)?);
                }
                Argument::Named(n) => {
                    named.push((n.name.value.into(), self.lower_expr(n.value)?));
                }
            }
        }
        Ok((args, named))
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
