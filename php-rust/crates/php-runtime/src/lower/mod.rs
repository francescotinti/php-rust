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
    AssignmentOperator, BinaryOperator,
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
mod curl_consts;
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
    seed_globals: &[Box<[u8]>],
    seed_aliases: &[(Vec<u8>, Vec<u8>)],
) -> Result<Program, LowerError> {
    lower_source_impl(
        name,
        source,
        Some((seed_classes, seed_static, seed_traits, seed_globals, seed_aliases)),
    )
}

type Seed<'a> = (
    &'a [crate::hir::ClassDecl],
    usize,
    &'a [(Vec<u8>, LoweredTrait)],
    &'a [Box<[u8]>],
    &'a [(Vec<u8>, Vec<u8>)],
);

fn lower_source_impl(
    name: &[u8],
    source: &[u8],
    seed: Option<Seed<'_>>,
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
    // Docblock trivia, for `getDocComment` retention (the AST drops comments).
    low.docs = program
        .trivia
        .iter()
        .filter(|t| t.kind.is_docblock())
        .map(|t| (t.span.start.offset, t.span.end.offset))
        .collect();
    // A misplaced `namespace` is a compile-time fatal in PHP; reject it here
    // (before hoisting) so it renders the exact error rather than being silently
    // accepted or tripping a later invariant.
    low.check_namespace_first(program.statements.as_slice())?;
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
        Some((sclasses, sstatic, straits, sglobals, saliases)) => {
            // Seed the shared global variable name→slot registry (step 57): a
            // seeded (`include`/`eval`) unit numbers its `$GLOBALS['x']` / `global
            // $x` slots to *agree* with `main`'s (and every earlier unit's), since
            // at run time all `DimBase::Global` ops index the one bottom (`main`)
            // frame. Pre-populating `globals` in order reproduces the shared slot
            // numbering; a genuinely new global name this unit introduces appends
            // past the seed (the VM grows the bottom frame to match). Without this,
            // each unit renumbers globals from 0 and cross-unit access aliases the
            // wrong cell or overflows `main`'s frame.
            for g in sglobals {
                low.globals.slot_for(g);
            }
            low.classes = sclasses.to_vec();
            let mut ci: HashMap<Vec<u8>, usize> = HashMap::new();
            for (i, cd) in sclasses.iter().enumerate() {
                ci.entry(cd.name.to_ascii_lowercase()).or_insert(i);
            }
            // Runtime `class_alias` entries resolve to the ORIGINAL decl (index
            // only — no clone), so `extends LegacyName` inherits the real class.
            for (alias, orig) in saliases {
                if let Some(&i) = ci.get(&orig.to_ascii_lowercase()) {
                    ci.entry(alias.to_ascii_lowercase()).or_insert(i);
                }
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
            let (pfunctions, pfn_index) = prelude_functions();
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
// unserialize() of an unknown class: the instance keeps its data plus the
// original class name in `__PHP_Incomplete_Class_Name` (set VM-side).
class __PHP_Incomplete_Class {}
// Incremental hashing (hash_init/update/final): the context buffers the fed
// data and the digest is computed at final by the one-shot hash()/hash_hmac()
// builtins (which are oracle-faithful), so every algorithm they support works
// incrementally too. Output-identical to streaming; memory-proportional to
// the fed data (fine for the workloads phpr targets).
final class HashContext {
    public $__algo = '';
    public $__buf = '';
    public $__key = null;
}
function hash_init($algo, $flags = 0, $key = '') {
    try { hash($algo, ''); } catch (ValueError $e) {
        // hash_init()'s message has no trailing `, "x" given` (unlike hash()).
        throw new ValueError('hash_init(): Argument #1 ($algo) must be a valid hashing algorithm');
    }
    $c = new HashContext;
    $c->__algo = $algo;
    if (($flags & HASH_HMAC) !== 0) {
        if ($key === '' || $key === null) {
            throw new ValueError('hash_init(): Argument #3 ($key) cannot be empty when HASH_HMAC is specified');
        }
        $c->__key = $key;
    }
    return $c;
}
function hash_update($context, $data) { $context->__buf .= $data; return true; }
function hash_update_stream($context, $stream, $length = -1) {
    $data = $length >= 0 ? stream_get_contents($stream, $length) : stream_get_contents($stream);
    if ($data === false) { return 0; }
    $context->__buf .= $data;
    return strlen($data);
}
function hash_update_file($context, $filename) {
    $d = @file_get_contents($filename);
    if ($d === false) { return false; }
    $context->__buf .= $d;
    return true;
}
function hash_final($context, $binary = false) {
    if ($context->__key !== null) {
        return hash_hmac($context->__algo, $context->__buf, $context->__key, $binary);
    }
    return hash($context->__algo, $context->__buf, $binary);
}
function hash_copy($context) { return clone $context; }
function hash_file($algo, $filename, $binary = false) {
    $d = @file_get_contents($filename);
    if ($d === false) { return false; }
    return hash($algo, $d, $binary);
}
function fsockopen($hostname, $port = -1, &$error_code = null, &$error_string = null, $timeout = null) {
    $r = __fsockopen((string)$hostname, (int)$port, $timeout === null ? -1.0 : (float)$timeout);
    $error_code = $r[1];
    $error_string = $r[2];
    return $r[0];
}
// phpr has no persistent-connection pool: pfsockopen connects fresh.
function pfsockopen($hostname, $port = -1, &$error_code = null, &$error_string = null, $timeout = null) {
    $r = __fsockopen((string)$hostname, (int)$port, $timeout === null ? -1.0 : (float)$timeout);
    $error_code = $r[1];
    $error_string = $r[2];
    return $r[0];
}
function stream_select(&$read, &$write, &$except, $seconds, $microseconds = null) {
    $r = __stream_select($read ?? [], $write ?? [], $except ?? [], $seconds, $microseconds);
    if ($r === false) { return false; }
    $read = $r[1]; $write = $r[2]; $except = $r[3];
    return $r[0];
}
function hash_hmac_file($algo, $filename, $key, $binary = false) {
    $d = @file_get_contents($filename);
    if ($d === false) { return false; }
    return hash_hmac($algo, $d, $key, $binary);
}
// ext/curl easy API: CurlHandle (a PHP 8 object, not a resource) wraps an id
// into the host-side handle table (__curl_* in php-builtins/curl.rs, over the
// same rustls/ureq transport as the http:// stream wrapper). curl_multi_* is
// deliberately NOT defined: consumers that probe for it (Composer) fall back
// to streams; function_exists('curl_exec') consumers (monolog, Guzzle sync)
// take this path.
final class CurlHandle {
    public $__id = 0;
}
function curl_init($url = null) {
    $h = new CurlHandle;
    $h->__id = __curl_init();
    if ($url !== null) { curl_setopt($h, CURLOPT_URL, $url); }
    return $h;
}
function curl_setopt($handle, $option, $value) { return __curl_setopt($handle->__id, $option, $value); }
function curl_setopt_array($handle, $options) {
    foreach ($options as $k => $v) {
        if (!curl_setopt($handle, $k, $v)) { return false; }
    }
    return true;
}
function curl_exec($handle) { return __curl_exec($handle->__id); }
function curl_errno($handle) { return __curl_errno($handle->__id); }
function curl_error($handle) { return __curl_error($handle->__id); }
function curl_getinfo($handle, $option = null) { return __curl_getinfo($handle->__id, $option); }
// curl_close() is a host builtin (no-op + 8.5 deprecation with caller attribution).
function curl_reset($handle) { __curl_reset($handle->__id); }
function curl_escape($handle, $string) { return rawurlencode($string); }
function curl_unescape($handle, $string) { return rawurldecode($string); }
function curl_version() {
    // Honest facade values: version mirrors a current libcurl line so version
    // gates pass, but ssl_version/host say what the backend really is. The
    // features bitmask claims only IPV6 (1) + SSL (4) - no HTTP2/libz/brotli.
    return [
        'version_number' => 526081,
        'age' => 11,
        'features' => 5,
        'feature_list' => [
            'AsynchDNS' => false,
            'IPv6' => true,
            'SSL' => true,
            'libz' => false,
            'HTTP2' => false,
            'brotli' => false,
            'zstd' => false,
        ],
        'ssl_version_number' => 0,
        'version' => '8.7.1',
        'host' => 'phpr-rustls',
        'ssl_version' => 'rustls',
        'libz_version' => '',
        'protocols' => ['http', 'https'],
        'ares' => '',
        'ares_num' => 0,
        'libidn' => '',
        'iconv_ver_num' => 0,
        'libssh_version' => '',
        'brotli_ver_num' => 0,
        'brotli_version' => '',
    ];
}
#[Attribute(Attribute::TARGET_CLASS)]
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
// Engine interfaces carry their real method signatures (compiled as
// abstract_sigs): hasMethod/getMethods and PHPUnit interface mocks read them.
interface Stringable {
    public function __toString(): string;
}
interface Throwable {}
interface Traversable {}
interface Iterator extends Traversable {
    public function current(): mixed;
    public function key(): mixed;
    public function next(): void;
    public function rewind(): void;
    public function valid(): bool;
}
interface IteratorAggregate extends Traversable {
    public function getIterator(): Traversable;
}
interface ArrayAccess {
    public function offsetExists(mixed $offset): bool;
    public function offsetGet(mixed $offset): mixed;
    public function offsetSet(mixed $offset, mixed $value): void;
    public function offsetUnset(mixed $offset): void;
}
interface Countable {
    public function count(): int;
}
interface JsonSerializable {
    public function jsonSerialize(): mixed;
}
interface Serializable {
    public function serialize();
    public function unserialize($data);
}
interface SeekableIterator extends Iterator {
    public function seek(int $offset);
}
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
class PharException extends Exception {}
class TypeError extends Error {}
class ArgumentCountError extends TypeError {}
class ValueError extends Error {}
class ArithmeticError extends Error {}
class DivisionByZeroError extends ArithmeticError {}
class UnhandledMatchError extends Error {}
class AssertionError extends Error {}
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
interface DateTimeInterface {
    const ATOM = 'Y-m-d\TH:i:sP';
    const COOKIE = 'l, d-M-Y H:i:s T';
    const ISO8601 = 'Y-m-d\TH:i:sO';
    const ISO8601_EXPANDED = 'X-m-d\TH:i:sP';
    const RFC822 = 'D, d M y H:i:s O';
    const RFC850 = 'l, d-M-y H:i:s T';
    const RFC1036 = 'D, d M y H:i:s O';
    const RFC1123 = 'D, d M Y H:i:s O';
    const RFC7231 = 'D, d M Y H:i:s \G\M\T';
    const RFC2822 = 'D, d M Y H:i:s O';
    const RFC3339 = 'Y-m-d\TH:i:sP';
    const RFC3339_EXTENDED = 'Y-m-d\TH:i:s.vP';
    const RSS = 'D, d M Y H:i:s O';
    const W3C = 'Y-m-d\TH:i:sP';
}
// phpr models instants as UTC unix timestamps, so a timezone is carried for
// `getName()`/`getTimezone()` but does not shift the stored timestamp (faithful
// for the UTC zone Composer uses; tz-aware display is deliberately out of scope).
class DateTimeZone {
    const AFRICA = 1;
    const AMERICA = 2;
    const ANTARCTICA = 4;
    const ARCTIC = 8;
    const ASIA = 16;
    const ATLANTIC = 32;
    const AUSTRALIA = 64;
    const EUROPE = 128;
    const INDIAN = 256;
    const PACIFIC = 512;
    const UTC = 1024;
    const ALL = 2047;
    const ALL_WITH_BC = 4095;
    const PER_COUNTRY = 4096;
    private $__name = "UTC";
    public function __construct($timezone = "UTC") { $this->__name = (string)$timezone; }
    public function getName() { return $this->__name; }
    public function __toString() { return $this->__name; }
    // The oracle's 419 identifiers (macOS tzdata), comma-packed to keep the
    // prelude compact. Group/country filtering is not modelled: real consumers
    // (monolog's setTimezoneProvider) call it bare.
    public static function listIdentifiers($timezoneGroup = DateTimeZone::ALL, $countryCode = null) {
        return explode(',', 'Africa/Abidjan,Africa/Accra,Africa/Addis_Ababa,Africa/Algiers,Africa/Asmara,Africa/Bamako,Africa/Bangui,Africa/Banjul,Africa/Bissau,Africa/Blantyre,Africa/Brazzaville,Africa/Bujumbura,Africa/Cairo,Africa/Casablanca,Africa/Ceuta,Africa/Conakry,Africa/Dakar,Africa/Dar_es_Salaam,Africa/Djibouti,Africa/Douala,Africa/El_Aaiun,Africa/Freetown,Africa/Gaborone,Africa/Harare,Africa/Johannesburg,Africa/Juba,Africa/Kampala,Africa/Khartoum,Africa/Kigali,Africa/Kinshasa,Africa/Lagos,Africa/Libreville,Africa/Lome,Africa/Luanda,Africa/Lubumbashi,Africa/Lusaka,Africa/Malabo,Africa/Maputo,Africa/Maseru,Africa/Mbabane,Africa/Mogadishu,Africa/Monrovia,Africa/Nairobi,Africa/Ndjamena,Africa/Niamey,Africa/Nouakchott,Africa/Ouagadougou,Africa/Porto-Novo,Africa/Sao_Tome,Africa/Tripoli,Africa/Tunis,Africa/Windhoek,America/Adak,America/Anchorage,America/Anguilla,America/Antigua,America/Araguaina,America/Argentina/Buenos_Aires,America/Argentina/Catamarca,America/Argentina/Cordoba,America/Argentina/Jujuy,America/Argentina/La_Rioja,America/Argentina/Mendoza,America/Argentina/Rio_Gallegos,America/Argentina/Salta,America/Argentina/San_Juan,America/Argentina/San_Luis,America/Argentina/Tucuman,America/Argentina/Ushuaia,America/Aruba,America/Asuncion,America/Atikokan,America/Bahia,America/Bahia_Banderas,America/Barbados,America/Belem,America/Belize,America/Blanc-Sablon,America/Boa_Vista,America/Bogota,America/Boise,America/Cambridge_Bay,America/Campo_Grande,America/Cancun,America/Caracas,America/Cayenne,America/Cayman,America/Chicago,America/Chihuahua,America/Ciudad_Juarez,America/Costa_Rica,America/Coyhaique,America/Creston,America/Cuiaba,America/Curacao,America/Danmarkshavn,America/Dawson,America/Dawson_Creek,America/Denver,America/Detroit,America/Dominica,America/Edmonton,America/Eirunepe,America/El_Salvador,America/Fort_Nelson,America/Fortaleza,America/Glace_Bay,America/Goose_Bay,America/Grand_Turk,America/Grenada,America/Guadeloupe,America/Guatemala,America/Guayaquil,America/Guyana,America/Halifax,America/Havana,America/Hermosillo,America/Indiana/Indianapolis,America/Indiana/Knox,America/Indiana/Marengo,America/Indiana/Petersburg,America/Indiana/Tell_City,America/Indiana/Vevay,America/Indiana/Vincennes,America/Indiana/Winamac,America/Inuvik,America/Iqaluit,America/Jamaica,America/Juneau,America/Kentucky/Louisville,America/Kentucky/Monticello,America/Kralendijk,America/La_Paz,America/Lima,America/Los_Angeles,America/Lower_Princes,America/Maceio,America/Managua,America/Manaus,America/Marigot,America/Martinique,America/Matamoros,America/Mazatlan,America/Menominee,America/Merida,America/Metlakatla,America/Mexico_City,America/Miquelon,America/Moncton,America/Monterrey,America/Montevideo,America/Montserrat,America/Nassau,America/New_York,America/Nome,America/Noronha,America/North_Dakota/Beulah,America/North_Dakota/Center,America/North_Dakota/New_Salem,America/Nuuk,America/Ojinaga,America/Panama,America/Paramaribo,America/Phoenix,America/Port-au-Prince,America/Port_of_Spain,America/Porto_Velho,America/Puerto_Rico,America/Punta_Arenas,America/Rankin_Inlet,America/Recife,America/Regina,America/Resolute,America/Rio_Branco,America/Santarem,America/Santiago,America/Santo_Domingo,America/Sao_Paulo,America/Scoresbysund,America/Sitka,America/St_Barthelemy,America/St_Johns,America/St_Kitts,America/St_Lucia,America/St_Thomas,America/St_Vincent,America/Swift_Current,America/Tegucigalpa,America/Thule,America/Tijuana,America/Toronto,America/Tortola,America/Vancouver,America/Whitehorse,America/Winnipeg,America/Yakutat,Antarctica/Casey,Antarctica/Davis,Antarctica/DumontDUrville,Antarctica/Macquarie,Antarctica/Mawson,Antarctica/McMurdo,Antarctica/Palmer,Antarctica/Rothera,Antarctica/Syowa,Antarctica/Troll,Antarctica/Vostok,Arctic/Longyearbyen,Asia/Aden,Asia/Almaty,Asia/Amman,Asia/Anadyr,Asia/Aqtau,Asia/Aqtobe,Asia/Ashgabat,Asia/Atyrau,Asia/Baghdad,Asia/Bahrain,Asia/Baku,Asia/Bangkok,Asia/Barnaul,Asia/Beirut,Asia/Bishkek,Asia/Brunei,Asia/Chita,Asia/Colombo,Asia/Damascus,Asia/Dhaka,Asia/Dili,Asia/Dubai,Asia/Dushanbe,Asia/Famagusta,Asia/Gaza,Asia/Hebron,Asia/Ho_Chi_Minh,Asia/Hong_Kong,Asia/Hovd,Asia/Irkutsk,Asia/Jakarta,Asia/Jayapura,Asia/Jerusalem,Asia/Kabul,Asia/Kamchatka,Asia/Karachi,Asia/Kathmandu,Asia/Khandyga,Asia/Kolkata,Asia/Krasnoyarsk,Asia/Kuala_Lumpur,Asia/Kuching,Asia/Kuwait,Asia/Macau,Asia/Magadan,Asia/Makassar,Asia/Manila,Asia/Muscat,Asia/Nicosia,Asia/Novokuznetsk,Asia/Novosibirsk,Asia/Omsk,Asia/Oral,Asia/Phnom_Penh,Asia/Pontianak,Asia/Pyongyang,Asia/Qatar,Asia/Qostanay,Asia/Qyzylorda,Asia/Riyadh,Asia/Sakhalin,Asia/Samarkand,Asia/Seoul,Asia/Shanghai,Asia/Singapore,Asia/Srednekolymsk,Asia/Taipei,Asia/Tashkent,Asia/Tbilisi,Asia/Tehran,Asia/Thimphu,Asia/Tokyo,Asia/Tomsk,Asia/Ulaanbaatar,Asia/Urumqi,Asia/Ust-Nera,Asia/Vientiane,Asia/Vladivostok,Asia/Yakutsk,Asia/Yangon,Asia/Yekaterinburg,Asia/Yerevan,Atlantic/Azores,Atlantic/Bermuda,Atlantic/Canary,Atlantic/Cape_Verde,Atlantic/Faroe,Atlantic/Madeira,Atlantic/Reykjavik,Atlantic/South_Georgia,Atlantic/St_Helena,Atlantic/Stanley,Australia/Adelaide,Australia/Brisbane,Australia/Broken_Hill,Australia/Darwin,Australia/Eucla,Australia/Hobart,Australia/Lindeman,Australia/Lord_Howe,Australia/Melbourne,Australia/Perth,Australia/Sydney,Europe/Amsterdam,Europe/Andorra,Europe/Astrakhan,Europe/Athens,Europe/Belgrade,Europe/Berlin,Europe/Bratislava,Europe/Brussels,Europe/Bucharest,Europe/Budapest,Europe/Busingen,Europe/Chisinau,Europe/Copenhagen,Europe/Dublin,Europe/Gibraltar,Europe/Guernsey,Europe/Helsinki,Europe/Isle_of_Man,Europe/Istanbul,Europe/Jersey,Europe/Kaliningrad,Europe/Kirov,Europe/Kyiv,Europe/Lisbon,Europe/Ljubljana,Europe/London,Europe/Luxembourg,Europe/Madrid,Europe/Malta,Europe/Mariehamn,Europe/Minsk,Europe/Monaco,Europe/Moscow,Europe/Oslo,Europe/Paris,Europe/Podgorica,Europe/Prague,Europe/Riga,Europe/Rome,Europe/Samara,Europe/San_Marino,Europe/Sarajevo,Europe/Saratov,Europe/Simferopol,Europe/Skopje,Europe/Sofia,Europe/Stockholm,Europe/Tallinn,Europe/Tirane,Europe/Ulyanovsk,Europe/Vaduz,Europe/Vatican,Europe/Vienna,Europe/Vilnius,Europe/Volgograd,Europe/Warsaw,Europe/Zagreb,Europe/Zurich,Indian/Antananarivo,Indian/Chagos,Indian/Christmas,Indian/Cocos,Indian/Comoro,Indian/Kerguelen,Indian/Mahe,Indian/Maldives,Indian/Mauritius,Indian/Mayotte,Indian/Reunion,Pacific/Apia,Pacific/Auckland,Pacific/Bougainville,Pacific/Chatham,Pacific/Chuuk,Pacific/Easter,Pacific/Efate,Pacific/Fakaofo,Pacific/Fiji,Pacific/Funafuti,Pacific/Galapagos,Pacific/Gambier,Pacific/Guadalcanal,Pacific/Guam,Pacific/Honolulu,Pacific/Kanton,Pacific/Kiritimati,Pacific/Kosrae,Pacific/Kwajalein,Pacific/Majuro,Pacific/Marquesas,Pacific/Midway,Pacific/Nauru,Pacific/Niue,Pacific/Norfolk,Pacific/Noumea,Pacific/Pago_Pago,Pacific/Palau,Pacific/Pitcairn,Pacific/Pohnpei,Pacific/Port_Moresby,Pacific/Rarotonga,Pacific/Saipan,Pacific/Tahiti,Pacific/Tarawa,Pacific/Tongatapu,Pacific/Wake,Pacific/Wallis,UTC');
    }
}
class DateTime implements DateTimeInterface {
    private $__ts = 0;
    private $__us = 0;
    private $__tz = "UTC";
    public function __construct($datetime = "now", $timezone = null) {
        if ($timezone !== null) { $this->__tz = $timezone->getName(); }
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $t = microtime(true);
            $this->__ts = (int) $t;
            $this->__us = (int) round(($t - (int) $t) * 1000000);
        } else {
            // A leading '@' (unix timestamp) forces the UTC-offset zone "+00:00",
            // ignoring any passed timezone (a PHP quirk).
            if (is_string($datetime) && isset($datetime[0]) && $datetime[0] === "@") {
                $this->__tz = "+00:00";
            }
            $parse = $datetime;
            if (is_string($datetime) && preg_match('/\.(\d{1,6})/', $datetime, $m) === 1) {
                $this->__us = (int) str_pad($m[1], 6, '0');
                $parse = preg_replace('/\.\d{1,6}/', '', $datetime, 1);
            }
            $r = strtotime($parse);
            if ($r === false) {
                throw new Exception("DateTime::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r;
        }
    }
    public function getTimezone() { return new DateTimeZone($this->__tz); }
    public function setTimezone($timezone) {
        $this->__tz = is_string($timezone) ? $timezone : $timezone->getName();
        return $this;
    }
    public static function createFromInterface($object) {
        $d = new DateTime("@" . $object->getTimestamp());
        return $d->setTimezone($object->getTimezone());
    }
    public static function createFromImmutable($object) { return static::createFromInterface($object); }
    public function format($format) {
        // 'u'/'v' (micro/milliseconds) come from this instance, not date():
        // substitute the digits as backslash-escaped literals in the format.
        $out = ''; $esc = false;
        for ($i = 0, $len = strlen($format); $i < $len; $i++) {
            $c = $format[$i];
            if ($esc) { $out .= '\\' . $c; $esc = false; continue; }
            if ($c === '\\') { $esc = true; continue; }
            if ($c === 'u' || $c === 'v') {
                $n = $c === 'u' ? sprintf('%06d', $this->__us) : sprintf('%03d', intdiv($this->__us, 1000));
                foreach (str_split($n) as $d) { $out .= '\\' . $d; }
                continue;
            }
            $out .= $c;
        }
        return date($out, $this->__ts);
    }
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
    public static function createFromFormat($format, $datetime, $timezone = null) {
        $r = __date_from_format($format, $datetime);
        if ($r === false) { return false; }
        $d = new DateTime("@" . $r[0]);
        // "@ts" leaves the "+00:00" offset tz; the real createFromFormat keeps
        // the parsed offset, the $timezone argument, or the default (UTC).
        $d->__tz = $r[1] !== null ? $r[1] : ($timezone !== null ? $timezone->getName() : 'UTC');
        $d->__us = $r[2];
        return $d;
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
    private $__us = 0;
    private $__tz = "UTC";
    public function __construct($datetime = "now", $timezone = null) {
        if ($timezone !== null) { $this->__tz = $timezone->getName(); }
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $t = microtime(true);
            $this->__ts = (int) $t;
            $this->__us = (int) round(($t - (int) $t) * 1000000);
        } else {
            // A leading '@' (unix timestamp) forces the UTC-offset zone "+00:00".
            if (is_string($datetime) && isset($datetime[0]) && $datetime[0] === "@") {
                $this->__tz = "+00:00";
            }
            $parse = $datetime;
            if (is_string($datetime) && preg_match('/\.(\d{1,6})/', $datetime, $m) === 1) {
                $this->__us = (int) str_pad($m[1], 6, '0');
                $parse = preg_replace('/\.\d{1,6}/', '', $datetime, 1);
            }
            $r = strtotime($parse);
            if ($r === false) {
                throw new Exception("DateTimeImmutable::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r;
        }
    }
    public function getTimezone() { return new DateTimeZone($this->__tz); }
    public function setTimezone($timezone) {
        // `clone` keeps the runtime class, so a userland subclass (monolog's
        // JsonSerializableDateTimeImmutable) survives, like PHP's `static`.
        $c = clone $this;
        $c->__tz = is_string($timezone) ? $timezone : $timezone->getName();
        return $c;
    }
    public static function createFromInterface($object) {
        $d = new DateTimeImmutable("@" . $object->getTimestamp());
        return $d->setTimezone($object->getTimezone());
    }
    public static function createFromMutable($object) { return static::createFromInterface($object); }
    public function format($format) {
        // 'u'/'v' (micro/milliseconds) come from this instance, not date():
        // substitute the digits as backslash-escaped literals in the format.
        $out = ''; $esc = false;
        for ($i = 0, $len = strlen($format); $i < $len; $i++) {
            $c = $format[$i];
            if ($esc) { $out .= '\\' . $c; $esc = false; continue; }
            if ($c === '\\') { $esc = true; continue; }
            if ($c === 'u' || $c === 'v') {
                $n = $c === 'u' ? sprintf('%06d', $this->__us) : sprintf('%03d', intdiv($this->__us, 1000));
                foreach (str_split($n) as $d) { $out .= '\\' . $d; }
                continue;
            }
            $out .= $c;
        }
        return date($out, $this->__ts);
    }
    public function getTimestamp() { return $this->__ts; }
    // Every "wither" clones: the runtime class survives (PHP returns `static`,
    // monolog's JsonSerializableDateTimeImmutable relies on it), and so do
    // the timezone label and (where PHP keeps them) the microseconds.
    public function setTimestamp($timestamp) {
        $c = clone $this; $c->__ts = $timestamp; $c->__us = 0; return $c;
    }
    public function setDate($year, $month, $day) {
        $c = clone $this;
        $c->__ts = mktime((int)date('G', $this->__ts), (int)date('i', $this->__ts), (int)date('s', $this->__ts), $month, $day, $year);
        return $c;
    }
    public function setTime($hour, $minute, $second = 0, $microsecond = 0) {
        $c = clone $this;
        $c->__ts = mktime($hour, $minute, $second, (int)date('n', $this->__ts), (int)date('j', $this->__ts), (int)date('Y', $this->__ts));
        $c->__us = $microsecond;
        return $c;
    }
    public static function createFromFormat($format, $datetime, $timezone = null) {
        $r = __date_from_format($format, $datetime);
        if ($r === false) { return false; }
        $d = new DateTimeImmutable("@" . $r[0]);
        $d->__tz = $r[1] !== null ? $r[1] : ($timezone !== null ? $timezone->getName() : 'UTC');
        $d->__us = $r[2];
        return $d;
    }
    public function modify($modifier) {
        $r = strtotime($modifier, $this->__ts);
        if ($r === false) { return false; }
        $c = clone $this; $c->__ts = $r; return $c;
    }
    public function add($interval) { $c = clone $this; $c->__ts = $this->__apply($interval, 1); return $c; }
    public function sub($interval) { $c = clone $this; $c->__ts = $this->__apply($interval, -1); return $c; }
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
// ZipArchive (ext/zip subset backing Composer's dist downloads): the prelude
// class delegates to the __zip_* host builtins (zip crate VM-side); the handle
// is an int id in Vm.zips. Read-only surface: open/close/count/statIndex/
// getNameIndex/locateName/getFromIndex/getFromName/extractTo. No writing.
class ZipArchive implements Countable {
    const CREATE = 1; const EXCL = 2; const CHECKCONS = 4; const OVERWRITE = 8; const RDONLY = 16;
    const FL_NOCASE = 1; const FL_NODIR = 2;
    const CM_DEFAULT = -1; const CM_STORE = 0; const CM_DEFLATE = 8;
    const EM_NONE = 0;
    const ER_OK = 0; const ER_MULTIDISK = 1; const ER_RENAME = 2; const ER_CLOSE = 3;
    const ER_SEEK = 4; const ER_READ = 5; const ER_WRITE = 6; const ER_CRC = 7;
    const ER_ZIPCLOSED = 8; const ER_NOENT = 9; const ER_EXISTS = 10; const ER_OPEN = 11;
    const ER_TMPOPEN = 12; const ER_ZLIB = 13; const ER_MEMORY = 14; const ER_CHANGED = 15;
    const ER_COMPNOTSUPP = 16; const ER_EOF = 17; const ER_INVAL = 18; const ER_NOZIP = 19;
    const ER_INTERNAL = 20; const ER_INCONS = 21; const ER_REMOVE = 22; const ER_DELETED = 23;
    public $numFiles = 0;
    public $status = 0;
    public $statusSys = 0;
    public $filename = '';
    public $comment = '';
    private $__h = null;
    public function open($filename, $flags = 0) {
        $r = __zip_open($filename);
        if (is_int($r)) { $this->status = $r; return $r; }
        $this->__h = $r[0];
        $this->numFiles = $r[1];
        $this->filename = $filename;
        $this->status = 0;
        return true;
    }
    public function close() {
        if ($this->__h === null) { return false; }
        $r = __zip_close($this->__h);
        $this->__h = null; $this->numFiles = 0; $this->filename = '';
        return $r;
    }
    public function count(): int { return $this->numFiles; }
    public function statIndex($index, $flags = 0) { return $this->__h === null ? false : __zip_stat_index($this->__h, $index); }
    public function getNameIndex($index, $flags = 0) { return $this->__h === null ? false : __zip_get_name_index($this->__h, $index); }
    public function locateName($name, $flags = 0) { return $this->__h === null ? false : __zip_locate_name($this->__h, $name); }
    public function getFromIndex($index, $len = 0, $flags = 0) { return $this->__h === null ? false : __zip_get_from_index($this->__h, $index); }
    public function getFromName($name, $len = 0, $flags = 0) {
        if ($this->__h === null) { return false; }
        $i = __zip_locate_name($this->__h, $name);
        return $i === false ? false : __zip_get_from_index($this->__h, $i);
    }
    public function extractTo($pathto, $files = null) { return $this->__h === null ? false : __zip_extract_to($this->__h, $pathto); }
    public function getStatusString() { return $this->status === 0 ? 'No error' : 'Unknown error ' . $this->status; }
}
// ext/pdo + pdo_sqlite (sqlite driver only, backing doctrine/dbal): the prelude
// classes delegate to the __pdo_* host builtins (rusqlite VM-side, vm/pdo.rs);
// the handle is an int id in Vm.pdo_conns. A failing host op returns the array
// [message, code, sqlstate|null, native-msg|null] and the PHP side raises the
// PDOException. Constants are the oracle's full PDO constant table (8.5.7).
// No custom ctor, like the C class: internal raises pass SQLSTATE-string codes
// through the (prelude-untyped) Exception ctor and set the public errorInfo
// from the raising side.
class PDOException extends RuntimeException {
    public $errorInfo = null;
}
class PDO {
    const PARAM_NULL = 0;
    const PARAM_BOOL = 5;
    const PARAM_INT = 1;
    const PARAM_STR = 2;
    const PARAM_LOB = 3;
    const PARAM_STMT = 4;
    const PARAM_INPUT_OUTPUT = 2147483648;
    const PARAM_STR_NATL = 1073741824;
    const PARAM_STR_CHAR = 536870912;
    const PARAM_EVT_ALLOC = 0;
    const PARAM_EVT_FREE = 1;
    const PARAM_EVT_EXEC_PRE = 2;
    const PARAM_EVT_EXEC_POST = 3;
    const PARAM_EVT_FETCH_PRE = 4;
    const PARAM_EVT_FETCH_POST = 5;
    const PARAM_EVT_NORMALIZE = 6;
    const FETCH_DEFAULT = 0;
    const FETCH_LAZY = 1;
    const FETCH_ASSOC = 2;
    const FETCH_NUM = 3;
    const FETCH_BOTH = 4;
    const FETCH_OBJ = 5;
    const FETCH_BOUND = 6;
    const FETCH_COLUMN = 7;
    const FETCH_CLASS = 8;
    const FETCH_INTO = 9;
    const FETCH_FUNC = 10;
    const FETCH_GROUP = 32;
    const FETCH_UNIQUE = 64;
    const FETCH_KEY_PAIR = 12;
    const FETCH_CLASSTYPE = 128;
    const FETCH_SERIALIZE = 512;
    const FETCH_PROPS_LATE = 256;
    const FETCH_NAMED = 11;
    const ATTR_AUTOCOMMIT = 0;
    const ATTR_PREFETCH = 1;
    const ATTR_TIMEOUT = 2;
    const ATTR_ERRMODE = 3;
    const ATTR_SERVER_VERSION = 4;
    const ATTR_CLIENT_VERSION = 5;
    const ATTR_SERVER_INFO = 6;
    const ATTR_CONNECTION_STATUS = 7;
    const ATTR_CASE = 8;
    const ATTR_CURSOR_NAME = 9;
    const ATTR_CURSOR = 10;
    const ATTR_ORACLE_NULLS = 11;
    const ATTR_PERSISTENT = 12;
    const ATTR_STATEMENT_CLASS = 13;
    const ATTR_FETCH_TABLE_NAMES = 14;
    const ATTR_FETCH_CATALOG_NAMES = 15;
    const ATTR_DRIVER_NAME = 16;
    const ATTR_STRINGIFY_FETCHES = 17;
    const ATTR_MAX_COLUMN_LEN = 18;
    const ATTR_EMULATE_PREPARES = 20;
    const ATTR_DEFAULT_FETCH_MODE = 19;
    const ATTR_DEFAULT_STR_PARAM = 21;
    const ERRMODE_SILENT = 0;
    const ERRMODE_WARNING = 1;
    const ERRMODE_EXCEPTION = 2;
    const CASE_NATURAL = 0;
    const CASE_LOWER = 2;
    const CASE_UPPER = 1;
    const NULL_NATURAL = 0;
    const NULL_EMPTY_STRING = 1;
    const NULL_TO_STRING = 2;
    const ERR_NONE = '00000';
    const FETCH_ORI_NEXT = 0;
    const FETCH_ORI_PRIOR = 1;
    const FETCH_ORI_FIRST = 2;
    const FETCH_ORI_LAST = 3;
    const FETCH_ORI_ABS = 4;
    const FETCH_ORI_REL = 5;
    const CURSOR_FWDONLY = 0;
    const CURSOR_SCROLL = 1;
    const DBLIB_ATTR_CONNECTION_TIMEOUT = 1000;
    const DBLIB_ATTR_QUERY_TIMEOUT = 1001;
    const DBLIB_ATTR_STRINGIFY_UNIQUEIDENTIFIER = 1002;
    const DBLIB_ATTR_VERSION = 1003;
    const DBLIB_ATTR_TDS_VERSION = 1004;
    const DBLIB_ATTR_SKIP_EMPTY_ROWSETS = 1005;
    const DBLIB_ATTR_DATETIME_CONVERT = 1006;
    const MYSQL_ATTR_USE_BUFFERED_QUERY = 1000;
    const MYSQL_ATTR_LOCAL_INFILE = 1001;
    const MYSQL_ATTR_INIT_COMMAND = 1002;
    const MYSQL_ATTR_COMPRESS = 1003;
    const MYSQL_ATTR_DIRECT_QUERY = 20;
    const MYSQL_ATTR_FOUND_ROWS = 1004;
    const MYSQL_ATTR_IGNORE_SPACE = 1005;
    const MYSQL_ATTR_SSL_KEY = 1006;
    const MYSQL_ATTR_SSL_CERT = 1007;
    const MYSQL_ATTR_SSL_CA = 1008;
    const MYSQL_ATTR_SSL_CAPATH = 1009;
    const MYSQL_ATTR_SSL_CIPHER = 1010;
    const MYSQL_ATTR_SERVER_PUBLIC_KEY = 1011;
    const MYSQL_ATTR_MULTI_STATEMENTS = 1012;
    const MYSQL_ATTR_SSL_VERIFY_SERVER_CERT = 1013;
    const MYSQL_ATTR_LOCAL_INFILE_DIRECTORY = 1014;
    const ODBC_ATTR_USE_CURSOR_LIBRARY = 1000;
    const ODBC_ATTR_ASSUME_UTF8 = 1001;
    const ODBC_SQL_USE_IF_NEEDED = 0;
    const ODBC_SQL_USE_DRIVER = 2;
    const ODBC_SQL_USE_ODBC = 1;
    const PGSQL_ATTR_DISABLE_PREPARES = 1000;
    const PGSQL_TRANSACTION_IDLE = 0;
    const PGSQL_TRANSACTION_ACTIVE = 1;
    const PGSQL_TRANSACTION_INTRANS = 2;
    const PGSQL_TRANSACTION_INERROR = 3;
    const PGSQL_TRANSACTION_UNKNOWN = 4;
    const SQLITE_DETERMINISTIC = 2048;
    const SQLITE_ATTR_OPEN_FLAGS = 1000;
    const SQLITE_OPEN_READONLY = 1;
    const SQLITE_OPEN_READWRITE = 2;
    const SQLITE_OPEN_CREATE = 4;
    const SQLITE_ATTR_READONLY_STATEMENT = 1001;
    const SQLITE_ATTR_EXTENDED_RESULT_CODES = 1002;
    private $__h = null;
    private $__attrs = array();
    private $__err = array('00000', null, null);
    // The driver-level einfo (native code, native msg): unlike __err it is NOT
    // reset by a later success -- a fresh statement's errorInfo leaks it
    // (oracle: ['', 1, 'near "BROKEN": syntax error'] after a past failure).
    private $__driverErr = array(null, null);
    public function __construct($dsn, $username = null, $password = null, $options = null) {
        $r = __pdo_open((string)$dsn);
        if (is_array($r)) {
            $e = new PDOException($r[0], $r[1]);
            if ($r[2] !== null) { $e->errorInfo = array($r[2], $r[1], $r[3]); }
            throw $e;
        }
        $this->__h = $r;
        $this->__attrs = array(PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION);
        if (is_array($options)) {
            foreach ($options as $k => $v) { $this->setAttribute($k, $v); }
        }
    }
    public function __destruct() {
        if ($this->__h !== null) { __pdo_close($this->__h); $this->__h = null; }
    }
    // PHP 8.4 static factory (doctrine/dbal's PDOConnect prefers it). The real
    // one returns the driver subclass (Pdo\Sqlite); phpr returns PDO itself,
    // an accepted slice until the subclass exists.
    public static function connect($dsn, $username = null, $password = null, $options = null) {
        return new static($dsn, $username, $password, $options);
    }
    // get/setAttribute follow pdo_dbh.c: the error state clears at method
    // ENTRY (a later errorCode() reads 00000 even after a failed call here);
    // an attribute outside the supported set raises SQLSTATE[IM001] on get
    // and plainly returns false on set.
    public function setAttribute($attribute, $value) {
        $this->__err = array('00000', null, null);
        if ($attribute === PDO::ATTR_ERRMODE || $attribute === PDO::ATTR_CASE || $attribute === PDO::ATTR_ORACLE_NULLS
            || $attribute === PDO::ATTR_STRINGIFY_FETCHES || $attribute === PDO::ATTR_DEFAULT_FETCH_MODE
            || $attribute === PDO::ATTR_DEFAULT_STR_PARAM || $attribute === PDO::ATTR_STATEMENT_CLASS
            || $attribute === PDO::ATTR_TIMEOUT || $attribute === PDO::SQLITE_ATTR_EXTENDED_RESULT_CODES) {
            $this->__attrs[$attribute] = $value;
            return true;
        }
        return false;
    }
    public function getAttribute($attribute) {
        $this->__err = array('00000', null, null);
        if ($attribute === PDO::ATTR_DRIVER_NAME) { return 'sqlite'; }
        if ($attribute === PDO::ATTR_SERVER_VERSION || $attribute === PDO::ATTR_CLIENT_VERSION) { return __pdo_sqlite_version(); }
        if ($attribute === PDO::ATTR_ERRMODE || $attribute === PDO::ATTR_CASE || $attribute === PDO::ATTR_ORACLE_NULLS
            || $attribute === PDO::ATTR_PERSISTENT || $attribute === PDO::ATTR_STRINGIFY_FETCHES
            || $attribute === PDO::ATTR_DEFAULT_FETCH_MODE || $attribute === PDO::ATTR_DEFAULT_STR_PARAM
            || $attribute === PDO::ATTR_STATEMENT_CLASS || $attribute === PDO::SQLITE_ATTR_EXTENDED_RESULT_CODES) {
            if (array_key_exists($attribute, $this->__attrs)) { return $this->__attrs[$attribute]; }
            if ($attribute === PDO::ATTR_CASE || $attribute === PDO::ATTR_ORACLE_NULLS || $attribute === PDO::SQLITE_ATTR_EXTENDED_RESULT_CODES) { return 0; }
            if ($attribute === PDO::ATTR_PERSISTENT || $attribute === PDO::ATTR_STRINGIFY_FETCHES) { return false; }
            if ($attribute === PDO::ATTR_DEFAULT_FETCH_MODE) { return PDO::FETCH_BOTH; }
            if ($attribute === PDO::ATTR_ERRMODE) { return PDO::ERRMODE_EXCEPTION; }
            return null;
        }
        return $this->__raise(array('SQLSTATE[IM001]: Driver does not support this function: driver does not support that attribute', 'IM001', null, null), 'PDO::getAttribute');
    }
    public static function getAvailableDrivers() { return array('sqlite'); }
    public function exec($statement) {
        $r = __pdo_exec($this->__h, (string)$statement);
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::exec'); }
        $this->__err = array('00000', null, null);
        return $r['changes'];
    }
    // pdo_sqlite implements in_transaction via sqlite3_get_autocommit (so a
    // manual exec('BEGIN') IS visible here); the state errors below throw
    // *unconditionally*, whatever ATTR_ERRMODE says (pdo_dbh.c).
    public function beginTransaction() {
        $this->__err = array('00000', null, null);
        if ($this->inTransaction()) { throw new PDOException('There is already an active transaction'); }
        $r = __pdo_exec($this->__h, 'BEGIN');
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::beginTransaction'); }
        $this->__err = array('00000', null, null);
        return true;
    }
    public function commit() {
        $this->__err = array('00000', null, null);
        if (!$this->inTransaction()) { throw new PDOException('There is no active transaction'); }
        $r = __pdo_exec($this->__h, 'COMMIT');
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::commit'); }
        $this->__err = array('00000', null, null);
        return true;
    }
    public function rollBack() {
        $this->__err = array('00000', null, null);
        if (!$this->inTransaction()) { throw new PDOException('There is no active transaction'); }
        $r = __pdo_exec($this->__h, 'ROLLBACK');
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::rollBack'); }
        $this->__err = array('00000', null, null);
        return true;
    }
    public function inTransaction() { return __pdo_in_txn($this->__h); }
    public function errorCode() { return $this->__err[0]; }
    public function errorInfo() { return array($this->__err[0], $this->__err[1], $this->__err[2]); }
    public function __driverError() { return $this->__driverErr; }
    public function quote($string, $type = PDO::PARAM_STR) {
        $s = (string)$string;
        if (strpos($s, "\0") !== false) { return false; }
        return "'" . str_replace("'", "''", $s) . "'";
    }
    public function prepare($query, $options = null) {
        // pdo_sqlite prepares eagerly: broken SQL fails here, not at execute.
        $r = __pdo_prepare($this->__h, (string)$query);
        if (is_array($r) && isset($r['err'])) { return $this->__raise($r['err'], 'PDO::prepare'); }
        $this->__err = array('00000', null, null);
        $st = new PDOStatement();
        $st->__pdoInit($this, $this->__h, (string)$query);
        return $st;
    }
    public function query($query, $mode = null, ...$args) {
        $st = $this->prepare($query);
        if ($st === false) { return false; }
        if ($mode !== null) { $st->setFetchMode($mode, ...$args); }
        if ($st->execute() === false) { return false; }
        return $st;
    }
    public function lastInsertId($name = null) {
        return (string)__pdo_last_id($this->__h);
    }
    // The shared error sink: record errorInfo and act per ATTR_ERRMODE.
    // Payload = [full message, sqlstate, native code|null, native msg|null];
    // runtime PDOException codes are the SQLSTATE *string* (connection-time
    // ctor failures use the native int instead, see __construct).
    public function __raise($e, $fn) {
        $this->__err = array($e[1], $e[2], $e[3]);
        if ($e[2] !== null) { $this->__driverErr = array($e[2], $e[3]); }
        $mode = isset($this->__attrs[PDO::ATTR_ERRMODE]) ? $this->__attrs[PDO::ATTR_ERRMODE] : PDO::ERRMODE_EXCEPTION;
        if ($mode === PDO::ERRMODE_EXCEPTION) {
            $ex = new PDOException($e[0], $e[1]);
            $ex->errorInfo = array($e[1], $e[2], $e[3]);
            throw $ex;
        }
        if ($mode === PDO::ERRMODE_WARNING) { trigger_error($fn . '(): ' . $e[0], E_USER_WARNING); }
        return false;
    }
}
// The statement: prepared-SQL + bound-params holder; execute() ships both to
// __pdo_run (the host re-prepares each time: sqlite has no server state to
// lose) and materializes the whole rowset VM-side, fetch* then walk it.
class PDOStatement implements IteratorAggregate {
    public $queryString = '';
    private $__pdo = null;
    private $__c = null;
    private $__cols = array();
    private $__rows = null;
    private $__pos = 0;
    private $__changes = 0;
    private $__bound = array();
    // null until the first execute-ish op: a fresh statement's errorCode() is
    // NULL while its errorInfo() shows '' plus the *connection's* last driver
    // einfo (oracle-verified pdo_stmt.c behaviour).
    private $__err = null;
    private $__mode = null;
    private $__modeArgs = array();
    private $__meta = array();
    private $__freed = false;
    public function __pdoInit($pdo, $h, $sql) {
        $this->__pdo = $pdo;
        $this->__c = $h;
        $this->queryString = $sql;
    }
    // bindValue/execute(array) coercions: execute(array) values are all
    // PARAM_STR (oracle: execute([1]) fetches back "1"); bindValue applies the
    // declared PARAM_* type. null always stays null; bools bind as sqlite ints.
    private function __coerce($v, $t) {
        if ($v === null) { return null; }
        $t = $t & ~PDO::PARAM_INPUT_OUTPUT;
        // A stream bound as PARAM_LOB sends its remaining contents.
        if ($t === PDO::PARAM_LOB && is_resource($v)) { return stream_get_contents($v); }
        if ($t === PDO::PARAM_INT) { return (int)$v; }
        if ($t === PDO::PARAM_BOOL) { return (bool)$v; }
        if ($t === PDO::PARAM_NULL) { return null; }
        if ($t === PDO::PARAM_STR || $t === PDO::PARAM_LOB) { return (string)$v; }
        return $v;
    }
    public function bindValue($param, $value, $type = PDO::PARAM_STR) {
        $this->__bound[$param] = array($this->__coerce($value, $type), $type, false, null);
        return true;
    }
    public function bindParam($param, &$var, $type = PDO::PARAM_STR, $maxLength = 0, $driverOptions = null) {
        $this->__bound[$param] = array(null, $type, true, null);
        $this->__bound[$param][3] =& $var;
        return true;
    }
    public function execute($params = null) {
        $send = array();
        $strict = false;
        if (is_array($params)) {
            $strict = true;
            foreach ($params as $k => $v) {
                // A 0-based execute(array) list feeds the 1-based placeholders.
                $key = is_int($k) ? $k + 1 : $k;
                $send[$key] = $v === null ? null : (string)$v;
            }
        } else {
            foreach ($this->__bound as $k => $b) {
                $send[$k] = $b[2] ? $this->__coerce($b[3], $b[1]) : $b[0];
            }
        }
        $r = __pdo_run($this->__c, $this->queryString, $send, $strict);
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDOStatement::execute'); }
        $this->__err = array('00000', null, null);
        $this->__cols = isset($r['cols']) ? $r['cols'] : array();
        $this->__rows = isset($r['rows']) ? $r['rows'] : array();
        $this->__meta = isset($r['meta']) ? $r['meta'] : array();
        $this->__pos = 0;
        $this->__freed = false;
        $this->__changes = isset($r['changes']) ? $r['changes'] : 0;
        return true;
    }
    // FETCH_CLASS instantiation: props are written BEFORE the constructor
    // runs, unless FETCH_PROPS_LATE flips the order (bug46139 semantics).
    private function __fetchClass($assoc, $class, $ctorArgs, $propsLate) {
        $rc = new ReflectionClass($class);
        $o = $rc->newInstanceWithoutConstructor();
        $ctor = $rc->getConstructor();
        if ($propsLate && $ctor !== null) { $o->__construct(...$ctorArgs); }
        foreach ($assoc as $k => $v) { $o->$k = $v; }
        if (!$propsLate && $ctor !== null) { $o->__construct(...$ctorArgs); }
        return $o;
    }
    private function __buildRow($row, $mode, $margs = null) {
        $pdo = $this->__pdo;
        if ($mode === null || $mode === PDO::FETCH_DEFAULT) { $mode = $this->__mode; }
        if ($mode === null || $mode === PDO::FETCH_DEFAULT) {
            $mode = $pdo !== null ? $pdo->getAttribute(PDO::ATTR_DEFAULT_FETCH_MODE) : PDO::FETCH_BOTH;
        }
        if ($margs === null) { $margs = $this->__modeArgs; }
        $flags = $mode & ~15;
        $mode = $mode & 15;
        $stringify = $pdo !== null && $pdo->getAttribute(PDO::ATTR_STRINGIFY_FETCHES);
        $case = $pdo !== null ? $pdo->getAttribute(PDO::ATTR_CASE) : PDO::CASE_NATURAL;
        $vals = array();
        foreach ($row as $i => $v) {
            if ($stringify && (is_int($v) || is_float($v))) { $v = (string)$v; }
            $vals[$i] = $v;
        }
        if ($mode === PDO::FETCH_NUM) { return $vals; }
        if ($mode === PDO::FETCH_COLUMN) {
            $col = isset($margs[0]) ? $margs[0] : 0;
            return array_key_exists($col, $vals) ? $vals[$col] : null;
        }
        $names = array();
        foreach ($this->__cols as $i => $n) {
            if ($case === PDO::CASE_LOWER) { $n = strtolower($n); }
            elseif ($case === PDO::CASE_UPPER) { $n = strtoupper($n); }
            $names[$i] = $n;
        }
        if ($mode === PDO::FETCH_CLASS) {
            $assoc = array();
            foreach ($vals as $i => $v) { $assoc[$names[$i]] = $v; }
            $class = isset($margs[0]) ? $margs[0] : 'stdClass';
            $ctorArgs = isset($margs[1]) && is_array($margs[1]) ? $margs[1] : array();
            return $this->__fetchClass($assoc, $class, $ctorArgs, ($flags & PDO::FETCH_PROPS_LATE) !== 0);
        }
        if ($mode === PDO::FETCH_INTO) {
            $obj = isset($margs[0]) ? $margs[0] : new stdClass();
            foreach ($vals as $i => $v) { $n = $names[$i]; $obj->$n = $v; }
            return $obj;
        }
        if ($mode === PDO::FETCH_KEY_PAIR) { return array($vals[0] => $vals[1]); }
        if ($mode === PDO::FETCH_NAMED) {
            $out = array();
            $dup = array();
            foreach ($vals as $i => $v) {
                $n = $names[$i];
                if (array_key_exists($n, $out)) {
                    if (!isset($dup[$n])) { $out[$n] = array($out[$n]); $dup[$n] = true; }
                    $out[$n][] = $v;
                } else { $out[$n] = $v; }
            }
            return $out;
        }
        $out = array();
        foreach ($vals as $i => $v) {
            if ($mode === PDO::FETCH_ASSOC || $mode === PDO::FETCH_OBJ || $mode === PDO::FETCH_BOTH) { $out[$names[$i]] = $v; }
            if ($mode === PDO::FETCH_BOTH || $mode === PDO::FETCH_NUM) { $out[$i] = $v; }
        }
        if ($mode === PDO::FETCH_OBJ) { return (object)$out; }
        return $out;
    }
    public function fetch($mode = null, $cursorOrientation = 0, $cursorOffset = 0) {
        if ($this->__rows === null || $this->__pos >= count($this->__rows)) { return false; }
        $row = $this->__rows[$this->__pos];
        $this->__pos = $this->__pos + 1;
        return $this->__buildRow($row, $mode);
    }
    public function fetchAll($mode = null, ...$args) {
        if ($this->__rows === null) { return array(); }
        $out = array();
        if ($mode === PDO::FETCH_COLUMN) {
            $col = isset($args[0]) ? $args[0] : (isset($this->__modeArgs[0]) ? $this->__modeArgs[0] : 0);
            while (($row = $this->fetch(PDO::FETCH_NUM)) !== false) { $out[] = array_key_exists($col, $row) ? $row[$col] : null; }
            return $out;
        }
        if ($mode === PDO::FETCH_KEY_PAIR) {
            while (($row = $this->fetch(PDO::FETCH_NUM)) !== false) { $out[$row[0]] = $row[1]; }
            return $out;
        }
        while ($this->__pos < count($this->__rows)) {
            $out[] = $this->__buildRow($this->__rows[$this->__pos], $mode, count($args) ? $args : null);
            $this->__pos = $this->__pos + 1;
        }
        return $out;
    }
    public function fetchColumn($column = 0) {
        $row = $this->fetch(PDO::FETCH_NUM);
        if ($row === false) { return false; }
        return array_key_exists($column, $row) ? $row[$column] : null;
    }
    public function fetchObject($class = 'stdClass', $constructorArgs = array()) {
        if (!is_string($class) || !class_exists($class)) {
            throw new TypeError('PDOStatement::fetchObject(): Argument #1 ($class) must be a valid class name, ' . $class . ' given');
        }
        $row = $this->fetch(PDO::FETCH_ASSOC);
        if ($row === false) { return false; }
        if ($class === 'stdClass') { return (object)$row; }
        $rc = new ReflectionClass($class);
        $o = $rc->newInstanceWithoutConstructor();
        foreach ($row as $k => $v) { $o->$k = $v; }
        if ($rc->getConstructor() !== null) { $o->__construct(...$constructorArgs); }
        return $o;
    }
    public function setFetchMode($mode, ...$args) {
        $this->__mode = $mode;
        $this->__modeArgs = $args;
        return true;
    }
    public function rowCount() { return $this->__changes; }
    public function columnCount() { return count($this->__cols); }
    // getColumnMeta statics come from the host ('meta': decl type + table,
    // absent on expression columns); native_type/pdo_type reflect the *value*
    // in the materialized first row, like sqlite3_column_type at execute.
    public function getColumnMeta($column) {
        if ($column < 0) { throw new ValueError('PDOStatement::getColumnMeta(): Argument #1 ($column) must be greater than or equal to 0'); }
        if ($this->__rows === null || $this->__freed) { return false; }
        if ($column >= count($this->__cols)) {
            // With rows still pending, pdo_sqlite surfaces the driver state
            // (SQLITE_ROW = 100); with the set exhausted it reports false
            // (PHP >= 8.3.18 behaviour, what DBAL's InvalidColumnIndex needs).
            if ($this->__pos < count($this->__rows)) {
                return $this->__raise(array('SQLSTATE[HY000]: General error: 100 another row available', 'HY000', 100, 'another row available'), 'PDOStatement::getColumnMeta');
            }
            return false;
        }
        $v = count($this->__rows) > 0 ? $this->__rows[0][$column] : null;
        if (is_int($v)) { $nt = 'integer'; $pt = PDO::PARAM_INT; }
        elseif (is_float($v)) { $nt = 'double'; $pt = PDO::PARAM_STR; }
        elseif ($v === null) { $nt = 'null'; $pt = PDO::PARAM_NULL; }
        else { $nt = 'string'; $pt = PDO::PARAM_STR; }
        $out = array('native_type' => $nt, 'pdo_type' => $pt);
        $m = isset($this->__meta[$column]) ? $this->__meta[$column] : array(null, null);
        if ($m[0] !== null) { $out['sqlite:decl_type'] = $m[0]; }
        if ($m[1] !== null) { $out['table'] = $m[1]; }
        $out['flags'] = array();
        $out['name'] = $this->__cols[$column];
        $out['len'] = -1;
        $out['precision'] = 0;
        return $out;
    }
    public function getAttribute($attribute) {
        if ($attribute === PDO::ATTR_EMULATE_PREPARES) { return false; }
        if ($attribute === PDO::SQLITE_ATTR_READONLY_STATEMENT) { return __pdo_stmt_readonly($this->__c, $this->queryString); }
        return $this->__raise(array('SQLSTATE[IM001]: Driver does not support this function: driver does not support that attribute', 'IM001', null, null), 'PDOStatement::getAttribute');
    }
    public function setAttribute($attribute, $value) { return false; }
    public function errorCode() { return $this->__err === null ? null : $this->__err[0]; }
    public function errorInfo() {
        if ($this->__err === null) {
            $d = $this->__pdo !== null ? $this->__pdo->__driverError() : array(null, null);
            return array('', $d[0], $d[1]);
        }
        return array($this->__err[0], $this->__err[1], $this->__err[2]);
    }
    public function closeCursor() {
        $this->__rows = array();
        $this->__pos = 0;
        $this->__freed = true;
        return true;
    }
    public function getIterator(): Iterator {
        $rows = array();
        while (($row = $this->fetch()) !== false) { $rows[] = $row; }
        return new ArrayIterator($rows);
    }
    // Mirrors PDO::__raise, reading the owner's ERRMODE.
    public function __raise($e, $fn) {
        $this->__err = array($e[1], $e[2], $e[3]);
        $mode = $this->__pdo !== null ? $this->__pdo->getAttribute(PDO::ATTR_ERRMODE) : PDO::ERRMODE_EXCEPTION;
        if ($mode === PDO::ERRMODE_EXCEPTION) {
            $ex = new PDOException($e[0], $e[1]);
            $ex->errorInfo = array($e[1], $e[2], $e[3]);
            throw $ex;
        }
        if ($mode === PDO::ERRMODE_WARNING) { trigger_error($fn . '(): ' . $e[0], E_USER_WARNING); }
        return false;
    }
}
final class PDORow {
    public $queryString = '';
    public function __construct() { throw new PDOException('You may not create a PDORow manually'); }
}
function pdo_drivers() { return PDO::getAvailableDrivers(); }
// ext/sqlite3 sul medesimo backing rusqlite dei __pdo_* (stessa registry di
// connessioni): SQLite3Stmt tiene SQL+parametri e ri-prepara a ogni execute,
// SQLite3Result e' il rowset materializzato. Quirk oracle-verificati: exec
// ritorna true (non changes); fetchArray oltre la fine ritorna false E
// RESETTA il cursore (sqlite3_step dopo DONE auto-resetta); BOTH e' in ordine
// NUM-poi-nome per colonna (inverso di PDO); columnType riflette la riga
// APPENA fetchata (false prima del primo fetch e dopo il false di fine);
// bindValue senza tipo inferisce dal tipo PHP (bool->int); il ctor lancia
// \Exception anche con exceptions OFF, gli errori runtime SQLite3Exception
// solo con enableExceptions(true), altrimenti warning + false.
class SQLite3Exception extends Exception {}
class SQLite3 {
    private $__h = null;
    private $__throw = false;
    private $__err = array(0, '');
    public function __construct($filename = '', $flags = 6, $encryptionKey = '') {
        $this->open($filename, $flags, $encryptionKey);
    }
    public function open($filename, $flags = 6, $encryptionKey = '') {
        $r = __pdo_open('sqlite:' . $filename);
        if (is_array($r)) {
            throw new Exception('Unable to open database: ' . ($r[3] !== null ? $r[3] : $r[0]));
        }
        $this->__h = $r;
    }
    public function enableExceptions($enable = false) {
        $old = $this->__throw;
        $this->__throw = (bool)$enable;
        return $old;
    }
    // Error sink: record lastErrorCode/Msg, throw or warn per enableExceptions.
    public function __fail($code, $msg, $full, $fn) {
        $this->__err = array($code, $msg);
        if ($this->__throw) { throw new SQLite3Exception($full, $code); }
        trigger_error($fn . '(): ' . $full, E_USER_WARNING);
        return false;
    }
    public function exec($query) {
        $r = __pdo_exec($this->__h, (string)$query);
        if (isset($r['err'])) { $e = $r['err']; return $this->__fail($e[2], $e[3], $e[3], 'SQLite3::exec'); }
        return true;
    }
    public function query($query) {
        $r = __pdo_run($this->__h, (string)$query, array(), false);
        if (isset($r['err'])) { $e = $r['err']; return $this->__fail($e[2], $e[3], $e[3], 'SQLite3::query'); }
        $res = new SQLite3Result();
        $res->__init(isset($r['cols']) ? $r['cols'] : array(), isset($r['rows']) ? $r['rows'] : array());
        return $res;
    }
    public function querySingle($query, $entireRow = false) {
        $res = $this->query($query);
        if ($res === false) { return false; }
        $row = $res->fetchArray($entireRow ? SQLITE3_ASSOC : SQLITE3_NUM);
        if ($row === false) { return $entireRow ? array() : null; }
        return $entireRow ? $row : $row[0];
    }
    public function prepare($query) {
        $r = __pdo_prepare($this->__h, (string)$query);
        if (is_array($r) && isset($r['err'])) {
            $e = $r['err'];
            return $this->__fail($e[2], $e[3], 'Unable to prepare statement: ' . $e[3], 'SQLite3::prepare');
        }
        $st = new SQLite3Stmt();
        $st->__init($this, $this->__h, (string)$query);
        return $st;
    }
    public function changes() { return __pdo_changes($this->__h); }
    public function lastInsertRowID() { return __pdo_last_id($this->__h); }
    public function lastErrorCode() { return $this->__err[0]; }
    public function lastErrorMsg() { return $this->__err[1]; }
    public function busyTimeout($milliseconds) { return true; }
    public function close() {
        if ($this->__h !== null) { __pdo_close($this->__h); $this->__h = null; }
        return true;
    }
    public function __destruct() {
        if ($this->__h !== null) { __pdo_close($this->__h); $this->__h = null; }
    }
    public static function escapeString($string) { return str_replace("'", "''", (string)$string); }
    public static function version() {
        $s = __pdo_sqlite_version();
        $p = explode('.', $s);
        $n = (int)$p[0] * 1000000 + (isset($p[1]) ? (int)$p[1] * 1000 : 0) + (isset($p[2]) ? (int)$p[2] : 0);
        return array('versionString' => $s, 'versionNumber' => $n);
    }
}
class SQLite3Stmt {
    private $__db = null;
    private $__c = null;
    private $__sql = '';
    private $__bound = array();
    public function __init($db, $h, $sql) {
        $this->__db = $db;
        $this->__c = $h;
        $this->__sql = $sql;
    }
    private function __coerce($v, $t) {
        if ($t === null) {
            // Senza tipo dichiarato binda per tipo PHP (bool -> int).
            if (is_bool($v)) { return (int)$v; }
            return $v;
        }
        if ($v === null || $t === SQLITE3_NULL) { return null; }
        if ($t === SQLITE3_INTEGER) { return (int)$v; }
        if ($t === SQLITE3_FLOAT) { return (float)$v; }
        return (string)$v; // TEXT/BLOB
    }
    public function bindValue($param, $value, $type = null) {
        $this->__bound[$param] = array($this->__coerce($value, $type));
        return true;
    }
    public function bindParam($param, &$var, $type = null) {
        $this->__bound[$param] = array(null, $type, true, null);
        $this->__bound[$param][3] =& $var;
        return true;
    }
    public function execute() {
        $send = array();
        foreach ($this->__bound as $k => $b) {
            $send[$k] = isset($b[2]) && $b[2] ? $this->__coerce($b[3], $b[1]) : $b[0];
        }
        $r = __pdo_run($this->__c, $this->__sql, $send, false);
        if (isset($r['err'])) {
            $e = $r['err'];
            return $this->__db->__fail($e[2], $e[3], 'Unable to execute statement: ' . $e[3], 'SQLite3Stmt::execute');
        }
        $res = new SQLite3Result();
        $res->__init(isset($r['cols']) ? $r['cols'] : array(), isset($r['rows']) ? $r['rows'] : array());
        return $res;
    }
    public function paramCount() { return __pdo_param_count($this->__c, $this->__sql); }
    public function readOnly() { return __pdo_stmt_readonly($this->__c, $this->__sql); }
    public function getSQL($expand = false) { return $this->__sql; }
    public function reset() { return true; }
    public function clear() { $this->__bound = array(); return true; }
    public function close() { return true; }
}
class SQLite3Result {
    private $__cols = array();
    private $__rows = array();
    private $__pos = 0;
    public function __init($cols, $rows) {
        $this->__cols = $cols;
        $this->__rows = $rows;
        $this->__pos = 0;
    }
    public function fetchArray($mode = SQLITE3_BOTH) {
        if ($this->__pos >= count($this->__rows)) {
            // sqlite3_step dopo DONE auto-resetta: il fetch successivo riparte.
            $this->__pos = 0;
            return false;
        }
        $row = $this->__rows[$this->__pos];
        $this->__pos = $this->__pos + 1;
        $out = array();
        foreach ($row as $i => $v) {
            if (($mode & SQLITE3_NUM) === SQLITE3_NUM) { $out[$i] = $v; }
            if (($mode & SQLITE3_ASSOC) === SQLITE3_ASSOC) { $out[$this->__cols[$i]] = $v; }
        }
        return $out;
    }
    public function numColumns() { return count($this->__cols); }
    public function columnName($column) {
        return isset($this->__cols[$column]) ? $this->__cols[$column] : false;
    }
    public function columnType($column) {
        // Tipo del valore nella riga APPENA fetchata; false prima del primo
        // fetch e dopo il false di fine (pos resettata a 0).
        if ($this->__pos < 1 || !isset($this->__cols[$column])) { return false; }
        $v = $this->__rows[$this->__pos - 1][$column];
        if (is_int($v)) { return SQLITE3_INTEGER; }
        if (is_float($v)) { return SQLITE3_FLOAT; }
        if ($v === null) { return SQLITE3_NULL; }
        return SQLITE3_TEXT;
    }
    public function finalize() { return true; }
    public function reset() { $this->__pos = 0; return true; }
}
// SplFileInfo + the directory-iterator family (Composer's Filesystem: rm -rf
// via CHILD_FIRST, copy-tree via SELF_FIRST + getSubPathname). The recursive
// iterator SNAPSHOTS the traversal at rewind() -- Composer's uses (delete after
// yield, copy) are order-compatible; live-mutation semantics, CATCH_GET_CHILD
// and the flag combinations beyond SKIP_DOTS/CURRENT_AS_PATHNAME are residues.
class SplFileInfo {
    protected $__path;
    public function __construct($path) { $this->__path = $path; }
    public function getPathname() { return $this->__path; }
    public function isDir() { return is_dir($this->__path); }
    public function isFile() { return is_file($this->__path); }
    public function isLink() { return is_link($this->__path); }
    public function getFilename() { return basename($this->__path); }
    public function getBasename($suffix = '') { return basename($this->__path, $suffix); }
    public function getPath() { return dirname($this->__path); }
    public function getRealPath() { return realpath($this->__path); }
    public function getSize() { return filesize($this->__path); }
    public function getPerms() { return fileperms($this->__path); }
    public function getMTime() { return filemtime($this->__path); }
    public function isReadable() { return is_readable($this->__path); }
    public function isWritable() { return is_writable($this->__path); }
    public function getExtension() {
        $f = basename($this->__path);
        $p = strrpos($f, '.');
        return $p === false || $p === 0 ? '' : substr($f, $p + 1);
    }
    public function __toString() { return $this->__path; }
}
class FilesystemIterator extends SplFileInfo {
    const CURRENT_AS_FILEINFO = 0; const CURRENT_AS_PATHNAME = 32; const CURRENT_AS_SELF = 16;
    const KEY_AS_PATHNAME = 0; const KEY_AS_FILENAME = 256;
    const FOLLOW_SYMLINKS = 512; const NEW_CURRENT_AND_KEY = 256;
    const SKIP_DOTS = 4096; const UNIX_PATHS = 8192;
}
// DirectoryIterator (flat, dots included): the native iterator is the current
// entry, like RecursiveDirectoryIterator below (Symfony Console's completion
// command scans /etc/bash_completion.d with it).
class DirectoryIterator extends SplFileInfo implements SeekableIterator {
    private $__dir;
    private $__names = [];
    private $__pos = 0;
    public function __construct($directory) {
        parent::__construct($directory);
        if (!is_dir($directory)) {
            throw new UnexpectedValueException("DirectoryIterator::__construct($directory): Failed to open directory: No such file or directory");
        }
        $this->__dir = rtrim($directory, '/');
        $this->__names = scandir($directory);
        $this->__sync();
    }
    private function __cur() { return $this->__dir . '/' . $this->__names[$this->__pos]; }
    private function __sync() {
        if ($this->__pos < count($this->__names)) { $this->__path = $this->__cur(); }
    }
    public function rewind(): void { $this->__pos = 0; $this->__sync(); }
    public function valid(): bool { return $this->__pos < count($this->__names); }
    public function next(): void { $this->__pos++; $this->__sync(); }
    public function seek($offset): void { $this->__pos = $offset; $this->__sync(); }
    public function key(): mixed { return $this->__pos; }
    public function current(): mixed { return $this; }
    public function isDot(): bool {
        $n = $this->__names[$this->__pos] ?? '';
        return $n === '.' || $n === '..';
    }
}
class RecursiveDirectoryIterator extends FilesystemIterator implements RecursiveIterator {
    private $__dir;
    private $__flags;
    private $__names = [];
    private $__pos = 0;
    private $__sub = ''; // path of this level relative to the traversal root
    public function __construct($directory, $flags = 0) {
        parent::__construct($directory);
        if (!is_dir($directory)) {
            throw new UnexpectedValueException("RecursiveDirectoryIterator::__construct($directory): Failed to open directory: No such file or directory");
        }
        $this->__dir = rtrim($directory, '/');
        $this->__flags = $flags;
        $names = scandir($directory);
        if (($flags & self::SKIP_DOTS) === self::SKIP_DOTS) {
            $keep = [];
            foreach ($names as $n) { if ($n !== '.' && $n !== '..') { $keep[] = $n; } }
            $names = $keep;
        }
        $this->__names = $names;
        $this->__sync();
    }
    private function __cur() { return $this->__dir . '/' . $this->__names[$this->__pos]; }
    // The native iterator *is* the current entry: every inherited SplFileInfo
    // accessor (getFilename/getPathname/isDir/...) reflects the position; Symfony
    // Finder's RecursiveDirectoryIterator::current() is built on exactly that.
    private function __sync() {
        if ($this->__pos < count($this->__names)) { $this->__path = $this->__cur(); }
    }
    public function rewind(): void { $this->__pos = 0; $this->__sync(); }
    public function valid(): bool { return $this->__pos < count($this->__names); }
    public function next(): void { $this->__pos++; $this->__sync(); }
    public function key(): mixed { return $this->__cur(); }
    public function current(): mixed {
        if (($this->__flags & self::CURRENT_AS_PATHNAME) === self::CURRENT_AS_PATHNAME) { return $this->__cur(); }
        return new SplFileInfo($this->__cur());
    }
    public function hasChildren($allowLinks = false) {
        $p = $this->__cur();
        return is_dir($p) && ($allowLinks || ($this->__flags & self::FOLLOW_SYMLINKS) === self::FOLLOW_SYMLINKS || !is_link($p));
    }
    public function getChildren() {
        // `new static`: a subclass (Symfony Finder's iterator) gets subclass
        // children, exactly like the native late-static getChildren.
        $child = new static($this->__cur(), $this->__flags);
        $name = $this->__names[$this->__pos];
        $child->__sub = $this->__sub === '' ? $name : $this->__sub . '/' . $name;
        return $child;
    }
    public function getSubPath() { return $this->__sub; }
    public function getSubPathname() {
        $name = $this->__names[$this->__pos];
        return $this->__sub === '' ? $name : $this->__sub . '/' . $name;
    }
}
class RecursiveIteratorIterator implements OuterIterator {
    const LEAVES_ONLY = 0; const SELF_FIRST = 1; const CHILD_FIRST = 2; const CATCH_GET_CHILD = 16;
    private $__it;
    private $__mode;
    private $__list = [];  // [[key, subpathname, current, depth], ...] in emit order
    private $__pos = 0;
    private $__maxDepth = -1; // -1 = unlimited (setMaxDepth(-1), the default)
    public function __construct($iterator, $mode = 0, $flags = 0) {
        $this->__it = $iterator;
        $this->__mode = $mode;
    }
    public function setMaxDepth($maxDepth = -1) {
        if ($maxDepth < -1) { throw new InvalidArgumentException('Parameter maxDepth must be >= -1'); }
        $this->__maxDepth = $maxDepth;
    }
    public function getMaxDepth() { return $this->__maxDepth === -1 ? false : $this->__maxDepth; }
    private function __collect($it, $depth) {
        for ($it->rewind(); $it->valid(); $it->next()) {
            $descend = method_exists($it, 'hasChildren') && $it->hasChildren()
                && ($this->__maxDepth === -1 || $depth < $this->__maxDepth);
            $entry = [$it->key(), method_exists($it, 'getSubPathname') ? $it->getSubPathname() : $it->key(), $it->current(), $depth];
            if ($descend) {
                if ($this->__mode === self::SELF_FIRST) { $this->__list[] = $entry; }
                $this->__collect($it->getChildren(), $depth + 1);
                if ($this->__mode === self::CHILD_FIRST) { $this->__list[] = $entry; }
            } else {
                $this->__list[] = $entry;
            }
        }
    }
    public function rewind(): void {
        $this->__list = [];
        $this->__pos = 0;
        $this->__collect($this->__it, 0);
    }
    public function valid(): bool { return $this->__pos < count($this->__list); }
    public function next(): void { $this->__pos++; }
    public function key(): mixed { return $this->__list[$this->__pos][0]; }
    public function current(): mixed { return $this->__list[$this->__pos][2]; }
    public function getSubPathname() { return $this->__list[$this->__pos][1]; }
    public function getDepth() { return $this->__list[$this->__pos][3]; }
    public function getInnerIterator() { return $this->__it; }
}
// SplDoublyLinkedList family (Composer's dependency solver: SplQueue work
// queues, RuleWatchChain extends the list and removes mid-iteration). Backed by
// a plain array kept dense via array_splice; the iteration cursor is a plain
// index that runs bottom-up (FIFO) or top-down (LIFO). Offsets index from the
// iteration head: $stack[0] is the most recently pushed element (oracle).
class SplDoublyLinkedList implements Iterator, Countable, ArrayAccess {
    const IT_MODE_LIFO = 2;
    const IT_MODE_FIFO = 0;
    const IT_MODE_DELETE = 1;
    const IT_MODE_KEEP = 0;
    protected $__items = [];
    protected $__pos = 0;
    protected $__mode = 0; // FIFO|KEEP; SplStack flips to LIFO
    private function __lifo() { return ($this->__mode & self::IT_MODE_LIFO) === self::IT_MODE_LIFO; }
    private function __real($index, $method) {
        $n = count($this->__items);
        $i = (int)$index;
        $r = $this->__lifo() ? $n - 1 - $i : $i;
        if ($i < 0 || $r < 0 || $r >= $n) {
            throw new OutOfRangeException("SplDoublyLinkedList::$method(): Argument #1 (\$index) is out of range");
        }
        return $r;
    }
    public function push($value) { $this->__items[] = $value; }
    public function pop() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't pop from an empty datastructure"); }
        return array_pop($this->__items);
    }
    public function shift() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't shift from an empty datastructure"); }
        return array_shift($this->__items);
    }
    public function unshift($value) { array_unshift($this->__items, $value); }
    public function top() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't peek at an empty datastructure"); }
        return $this->__items[count($this->__items) - 1];
    }
    public function bottom() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't peek at an empty datastructure"); }
        return $this->__items[0];
    }
    public function isEmpty() { return count($this->__items) === 0; }
    public function count(): int { return count($this->__items); }
    public function setIteratorMode($mode) { $this->__mode = (int)$mode; }
    public function getIteratorMode() { return $this->__mode; }
    public function toArray() { return $this->__items; }
    public function rewind(): void { $this->__pos = $this->__lifo() ? count($this->__items) - 1 : 0; }
    public function valid(): bool { return $this->__pos >= 0 && $this->__pos < count($this->__items); }
    public function current(): mixed { return $this->valid() ? $this->__items[$this->__pos] : null; }
    public function key(): mixed { return $this->__pos; }
    public function next(): void {
        if ($this->__lifo()) {
            if (($this->__mode & self::IT_MODE_DELETE) === self::IT_MODE_DELETE && $this->valid()) { $this->pop(); }
            $this->__pos--;
        } else {
            if (($this->__mode & self::IT_MODE_DELETE) === self::IT_MODE_DELETE && $this->valid()) {
                $this->shift();
            } else {
                $this->__pos++;
            }
        }
    }
    public function prev(): void { if ($this->__lifo()) { $this->__pos++; } else { $this->__pos--; } }
    public function offsetExists($index): bool {
        $n = count($this->__items);
        $i = (int)$index;
        return $i >= 0 && $i < $n;
    }
    public function offsetGet($index): mixed { return $this->__items[$this->__real($index, "offsetGet")]; }
    public function offsetSet($index, $value): void {
        if ($index === null) { $this->__items[] = $value; return; }
        $this->__items[$this->__real($index, "offsetSet")] = $value;
    }
    public function offsetUnset($index): void {
        array_splice($this->__items, $this->__real($index, "offsetUnset"), 1);
    }
    public function add($index, $value) {
        $n = count($this->__items);
        $i = (int)$index;
        if ($i < 0 || $i > $n) {
            throw new OutOfRangeException("SplDoublyLinkedList::add(): Argument #1 (\$index) is out of range");
        }
        array_splice($this->__items, $i, 0, [$value]);
    }
}
class SplQueue extends SplDoublyLinkedList {
    public function enqueue($value) { $this->push($value); }
    public function dequeue() { return $this->shift(); }
}
class SplStack extends SplDoublyLinkedList {
    protected $__mode = 2; // IT_MODE_LIFO | IT_MODE_KEEP
}
class SplObjectStorage implements Countable, Iterator, ArrayAccess {
    private $__objs = [];  // spl_object_id => object (strong ref, as ext/spl)
    private $__data = [];  // spl_object_id => attached info
    private $__pos = 0;
    private $__ids = [];   // iteration snapshot (rewind)
    private function __attach($object, $info) {
        $id = spl_object_id($object);
        $this->__objs[$id] = $object;
        $this->__data[$id] = $info;
    }
    public function attach($object, $info = null) {
        __deprecated_from_caller('Method SplObjectStorage::attach() is deprecated since 8.5, use method SplObjectStorage::offsetSet() instead');
        $this->__attach($object, $info);
    }
    public function detach($object) {
        __deprecated_from_caller('Method SplObjectStorage::detach() is deprecated since 8.5, use method SplObjectStorage::offsetUnset() instead');
        $id = spl_object_id($object);
        unset($this->__objs[$id], $this->__data[$id]);
    }
    public function contains($object) {
        __deprecated_from_caller('Method SplObjectStorage::contains() is deprecated since 8.5, use method SplObjectStorage::offsetExists() instead');
        return isset($this->__objs[spl_object_id($object)]);
    }
    public function addAll($storage) {
        foreach ($storage->__objs as $id => $obj) {
            $this->__objs[$id] = $obj;
            $this->__data[$id] = $storage->__data[$id];
        }
        return $this->count();
    }
    public function removeAll($storage) {
        foreach ($storage->__objs as $id => $obj) {
            unset($this->__objs[$id], $this->__data[$id]);
        }
        return $this->count();
    }
    public function removeAllExcept($storage) {
        foreach ($this->__objs as $id => $obj) {
            if (!isset($storage->__objs[$id])) {
                unset($this->__objs[$id], $this->__data[$id]);
            }
        }
        return $this->count();
    }
    public function getHash($object) { return spl_object_hash($object); }
    public function count($mode = COUNT_NORMAL) { return count($this->__objs); }
    public function rewind() { $this->__ids = array_keys($this->__objs); $this->__pos = 0; }
    public function valid() { return isset($this->__ids[$this->__pos]) && isset($this->__objs[$this->__ids[$this->__pos]]); }
    public function key() { return $this->__pos; }
    public function current() { return $this->__objs[$this->__ids[$this->__pos]]; }
    public function next() { $this->__pos++; }
    public function getInfo() {
        if (!$this->valid()) { return null; }
        return $this->__data[$this->__ids[$this->__pos]];
    }
    public function setInfo($info) {
        if ($this->valid()) { $this->__data[$this->__ids[$this->__pos]] = $info; }
    }
    public function offsetExists($object) { return isset($this->__objs[spl_object_id($object)]); }
    public function offsetSet($object, $info = null) { $this->__attach($object, $info); }
    public function offsetUnset($object) {
        $id = spl_object_id($object);
        unset($this->__objs[$id], $this->__data[$id]);
    }
    public function offsetGet($object) {
        $id = spl_object_id($object);
        if (!isset($this->__objs[$id])) {
            throw new UnexpectedValueException('Object not found');
        }
        return $this->__data[$id];
    }
}
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
    // SPL's dual-iterators forward unknown method calls to the inner iterator
    // (spl_dual_it_call_method): `$filterIt->getFilename()` reaches the
    // wrapped (Recursive)DirectoryIterator down the chain. Symfony Finder's
    // whole Iterator/ stack leans on this.
    public function __call($name, $args) { return $this->__it->$name(...$args); }
}
interface OuterIterator extends Iterator {
    public function getInnerIterator();
}
interface RecursiveIterator extends Iterator {
    public function hasChildren();
    public function getChildren();
}
// `FilterIterator`: skip inner entries the subclass's accept() rejects --
// rewind/next fast-forward to the next accepted position (Symfony Finder's
// whole Iterator/ directory is built on it).
abstract class FilterIterator extends IteratorIterator {
    abstract public function accept(): bool;
    public function rewind() {
        parent::rewind();
        while (parent::valid() && !$this->accept()) { parent::next(); }
    }
    public function next() {
        parent::next();
        while (parent::valid() && !$this->accept()) { parent::next(); }
    }
}
class CallbackFilterIterator extends FilterIterator {
    private $__cb;
    public function __construct($iterator, $callback) {
        parent::__construct($iterator);
        $this->__cb = $callback;
    }
    public function accept(): bool {
        $cb = $this->__cb;
        return (bool) $cb($this->current(), $this->key(), $this->getInnerIterator());
    }
}
// `RecursiveFilterIterator`: FilterIterator over a RecursiveIterator; children
// are wrapped in the SUBCLASS (new static) so the filter applies at every
// depth (PHPUnit's Runner/Filter iterators are built on it).
abstract class RecursiveFilterIterator extends FilterIterator implements RecursiveIterator {
    public function hasChildren(): bool {
        return $this->getInnerIterator()->hasChildren();
    }
    public function getChildren() {
        return new static($this->getInnerIterator()->getChildren());
    }
}
// `AppendIterator`: iterate several iterators in sequence. Each appended
// iterator is rewound when it becomes current (append-after-start supported).
class AppendIterator implements OuterIterator {
    private $__its = [];
    private $__idx = 0;
    public function __construct() {}
    public function append($iterator) { $this->__its[] = $iterator; }
    public function getInnerIterator() { return $this->__its[$this->__idx] ?? null; }
    private function __settle() {
        while ($this->__idx < count($this->__its) && !$this->__its[$this->__idx]->valid()) {
            $this->__idx++;
            if ($this->__idx < count($this->__its)) { $this->__its[$this->__idx]->rewind(); }
        }
    }
    public function rewind() {
        $this->__idx = 0;
        if (count($this->__its) > 0) { $this->__its[0]->rewind(); }
        $this->__settle();
    }
    public function valid() { return $this->__idx < count($this->__its) && $this->__its[$this->__idx]->valid(); }
    public function current() { return $this->valid() ? $this->__its[$this->__idx]->current() : null; }
    public function key() { return $this->valid() ? $this->__its[$this->__idx]->key() : null; }
    public function next() {
        if ($this->valid()) { $this->__its[$this->__idx]->next(); }
        $this->__settle();
    }
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
    const IS_INSTANCEOF = 2;
    // Validate the $flags argument and apply the IS_INSTANCEOF filter. `$label` is
    // the Reflection class reported in the invalid-flag error (PHP reports the
    // declaring scope, e.g. ReflectionFunctionAbstract for func/method). When the
    // flag is set, `$all` was fetched unfiltered (host name = null) and we keep
    // only attributes whose class is `$name` or a subclass/implementor of it; the
    // filter class must exist (else PHP throws "Class X not found"). Without the
    // flag the host already applied the exact-name filter, so `$all` is as-is.
    public static function __filter($all, $name, $flags, $label) {
        if ($flags !== 0 && $flags !== self::IS_INSTANCEOF) {
            throw new Error($label . '::getAttributes(): Argument #2 ($flags) must be a valid attribute filter flag');
        }
        if (($flags & self::IS_INSTANCEOF) === 0 || $name === null) { return $all; }
        if (!class_exists($name) && !interface_exists($name)) {
            throw new Error('Class "' . $name . '" not found');
        }
        $want = strtolower($name);
        $out = [];
        foreach ($all as $a) {
            $x = $a->getName();
            $isa = strcasecmp($x, $name) === 0;
            // Only walk the hierarchy for a class that actually exists; an
            // unresolved attribute class is simply not an instance of anything but
            // its own name (mirrors PHP, which emits no warning here).
            if (!$isa && (class_exists($x) || interface_exists($x))) {
                foreach (class_parents($x) as $p) { if (strtolower($p) === $want) { $isa = true; break; } }
                if (!$isa) {
                    foreach (class_implements($x) as $i) { if (strtolower($i) === $want) { $isa = true; break; } }
                }
            }
            if ($isa) { $out[] = $a; }
        }
        return $out;
    }
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
    public $__classconst;
    public $__paramfunc;
    public $__paramclass;
    public $__parampos;
    public $__closure_val;
    public function getName() { return $this->name; }
    public function getArguments() {
        if (isset($this->__paramfunc)) {
            return __reflect_param_attr_args($this->__paramclass, $this->__paramfunc, $this->__parampos, $this->__index);
        }
        if (isset($this->__classconst)) {
            return __reflect_classconst_attr_args($this->__class, $this->__classconst, $this->__index);
        }
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
        if ($this->__closure_val !== null) {
            return __reflect_closure_attr_args($this->__closure_val, $this->__index);
        }
        return __reflect_attr_arguments($this->__class, $this->__index);
    }
    public function newInstance() {
        if (isset($this->__paramfunc)) {
            return __reflect_param_attr_new($this->__paramclass, $this->__paramfunc, $this->__parampos, $this->__index);
        }
        if (isset($this->__classconst)) {
            return __reflect_classconst_attr_new($this->__class, $this->__classconst, $this->__index);
        }
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
        if ($this->__closure_val !== null) {
            return __reflect_closure_attr_new($this->__closure_val, $this->__index);
        }
        return __reflect_attr_newinstance($this->__class, $this->__index);
    }
}
class ReflectionClass {
    public $name;
    public function __construct($objectOrClass) {
        $this->name = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        // An *object* argument is always reflectable (engine values like a
        // Closure included); only a class-name string is checked for existence.
        if (!is_object($objectOrClass) && !class_exists($this->name) && !interface_exists($this->name) && !trait_exists($this->name)) {
            throw new ReflectionException(sprintf('Class "%s" does not exist', $this->name));
        }
    }
    public function getFileName() { $l = __reflect_class_loc($this->name); return $l[0]; }
    public function isInternal() { return $this->getFileName() === false; }
    public function isUserDefined() { return $this->getFileName() !== false; }
    public function getDocComment() { return __reflect_class_doc($this->name); }
    public function isReadOnly() { return __reflect_class_modifiers($this->name)['readonly'] ?? false; }
    public function getStartLine() { $l = __reflect_class_loc($this->name); return $l[0] === false ? false : $l[1]; }
    public function getEndLine() { $l = __reflect_class_loc($this->name); return $l[0] === false ? false : $l[2]; }
    // phpr mangles anonymous classes exactly like PHP: `class@anonymous\0N`.
    public function isAnonymous() { return strpos($this->name, 'class@anonymous') === 0; }
    public function getName() { return $this->name; }
    public function getShortName() {
        $p = strrpos($this->name, '\\');
        return $p === false ? $this->name : substr($this->name, $p + 1);
    }
    // Attributes are retained at lowering; the host builds one ReflectionAttribute
    // per attribute declared on the class (optionally filtered by name).
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_class_attributes($this->name, $hostName), $name, $flags, 'ReflectionClass');
    }
    public function newInstance(...$args) { return new $this->name(...$args); }
    public function newInstanceArgs($args = []) { return new $this->name(...$args); }
    public function newInstanceWithoutConstructor() {
        // Internal final classes cannot skip their constructor (Zend rejects
        // it; doctrine/instantiator's PDORow probe relies on the refusal).
        if ($this->isInternal() && $this->isFinal()) {
            throw new ReflectionException('Class ' . $this->name . ' is an internal class marked as final that cannot be instantiated without invoking its constructor');
        }
        return __reflect_new_no_ctor($this->name);
    }
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
    public function isCloneable() {
        if ($this->isInterface() || $this->isAbstract() || $this->isEnum()) { return false; }
        if ($this->hasMethod('__clone')) {
            return $this->getMethod('__clone')->isPublic();
        }
        return true;
    }
    public function isInterface() { return interface_exists($this->name); }
    public function isEnum() { return in_array('UnitEnum', class_implements($this->name)); }
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
    public function getConstants($filter = null) {
        return $filter === null
            ? __reflect_class_constants($this->name)
            : __reflect_class_constants($this->name, $filter);
    }
    public function getReflectionConstant($name) {
        try { return new ReflectionClassConstant($this->name, $name); }
        catch (ReflectionException $e) { return false; }
    }
    public function getReflectionConstants($filter = null) {
        $out = [];
        foreach (__reflect_class_const_names($this->name) as $n) {
            $rc = new ReflectionClassConstant($this->name, $n);
            if ($filter === null || ($rc->getModifiers() & $filter)) { $out[] = $rc; }
        }
        return $out;
    }
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
    public function getMethods($filter = null) {
        $out = [];
        // All visibilities, parent chain included (get_class_methods filters
        // to public outside the class -- PHPUnit's #[Before] hooks are protected).
        foreach (__reflect_method_names($this->name) as $m) {
            $rm = new ReflectionMethod($this->name, $m);
            if ($filter !== null && ($rm->getModifiers() & $filter) === 0) { continue; }
            $out[] = $rm;
        }
        return $out;
    }
    public function hasMethod($name) { return method_exists($this->name, $name); }
}
abstract class ReflectionType {
    abstract public function allowsNull(): bool;
    abstract public function __toString(): string;
}
class ReflectionUnionType extends ReflectionType {
    public $__types; public $__nullable;
    public function getTypes() { return $this->__types; }
    public function allowsNull(): bool { return $this->__nullable; }
    public function __toString(): string {
        $parts = [];
        foreach ($this->__types as $t) { $parts[] = $t->getName(); }
        return implode('|', $parts);
    }
    public static function __fromInfo($t) {
        $r = new ReflectionUnionType();
        $types = [];
        foreach ($t['types'] as $m) { $types[] = ReflectionNamedType::__fromInfo($m); }
        $r->__types = $types;
        $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionIntersectionType extends ReflectionType {
    public $__types;
    public function getTypes() { return $this->__types; }
    public function allowsNull(): bool { return false; }
    public function __toString(): string {
        $parts = [];
        foreach ($this->__types as $t) { $parts[] = $t->getName(); }
        return implode('&', $parts);
    }
    public static function __fromInfo($t) {
        $r = new ReflectionIntersectionType();
        $types = [];
        foreach ($t['types'] as $m) { $types[] = ReflectionNamedType::__fromInfo($m); }
        $r->__types = $types;
        return $r;
    }
}
class ReflectionNamedType extends ReflectionType {
    public $__name; public $__builtin; public $__nullable;
    public function getName() { return $this->__name; }
    public function allowsNull(): bool { return $this->__nullable; }
    public function isBuiltin() { return $this->__builtin; }
    public function __toString(): string {
        $q = ($this->__nullable && $this->__name !== 'mixed' && $this->__name !== 'null') ? '?' : '';
        return $q . $this->__name;
    }
    public static function __fromInfo($t) {
        if ($t === false) { return null; }
        if (isset($t['kind'])) {
            return $t['kind'] === 'intersection'
                ? ReflectionIntersectionType::__fromInfo($t)
                : ReflectionUnionType::__fromInfo($t);
        }
        $r = new ReflectionNamedType();
        $r->__name = $t['name']; $r->__builtin = $t['builtin']; $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionParameter {
    public $name;
    public $__pos; public $__optional; public $__variadic; public $__byref;
    public $__type; public $__hasDefault; public $__default;
    public $__declClass; public $__declFunc;
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
        $this->__declClass = $p['declClass'] ?? ''; $this->__declFunc = $p['declFunc'] ?? '';
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
    // `Parameter #N [ <optional> Type $name = DEFAULT ]` (oracle format).
    // PHPUnit's mock generator parses the piece after ' = ' as *source code*
    // for an object default, so enum cases render `\FQCN::CASE`.
    public function __toString() {
        $opt = $this->isDefaultValueAvailable();
        $s = 'Parameter #' . $this->getPosition() . ' [ <' . ($opt ? 'optional' : 'required') . '> ';
        $t = $this->getType();
        $ts = '';
        if ($t !== null) {
            if ($t instanceof ReflectionUnionType) {
                $parts = array();
                foreach ($t->getTypes() as $tt) { $parts[] = $tt->getName(); }
                $ts = implode('|', $parts);
            } elseif ($t instanceof ReflectionIntersectionType) {
                $parts = array();
                foreach ($t->getTypes() as $tt) { $parts[] = $tt->getName(); }
                $ts = implode('&', $parts);
            } else {
                $ts = $t->getName();
                if ($t->allowsNull() && $ts !== 'null' && $ts !== 'mixed') { $ts = '?' . $ts; }
            }
        }
        $s .= ($ts !== '' ? $ts . ' ' : '') . ($this->isPassedByReference() ? '&' : '') . ($this->isVariadic() ? '...' : '') . '$' . $this->getName();
        if ($opt && !$this->isVariadic()) {
            $v = $this->__default;
            if ($v === null) { $d = 'NULL'; }
            elseif (is_object($v)) {
                $d = ($v instanceof UnitEnum) ? '\\' . get_class($v) . '::' . $v->name : 'new \\' . get_class($v) . '(...)';
            }
            elseif (is_array($v)) { $d = str_replace("\n", '', var_export($v, true)); }
            else { $d = var_export($v, true); }
            $s .= ' = ' . $d;
        }
        return $s . ' ]';
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_param_attributes($this->__declClass, $this->__declFunc, $this->__pos, $hostName), $name, $flags, 'ReflectionParameter');
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
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_const_attributes($this->name, $hostName), $name, $flags, 'ReflectionConstant');
    }
    public function __toString() { return sprintf("Constant [ %s ]\n", $this->name); }
}
class ReflectionClassConstant {
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_FINAL = 32;
    public $name;
    public $class;
    public $__info;
    public function __construct($class, $constant) {
        $cls = is_object($class) ? get_class($class) : $class;
        $info = __reflect_class_const_info($cls, $constant);
        if ($info === false) {
            throw new ReflectionException(sprintf('Constant %s::%s does not exist', $cls, $constant));
        }
        $this->name = $constant;
        $this->class = $info['declaringClass'];
        $this->__info = $info;
    }
    public function getName() { return $this->name; }
    public function getValue() { return $this->__info['value']; }
    public function getDeclaringClass() { return new ReflectionClass($this->class); }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function isFinal() { return $this->__info['final']; }
    public function isEnumCase() { return $this->__info['enumCase']; }
    public function getModifiers() {
        $m = 0;
        if ($this->__info['visibility'] === 'public') { $m |= self::IS_PUBLIC; }
        elseif ($this->__info['visibility'] === 'protected') { $m |= self::IS_PROTECTED; }
        else { $m |= self::IS_PRIVATE; }
        if ($this->__info['final']) { $m |= self::IS_FINAL; }
        return $m;
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_classconst_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionClassConstant');
    }
}
class ReflectionEnumUnitCase extends ReflectionClassConstant {
    // getValue() is inherited: __reflect_class_const_info returns the case
    // singleton as the constant's value.
    public function getEnum() { return new ReflectionEnum($this->class); }
}
class ReflectionEnumBackedCase extends ReflectionEnumUnitCase {
    public function getBackingValue() { return $this->getValue()->value; }
}
class ReflectionEnum extends ReflectionClass {
    public function isBacked() { return in_array('BackedEnum', class_implements($this->name)); }
    public function getBackingType() { return ReflectionNamedType::__fromInfo(__reflect_enum_backing($this->name)); }
    public function hasCase($name) {
        $cls = $this->name;
        foreach ($cls::cases() as $c) { if ($c->name === $name) { return true; } }
        return false;
    }
    public function getCase($name) {
        if (!$this->hasCase($name)) {
            throw new ReflectionException(sprintf('Case %s::%s does not exist', $this->name, $name));
        }
        return $this->isBacked()
            ? new ReflectionEnumBackedCase($this->name, $name)
            : new ReflectionEnumUnitCase($this->name, $name);
    }
    public function getCases() {
        $out = [];
        $cls = $this->name;
        $backed = $this->isBacked();
        foreach ($cls::cases() as $c) {
            $out[] = $backed
                ? new ReflectionEnumBackedCase($this->name, $c->name)
                : new ReflectionEnumUnitCase($this->name, $c->name);
        }
        return $out;
    }
}
abstract class ReflectionFunctionAbstract {}
class ReflectionFunction extends ReflectionFunctionAbstract {
    public $name;
    public $__info;
    public $__closure;
    public function __construct($name) {
        if ($name instanceof Closure) {
            $this->name = '{closure}';
            $this->__closure = $name;
            $this->__info = __reflect_closure_info($name);
        } else {
            $this->name = is_string($name) ? $name : '{closure}';
            $this->__info = __reflect_func_info($this->name);
        }
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
    public function isVariadic() {
        foreach ($this->__info['params'] as $p) { if ($p['variadic']) { return true; } }
        return false;
    }
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    public function invoke(...$args) { return call_user_func_array($this->name, $args); }
    public function invokeArgs($args) { return call_user_func_array($this->name, $args); }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        $all = $this->__closure !== null
            ? __reflect_closure_attributes($this->__closure, $hostName)
            : __reflect_func_attributes($this->name, $hostName);
        return ReflectionAttribute::__filter($all, $name, $flags, 'ReflectionFunctionAbstract');
    }
}
class ReflectionMethod extends ReflectionFunctionAbstract {
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_STATIC = 16;
    const IS_FINAL = 32;
    const IS_ABSTRACT = 64;
    public $name;
    public $class;
    public $__info;
    public function getName() { return $this->name; }
    public function getModifiers() {
        $bits = 0;
        if ($this->isPublic()) { $bits |= 1; }
        if ($this->isProtected()) { $bits |= 2; }
        if ($this->isPrivate()) { $bits |= 4; }
        if ($this->isStatic()) { $bits |= 16; }
        if ($this->isFinal()) { $bits |= 32; }
        if ($this->isAbstract()) { $bits |= 64; }
        return $bits;
    }
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
    public function isVariadic() {
        foreach ($this->__info['params'] as $p) { if ($p['variadic']) { return true; } }
        return false;
    }
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    public function __construct($objectOrClass, $method = null) {
        // A class-name string autoloads (Zend does; the info lookup below is
        // autoload-blind and would report "does not exist" for a not-yet-loaded
        // class).
        if (is_string($objectOrClass)) { class_exists(strpos($objectOrClass, '::') !== false ? explode('::', $objectOrClass, 2)[0] : $objectOrClass); }
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
    public function isConstructor() { return strcasecmp($this->name, '__construct') === 0; }
    public function returnsReference() { return $this->__info['byRef'] ?? false; }
    public function hasTentativeReturnType() { return false; }
    public function getTentativeReturnType() { return null; }
    public function isUserDefined() { return ($this->__info['file'] ?? false) !== false; }
    public function isInternal() { return ($this->__info['file'] ?? false) === false; }
    public function isDestructor() { return strcasecmp($this->name, '__destruct') === 0; }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function getFileName() { return $this->__info['file']; }
    public function getStartLine() { return $this->__info['startLine']; }
    public function getEndLine() { return $this->__info['endLine']; }
    public function isStatic() { return $this->__info['static']; }
    public function isFinal() { return $this->__info['final']; }
    public function isAbstract() { return $this->__info['abstract']; }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function setAccessible($accessible) {}
    public function invoke($object, ...$args) {
        return __reflect_invoke($object, $this->class, $this->name, $args);
    }
    public function invokeArgs($object, $args) {
        return __reflect_invoke($object, $this->class, $this->name, $args);
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_method_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionFunctionAbstract');
    }
}
class ReflectionProperty {
    public function getDocComment() { return false; }
    public function isFinal() { return false; }
    public function isAbstract() { return false; }
    public function isVirtual() { return false; }
    public function hasHooks() { return false; }
    public function getHooks() { return []; }
    public function hasHook($type) { return false; }
    public function getHook($type) { return null; }
    public function isDynamic() { return false; }
    public function isLazy() { return false; }
    const IS_STATIC = 16;
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_READONLY = 128;
    const IS_PROTECTED_SET = 2048;
    const IS_PRIVATE_SET = 4096;
    public $name;
    public $class;
    public $__info;
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
        $this->__info = __reflect_prop_details($this->class, $this->name);
    }
    public function getName() { return $this->name; }
    public function getValue($object = null) {
        if (__reflect_prop_is_static($this->class, $this->name)) {
            return __reflect_static_prop_get($this->class, $this->name);
        }
        return __reflect_prop_get($this->class, $this->name, $object);
    }
    public function setValue($object, $value = null) {
        if (__reflect_prop_is_static($this->class, $this->name)) {
            // Static form: setValue($value) and setValue(null, $value) both write
            // the class-level slot (Composer pokes InstalledVersions::$selfDir).
            if (func_num_args() === 1) { $value = $object; }
            __reflect_static_prop_set($this->class, $this->name, $value);
            return;
        }
        __reflect_prop_set($this->class, $this->name, $object, $value);
    }
    // PHP 8.4 raw accessors bypass property hooks; phpr's reflection reads the
    // backing slots directly already, so they alias get/setValue.
    public function getRawValue($object) { return $this->getValue($object); }
    public function setRawValue($object, $value) { $this->setValue($object, $value); }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_prop_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionProperty');
    }
    public function isStatic() { return __reflect_prop_is_static($this->class, $this->name); }
    public function hasType() { return __reflect_prop_type($this->class, $this->name) !== false; }
    public function getType() { return ReflectionNamedType::__fromInfo(__reflect_prop_type($this->class, $this->name)); }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function isReadOnly() { return $this->__info['readonly']; }
    public function getModifiers() {
        $m = 0;
        if ($this->__info['visibility'] === 'public') { $m |= self::IS_PUBLIC; }
        elseif ($this->__info['visibility'] === 'protected') { $m |= self::IS_PROTECTED; }
        else { $m |= self::IS_PRIVATE; }
        if ($this->__info['static']) { $m |= self::IS_STATIC; }
        if ($this->__info['readonly']) {
            $m |= self::IS_READONLY;
            // PHP 8.4: a `public readonly` property's implicit set-visibility is
            // downgraded to protected (asymmetric visibility), adding IS_PROTECTED_SET.
            if ($this->__info['visibility'] === 'public') { $m |= self::IS_PROTECTED_SET; }
        }
        return $m;
    }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function hasDefaultValue() { return $this->__info['hasDefault']; }
    public function getDefaultValue() { return $this->__info['default']; }
    public function isInitialized($object = null) { return __reflect_prop_initialized($this->class, $this->name, $object); }
    public function skipLazyInitialization($object) {
        $msg = __lazy_skip_init($object, $this->class, $this->name);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
    public function setRawValueWithoutLazyInitialization($object, $value) {
        $msg = __lazy_set_raw($object, $this->class, $this->name, $value);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
}
class ReflectionExtension {
    public $name;
    public function __construct($name) {
        if (!extension_loaded($name)) {
            throw new ReflectionException(sprintf('Extension "%s" does not exist', $name));
        }
        // Canonical casing as get_loaded_extensions reports it, mirroring
        // ReflectionExtension::getName().
        foreach (get_loaded_extensions() as $ext) {
            if (strcasecmp($ext, $name) === 0) { $name = $ext; break; }
        }
        $this->name = $name;
    }
    public function getName() { return $this->name; }
    public function getVersion() { return phpversion($this->name); }
    // info() prints the phpinfo() block for the extension. phpr models these
    // extensions with Rust crates, not the C internals whose text this reports,
    // so it emits nothing (the OpenSSL-text-rendering class of rabbit hole);
    // callers parse it defensively (e.g. Composer regexes for an optional
    // sub-library version and simply skips it when absent).
    public function info() {}
    public function __toString() { return ''; }
    // Constants per extension. pcntl's table is what real consumers read
    // (monolog's SignalHandler maps signo -> "SIG*" name through it); values
    // are the macOS oracle's. Other extensions read as empty for now.
    public function getConstants() {
        if (strcasecmp($this->name, 'pcntl') === 0) {
            return [
                'WNOHANG' => 1, 'WUNTRACED' => 2, 'WCONTINUED' => 16, 'WEXITED' => 4,
                'WSTOPPED' => 8, 'WNOWAIT' => 32, 'P_ALL' => 0, 'P_PID' => 1, 'P_PGID' => 2,
                'SIG_IGN' => 1, 'SIG_DFL' => 0, 'SIG_ERR' => -1, 'SIGHUP' => 1, 'SIGINT' => 2,
                'SIGQUIT' => 3, 'SIGILL' => 4, 'SIGTRAP' => 5, 'SIGABRT' => 6, 'SIGIOT' => 6,
                'SIGBUS' => 10, 'SIGFPE' => 8, 'SIGKILL' => 9, 'SIGUSR1' => 30, 'SIGSEGV' => 11,
                'SIGUSR2' => 31, 'SIGPIPE' => 13, 'SIGALRM' => 14, 'SIGTERM' => 15,
                'SIGCHLD' => 20, 'SIGCONT' => 19, 'SIGSTOP' => 17, 'SIGTSTP' => 18,
                'SIGTTIN' => 21, 'SIGTTOU' => 22, 'SIGURG' => 16, 'SIGXCPU' => 24,
                'SIGXFSZ' => 25, 'SIGVTALRM' => 26, 'SIGPROF' => 27, 'SIGWINCH' => 28,
                'SIGIO' => 23, 'SIGINFO' => 29, 'SIGSYS' => 12, 'SIGBABY' => 12,
                'PRIO_PGRP' => 1, 'PRIO_USER' => 2, 'PRIO_PROCESS' => 0,
                'PRIO_DARWIN_BG' => 4096, 'PRIO_DARWIN_THREAD' => 3, 'SIG_BLOCK' => 1,
                'SIG_UNBLOCK' => 2, 'SIG_SETMASK' => 3, 'PCNTL_EINTR' => 4,
                'PCNTL_ECHILD' => 10, 'PCNTL_EINVAL' => 22, 'PCNTL_EAGAIN' => 35,
                'PCNTL_ESRCH' => 3, 'PCNTL_EACCES' => 13, 'PCNTL_EPERM' => 1,
                'PCNTL_ENOMEM' => 12, 'PCNTL_E2BIG' => 7, 'PCNTL_EFAULT' => 14,
                'PCNTL_EIO' => 5, 'PCNTL_EISDIR' => 21, 'PCNTL_ELOOP' => 62,
                'PCNTL_EMFILE' => 24, 'PCNTL_ENAMETOOLONG' => 63, 'PCNTL_ENFILE' => 23,
                'PCNTL_ENOENT' => 2, 'PCNTL_ENOEXEC' => 8, 'PCNTL_ENOTDIR' => 20,
                'PCNTL_ETXTBSY' => 26, 'PCNTL_ENOSPC' => 28, 'PCNTL_EUSERS' => 68,
            ];
        }
        return [];
    }
}
function enum_exists($enum, $autoload = true) {
    // Reuse class_exists for the (autoload-aware) existence check, then confirm
    // the class is an enum via its implicit UnitEnum interface.
    return class_exists($enum, $autoload) && in_array('UnitEnum', class_implements($enum));
}

// `preg_replace_callback_array(['/rx/' => cb, ...], $subject)`: sequential
// preg_replace_callback over the pattern map (the replacement-count out-param
// stays unreported, matching phpr's preg_replace_callback).
function preg_replace_callback_array($pattern, $subject, $limit = -1, &$count = null, $flags = 0) {
    $count = 0;
    foreach ($pattern as $rx => $cb) {
        $subject = preg_replace_callback($rx, $cb, $subject, $limit);
        if ($subject === null) { return null; }
    }
    return $subject;
}

// ----- ext/dom (host arena behind the __dom_* builtins; see vm/dom.rs) -----
// Node identity is the (docId, nodeId) handle pair; every accessor re-wraps a
// fresh PHP object around the handle (isSameNode compares handles, as ext/dom
// offers for the same reason).
class DOMException extends Exception {}

class LibXMLError {
    public $level = 0;
    public $code = 0;
    public $column = 0;
    public $message = '';
    public $file = '';
    public $line = 0;
}
function is_countable($value) { return is_array($value) || $value instanceof Countable; }
function libxml_get_errors() {
    $out = array();
    foreach (__libxml_get_errors() as $e) {
        $o = new LibXMLError();
        $o->level = $e['level']; $o->code = $e['code']; $o->column = $e['column'];
        $o->message = $e['message']; $o->file = $e['file']; $o->line = $e['line'];
        $out[] = $o;
    }
    return $out;
}
function libxml_get_last_error() {
    $all = libxml_get_errors();
    $n = count($all);
    return $n > 0 ? $all[$n - 1] : false;
}

class DOMNodeList implements IteratorAggregate, Countable {
    public $length = 0;
    public $__items = array();
    public static function __make($items) {
        $l = new DOMNodeList();
        $l->__items = $items;
        $l->length = count($items);
        return $l;
    }
    public function item($index) { return isset($this->__items[$index]) ? $this->__items[$index] : null; }
    public function count(): int { return $this->length; }
    public function getIterator(): Iterator { return new ArrayIterator($this->__items); }
}

class DOMNamedNodeMap implements IteratorAggregate, Countable {
    public $length = 0;
    public $__items = array(); // name => DOMAttr
    public static function __make($items) {
        $m = new DOMNamedNodeMap();
        $m->__items = $items;
        $m->length = count($items);
        return $m;
    }
    public function getNamedItem($name) { return isset($this->__items[$name]) ? $this->__items[$name] : null; }
    public function item($index) {
        $i = 0;
        foreach ($this->__items as $v) { if ($i === (int)$index) return $v; $i++; }
        return null;
    }
    public function count(): int { return $this->length; }
    public function getIterator(): Iterator { return new ArrayIterator($this->__items); }
}

class DOMNode {
    public $__d = -1;
    public $__n = -1;
    public static function __wrap($d, $n) {
        if ($d < 0 || $n < 0) { return null; }
        $i = __dom_info($d, $n);
        switch ($i[0]) {
            case 1: $c = 'DOMElement'; break;
            case 3: $c = 'DOMText'; break;
            case 4: $c = 'DOMCdataSection'; break;
            case 7: $c = 'DOMProcessingInstruction'; break;
            case 8: $c = 'DOMComment'; break;
            case 9: $c = 'DOMDocument'; break;
            case 10: $c = 'DOMDocumentType'; break;
            case 11: $c = 'DOMDocumentFragment'; break;
            default: $c = 'DOMNode';
        }
        $r = new ReflectionClass($c);
        $o = $r->newInstanceWithoutConstructor();
        $o->__d = $d;
        $o->__n = $n;
        return $o;
    }
    public function __get($name) {
        switch ($name) {
            case 'nodeType': $i = __dom_info($this->__d, $this->__n); return $i[0];
            case 'nodeName': $i = __dom_info($this->__d, $this->__n); return $i[1];
            case 'nodeValue':
                $i = __dom_info($this->__d, $this->__n);
                // ext/dom: an element/fragment reports its text content here
                // (xmlNodeGetContent); a document reports NULL.
                if ($i[0] === 1 || $i[0] === 11) { return __dom_text($this->__d, $this->__n); }
                return $i[2];
            case 'textContent': return __dom_text($this->__d, $this->__n);
            case 'parentNode': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 0));
            case 'firstChild': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 1));
            case 'lastChild': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 2));
            case 'nextSibling': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 3));
            case 'previousSibling': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 4));
            case 'ownerDocument':
                $i = __dom_info($this->__d, $this->__n);
                return $i[0] === 9 ? null : DOMNode::__wrap($this->__d, 0);
            case 'childNodes':
                $items = array();
                foreach (__dom_children($this->__d, $this->__n) as $c) {
                    $items[] = DOMNode::__wrap($this->__d, $c);
                }
                return DOMNodeList::__make($items);
            case 'attributes':
                $i = __dom_info($this->__d, $this->__n);
                if ($i[0] !== 1) { return null; }
                $items = array();
                foreach (__dom_attr($this->__d, $this->__n, 4, '', '') as $an) {
                    $items[$an] = DOMAttr::__wrapAttr($this->__d, $this->__n, $an);
                }
                return DOMNamedNodeMap::__make($items);
            case 'namespaceURI': case 'prefix': case 'localName':
                $r = __dom_ns($this->__d, $this->__n, '');
                if ($name === 'namespaceURI') { return $r[0]; }
                if ($name === 'prefix') { return $r[1]; }
                return $r[2];
            case 'baseURI':
                return null;
        }
        return null;
    }
    public function __set($name, $value) {
        if ($name === 'nodeValue' || $name === 'textContent') {
            __dom_set_value($this->__d, $this->__n, (string)$value);
        }
        // Other magic props are read-only in ext/dom; silently ignore like a no-op.
    }
    public function appendChild($node) {
        if ($node->__d !== $this->__d) { throw new DOMException('Wrong Document Error'); }
        if (!__dom_mutate($this->__d, 0, $this->__n, $node->__n, -1)) {
            throw new DOMException('Hierarchy Request Error');
        }
        return $node;
    }
    public function insertBefore($node, $refNode = null) {
        if ($node->__d !== $this->__d) { throw new DOMException('Wrong Document Error'); }
        $ref = $refNode === null ? -1 : $refNode->__n;
        if (!__dom_mutate($this->__d, 1, $this->__n, $node->__n, $ref)) {
            throw new DOMException('Hierarchy Request Error');
        }
        return $node;
    }
    public function removeChild($node) {
        if (!__dom_mutate($this->__d, 2, $this->__n, $node->__n, -1)) {
            throw new DOMException('Not Found Error');
        }
        return $node;
    }
    public function replaceChild($newNode, $oldNode) {
        $this->insertBefore($newNode, $oldNode);
        return $this->removeChild($oldNode);
    }
    public function cloneNode($deep = false) {
        $n = __dom_copy($this->__d, $this->__d, $this->__n, $deep ? 1 : 0);
        return DOMNode::__wrap($this->__d, $n);
    }
    public function hasChildNodes() { return __dom_nav($this->__d, $this->__n, 1) >= 0; }
    public function hasAttributes() {
        $i = __dom_info($this->__d, $this->__n);
        if ($i[0] !== 1) { return false; }
        return count(__dom_attr($this->__d, $this->__n, 4, '', '')) > 0;
    }
    public function isSameNode($other) {
        return $other instanceof DOMNode && $other->__d === $this->__d && $other->__n === $this->__n;
    }
    public function normalize() {}
    public function getLineNo() { return 0; }
    public function getNodePath() { return null; }
    public function lookupNamespaceURI($prefix) { return null; }
    public function lookupPrefix($namespace) { return null; }
    public function isDefaultNamespace($namespace) { return false; }
    public function isSupported($feature, $version) { return false; }
    public function contains($other) {
        if (!($other instanceof DOMNode) || $other->__d !== $this->__d) { return false; }
        $n = $other->__n;
        while ($n >= 0) {
            if ($n === $this->__n) { return true; }
            $n = __dom_nav($this->__d, $n, 0);
        }
        return false;
    }
}

