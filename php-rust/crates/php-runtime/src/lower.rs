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
use std::collections::HashMap;

use bumpalo::Bump;
use mago_database::file::File;
use mago_span::{HasSpan, Span};
use mago_syntax::ast::{
    Argument, ArrayElement, AssignmentOperator, BinaryOperator, Call, Construct, Expression,
    ForeachTarget, Function, Identifier, Literal, LiteralInteger, MatchArm as AstMatchArm,
    Statement, UnaryPostfixOperator, UnaryPrefixOperator, Variable,
};
use mago_syntax::parser::parse_file;

use crate::hir::{
    ArrayElem, BinOp, Case, CastKind, Expr, ExprKind, FnDecl, GlobalBinding, Line, MatchArm, Param,
    Place, PlaceBase, PlaceStep, Program, Slot, Stmt, StmtKind, UnOp,
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

    let mut low = Lowerer::new(&file);
    // Hoist top-level function declarations first, so a call may textually
    // precede its definition (PHP's function hoisting). Bodies are lowered here;
    // the main pass below skips the declaration statements (they are no-ops).
    for s in program.statements.as_slice() {
        if let Statement::Function(func) = s {
            low.hoist_function(func)?;
        }
    }
    let body = low.lower_stmts(program.statements.as_slice())?;
    Ok(Program {
        body,
        file: name.into(),
        slots: low.globals.slots,
        functions: low.functions,
    })
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
}

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
}

impl<'f> Lowerer<'f> {
    fn new(file: &'f File) -> Self {
        Lowerer {
            file,
            globals: Scope::default(),
            locals: None,
            after_closing_tag: false,
            functions: Vec::new(),
            fn_index: HashMap::new(),
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

            Statement::Break(node) => StmtKind::Break(self.lower_level(node.level, line)?),
            Statement::Continue(node) => StmtKind::Continue(self.lower_level(node.level, line)?),

            Statement::Return(node) => StmtKind::Return(match node.value {
                Some(e) => Some(self.lower_expr(e)?),
                None => None,
            }),

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

    /// Lower a function body in a *fresh* local slot scope (PHP functions do not
    /// capture the enclosing scope). The outer scope is saved and restored even
    /// on error so the caller's slot table is never corrupted.
    fn lower_function(&mut self, func: &Function) -> Result<FnDecl, LowerError> {
        let line = self.line_of(func.span());
        let name: Box<[u8]> = func.name.value.into();
        if func.ampersand.is_some() {
            return Err(LowerError::Unsupported {
                what: "function returning by reference",
                line,
            });
        }

        // Install a fresh local overlay; the global scope stays reachable so a
        // global slot can be pre-registered from inside this body (D-12.1).
        // Save/restore the previous overlay so nested lowering nests correctly.
        let saved_locals = self.locals.replace(Scope::default());
        let saved_tag = std::mem::replace(&mut self.after_closing_tag, false);

        let inner = self.lower_function_body(func, line);

        // Reclaim the function's local scope and restore the outer one.
        let local_scope = std::mem::replace(&mut self.locals, saved_locals)
            .expect("local scope installed for function body");
        self.after_closing_tag = saved_tag;

        let (params, body) = inner?;
        Ok(FnDecl {
            name,
            params,
            body,
            slots: local_scope.slots,
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
        let mut params = Vec::new();
        for p in func.parameter_list.parameters.iter() {
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
            });
        }
        let body = self.lower_stmts(func.body.statements.as_slice())?;
        Ok((params, body))
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

            Expression::Variable(Variable::Direct(d)) => ExprKind::Var(self.slot_for(strip_dollar(d.name))),
            Expression::Variable(_) => {
                return Err(LowerError::Unsupported {
                    what: "variable variable",
                    line,
                })
            }

            Expression::Binary(b) => {
                let l = Box::new(self.lower_expr(b.lhs)?);
                let r = Box::new(self.lower_expr(b.rhs)?);
                match b.operator {
                    BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => ExprKind::And(l, r),
                    BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => ExprKind::Or(l, r),
                    BinaryOperator::LowXor(_) => ExprKind::Xor(l, r),
                    BinaryOperator::NullCoalesce(_) => ExprKind::Coalesce(l, r),
                    BinaryOperator::Instanceof(_) => {
                        return Err(LowerError::Unsupported {
                            what: "instanceof",
                            line,
                        })
                    }
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
                            // Both sides are places: a bare variable or an array
                            // element (step 11d-2). `lower_place` rejects anything
                            // that is not an lvalue.
                            let target = self.lower_place(a.lhs, line)?;
                            let source = self.lower_place(u.operand, line)?;
                            return Ok(Expr {
                                line,
                                kind: ExprKind::AssignRef { target, source },
                            });
                        }
                    }
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
            Expression::Variable(Variable::Direct(d)) => Ok(ExprKind::IncDec {
                slot: self.slot_for(strip_dollar(d.name)),
                inc,
                pre,
            }),
            _ => Err(LowerError::Unsupported {
                what: "increment/decrement of non-variable",
                line,
            }),
        }
    }

    /// Lower a call. Tier 1 supports only direct calls to a named function with
    /// positional arguments (builtins); methods, static calls, dynamic callees,
    /// and named/variadic arguments are deferred.
    fn lower_call(&mut self, call: &Call, line: Line) -> Result<ExprKind, LowerError> {
        let fc = match call {
            Call::Function(fc) => fc,
            _ => {
                return Err(LowerError::Unsupported {
                    what: "method/static call",
                    line,
                })
            }
        };
        let name = match fc.function {
            Expression::Identifier(id) => function_name(id),
            _ => {
                return Err(LowerError::Unsupported {
                    what: "dynamic function call",
                    line,
                })
            }
        };
        let mut args = Vec::new();
        for arg in fc.argument_list.arguments.iter() {
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
        Ok(ExprKind::Call {
            name: name.into(),
            args,
        })
    }

    /// Resolve an lvalue: a base variable plus a chain of index steps. `$x`
    /// yields an empty step list; `$a[k]`, `$a[]`, and nested forms append
    /// [`PlaceStep`]s. Property and `list()` targets stay out of Tier 1 scope.
    fn lower_place(&mut self, lhs: &Expression, line: Line) -> Result<Place, LowerError> {
        match lhs {
            Expression::Parenthesized(p) => self.lower_place(p.expression, line),
            Expression::Variable(Variable::Direct(d)) => Ok(Place {
                base: PlaceBase::Local(self.slot_for(strip_dollar(d.name))),
                steps: Vec::new(),
            }),
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
