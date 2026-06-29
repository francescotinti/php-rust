//! HIR lowering of expressions: the `lower_expr` dispatch, interpolation/
//! heredoc, calls, instantiation, member access, args, places and array
//! elements. Split out of `lower.rs` (step 61).

use mago_span::HasSpan;
use mago_syntax::ast::{
    Access, Argument, ArrayElement, AssignmentOperator, BinaryOperator, Call,
    ClassLikeConstantSelector, ClassLikeMemberSelector,
    CompositeString, Construct, DocumentIndentation, DocumentKind, DocumentString,
    Expression, Instantiation, Literal,
    MatchArm as AstMatchArm, PartialApplication,
    StringPart, UnaryPostfixOperator, UnaryPrefixOperator, Variable, Yield,
};

use crate::hir::{
    ArrayElem, BinOp, CastKind, ClassRef, Expr, ExprKind, Line, MatchArm, Place, PlaceBase, PlaceStep, Slot, UnOp,
};


use super::*;

/// The lowered form of a member selector (`->name`): a compile-time static name
/// or a runtime-evaluated dynamic name (`$obj->$n` / `$obj->{expr}`), step 51.
enum MemberSel {
    Static(Box<[u8]>),
    Dynamic(Expr),
}

impl<'f> Lowerer<'f> {
    // --- expressions ---

    pub(super) fn lower_expr_list<'a, I>(&mut self, it: I) -> Result<Vec<Expr>, LowerError>
    where
        I: Iterator<Item = &'a &'a Expression<'a>>,
    {
        let mut out = Vec::new();
        for e in it {
            out.push(self.lower_expr(e)?);
        }
        Ok(out)
    }