// Class-shape stub of ext/xmlreader's pull parser: enough for code that
// subclasses or type-checks XMLReader (doctrine/instantiator's test assets).
// Actual pull-parsing is out of slice: there are deliberately no methods, so
// any real use fails loudly with "undefined method" instead of misparsing.
class XMLReader {
    const NONE = 0;
    const ELEMENT = 1;
    const ATTRIBUTE = 2;
    const TEXT = 3;
    const CDATA = 4;
    const ENTITY_REF = 5;
    const ENTITY = 6;
    const PI = 7;
    const COMMENT = 8;
    const DOC = 9;
    const DOC_TYPE = 10;
    const DOC_FRAGMENT = 11;
    const NOTATION = 12;
    const WHITESPACE = 13;
    const SIGNIFICANT_WHITESPACE = 14;
    const END_ELEMENT = 15;
    const END_ENTITY = 16;
    const XML_DECLARATION = 17;
    const LOADDTD = 1;
    const DEFAULTATTRS = 2;
    const VALIDATE = 3;
    const SUBST_ENTITIES = 4;
}
class DOMDocument extends DOMNode {
    public $preserveWhiteSpace = true;
    public $formatOutput = false;
    public $validateOnParse = false;
    public $recover = false;
    public $resolveExternals = false;
    public $substituteEntities = false;
    public $strictErrorChecking = true;
    public $documentURI = null;
    public function __construct($version = '1.0', $encoding = '') {
        $this->__d = __dom_new_doc($version, $encoding);
        $this->__n = 0;
    }
    public function loadXML($source, $options = 0) {
        return __dom_load($this->__d, (string)$source, 0);
    }
    public function load($filename, $options = 0) {
        $this->documentURI = (string)$filename;
        return __dom_load($this->__d, (string)$filename, 1);
    }
    public function saveXML($node = null, $options = 0) {
        return __dom_save_xml($this->__d, $node === null ? -1 : $node->__n);
    }
    public function save($filename) {
        return file_put_contents($filename, __dom_save_xml($this->__d, -1));
    }
    public function schemaValidate($filename, $flags = 0) {
        // XSD validation is out of slice: a well-formed document is accepted.
        // (PHPUnit only uses this to warn about an invalid phpunit.xml.)
        return true;
    }
    public function schemaValidateSource($source, $flags = 0) { return true; }
    public function relaxNGValidate($filename) { return true; }
    public function relaxNGValidateSource($source) { return true; }
    public function xinclude($options = 0) {
        // XInclude substitution (PHPUnit's config loader calls this on every
        // phpunit.xml). A document with no XInclude elements is untouched and
        // the count is 0; actual substitution is out of slice, so report -1
        // (libxml's processing-error result) instead of pretending it worked.
        foreach (__dom_by_tag($this->__d, -1, 'xi:include') as $n) { return -1; }
        foreach (__dom_by_tag($this->__d, -1, 'xinclude') as $n) { return -1; }
        return 0;
    }
    public function createElement($localName, $value = '') {
        $n = __dom_create($this->__d, 1, (string)$localName, '');
        if ($n < 0) { throw new DOMException('Invalid Character Error'); }
        if ($value !== '' && $value !== null) {
            $t = __dom_create($this->__d, 3, (string)$value, '');
            __dom_mutate($this->__d, 0, $n, $t, -1);
        }
        return DOMNode::__wrap($this->__d, $n);
    }
    public function createTextNode($data = '') {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 3, (string)$data, ''));
    }
    public function createComment($data = '') {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 8, (string)$data, ''));
    }
    public function createCDATASection($data) {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 4, (string)$data, ''));
    }
    public function createProcessingInstruction($target, $data = '') {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 7, (string)$target, (string)$data));
    }
    public function createDocumentFragment() {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 11, '', ''));
    }
    public function createAttribute($localName) {
        return DOMAttr::__wrapDetached($this->__d, (string)$localName);
    }
    public function getElementsByTagName($qualifiedName) {
        $items = array();
        foreach (__dom_by_tag($this->__d, -1, (string)$qualifiedName) as $n) {
            $items[] = DOMNode::__wrap($this->__d, $n);
        }
        return DOMNodeList::__make($items);
    }
    public function getElementById($elementId) {
        // Without DTD machinery only xml:id qualifies, as in PHP with no DTD.
        foreach (__dom_by_tag($this->__d, -1, '*') as $n) {
            $v = __dom_attr($this->__d, $n, 0, 'xml:id', '');
            if ($v !== false && $v === (string)$elementId) { return DOMNode::__wrap($this->__d, $n); }
        }
        return null;
    }
    public function importNode($node, $deep = false) {
        $n = __dom_copy($this->__d, $node->__d, $node->__n, $deep ? 1 : 0);
        return DOMNode::__wrap($this->__d, $n);
    }
    public function adoptNode($node) { return $this->importNode($node, true); }
    public function __get($name) {
        switch ($name) {
            case 'documentElement': return DOMNode::__wrap($this->__d, __dom_doc_element($this->__d));
            case 'doctype':
                foreach (__dom_children($this->__d, 0) as $c) {
                    $i = __dom_info($this->__d, $c);
                    if ($i[0] === 10) { return DOMNode::__wrap($this->__d, $c); }
                }
                return null;
            case 'xmlVersion': case 'version':
                $m = __dom_doc_meta($this->__d); return $m[0];
            case 'xmlEncoding': case 'encoding': case 'actualEncoding':
                $m = __dom_doc_meta($this->__d); return $m[1];
            case 'xmlStandalone': case 'standalone': return true;
        }
        return parent::__get($name);
    }
}

