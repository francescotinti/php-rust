//! HIR lowering of expressions: the `lower_expr` dispatch, interpolation/
//! heredoc, calls, instantiation, member access, args, places and array
//! elements. Split out of `lower.rs` (step 61).

use mago_span::HasSpan;
use mago_syntax::ast::{
    Access, Argument, ArrayElement, AssignmentOperator, BinaryOperator, Call, ClassLikeConstantSelector,
    CompositeString, Construct, DocumentIndentation, DocumentKind, DocumentString,
    Expression, Instantiation, Literal,
    MatchArm as AstMatchArm, PartialApplication,
    StringPart, UnaryPostfixOperator, UnaryPrefixOperator, Variable, Yield,
};

use crate::hir::{
    ArrayElem, BinOp, CastKind, ClassRef, Expr, ExprKind, Line, MatchArm, Place, PlaceBase, PlaceStep, Slot, UnOp,
};


use super::*;

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

            // Magic constants (`__LINE__`, `__CLASS__`, …) are compile-time in
            // PHP: substitute each to a literal from the current file/line and
            // the lexical class/function/trait scope tracked above (step 49).
            Expression::MagicConstant(m) => self.lower_magic_constant(m, line),

            // A bare `NAME` constant: known engine constants fold to a literal
            // here (D-18.7); any other name becomes a runtime `Const` read,
            // resolved against `define()`'d constants at eval time (step 49c).
            Expression::ConstantAccess(ca) => {
                let name = function_name(&ca.name);
                match resolve_constant(name) {
                    Some(kind) => kind,
                    None => ExprKind::Const(name.to_vec().into_boxed_slice()),
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

    /// Lower a class-position expression to a [`ClassRef`] (step 19; dynamic
    /// forms added in step 48). `self`/`parent`/`static` and a bare identifier
    /// resolve statically; anything else (`$cls`, `$obj`, `Foo::CONST`, a call,
    /// …) becomes `ClassRef::Dynamic` and is resolved to a class at runtime.
    fn class_ref_of(&mut self, class: &Expression, _line: Line) -> Result<ClassRef, LowerError> {
        match class {
            Expression::Self_(_) => Ok(ClassRef::SelfClass),
            Expression::Parent(_) => Ok(ClassRef::Parent),
            Expression::Static(_) => Ok(ClassRef::Static),
            Expression::Identifier(id) => Ok(ClassRef::Named(function_name(id).into())),
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
    pub(super) fn lower_place(&mut self, lhs: &Expression, line: Line) -> Result<Place, LowerError> {
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
