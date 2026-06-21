//! HIR lowering of statements and the function/class hoist passes. Split out of `lower.rs` (step 61); behaviour is unchanged.

use mago_span::HasSpan;
use mago_syntax::ast::{
    Class,
    DeclareBody, Enum,
    Expression, ForeachTarget, Function, Interface, Literal, Statement, StaticItem, Variable,
};

use crate::hir::{
    Case,
    GlobalBinding, Line, StaticBinding, Stmt, StmtKind,
};

use super::*;

impl<'f> Lowerer<'f> {
    pub(super) fn lower_stmts(&mut self, stmts: &[Statement]) -> Result<Vec<Stmt>, LowerError> {
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
                    what: stmt_variant_name(stmt),
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
    pub(super) fn hoist_function(&mut self, func: &Function) -> Result<(), LowerError> {
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
    pub(super) fn hoist_classes(&mut self, stmts: &[Statement]) -> Result<(), LowerError> {
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

}