class DOMElement extends DOMNode {
    public function __construct($qualifiedName, $value = null, $namespace = '') {
        // A standalone element lives in its own private document until adopted
        // (appendChild across documents raises Wrong Document, as in PHP before
        // importNode).
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 1, (string)$qualifiedName, '');
        if ($this->__n < 0) { throw new DOMException('Invalid Character Error'); }
        if ($value !== null && $value !== '') {
            $t = __dom_create($this->__d, 3, (string)$value, '');
            __dom_mutate($this->__d, 0, $this->__n, $t, -1);
        }
    }
    public function __get($name) {
        if ($name === 'tagName') {
            $i = __dom_info($this->__d, $this->__n);
            return $i[1];
        }
        return parent::__get($name);
    }
    public function getAttribute($qualifiedName) {
        $v = __dom_attr($this->__d, $this->__n, 0, (string)$qualifiedName, '');
        return $v === false ? '' : $v;
    }
    public function hasAttribute($qualifiedName) {
        return __dom_attr($this->__d, $this->__n, 2, (string)$qualifiedName, '');
    }
    public function setAttribute($qualifiedName, $value) {
        __dom_attr($this->__d, $this->__n, 1, (string)$qualifiedName, (string)$value);
        return DOMAttr::__wrapAttr($this->__d, $this->__n, (string)$qualifiedName);
    }
    public function removeAttribute($qualifiedName) {
        return __dom_attr($this->__d, $this->__n, 3, (string)$qualifiedName, '');
    }
    public function getAttributeNames() {
        return __dom_attr($this->__d, $this->__n, 4, '', '');
    }
    public function getAttributeNode($qualifiedName) {
        if (!$this->hasAttribute($qualifiedName)) { return false; }
        return DOMAttr::__wrapAttr($this->__d, $this->__n, (string)$qualifiedName);
    }
    public function setAttributeNode($attr) {
        __dom_attr($this->__d, $this->__n, 1, $attr->name, $attr->value);
        $attr->__d = $this->__d;
        $attr->__e = $this->__n;
        return null;
    }
    public function removeAttributeNode($attr) {
        __dom_attr($this->__d, $this->__n, 3, $attr->name, '');
        return $attr;
    }
    public function toggleAttribute($qualifiedName, $force = null) {
        $has = $this->hasAttribute($qualifiedName);
        $want = $force === null ? !$has : (bool)$force;
        if ($want && !$has) { $this->setAttribute($qualifiedName, ''); }
        if (!$want && $has) { $this->removeAttribute($qualifiedName); }
        return $want;
    }
    public function getElementsByTagName($qualifiedName) {
        $items = array();
        foreach (__dom_by_tag($this->__d, $this->__n, (string)$qualifiedName) as $n) {
            $items[] = DOMNode::__wrap($this->__d, $n);
        }
        return DOMNodeList::__make($items);
    }
    public function setIdAttribute($qualifiedName, $isId) {}
    public function remove() {
        $p = __dom_nav($this->__d, $this->__n, 0);
        if ($p >= 0) { __dom_mutate($this->__d, 2, $p, $this->__n, -1); }
    }
}