    pub(super) fn lower_expr(&mut self, e: &Expression) -> Result<Expr, LowerError> {
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
                } else if let Some(idx) = crate::bytecode::superglobal_index(name) {
                    // Superglobals (`$_SERVER`, …) are auto-global: in any scope
                    // they read the VM-level superglobal store by name, so they
                    // resolve identically across units/frames (incl. included
                    // files). `$GLOBALS` keeps its own dedicated slot handling.
                    ExprKind::Superglobal(idx)
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
                    let class = self.class_ref_of(b.rhs, line)?;
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
                    let class = self.class_ref_of(sp.class, line)?;
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
                // `[$a, $b] = rhs` / `list($a, $b) = rhs` array destructuring: the
                // LHS is an array/list *pattern*, only valid with plain `=` (step 51).
                if let AssignmentOperator::Assign(_) = a.operator {
                    match a.lhs {
                        Expression::Array(arr) => {
                            return Ok(Expr {
                                line,
                                kind: self.lower_destructure_assign(arr.elements.iter(), a.rhs, line)?,
                            })
                        }
                        Expression::List(l) => {
                            return Ok(Expr {
                                line,
                                kind: self.lower_destructure_assign(l.elements.iter(), a.rhs, line)?,
                            })
                        }
                        Expression::LegacyArray(la) => {
                            return Ok(Expr {
                                line,
                                kind: self.lower_destructure_assign(la.elements.iter(), a.rhs, line)?,
                            })
                        }
                        _ => {}
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
                if let (PlaceBase::Local(slot), true) = (&place.base, place.steps.is_empty()) {
                    let slot = *slot;
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

            // `input |> callable` (PHP 8.5): `callable(input)`, operands evaluated
            // left-to-right. The callable is any expression resolving to a callable
            // (string name, closure, first-class callable, `[obj, m]`).
            Expression::Pipe(p) => ExprKind::Pipe {
                input: Box::new(self.lower_expr(p.input)?),
                callable: Box::new(self.lower_expr(p.callable)?),
            },

            // `new ClassName(args)` (step 19, D-19.6). Tier-1 resolves the class
            // as a literal identifier; `new $var` / `new self` / `new static`
            // arrive in later sub-steps.
            Expression::Instantiation(inst) => self.lower_instantiation(inst, line)?,

            // `new class(args) extends P implements I { … }` (step 51).
            Expression::AnonymousClass(anon) => self.lower_anonymous_class(anon, line)?,

            // `clone $obj` (step 51).
            Expression::Clone(c) => ExprKind::Clone(Box::new(self.lower_expr(c.object)?)),

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

            // Magic constants (`__LINE__`, `__CLASS__`, …) are compile-time in
            // PHP: substitute each to a literal from the current file/line and
            // the lexical class/function/trait scope tracked above (step 49).
            Expression::MagicConstant(m) => self.lower_magic_constant(m, line),

            // A bare `NAME` constant: known engine constants fold to a literal
            // here (D-18.7); any other name becomes a runtime `Const` read,
            // resolved against `define()`'d constants at eval time (step 49c).
            Expression::ConstantAccess(ca) => {
                // Engine constants (`true`/`PHP_INT_MAX`/…) fold here, on the bare
                // last segment so they resolve regardless of the current namespace
                // (they are global, case-insensitive for the language ones).
                if let Some(kind) = resolve_constant(bare_last_segment(&ca.name)) {
                    kind
                } else {
                    // Otherwise a runtime read. An *unqualified* name inside a
                    // namespace resolves to `CURNS\NAME`, falling back to the global
                    // `NAME` (step 50); the namespaced name is what an "Undefined
                    // constant" error reports, like PHP.
                    let (name, fallback) = self.resolve_const_fetch(&ca.name);
                    ExprKind::Const { name, fallback }
                }
            }

            Expression::Array(arr) => ExprKind::Array(self.lower_array_elements(arr.elements.iter(), line)?),
            Expression::LegacyArray(arr) => {
                ExprKind::Array(self.lower_array_elements(arr.elements.iter(), line)?)
            }

            Expression::ArrayAccess(aa) => {
                // `$GLOBALS['x']` reads as the global slot directly; a nested
                // `$GLOBALS['x'][k]` becomes `Index { base: GlobalVar, .. }`
                // since the inner access lowers to `GlobalVar` (D-12.3).
                if let Some(key) = globals_key(aa.array, aa.index) {
                    // `$GLOBALS['_SERVER']` aliases the data superglobal store so
                    // it stays consistent with a bare `$_SERVER`.
                    if let Some(idx) = crate::bytecode::superglobal_index(&key) {
                        ExprKind::Superglobal(idx)
                    } else {
                        ExprKind::GlobalVar(self.globals.slot_for(&key))
                    }
                } else {
                    let index = match aa.index {
                        // A bare identifier as an array index only arises from
                        // string interpolation (`"$a[k]"`), where mago rewrites
                        // the unquoted key to an identifier; it is a string key,
                        // not a constant (step 25).
                        Expression::Identifier(id) => {
                            Expr { line, kind: ExprKind::Str(bare_last_segment(id).into()) }
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
                        places.push(self.lower_test_place(v, line)?);
                    }
                    ExprKind::Isset(places)
                }
                Construct::Empty(em) => ExprKind::Empty(self.lower_test_place(em.value, line)?),
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
                Construct::Eval(ev) => ExprKind::Eval(Box::new(self.lower_expr(ev.value)?)),
                Construct::Include(i) => ExprKind::Include {
                    mode: crate::hir::IncludeMode::Include,
                    path: Box::new(self.lower_expr(i.value)?),
                },
                Construct::IncludeOnce(i) => ExprKind::Include {
                    mode: crate::hir::IncludeMode::IncludeOnce,
                    path: Box::new(self.lower_expr(i.value)?),
                },
                Construct::Require(r) => ExprKind::Include {
                    mode: crate::hir::IncludeMode::Require,
                    path: Box::new(self.lower_expr(r.value)?),
                },
                Construct::RequireOnce(r) => ExprKind::Include {
                    mode: crate::hir::IncludeMode::RequireOnce,
                    path: Box::new(self.lower_expr(r.value)?),
                },
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
                    what: expr_variant_name(e),
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
                ExprKind::Suppress(Box::new(self.lower_expr(operand)?))
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
                class: self.class_ref_of(sp.class, line)?,
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

    /// Lower a member selector (`->name`) that may be dynamic. A static identifier
    /// yields [`MemberSel::Static`]; a `$obj->$n` / `$obj->{expr}` form is lowered
    /// to an expression evaluated to the member name at runtime (step 51).
    fn member_sel(
        &mut self,
        sel: &ClassLikeMemberSelector,
        line: Line,
    ) -> Result<MemberSel, LowerError> {
        match sel {
            ClassLikeMemberSelector::Identifier(id) => Ok(MemberSel::Static(id.value.into())),
            ClassLikeMemberSelector::Variable(v) => {
                // `$obj->$n`: the selector is a plain variable. A variable-variable
                // selector (`$obj->$$x`) stays unsupported.
                let kind = match v {
                    Variable::Direct(d) => {
                        let nm = strip_dollar(d.name);
                        if nm == b"this" {
                            ExprKind::This
                        } else {
                            ExprKind::Var(self.slot_for(nm))
                        }
                    }
                    _ => {
                        return Err(LowerError::Unsupported {
                            what: "variable variable member name",
                            line,
                        })
                    }
                };
                Ok(MemberSel::Dynamic(Expr { line, kind }))
            }
            ClassLikeMemberSelector::Expression(e) => {
                Ok(MemberSel::Dynamic(self.lower_expr(e.expression)?))
            }
            ClassLikeMemberSelector::Missing(_) => Err(LowerError::Unsupported {
                what: "missing member selector",
                line,
            }),
        }
    }

    /// Build the right method-call HIR node from a (possibly dynamic) selector.
    fn method_call_kind(
        &mut self,
        object: Box<Expr>,
        method: MemberSel,
        args: Vec<Expr>,
        named: Vec<(Box<[u8]>, Expr)>,
        nullsafe: bool,
    ) -> ExprKind {
        match method {
            MemberSel::Static(method) => ExprKind::MethodCall { object, method, args, named, nullsafe },
            MemberSel::Dynamic(method) => ExprKind::MethodCallDyn {
                object,
                method: Box::new(method),
                args,
                named,
                nullsafe,
            },
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
                let method = self.member_sel(&mc.method, line)?;
                let (args, named) = self.lower_args(&mc.argument_list, line)?;
                return Ok(self.method_call_kind(object, method, args, named, false));
            }
            Call::NullSafeMethod(mc) => {
                let object = Box::new(self.lower_expr(mc.object)?);
                let method = self.member_sel(&mc.method, line)?;
                let (args, named) = self.lower_args(&mc.argument_list, line)?;
                return Ok(self.method_call_kind(object, method, args, named, true));
            }
            // `Class::m()` / `self::m()` / `parent::m()` / `static::m()`.
            Call::StaticMethod(sm) => {
                // PHP 8.4 parent property-hook call: the class position is itself a
                // static-property access (`parent::$prop`) and the method is
                // `get`/`set` — `parent::$prop::get()` / `parent::$prop::set($v)`.
                if let Expression::Access(Access::StaticProperty(sp)) = sm.class {
                    let m = member_name(&sm.method, line)?;
                    let is_get = m.eq_ignore_ascii_case(b"get");
                    let is_set = m.eq_ignore_ascii_case(b"set");
                    if is_get || is_set {
                        let class = self.class_ref_of(sp.class, line)?;
                        let prop = static_prop_name(&sp.property, line)?;
                        let (args, _named) = self.lower_args(&sm.argument_list, line)?;
                        return Ok(ExprKind::ParentHookCall {
                            class,
                            prop: prop.into(),
                            set: is_set,
                            args,
                        });
                    }
                }
                let class = self.class_ref_of(sm.class, line)?;
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
            Expression::Identifier(id) => self.resolve_fn_name(id),
            other => {
                let callee = Box::new(self.lower_expr(other)?);
                let args = self.lower_positional_args(&fc.argument_list, line)?;
                return Ok(ExprKind::CallDynamic { callee, args });
            }
        };
        let (args, named) = self.lower_args(&fc.argument_list, line)?;
        Ok(ExprKind::Call {
            name,
            args,
            named,
        })
    }

    /// Lower a class-position expression to a [`ClassRef`] (step 19; dynamic
    /// forms added in step 48). `self`/`parent`/`static` and a bare identifier
    /// resolve statically; anything else (`$cls`, `$obj`, `Foo::CONST`, a call,
    /// …) becomes `ClassRef::Dynamic` and is resolved to a class at runtime.
    fn class_ref_of(&mut self, class: &Expression, _line: Line) -> Result<ClassRef, LowerError> {
        match class {
            Expression::Self_(_) => Ok(ClassRef::SelfClass),
            Expression::Parent(_) => Ok(ClassRef::Parent),
            Expression::Static(_) => Ok(ClassRef::Static),
            Expression::Identifier(id) => Ok(ClassRef::Named(self.resolve_class(id))),
            other => Ok(ClassRef::Dynamic(Box::new(self.lower_expr(other)?))),
        }
    }

    /// Lower `new ClassName(args)` (step 19, D-19.6). A dynamic class
    /// (`new $cls`) lowers to `ClassRef::Dynamic` (step 48).
    fn lower_instantiation(
        &mut self,
        inst: &Instantiation,
        line: Line,
    ) -> Result<ExprKind, LowerError> {
        let class = self.class_ref_of(inst.class, line)?;
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
            Access::Property(p) => {
                let object = self.lower_expr(p.object)?;
                match self.member_sel(&p.property, line)? {
                    MemberSel::Static(name) => {
                        if matches!(object.kind, ExprKind::This) {
                            self.note_this_prop(&name); // backing read inside a hook (step 50)
                        }
                        Ok(ExprKind::PropGet { object: Box::new(object), name, nullsafe: false })
                    }
                    MemberSel::Dynamic(name) => Ok(ExprKind::PropGetDyn {
                        object: Box::new(object),
                        name: Box::new(name),
                        nullsafe: false,
                    }),
                }
            }
            Access::NullSafeProperty(p) => {
                let object = Box::new(self.lower_expr(p.object)?);
                match self.member_sel(&p.property, line)? {
                    MemberSel::Static(name) => Ok(ExprKind::PropGet { object, name, nullsafe: true }),
                    MemberSel::Dynamic(name) => Ok(ExprKind::PropGetDyn {
                        object,
                        name: Box::new(name),
                        nullsafe: true,
                    }),
                }
            }
            // `Class::CONST` / `self::CONST` / `Class::class` (step 19-4).
            Access::ClassConstant(cc) => {
                let class = self.class_ref_of(cc.class, line)?;
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
                let class = self.class_ref_of(sp.class, line)?;
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
            Expression::Identifier(id) => self.resolve_fn_name(id),
            _ => {
                return Err(LowerError::Unsupported {
                    what: "dynamic first-class callable",
                    line,
                })
            }
        };
        Ok(ExprKind::FirstClassCallable(name))
    }

    /// Lower a dynamic call's argument list: plain positional arguments plus
    /// variadic spread (`$cb(...$a)`, expanded at run time into an argument
    /// array). Named arguments on a dynamic call stay out of scope (the callee's
    /// parameters are unknown at compile time). A positional after a spread is a
    /// PHP compile-time Fatal, surfaced here.
    fn lower_positional_args(
        &mut self,
        list: &mago_syntax::ast::ArgumentList,
        line: Line,
    ) -> Result<Vec<Expr>, LowerError> {
        let mut args = Vec::new();
        let mut saw_spread = false;
        for arg in list.arguments.iter() {
            match arg {
                Argument::Positional(p) if p.ellipsis.is_some() => {
                    saw_spread = true;
                    let inner = self.lower_expr(p.value)?;
                    args.push(Expr {
                        kind: ExprKind::Spread(Box::new(inner)),
                        line,
                    });
                }
                Argument::Positional(p) => {
                    if saw_spread {
                        return Err(LowerError::Fatal {
                            message: "Cannot use positional argument after argument unpacking"
                                .to_string(),
                            line,
                        });
                    }
                    args.push(self.lower_expr(p.value)?);
                }
                Argument::Named(_) => {
                    return Err(LowerError::Unsupported {
                        what: "named argument on a dynamic call",
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
    pub(super) fn lower_args(
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
    /// Lower an `isset()` / `empty()` operand. A *bare* static property reached
    /// through `self::`/`parent::`/`static::` (`empty(self::$stack)`) is a valid
    /// read-only test place: the class is statically resolved and the property is
    /// always visible from the current scope, so the compiler can read it through
    /// `static_prop_read` (no `StaticPropSet`, so no visibility-checked write on a
    /// read) without risking a fatal that `isset`/`empty` must never raise.
    ///
    /// A *named* (`A::$priv`) or *dynamic* (`$cls::$p`) class is deliberately left
    /// to the general path: `isset` must stay silent about an inaccessible private
    /// or an undefined dynamic class name, which a plain static read would not be,
    /// and a bare static property is not a general lvalue anyway (a write goes
    /// through `StaticPropAssign`). Every other operand defers to `lower_place`.
    pub(super) fn lower_test_place(
        &mut self,
        e: &Expression,
        line: Line,
    ) -> Result<Place, LowerError> {
        if let Expression::Access(Access::StaticProperty(sp)) = e {
            // Only the same-hierarchy keywords guarantee a fatal-free read.
            if matches!(
                sp.class,
                Expression::Self_(_) | Expression::Parent(_) | Expression::Static(_)
            ) {
                let class = self.class_ref_of(sp.class, line)?;
                let name = static_prop_name(&sp.property, line)?.into();
                return Ok(Place {
                    base: PlaceBase::StaticProp { class, name },
                    steps: Vec::new(),
                });
            }
        }
        // `isset(self::TABLE[$k])` / `empty(Foo::MAP[$k])` — an index into a class
        // constant array. A class constant is a read-only container, so it is a
        // valid test place: root the place at the constant and carry the index
        // steps; the compiler materialises the value and tests the offset.
        if let Some(place) = self.class_const_test_place(e, line)? {
            return Ok(place);
        }
        self.lower_place(e, line)
    }

    /// Recognise a class-constant-rooted read path for [`Self::lower_test_place`]:
    /// `Class::CONST` optionally followed by `[k]` index steps. Returns `None` for
    /// anything else (including a dynamic `Class::{$e}` constant name), so the
    /// caller falls back to the general lvalue path.
    fn class_const_test_place(
        &mut self,
        e: &Expression,
        line: Line,
    ) -> Result<Option<Place>, LowerError> {
        match e {
            // `self`/`parent`/`static` always; a *named* class (`Foo::C[...]`) too
            // — class constants are public by default and `isset(Foo::C[$k])` is a
            // common pattern (Composer's `BasePackage::STABILITIES[$k]`). A
            // *dynamic* class (`$c::C`) stays on the general path. Constant
            // visibility is not enforced here, so an `isset` on an inaccessible
            // private constant returns silently instead of PHP's fatal — a rare
            // edge accepted for the common public-constant case.
            Expression::Access(Access::ClassConstant(cc))
                if matches!(
                    cc.class,
                    Expression::Self_(_)
                        | Expression::Parent(_)
                        | Expression::Static(_)
                        | Expression::Identifier(_)
                ) =>
            {
                let name = match &cc.constant {
                    ClassLikeConstantSelector::Identifier(id) => id.value,
                    _ => return Ok(None),
                };
                let class = self.class_ref_of(cc.class, line)?;
                Ok(Some(Place {
                    base: PlaceBase::ClassConst { class, name: name.into() },
                    steps: Vec::new(),
                }))
            }
            Expression::ArrayAccess(aa) => {
                let Some(mut place) = self.class_const_test_place(aa.array, line)? else {
                    return Ok(None);
                };
                place.steps.push(PlaceStep::Index(self.lower_expr(aa.index)?));
                Ok(Some(place))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn lower_place(&mut self, lhs: &Expression, line: Line) -> Result<Place, LowerError> {
        match lhs {
            Expression::Parenthesized(p) => self.lower_place(p.expression, line),
            Expression::Variable(Variable::Direct(d)) => {
                let name = strip_dollar(d.name);
                let base = if name == b"this" {
                    PlaceBase::This
                } else if let Some(idx) = crate::bytecode::superglobal_index(name) {
                    // Auto-global: `$_SERVER[$k] = …` inside any scope writes the
                    // VM-level superglobal store by name (mirrors the read path).
                    PlaceBase::Superglobal(idx)
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
                    let base = if let Some(idx) = crate::bytecode::superglobal_index(&key) {
                        PlaceBase::Superglobal(idx)
                    } else {
                        PlaceBase::Global(self.globals.slot_for(&key))
                    };
                    return Ok(Place { base, steps: Vec::new() });
                }
                // `Class::$arr[k]` — an *indexed* static-property target. Only the
                // indexed form roots a `Place` at a static property (a bare
                // `Class::$p` write goes through `StaticPropAssign`, and a bare
                // read/`isset` stays unsupported as a place), so the base is built
                // here rather than in a standalone `lower_place` arm (step 19-4).
                if let Expression::Access(Access::StaticProperty(sp)) = aa.array {
                    let class = self.class_ref_of(sp.class, line)?;
                    let name = static_prop_name(&sp.property, line)?.into();
                    let index = PlaceStep::Index(self.lower_expr(aa.index)?);
                    return Ok(Place {
                        base: PlaceBase::StaticProp { class, name },
                        steps: vec![index],
                    });
                }
                let mut place = self.lower_place(aa.array, line)?;
                place.steps.push(PlaceStep::Index(self.lower_expr(aa.index)?));
                Ok(place)
            }
            Expression::ArrayAppend(ap) => {
                // `Class::$arr[]` — append to a static-property array (mirrors the
                // indexed `Class::$arr[k]` target above).
                if let Expression::Access(Access::StaticProperty(sp)) = ap.array {
                    let class = self.class_ref_of(sp.class, line)?;
                    let name = static_prop_name(&sp.property, line)?.into();
                    return Ok(Place {
                        base: PlaceBase::StaticProp { class, name },
                        steps: vec![PlaceStep::Append],
                    });
                }
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
                match self.member_sel(&p.property, line)? {
                    MemberSel::Static(name) => {
                        if matches!(place.base, PlaceBase::This) && place.steps.is_empty() {
                            self.note_this_prop(&name); // backing write inside a hook (step 50)
                        }
                        place.steps.push(PlaceStep::Prop(name));
                    }
                    MemberSel::Dynamic(name) => place.steps.push(PlaceStep::PropDyn(name)),
                }
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
    pub(super) fn foreach_slot(&mut self, target: &Expression, line: Line) -> Result<Slot, LowerError> {
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
    pub(super) fn foreach_value_slot(
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
                // `[...$src]` array spread (PHP 8.1): a keyless element whose value
                // is a `Spread`, expanded at run time (step 51).
                ArrayElement::Variadic(v) => out.push(ArrayElem {
                    key: None,
                    value: Expr {
                        line,
                        kind: ExprKind::Spread(Box::new(self.lower_expr(v.value)?)),
                    },
                }),
                // A missing element `[, ]` is only valid in a destructuring pattern,
                // not an array literal.
                ArrayElement::Missing(_) => {
                    return Err(LowerError::Unsupported {
                        what: "missing element in array literal",
                        line,
                    })
                }
            }
        }
        Ok(out)
    }

    /// Lower `[targets] = rhs` / `list(targets) = rhs` array destructuring (step
    /// 51): evaluate `rhs` once, then destructure it into the target list. When the
    /// pattern binds any target by reference (`[&$x] = …`) and `rhs` is itself a
    /// place, the source place is threaded through so the references write back into
    /// the source array (list-reference semantics).
    pub(super) fn lower_destructure_assign<'a, I>(
        &mut self,
        elements: I,
        rhs: &Expression,
        line: Line,
    ) -> Result<ExprKind, LowerError>
    where
        I: Iterator<Item = &'a ArrayElement<'a>> + Clone,
    {
        // A by-reference target needs the source as a navigable place; only attempt
        // it when `rhs` is an lvalue (a non-place rhs — e.g. `[&$v] = f()` — aliases
        // the value copy instead, with no writeback).
        let src_place = if elements.clone().any(elem_has_ref) {
            self.lower_place(rhs, line).ok()
        } else {
            None
        };
        let source = self.lower_expr(rhs)?;
        let kind = self.destructure_into(elements, source, src_place, line)?.kind;
        Ok(kind)
    }

    /// Build a [`ExprKind::ListAssign`] that destructures the already-lowered
    /// `source` value into `elements`. Each target reads `temp[key]`; a nested
    /// list target recurses (its source is that `temp[key]`). When `src_place` is
    /// given and the pattern (here or nested) binds a target by reference, `temp` is
    /// aliased to `src_place` so a `&temp[key]` promotes the real source element.
    fn destructure_into<'a, I>(
        &mut self,
        elements: I,
        source: Expr,
        src_place: Option<Place>,
        line: Line,
    ) -> Result<Expr, LowerError>
    where
        I: Iterator<Item = &'a ArrayElement<'a>> + Clone,
    {
        let temp = self.fresh_list_temp();
        let mut assigns = Vec::new();
        let mut pos: i64 = 0;
        for el in elements {
            let (target, key) = match el {
                // `[, $b]`: a hole still advances the positional index.
                ArrayElement::Missing(_) => {
                    pos += 1;
                    continue;
                }
                ArrayElement::Value(v) => {
                    let key = Expr { line, kind: ExprKind::Int(pos) };
                    pos += 1;
                    (v.value, key)
                }
                ArrayElement::KeyValue(kv) => (kv.value, self.lower_expr(kv.key)?),
                ArrayElement::Variadic(_) => {
                    return Err(LowerError::Unsupported { what: "spread in destructuring", line })
                }
            };
            // The real source element for this target: `rhs_place[key]` when `rhs`
            // is a place (so a `&$x` leaf promotes the actual element, navigating
            // only the leaf — not the intermediate levels). `None` when `rhs` was a
            // value (refs then alias `temp[key]` instead, with no writeback).
            let child_src = src_place.as_ref().map(|p| {
                let mut q = p.clone();
                q.steps.push(PlaceStep::Index(key.clone()));
                q
            });
            assigns.push(self.destructure_target(target, temp, key, child_src, line)?);
        }
        Ok(Expr {
            line,
            kind: ExprKind::ListAssign { temp, rhs: Box::new(source), assigns },
        })
    }

    /// Build the assignment for one destructuring target. A nested `[...]`/`list(...)`
    /// recurses; a by-reference leaf `&$x` binds `$x` to the real source element
    /// `src` (the original `$arr[key]` when known, else the value copy `temp[key]`);
    /// a plain leaf copies `temp[key]`.
    fn destructure_target(
        &mut self,
        target: &Expression,
        temp: Slot,
        key: Expr,
        src: Option<Place>,
        line: Line,
    ) -> Result<Expr, LowerError> {
        // `[&$x] = …`: bind `$x` as a reference to the source element. When `src`
        // names the real source array element, the reference is promoted there
        // (list-reference writeback); otherwise it aliases the value copy `temp[key]`.
        if let Expression::UnaryPrefix(u) = target {
            if let UnaryPrefixOperator::Reference(_) = u.operator {
                let tgt_place = self.lower_place(u.operand, line)?;
                let source = src.unwrap_or_else(|| Place {
                    base: PlaceBase::Local(temp),
                    steps: vec![PlaceStep::Index(key.clone())],
                });
                return Ok(Expr {
                    line,
                    kind: ExprKind::AssignRef { target: tgt_place, source },
                });
            }
        }
        // The value read `temp[key]`.
        let elem = Expr {
            line,
            kind: ExprKind::Index {
                base: Box::new(Expr { line, kind: ExprKind::Var(temp) }),
                index: Box::new(key),
            },
        };
        // A nested list/array target destructures `temp[key]` in turn; its real
        // source (for by-reference grandchildren) is `src` = `rhs_place[key]`.
        match target {
            Expression::Array(arr) => return self.destructure_into(arr.elements.iter(), elem, src, line),
            Expression::List(l) => return self.destructure_into(l.elements.iter(), elem, src, line),
            Expression::LegacyArray(la) => {
                return self.destructure_into(la.elements.iter(), elem, src, line)
            }
            _ => {}
        }
        // A leaf lvalue (`$x`, `$a[i]`, `$o->p`) gets a plain assignment.
        let place = self.lower_place(target, line)?;
        let kind = if let (PlaceBase::Local(slot), true) = (&place.base, place.steps.is_empty()) {
            ExprKind::Assign(*slot, Box::new(elem))
        } else {
            ExprKind::AssignPlace(place, Box::new(elem))
        };
        Ok(Expr { line, kind })
    }

    /// `foreach (… as [$a,$b])` / `as list(...)`: when the value target is a list
    /// pattern, bind the element to a fresh temp and return that temp plus the
    /// destructuring statement to prepend to the loop body (step 51). Returns
    /// `None` for an ordinary (single-variable) value target.
    pub(super) fn foreach_destructure(
        &mut self,
        target: &Expression,
        line: Line,
    ) -> Result<Option<(Slot, crate::hir::Stmt)>, LowerError> {
        if !matches!(
            target,
            Expression::Array(_) | Expression::List(_) | Expression::LegacyArray(_)
        ) {
            return Ok(None);
        }
        // A by-reference target inside a `foreach` value pattern (`foreach ($a as
        // [&$x])`) needs the loop to iterate by reference so the destructure can
        // write back; that combination is unsupported, so keep the whole test a skip
        // rather than silently iterating a value copy (no writeback).
        if target_has_ref(target) {
            return Err(LowerError::Unsupported { what: "by-reference destructuring", line });
        }
        let temp = self.fresh_list_temp();
        let src = Expr { line, kind: ExprKind::Var(temp) };
        // `foreach` binds the element into `temp` by value, so list-reference
        // targets inside the pattern have no source place to write back to (passed
        // as `None`); a `&$x` leaf there stays unsupported via `lower_place`.
        let kind = match target {
            Expression::Array(arr) => {
                self.destructure_into(arr.elements.iter(), src, None, line)?.kind
            }
            Expression::List(l) => self.destructure_into(l.elements.iter(), src, None, line)?.kind,
            Expression::LegacyArray(la) => {
                self.destructure_into(la.elements.iter(), src, None, line)?.kind
            }
            _ => unreachable!(),
        };
        let stmt = crate::hir::Stmt {
            line,
            kind: crate::hir::StmtKind::Expr(Expr { line, kind }),
        };
        Ok(Some((temp, stmt)))
    }
}

/// Whether a destructuring element binds (or nests) a by-reference target
/// (`&$x`) — decides whether `lower_destructure_assign` must thread the source as
/// a place so the references can write back into it.
fn elem_has_ref(el: &ArrayElement) -> bool {
    match el {
        ArrayElement::Value(v) => target_has_ref(v.value),
        ArrayElement::KeyValue(kv) => target_has_ref(kv.value),
        _ => false,
    }
}

/// Whether a destructuring *target* is, or recursively contains, a `&$x` leaf.
fn target_has_ref(target: &Expression) -> bool {
    match target {
        Expression::UnaryPrefix(u) => matches!(u.operator, UnaryPrefixOperator::Reference(_)),
        Expression::Array(arr) => arr.elements.iter().any(elem_has_ref),
        Expression::List(l) => l.elements.iter().any(elem_has_ref),
        Expression::LegacyArray(la) => la.elements.iter().any(elem_has_ref),
        _ => false,
    }
}
