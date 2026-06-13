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
    AssignmentOperator, BinaryOperator, Expression, Literal, LiteralInteger, Statement,
    UnaryPostfixOperator, UnaryPrefixOperator, Variable,
};
use mago_syntax::parser::parse_file;

use crate::hir::{BinOp, CastKind, Expr, ExprKind, Line, Program, Slot, Stmt, StmtKind, UnOp};

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
    let body = low.lower_stmts(program.statements.as_slice())?;
    Ok(Program {
        body,
        slots: low.slots,
    })
}

struct Lowerer<'f> {
    file: &'f File,
    slots: Vec<Box<[u8]>>,
    index: HashMap<Vec<u8>, Slot>,
}

impl<'f> Lowerer<'f> {
    fn new(file: &'f File) -> Self {
        Lowerer {
            file,
            slots: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// 1-based source line for a span's start offset (`File::line_number` is 0-based).
    fn line_of(&self, span: Span) -> Line {
        self.file.line_number(span.start.offset) + 1
    }

    /// Resolve `$name` (name given *without* the leading `$`) to a stable slot.
    fn slot_for(&mut self, name: &[u8]) -> Slot {
        if let Some(&s) = self.index.get(name) {
            return s;
        }
        let s = self.slots.len() as Slot;
        self.slots.push(name.into());
        self.index.insert(name.to_vec(), s);
        s
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
            // `<?php` / `?>` markers carry no runtime behaviour.
            Statement::OpeningTag(_) | Statement::ClosingTag(_) => return Ok(None),

            Statement::Inline(inline) => StmtKind::InlineHtml(inline.value.into()),
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

            Statement::Break(node) => StmtKind::Break(self.lower_level(node.level, line)?),
            Statement::Continue(node) => StmtKind::Continue(self.lower_level(node.level, line)?),

            Statement::Return(node) => StmtKind::Return(match node.value {
                Some(e) => Some(self.lower_expr(e)?),
                None => None,
            }),

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
                let slot = self.assign_target(a.lhs, line)?;
                let rhs = Box::new(self.lower_expr(a.rhs)?);
                match a.operator {
                    AssignmentOperator::Assign(_) => ExprKind::Assign(slot, rhs),
                    AssignmentOperator::Coalesce(_) => ExprKind::AssignCoalesce(slot, rhs),
                    AssignmentOperator::Addition(_) => ExprKind::AssignOp(BinOp::Add, slot, rhs),
                    AssignmentOperator::Subtraction(_) => ExprKind::AssignOp(BinOp::Sub, slot, rhs),
                    AssignmentOperator::Multiplication(_) => ExprKind::AssignOp(BinOp::Mul, slot, rhs),
                    AssignmentOperator::Division(_) => ExprKind::AssignOp(BinOp::Div, slot, rhs),
                    AssignmentOperator::Modulo(_) => ExprKind::AssignOp(BinOp::Mod, slot, rhs),
                    AssignmentOperator::Exponentiation(_) => ExprKind::AssignOp(BinOp::Pow, slot, rhs),
                    AssignmentOperator::Concat(_) => ExprKind::AssignOp(BinOp::Concat, slot, rhs),
                    AssignmentOperator::BitwiseAnd(_) => ExprKind::AssignOp(BinOp::BitAnd, slot, rhs),
                    AssignmentOperator::BitwiseOr(_) => ExprKind::AssignOp(BinOp::BitOr, slot, rhs),
                    AssignmentOperator::BitwiseXor(_) => ExprKind::AssignOp(BinOp::BitXor, slot, rhs),
                    AssignmentOperator::LeftShift(_) => ExprKind::AssignOp(BinOp::Shl, slot, rhs),
                    AssignmentOperator::RightShift(_) => ExprKind::AssignOp(BinOp::Shr, slot, rhs),
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

    /// Resolve an assignment target to a slot. Only direct variables in Tier 1
    /// (array-element and property targets arrive in step 7 / Tier 2).
    fn assign_target(&mut self, lhs: &Expression, line: Line) -> Result<Slot, LowerError> {
        match lhs {
            Expression::Variable(Variable::Direct(d)) => Ok(self.slot_for(strip_dollar(d.name))),
            _ => Err(LowerError::Unsupported {
                what: "assignment target",
                line,
            }),
        }
    }
}

/// Strip the leading `$` from a mago direct-variable name (`b"$foo"` → `b"foo"`).
fn strip_dollar(name: &[u8]) -> &[u8] {
    if name.first() == Some(&b'$') {
        &name[1..]
    } else {
        name
    }
}

/// PHP integer literal → HIR. Values exceeding `i64::MAX` promote to float,
/// matching PHP's lexer (`0x...`, decimal, etc. all overflow to double).
fn lower_int(lit: &LiteralInteger, line: Line) -> Result<ExprKind, LowerError> {
    match lit.value {
        Some(v) if v <= i64::MAX as u64 => Ok(ExprKind::Int(v as i64)),
        Some(v) => Ok(ExprKind::Float(v as f64)),
        None => {
            // Beyond u64: PHP still promotes to float. Re-parse the decimal digits.
            let raw = std::str::from_utf8(lit.raw).map_err(|_| LowerError::Unsupported {
                what: "integer literal",
                line,
            })?;
            let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
            cleaned
                .parse::<f64>()
                .map(ExprKind::Float)
                .map_err(|_| LowerError::Unsupported {
                    what: "integer literal overflow",
                    line,
                })
        }
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