class DOMAttr extends DOMNode {
    public $name = '';
    public $value = '';
    public $__e = -1; // owner element node id (-1 = detached)
    public function __construct($name, $value = '') {
        $this->name = (string)$name;
        $this->value = (string)$value;
    }
    public static function __wrapAttr($d, $elem, $name) {
        $a = new DOMAttr($name);
        $a->__d = $d;
        $a->__e = $elem;
        $v = __dom_attr($d, $elem, 0, $name, '');
        $a->value = $v === false ? '' : $v;
        return $a;
    }
    public static function __wrapDetached($d, $name) {
        $a = new DOMAttr($name);
        $a->__d = $d;
        return $a;
    }
    public function __get($prop) {
        switch ($prop) {
            case 'nodeType': return 2;
            case 'nodeName': return $this->name;
            case 'nodeValue': case 'textContent':
                if ($this->__e >= 0) {
                    $v = __dom_attr($this->__d, $this->__e, 0, $this->name, '');
                    if ($v !== false) { return $v; }
                }
                return $this->value;
            case 'ownerElement':
                return $this->__e >= 0 ? DOMNode::__wrap($this->__d, $this->__e) : null;
            case 'specified': return true;
            case 'namespaceURI': case 'prefix': case 'localName':
                if ($this->__e >= 0) {
                    $r = __dom_ns($this->__d, $this->__e, $this->name);
                    if ($prop === 'namespaceURI') { return $r[0]; }
                    if ($prop === 'prefix') { return $r[1]; }
                    return $r[2];
                }
                $p = strpos($this->name, ':');
                if ($prop === 'prefix') { return $p === false ? '' : substr($this->name, 0, $p); }
                if ($prop === 'localName') { return $p === false ? $this->name : substr($this->name, $p + 1); }
                return null;
        }
        return parent::__get($prop);
    }
    public function __set($prop, $v) {
        if ($prop === 'value' || $prop === 'nodeValue') {
            $this->value = (string)$v;
            if ($this->__e >= 0) {
                __dom_attr($this->__d, $this->__e, 1, $this->name, (string)$v);
            }
            return;
        }
        parent::__set($prop, $v);
    }
    public function isId() { return false; }
}

class DOMCharacterData extends DOMNode {
    public function __get($name) {
        if ($name === 'data') {
            $i = __dom_info($this->__d, $this->__n);
            return $i[2];
        }
        if ($name === 'length') {
            $i = __dom_info($this->__d, $this->__n);
            return strlen($i[2]);
        }
        return parent::__get($name);
    }
    public function __set($name, $value) {
        if ($name === 'data') {
            __dom_set_value($this->__d, $this->__n, (string)$value);
            return;
        }
        parent::__set($name, $value);
    }
    public function appendData($data) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, $i[2] . (string)$data);
        return true;
    }
    public function substringData($offset, $count) {
        $i = __dom_info($this->__d, $this->__n);
        return substr($i[2], $offset, $count);
    }
    public function insertData($offset, $data) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, substr($i[2], 0, $offset) . (string)$data . substr($i[2], $offset));
        return true;
    }
    public function deleteData($offset, $count) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, substr($i[2], 0, $offset) . substr($i[2], $offset + $count));
        return true;
    }
    public function replaceData($offset, $count, $data) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, substr($i[2], 0, $offset) . (string)$data . substr($i[2], $offset + $count));
        return true;
    }
    public function remove() {
        $p = __dom_nav($this->__d, $this->__n, 0);
        if ($p >= 0) { __dom_mutate($this->__d, 2, $p, $this->__n, -1); }
    }
}

class DOMText extends DOMCharacterData {
    public function __construct($data = '') {
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 3, (string)$data, '');
    }
    public function isElementContentWhitespace() {
        $i = __dom_info($this->__d, $this->__n);
        return trim($i[2]) === '';
    }
}

class DOMComment extends DOMCharacterData {
    public function __construct($data = '') {
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 8, (string)$data, '');
    }
}

class DOMCdataSection extends DOMText {
    public function __construct($data) {
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 4, (string)$data, '');
    }
}

class DOMProcessingInstruction extends DOMNode {
    public function __get($name) {
        $i = __dom_info($this->__d, $this->__n);
        if ($name === 'target') { return $i[1]; }
        if ($name === 'data') { return $i[2]; }
        return parent::__get($name);
    }
}

class DOMDocumentFragment extends DOMNode {
    public function appendXML($data) {
        // Parse via a throwaway wrapper document, then copy the children in.
        $tmp = new DOMDocument();
        if (!__dom_load($tmp->__d, '<r>' . (string)$data . '</r>', 0)) { return false; }
        $root = __dom_doc_element($tmp->__d);
        foreach (__dom_children($tmp->__d, $root) as $c) {
            $copied = __dom_copy($this->__d, $tmp->__d, $c, 1);
            __dom_mutate($this->__d, 0, $this->__n, $copied, -1);
        }
        return true;
    }
}

class DOMDocumentType extends DOMNode {
    public function __get($name) {
        if ($name === 'name') {
            $i = __dom_info($this->__d, $this->__n);
            return $i[1];
        }
        if ($name === 'publicId' || $name === 'systemId') { return ''; }
        return parent::__get($name);
    }
}

class DOMImplementation {
    public function hasFeature($feature, $version) { return true; }
    public function createDocument($namespace = null, $qualifiedName = '', $doctype = null) {
        $doc = new DOMDocument();
        if ($qualifiedName !== '') {
            $doc->appendChild($doc->createElement($qualifiedName));
        }
        return $doc;
    }
}

class DOMXPath {
    public $document;
    public $__ns = array();
    public function __construct($document, $registerNodeNS = true) {
        $this->document = $document;
    }
    public function registerNamespace($prefix, $namespace) {
        $this->__ns[(string)$prefix] = (string)$namespace;
        return true;
    }
    public function query($expression, $contextNode = null, $registerNodeNS = true) {
        $r = __dom_xpath($this->document->__d, $contextNode === null ? -1 : $contextNode->__n, (string)$expression, $this->__ns);
        if (!is_array($r)) { return false; }
        return DOMNodeList::__make($this->__wrapAll($r));
    }
    public function evaluate($expression, $contextNode = null, $registerNodeNS = true) {
        $r = __dom_xpath($this->document->__d, $contextNode === null ? -1 : $contextNode->__n, (string)$expression, $this->__ns);
        if (is_array($r)) { return DOMNodeList::__make($this->__wrapAll($r)); }
        return $r;
    }
    public function __wrapAll($items) {
        $out = array();
        foreach ($items as $it) {
            if ($it[0] === 'a') {
                $out[] = DOMAttr::__wrapAttr($this->document->__d, $it[1], $it[2]);
            } else {
                $out[] = DOMNode::__wrap($this->document->__d, $it[1]);
            }
        }
        return $out;
    }
    public static function quote($str) {
        $s = (string)$str;
        if (strpos($s, "'") === false) { return "'" . $s . "'"; }
        if (strpos($s, '"') === false) { return '"' . $s . '"'; }
        // Mixed quotes: concat() form, exactly like PHP 8.4's implementation.
        $parts = explode("'", $s);
        $enc = array();
        foreach ($parts as $k => $p) {
            if ($k > 0) { $enc[] = '"\'"'; }
            if ($p !== '') { $enc[] = "'" . $p . "'"; }
        }
        return 'concat(' . implode(',', $enc) . ')';
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

thread_local! {
    /// One lowering of the prelude per thread. Parsing + hoisting the whole
    /// embedded PHP prelude is a large constant, and every `include`/`eval`
    /// unit needs (a clone of) its products — re-lowering it per included file
    /// dominates an autoload storm (PHPUnit's `preload()` requires ~1200 files).
    static PRELUDE_CACHE: std::cell::OnceCell<LoweredPrelude> = const { std::cell::OnceCell::new() };
}

/// Lower [`PRELUDE_SRC`] once per thread (cached) and return a clone of its
/// owned class table + name→id index (step 20) plus the global functions it
/// declares (step 35: the procedural date API). Function/`new` call sites
/// resolve by *name* (the evaluator rebuilds its `fn_index`/class table from
/// `Program`), so the prelude bodies need no index fix-up after being merged in.
fn lower_prelude() -> LoweredPrelude {
    PRELUDE_CACHE.with(|c| c.get_or_init(lower_prelude_uncached).clone())
}

/// The prelude's *functions* only (table + name→index), for the seeded
/// (`include`/`eval`) lowering path — it takes its classes from the seed image,
/// so cloning the cached prelude classes there would be pure waste.
fn prelude_functions() -> (Vec<FnDecl>, HashMap<Vec<u8>, usize>) {
    PRELUDE_CACHE.with(|c| {
        let p = c.get_or_init(lower_prelude_uncached);
        (p.2.clone(), p.3.clone())
    })
}

fn lower_prelude_uncached() -> LoweredPrelude {
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
    /// `/** ... */` docblock trivia spans of the unit, `(start, end)` byte
    /// offsets in lexical order — the parser drops comments from the AST, so
    /// [`Self::doc_for`] re-attaches them to declarations Zend-style.
    docs: Vec<(u32, u32)>,
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
            docs: Vec::new(),
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

    /// Resolve a *called* function's name to its primary name plus an optional
    /// global fallback, mirroring [`Self::resolve_const_fetch`]. An **unqualified**
    /// call inside a namespace primarily names `CURNS\foo` and falls back to the
    /// global `foo` — PHP tries the namespaced function first at run time, then the
    /// global one, so a namespaced function defined in *another* compilation unit
    /// (autoloaded / included) still binds. Unlike [`Self::resolve_fn_name`] this
    /// does *not* consult `fn_index`: whether `CURNS\foo` is a hoisted user
    /// function, a builtin, or defined elsewhere is decided by the call lowering.
    /// Qualified / fully-qualified / imported and global-scope names have no
    /// fallback.
    fn resolve_fn_call(&self, id: &Identifier) -> (Box<[u8]>, Option<Box<[u8]>>) {
        match id {
            Identifier::FullyQualified(f) => (strip_leading_backslash(f.value).into(), None),
            Identifier::Qualified(q) => (self.resolve_qualified(q.value), None),
            Identifier::Local(l) => {
                if let Some(fqn) = self.use_functions.get(&l.value.to_ascii_lowercase()) {
                    (fqn.clone().into(), None)
                } else if self.cur_namespace.is_empty() {
                    (l.value.into(), None)
                } else {
                    (join_ns(&self.cur_namespace, l.value), Some(l.value.into()))
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

    /// The `/** ... */` doc comment lexically attached to a declaration starting
    /// at byte `decl_start` (Zend semantics): the closest preceding docblock
    /// separated from the declaration only by whitespace, attribute lists
    /// (`#[...]`) and member-modifier keywords. `None` otherwise — "none", never
    /// a wrong one.
    fn doc_for(&self, decl_start: u32) -> Option<Box<[u8]>> {
        let idx = self.docs.partition_point(|&(_, end)| end <= decl_start);
        let &(ds, de) = self.docs.get(idx.checked_sub(1)?)?;
        let src = self.file.contents.as_ref();
        let target = decl_start as usize;
        let mut i = de as usize;
        while i < target {
            let b = src[i];
            if b.is_ascii_whitespace() {
                i += 1;
            } else if b == b'#' && src.get(i + 1) == Some(&b'[') {
                // Skip an attribute list, tracking bracket depth and quoted
                // strings (a `']'` inside an argument must not end it).
                i += 2;
                let mut depth = 1usize;
                while i < target && depth > 0 {
                    match src[i] {
                        b'[' => depth += 1,
                        b']' => depth -= 1,
                        q @ (b'\'' | b'"') => {
                            i += 1;
                            while i < target && src[i] != q {
                                if src[i] == b'\\' {
                                    i += 1;
                                }
                                i += 1;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            } else if b.is_ascii_alphabetic() {
                let start = i;
                while i < target && (src[i].is_ascii_alphanumeric() || src[i] == b'_') {
                    i += 1;
                }
                const MODIFIERS: &[&[u8]] = &[
                    b"public", b"protected", b"private", b"static", b"final", b"abstract",
                    b"readonly", b"var",
                ];
                if !MODIFIERS.iter().any(|m| src[start..i].eq_ignore_ascii_case(m)) {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(src.get(ds as usize..de as usize)?.to_vec().into())
    }

    /// PHP requires a `namespace` declaration to be the very first statement in
    /// the script — only `declare(...)` (and empty `;` statements) may precede it.
    /// When the *first* namespace is preceded by anything else (output, code,
    /// inline HTML, `use`, `const`, …) PHP raises a compile-time fatal; emit it
    /// verbatim rather than silently accepting the misplaced namespace (or
    /// panicking downstream). Subsequent `namespace` declarations are unrestricted
    /// (`namespace A; $x=1; namespace B;` is legal), so the scan stops at the first.
    fn check_namespace_first(&self, stmts: &[Statement]) -> Result<(), LowerError> {
        let mut seen_other = false;
        for s in stmts {
            match s {
                Statement::Namespace(_) => {
                    if seen_other {
                        return Err(LowerError::Fatal {
                            message: "Namespace declaration statement has to be the \
                                      very first statement or after any declare call \
                                      in the script"
                                .to_string(),
                            line: self.line_of(s.span()),
                        });
                    }
                    return Ok(());
                }
                // The open tag, `declare(...)`, and empty `;` statements may precede
                // a namespace; anything else makes it no longer the first statement.
                Statement::OpeningTag(_) | Statement::Declare(_) | Statement::Noop(_) => {}
                _ => seen_other = true,
            }
        }
        Ok(())
    }


    /// The unit's source path (matches `Program::file`), stamped onto each lowered
    /// `FnDecl` so traces/`getFile()` report the defining file across includes.
    fn unit_file(&self) -> Box<[u8]> {
        self.file.name.as_ref().into()
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
        b"PHP_EXTRA_VERSION" => str_lit(b""),
        b"PHP_SAPI" => str_lit(b"cli"),
        // Build flavour — PHP 8.5 reports these as bool. phpr is a non-debug,
        // non-thread-safe build, matching the oracle.
        b"PHP_DEBUG" => ExprKind::Bool(false),
        b"PHP_ZTS" => ExprKind::Bool(false),
        // The ZEND_* spellings exist on every build (Symfony Process gates its
        // cwd fallback on defined('ZEND_THREAD_SAFE') — the constant must BE
        // defined, whatever its value).
        b"ZEND_THREAD_SAFE" => ExprKind::Bool(false),
        b"ZEND_DEBUG_BUILD" => ExprKind::Bool(false),
        b"PHP_MAXPATHLEN" => ExprKind::Int(1024),
        // phpinfo() section selectors. phpr's `phpinfo` is a stub, but callers
        // (Composer's DiagnoseCommand does `phpinfo(INFO_GENERAL)`) still name the
        // flags, so they must resolve to their canonical bitmask values.
        b"INFO_GENERAL" => ExprKind::Int(1),
        b"INFO_CREDITS" => ExprKind::Int(2),
        b"INFO_CONFIGURATION" => ExprKind::Int(4),
        b"INFO_MODULES" => ExprKind::Int(8),
        b"INFO_ENVIRONMENT" => ExprKind::Int(16),
        b"INFO_VARIABLES" => ExprKind::Int(32),
        b"INFO_LICENSE" => ExprKind::Int(64),
        b"INFO_ALL" => ExprKind::Int(0x7FFF_FFFF),
        // Nominal platform-library versions. phpr does not link these C libraries
        // (it uses Rust crates for regex, mbstring, and TLS via rustls), so these
        // are representative constants for consumers that read them (e.g. Composer's
        // PlatformRepository builds `lib-*` packages from them); they are not tied
        // to any host install and need not match a given machine's libraries.
        b"PCRE_VERSION" => str_lit(b"10.44 2024-06-07"),
        b"MB_ONIGURUMA_VERSION" => str_lit(b"6.9.9"),
        b"OPENSSL_VERSION_TEXT" => str_lit(b"OpenSSL 3.0.0 7 sep 2023"),
        b"OPENSSL_VERSION_NUMBER" => ExprKind::Int(0x3000_0000),
        // Platform identity (macOS oracle, matching `DIRECTORY_SEPARATOR`).
        b"PHP_OS" => str_lit(b"Darwin"),
        b"PHP_OS_FAMILY" => str_lit(b"Darwin"),
        b"DIRECTORY_SEPARATOR" => str_lit(b"/"),
        b"PATH_SEPARATOR" => str_lit(b":"),
        // Global DATE_* format constants (ext/date mirrors of the
        // DateTimeInterface class constants; values from the oracle).
        b"DATE_ATOM" => str_lit(b"Y-m-d\\TH:i:sP"),
        b"DATE_COOKIE" => str_lit(b"l, d-M-Y H:i:s T"),
        b"DATE_ISO8601" => str_lit(b"Y-m-d\\TH:i:sO"),
        b"DATE_ISO8601_EXPANDED" => str_lit(b"X-m-d\\TH:i:sP"),
        b"DATE_RFC822" => str_lit(b"D, d M y H:i:s O"),
        b"DATE_RFC850" => str_lit(b"l, d-M-y H:i:s T"),
        b"DATE_RFC1036" => str_lit(b"D, d M y H:i:s O"),
        b"DATE_RFC1123" => str_lit(b"D, d M Y H:i:s O"),
        b"DATE_RFC7231" => str_lit(b"D, d M Y H:i:s \\G\\M\\T"),
        b"DATE_RFC2822" => str_lit(b"D, d M Y H:i:s O"),
        b"DATE_RFC3339" => str_lit(b"Y-m-d\\TH:i:sP"),
        b"DATE_RFC3339_EXTENDED" => str_lit(b"Y-m-d\\TH:i:s.vP"),
        b"DATE_RSS" => str_lit(b"D, d M Y H:i:s O"),
        b"DATE_W3C" => str_lit(b"Y-m-d\\TH:i:sP"),
        // setlocale() category selectors (macOS values, matching the oracle).
        b"LC_ALL" => ExprKind::Int(0),
        b"LC_COLLATE" => ExprKind::Int(1),
        b"LC_CTYPE" => ExprKind::Int(2),
        b"LC_MONETARY" => ExprKind::Int(3),
        b"LC_NUMERIC" => ExprKind::Int(4),
        b"LC_TIME" => ExprKind::Int(5),
        b"LC_MESSAGES" => ExprKind::Int(6),
        // ext/dom node-type codes (W3C nodeType) + ext/libxml option bitflags
        // and error levels, with the canonical values PHP documents. phpr's DOM
        // parser ignores most option flags (callers pass them for libxml).
        b"XML_ELEMENT_NODE" => ExprKind::Int(1),
        b"XML_ATTRIBUTE_NODE" => ExprKind::Int(2),
        b"XML_TEXT_NODE" => ExprKind::Int(3),
        b"XML_CDATA_SECTION_NODE" => ExprKind::Int(4),
        b"XML_ENTITY_REF_NODE" => ExprKind::Int(5),
        b"XML_ENTITY_NODE" => ExprKind::Int(6),
        b"XML_PI_NODE" => ExprKind::Int(7),
        b"XML_COMMENT_NODE" => ExprKind::Int(8),
        b"XML_DOCUMENT_NODE" => ExprKind::Int(9),
        b"XML_DOCUMENT_TYPE_NODE" => ExprKind::Int(10),
        b"XML_DOCUMENT_FRAG_NODE" => ExprKind::Int(11),
        b"XML_NOTATION_NODE" => ExprKind::Int(12),
        b"XML_HTML_DOCUMENT_NODE" => ExprKind::Int(13),
        b"XML_DTD_NODE" => ExprKind::Int(14),
        b"XML_ELEMENT_DECL_NODE" => ExprKind::Int(15),
        b"XML_ATTRIBUTE_DECL_NODE" => ExprKind::Int(16),
        b"XML_ENTITY_DECL_NODE" => ExprKind::Int(17),
        b"XML_NAMESPACE_DECL_NODE" => ExprKind::Int(18),
        b"XML_ATTRIBUTE_CDATA" => ExprKind::Int(1),
        b"XML_ATTRIBUTE_ID" => ExprKind::Int(2),
        b"LIBXML_NOENT" => ExprKind::Int(2),
        b"LIBXML_DTDLOAD" => ExprKind::Int(4),
        b"LIBXML_DTDATTR" => ExprKind::Int(8),
        b"LIBXML_DTDVALID" => ExprKind::Int(16),
        b"LIBXML_NOERROR" => ExprKind::Int(32),
        b"LIBXML_NOWARNING" => ExprKind::Int(64),
        b"LIBXML_PEDANTIC" => ExprKind::Int(128),
        b"LIBXML_NOBLANKS" => ExprKind::Int(256),
        b"LIBXML_XINCLUDE" => ExprKind::Int(1024),
        b"LIBXML_NONET" => ExprKind::Int(2048),
        b"LIBXML_NSCLEAN" => ExprKind::Int(8192),
        b"LIBXML_NOCDATA" => ExprKind::Int(16384),
        b"LIBXML_NOEMPTYTAG" => ExprKind::Int(4),
        b"LIBXML_NOXMLDECL" => ExprKind::Int(2),
        b"LIBXML_COMPACT" => ExprKind::Int(65536),
        b"LIBXML_PARSEHUGE" => ExprKind::Int(524288),
        b"LIBXML_BIGLINES" => ExprKind::Int(4194304),
        b"LIBXML_SCHEMA_CREATE" => ExprKind::Int(1),
        b"LIBXML_HTML_NOIMPLIED" => ExprKind::Int(8192),
        b"LIBXML_HTML_NODEFDTD" => ExprKind::Int(4),
        b"LIBXML_ERR_NONE" => ExprKind::Int(0),
        b"LIBXML_ERR_WARNING" => ExprKind::Int(1),
        b"LIBXML_ERR_ERROR" => ExprKind::Int(2),
        b"LIBXML_ERR_FATAL" => ExprKind::Int(3),
        // Nominal libxml version (phpr parses XML with a Rust crate; these are
        // representative, like PCRE_VERSION above).
        b"LIBXML_VERSION" => ExprKind::Int(21300),
        b"LIBXML_LOADED_VERSION" => ExprKind::Int(21300),
        b"LIBXML_DOTTED_VERSION" => str_lit(b"2.13.0"),
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
        // parse_url() component selectors.
        b"PHP_URL_SCHEME" => ExprKind::Int(0),
        b"PHP_URL_HOST" => ExprKind::Int(1),
        b"PHP_URL_PORT" => ExprKind::Int(2),
        b"PHP_URL_USER" => ExprKind::Int(3),
        b"PHP_URL_PASS" => ExprKind::Int(4),
        b"PHP_URL_PATH" => ExprKind::Int(5),
        b"PHP_URL_QUERY" => ExprKind::Int(6),
        b"PHP_URL_FRAGMENT" => ExprKind::Int(7),
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
        // hash_init() flag.
        b"HASH_HMAC" => ExprKind::Int(1),
        // http_build_query() encoding selectors.
        b"PHP_QUERY_RFC1738" => ExprKind::Int(1),
        b"PHP_QUERY_RFC3986" => ExprKind::Int(2),
        // POSIX signal numbers (macOS values, brew-php-pinned; ext/pcntl defines
        // these — monolog's SignalHandler references them even when the pcntl
        // *functions* are unavailable). SIGEMT is undefined on macOS PHP too.
        b"SIGHUP" => ExprKind::Int(1),
        b"SIGINT" => ExprKind::Int(2),
        b"SIGQUIT" => ExprKind::Int(3),
        b"SIGILL" => ExprKind::Int(4),
        b"SIGTRAP" => ExprKind::Int(5),
        b"SIGABRT" => ExprKind::Int(6),
        b"SIGFPE" => ExprKind::Int(8),
        b"SIGKILL" => ExprKind::Int(9),
        b"SIGBUS" => ExprKind::Int(10),
        b"SIGSEGV" => ExprKind::Int(11),
        b"SIGSYS" => ExprKind::Int(12),
        b"SIGPIPE" => ExprKind::Int(13),
        b"SIGALRM" => ExprKind::Int(14),
        b"SIGTERM" => ExprKind::Int(15),
        b"SIGURG" => ExprKind::Int(16),
        b"SIGSTOP" => ExprKind::Int(17),
        b"SIGTSTP" => ExprKind::Int(18),
        b"SIGCONT" => ExprKind::Int(19),
        b"SIGCHLD" => ExprKind::Int(20),
        b"SIGTTIN" => ExprKind::Int(21),
        b"SIGTTOU" => ExprKind::Int(22),
        b"SIGIO" => ExprKind::Int(23),
        b"SIGXCPU" => ExprKind::Int(24),
        b"SIGXFSZ" => ExprKind::Int(25),
        b"SIGVTALRM" => ExprKind::Int(26),
        b"SIGPROF" => ExprKind::Int(27),
        b"SIGWINCH" => ExprKind::Int(28),
        b"SIGINFO" => ExprKind::Int(29),
        b"SIGUSR1" => ExprKind::Int(30),
        b"SIGUSR2" => ExprKind::Int(31),
        b"SIG_DFL" => ExprKind::Int(0),
        b"SIG_IGN" => ExprKind::Int(1),
        b"SIG_ERR" => ExprKind::Int(-1),
        b"SIG_BLOCK" => ExprKind::Int(1),
        b"SIG_UNBLOCK" => ExprKind::Int(2),
        b"SIG_SETMASK" => ExprKind::Int(3),
        // flock operations / debug_backtrace flags (oracle-pinned).
        // (`LOCK_EX` lives with the file_put_contents flags above.)
        b"LOCK_SH" => ExprKind::Int(1),
        b"LOCK_UN" => ExprKind::Int(3),
        b"LOCK_NB" => ExprKind::Int(4),
        b"DEBUG_BACKTRACE_PROVIDE_OBJECT" => ExprKind::Int(1),
        b"DEBUG_BACKTRACE_IGNORE_ARGS" => ExprKind::Int(2),
        // syslog() priorities / facilities / options (macOS values, oracle-pinned).
        b"LOG_EMERG" => ExprKind::Int(0),
        b"LOG_ALERT" => ExprKind::Int(1),
        b"LOG_CRIT" => ExprKind::Int(2),
        b"LOG_ERR" => ExprKind::Int(3),
        b"LOG_WARNING" => ExprKind::Int(4),
        b"LOG_NOTICE" => ExprKind::Int(5),
        b"LOG_INFO" => ExprKind::Int(6),
        b"LOG_DEBUG" => ExprKind::Int(7),
        b"LOG_AUTH" => ExprKind::Int(32),
        b"LOG_AUTHPRIV" => ExprKind::Int(80),
        b"LOG_CRON" => ExprKind::Int(72),
        b"LOG_DAEMON" => ExprKind::Int(24),
        b"LOG_KERN" => ExprKind::Int(0),
        b"LOG_LPR" => ExprKind::Int(48),
        b"LOG_MAIL" => ExprKind::Int(16),
        b"LOG_NEWS" => ExprKind::Int(56),
        b"LOG_SYSLOG" => ExprKind::Int(40),
        b"LOG_UUCP" => ExprKind::Int(64),
        b"LOG_USER" => ExprKind::Int(8),
        b"LOG_LOCAL0" => ExprKind::Int(128),
        b"LOG_LOCAL1" => ExprKind::Int(136),
        b"LOG_LOCAL2" => ExprKind::Int(144),
        b"LOG_LOCAL3" => ExprKind::Int(152),
        b"LOG_LOCAL4" => ExprKind::Int(160),
        b"LOG_LOCAL5" => ExprKind::Int(168),
        b"LOG_LOCAL6" => ExprKind::Int(176),
        b"LOG_LOCAL7" => ExprKind::Int(184),
        b"LOG_NDELAY" => ExprKind::Int(8),
        b"LOG_ODELAY" => ExprKind::Int(4),
        b"LOG_PERROR" => ExprKind::Int(32),
        b"LOG_PID" => ExprKind::Int(1),
        b"LOG_CONS" => ExprKind::Int(2),
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
        // array_change_key_case flags.
        b"CASE_LOWER" => ExprKind::Int(0),
        b"CASE_UPPER" => ExprKind::Int(1),
        // ext/sqlite3 (fetch modes, value types, open flags).
        b"SQLITE3_ASSOC" => ExprKind::Int(1),
        b"SQLITE3_NUM" => ExprKind::Int(2),
        b"SQLITE3_BOTH" => ExprKind::Int(3),
        b"SQLITE3_INTEGER" => ExprKind::Int(1),
        b"SQLITE3_FLOAT" => ExprKind::Int(2),
        b"SQLITE3_TEXT" => ExprKind::Int(3),
        b"SQLITE3_BLOB" => ExprKind::Int(4),
        b"SQLITE3_NULL" => ExprKind::Int(5),
        b"SQLITE3_OPEN_READONLY" => ExprKind::Int(1),
        b"SQLITE3_OPEN_READWRITE" => ExprKind::Int(2),
        b"SQLITE3_OPEN_CREATE" => ExprKind::Int(4),
        // json_encode / json_decode flags (step 26).
        b"JSON_HEX_TAG" => ExprKind::Int(1),
        b"JSON_HEX_AMP" => ExprKind::Int(2),
        b"JSON_HEX_APOS" => ExprKind::Int(4),
        b"JSON_HEX_QUOT" => ExprKind::Int(8),
        b"JSON_FORCE_OBJECT" => ExprKind::Int(16),
        b"JSON_NUMERIC_CHECK" => ExprKind::Int(32),
        b"JSON_UNESCAPED_SLASHES" => ExprKind::Int(64),
        b"JSON_PRETTY_PRINT" => ExprKind::Int(128),
        b"JSON_UNESCAPED_UNICODE" => ExprKind::Int(256),
        b"JSON_PARTIAL_OUTPUT_ON_ERROR" => ExprKind::Int(512),
        b"JSON_PRESERVE_ZERO_FRACTION" => ExprKind::Int(1024),
        b"JSON_INVALID_UTF8_IGNORE" => ExprKind::Int(1_048_576),
        b"JSON_INVALID_UTF8_SUBSTITUTE" => ExprKind::Int(2_097_152),
        b"JSON_THROW_ON_ERROR" => ExprKind::Int(4_194_304),
        b"JSON_OBJECT_AS_ARRAY" => ExprKind::Int(1),
        b"JSON_BIGINT_AS_STRING" => ExprKind::Int(2),
        b"JSON_ERROR_NONE" => ExprKind::Int(0),
        b"JSON_ERROR_DEPTH" => ExprKind::Int(1),
        b"JSON_ERROR_STATE_MISMATCH" => ExprKind::Int(2),
        b"JSON_ERROR_CTRL_CHAR" => ExprKind::Int(3),
        b"JSON_ERROR_SYNTAX" => ExprKind::Int(4),
        b"JSON_ERROR_UTF8" => ExprKind::Int(5),
        b"JSON_ERROR_RECURSION" => ExprKind::Int(6),
        b"JSON_ERROR_INF_OR_NAN" => ExprKind::Int(7),
        b"JSON_ERROR_UNSUPPORTED_TYPE" => ExprKind::Int(8),
        b"JSON_ERROR_INVALID_PROPERTY_NAME" => ExprKind::Int(9),
        b"JSON_ERROR_UTF16" => ExprKind::Int(10),
        b"JSON_ERROR_NON_BACKED_ENUM" => ExprKind::Int(11),
        // preg flags (step 31).
        b"PREG_PATTERN_ORDER" => ExprKind::Int(1),
        b"PREG_SET_ORDER" => ExprKind::Int(2),
        b"PREG_OFFSET_CAPTURE" => ExprKind::Int(256),
        b"PREG_UNMATCHED_AS_NULL" => ExprKind::Int(512),
        b"PREG_SPLIT_NO_EMPTY" => ExprKind::Int(1),
        b"PREG_SPLIT_DELIM_CAPTURE" => ExprKind::Int(2),
        b"PREG_SPLIT_OFFSET_CAPTURE" => ExprKind::Int(4),
        b"PREG_GREP_INVERT" => ExprKind::Int(1),
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
        // The 689 ext/curl constants live in a generated sorted table
        // (lower/curl_consts.rs) rather than as match arms here.
        _ => return curl_consts::curl_constant(name).map(ExprKind::Int),
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
        // `return $obj->prop;` / `return $this->$name;` — a (non-nullsafe) property
        // access is a place `lower_place` accepts, so it returns a real reference
        // (D-13.3) rather than copying with the by-ref Notice.
        Expression::Access(mago_syntax::ast::Access::Property(_)) => true,
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
        // `A|B[|null]`: enforced when EVERY member is itself enforceable —
        // `null` folds into nullability, a single surviving member collapses
        // to a plain hint. Any unenforceable member (mixed, `self`, literal
        // `true`, a nested intersection, …) keeps the whole union unenforced.
        Hint::Union(_) => {
            let mut members: Vec<HintKind> = Vec::new();
            let mut nullable = false;
            if !collect_enforced_union(lo, hint, &mut members, &mut nullable) {
                return None;
            }
            return match members.len() {
                0 => None,
                1 => Some(TypeHint { kind: members.pop().expect("len checked"), nullable }),
                _ => Some(TypeHint { kind: HintKind::Union(members), nullable }),
            };
        }
        // Intersection / `self`/`parent`/`static` / `mixed` / `void` /
        // literal types stay unenforced (lowered to `None`) for now.
        _ => return None,
    };
    Some(TypeHint { kind, nullable: false })
}

/// Collect the enforceable members of a union hint, flattening nested unions
/// and folding `null` / `?T` into `nullable`. Returns `false` when any member
/// is not enforceable — the caller then leaves the whole union unenforced.
fn collect_enforced_union(
    lo: &Lowerer,
    hint: &Hint,
    out: &mut Vec<HintKind>,
    nullable: &mut bool,
) -> bool {
    match hint {
        Hint::Union(u) => {
            collect_enforced_union(lo, u.left, out, nullable)
                && collect_enforced_union(lo, u.right, out, nullable)
        }
        Hint::Parenthesized(p) => collect_enforced_union(lo, p.hint, out, nullable),
        Hint::Null(_) => {
            *nullable = true;
            true
        }
        other => match lower_hint(lo, other) {
            Some(TypeHint { kind, nullable: n }) => {
                if n {
                    *nullable = true;
                }
                out.push(kind);
                true
            }
            None => false,
        },
    }
}


/// Capture a *composite* (union/intersection) declared type for reflection only.
/// Returns `None` for any single type (which reflects through the enforced
/// [`TypeHint`]) — so this never participates in the coercion binder and cannot
/// change run-time behaviour. `A|B|null` / `A&B` flatten to their members.
fn lower_reflect_type(lo: &Lowerer, hint: &Hint) -> Option<crate::hir::ReflectType> {
    use crate::hir::ReflectType;
    match hint {
        Hint::Union(_) => {
            let mut out = Vec::new();
            collect_union_members(lo, hint, &mut out);
            Some(ReflectType::Union(out))
        }
        Hint::Intersection(_) => {
            let mut out = Vec::new();
            collect_intersection_members(lo, hint, &mut out);
            Some(ReflectType::Intersection(out))
        }
        Hint::Parenthesized(p) => lower_reflect_type(lo, p.hint),
        // `?T` where T is a special single type the enforced hint can't model.
        Hint::Nullable(n) => single_special(lo, n.hint).map(|(named, _)| ReflectType::Single(named, true)),
        // A bare special single type (`mixed`/`void`/`self`/…); scalars, `array`,
        // class names, etc. return `None` here and reflect via the enforced hint.
        other => single_special(lo, other).map(|(named, nullable)| ReflectType::Single(named, nullable)),
    }
}

/// A *special* single type (one the enforced [`TypeHint`] lowers to `None`) →
/// its reflection `(name, builtin)` plus implicit `allowsNull`. Returns `None`
/// for an ordinary type (scalar / `array` / class / object / …) so the enforced
/// hint keeps reflecting it.
fn single_special(lo: &Lowerer, hint: &Hint) -> Option<(crate::hir::ReflectNamed, bool)> {
    use crate::hir::ReflectNamed;
    let mk = |name: &[u8], builtin: bool, nullable: bool| {
        Some((ReflectNamed { name: name.to_vec().into_boxed_slice(), builtin }, nullable))
    };
    match hint {
        Hint::Mixed(_) => mk(b"mixed", true, true),
        Hint::Null(_) => mk(b"null", true, true),
        Hint::Void(_) => mk(b"void", true, false),
        Hint::Never(_) => mk(b"never", true, false),
        Hint::True(_) => mk(b"true", true, false),
        Hint::False(_) => mk(b"false", true, false),
        Hint::Static(_) => mk(b"static", false, false),
        Hint::Parent(_) => mk(b"parent", false, false),
        // `self` reflects as the enclosing class name (PHP resolves it).
        Hint::Self_(_) => Some((
            ReflectNamed {
                name: lo.cur_class.clone().unwrap_or_else(|| b"self".to_vec().into_boxed_slice()),
                builtin: false,
            },
            false,
        )),
        _ => None,
    }
}

fn collect_union_members(lo: &Lowerer, hint: &Hint, out: &mut Vec<crate::hir::ReflectNamed>) {
    match hint {
        Hint::Union(u) => {
            collect_union_members(lo, u.left, out);
            collect_union_members(lo, u.right, out);
        }
        Hint::Parenthesized(p) => collect_union_members(lo, p.hint, out),
        other => {
            if let Some(n) = reflect_named(lo, other) {
                out.push(n);
            }
        }
    }
}

fn collect_intersection_members(lo: &Lowerer, hint: &Hint, out: &mut Vec<crate::hir::ReflectNamed>) {
    match hint {
        Hint::Intersection(i) => {
            collect_intersection_members(lo, i.left, out);
            collect_intersection_members(lo, i.right, out);
        }
        Hint::Parenthesized(p) => collect_intersection_members(lo, p.hint, out),
        other => {
            if let Some(n) = reflect_named(lo, other) {
                out.push(n);
            }
        }
    }
}

/// One leaf of a composite type → its reflection name + `builtin` flag.
fn reflect_named(lo: &Lowerer, hint: &Hint) -> Option<crate::hir::ReflectNamed> {
    use crate::hir::ReflectNamed;
    let (name, builtin): (&[u8], bool) = match hint {
        Hint::Integer(_) => (b"int", true),
        Hint::Float(_) => (b"float", true),
        Hint::String(_) => (b"string", true),
        Hint::Bool(_) => (b"bool", true),
        Hint::Array(_) => (b"array", true),
        Hint::Callable(_) => (b"callable", true),
        Hint::Iterable(_) => (b"iterable", true),
        Hint::Object(_) => (b"object", true),
        Hint::Null(_) => (b"null", true),
        Hint::True(_) => (b"true", true),
        Hint::False(_) => (b"false", true),
        Hint::Void(_) => (b"void", true),
        Hint::Never(_) => (b"never", true),
        Hint::Mixed(_) => (b"mixed", true),
        Hint::Static(_) => (b"static", false),
        // `self` in a union reflects as the enclosing class name (PHP resolves it).
        Hint::Self_(_) => {
            return Some(ReflectNamed {
                name: lo.cur_class.clone().unwrap_or_else(|| b"self".to_vec().into_boxed_slice()),
                builtin: false,
            })
        }
        Hint::Parent(_) => (b"parent", false),
        Hint::Identifier(id) => return Some(ReflectNamed { name: lo.resolve_class(id), builtin: false }),
        _ => return None,
    };
    Some(ReflectNamed { name: name.into(), builtin })
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
