//! Smoke tests for the mago→HIR bridge (plan step 3).
//!
//! These assert the *shape* of the lowered HIR for representative Tier 1
//! scripts: slot resolution, control-flow nesting, operator mapping, line
//! numbers, and the scope boundary (unsupported constructs are reported, not
//! silently dropped).

use php_runtime::hir::*;
use php_runtime::{lower_source, LowerError};

fn lower(src: &str) -> Program {
    lower_source(b"test.php", src.as_bytes()).expect("should lower")
}

fn err(src: &str) -> LowerError {
    lower_source(b"test.php", src.as_bytes()).expect_err("should fail")
}

#[test]
fn echo_string_literal() {
    let p = lower(r#"<?php echo "hi";"#);
    assert!(p.slots.is_empty());
    assert_eq!(p.body.len(), 1);
    match &p.body[0].kind {
        StmtKind::Echo(vals) => {
            assert_eq!(vals.len(), 1);
            assert_eq!(vals[0].kind, ExprKind::Str(b"hi".to_vec().into_boxed_slice()));
        }
        other => panic!("expected echo, got {other:?}"),
    }
}

#[test]
fn echo_multiple_values() {
    let p = lower(r#"<?php echo 1, 2, 3;"#);
    match &p.body[0].kind {
        StmtKind::Echo(vals) => assert_eq!(vals.len(), 3),
        other => panic!("got {other:?}"),
    }
}

#[test]
fn assignment_creates_and_reuses_slot() {
    let p = lower("<?php $x = 1; $y = 2; $x = 3;");
    // Two distinct variables → two slots; the third statement reuses slot 0.
    assert_eq!(p.slots, vec![b"x".to_vec().into_boxed_slice(), b"y".to_vec().into_boxed_slice()]);
    assert_eq!(p.body.len(), 3);
    match &p.body[2].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(slot, rhs), .. }) => {
            assert_eq!(*slot, 0);
            assert_eq!(rhs.kind, ExprKind::Int(3));
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn arithmetic_and_var_read() {
    let p = lower("<?php $x = 1 + 2 * 3;");
    // Precedence is mago's job: 1 + (2 * 3).
    match &p.body[0].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(0, rhs), .. }) => match &rhs.kind {
            ExprKind::Binary(BinOp::Add, l, r) => {
                assert_eq!(l.kind, ExprKind::Int(1));
                assert!(matches!(r.kind, ExprKind::Binary(BinOp::Mul, _, _)));
            }
            other => panic!("got {other:?}"),
        },
        other => panic!("got {other:?}"),
    }
}

#[test]
fn integer_overflow_promotes_to_float() {
    let p = lower("<?php $x = 9223372036854775808;");
    match &p.body[0].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, rhs), .. }) => {
            assert!(matches!(rhs.kind, ExprKind::Float(_)));
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn if_elseif_else() {
    let p = lower(
        "<?php if ($a) { echo 1; } elseif ($b) { echo 2; } else { echo 3; }",
    );
    match &p.body[0].kind {
        StmtKind::If { cond, then, elseifs, otherwise } => {
            assert!(matches!(cond.kind, ExprKind::Var(_)));
            assert_eq!(then.len(), 1);
            assert_eq!(elseifs.len(), 1);
            assert_eq!(otherwise.len(), 1);
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn if_without_braces() {
    let p = lower("<?php if ($a) echo 1;");
    match &p.body[0].kind {
        StmtKind::If { then, elseifs, otherwise, .. } => {
            assert_eq!(then.len(), 1);
            assert!(elseifs.is_empty());
            assert!(otherwise.is_empty());
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn while_loop() {
    let p = lower("<?php while ($i < 10) { $i = $i + 1; }");
    match &p.body[0].kind {
        StmtKind::While { cond, body } => {
            assert!(matches!(cond.kind, ExprKind::Binary(BinOp::Lt, _, _)));
            assert_eq!(body.len(), 1);
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn for_loop() {
    let p = lower("<?php for ($i = 0; $i < 3; $i++) { echo $i; }");
    match &p.body[0].kind {
        StmtKind::For { init, cond, step, body } => {
            assert_eq!(init.len(), 1);
            assert_eq!(cond.len(), 1);
            assert_eq!(step.len(), 1);
            assert!(matches!(step[0].kind, ExprKind::IncDec { inc: true, pre: false, .. }));
            assert_eq!(body.len(), 1);
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn do_while() {
    let p = lower("<?php do { echo 1; } while ($x);");
    assert!(matches!(p.body[0].kind, StmtKind::DoWhile { .. }));
}

#[test]
fn ternary_full_and_short() {
    let p = lower("<?php $a = $x ? 1 : 2; $b = $x ?: 3;");
    match &p.body[0].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, rhs), .. }) => match &rhs.kind {
            ExprKind::Ternary { then, .. } => assert!(then.is_some()),
            other => panic!("got {other:?}"),
        },
        other => panic!("got {other:?}"),
    }
    match &p.body[1].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, rhs), .. }) => match &rhs.kind {
            ExprKind::Ternary { then, .. } => assert!(then.is_none()),
            other => panic!("got {other:?}"),
        },
        other => panic!("got {other:?}"),
    }
}

#[test]
fn logical_and_coalesce() {
    let p = lower("<?php $a = $x && $y; $b = $x || $y; $c = $x ?? $y;");
    assert!(matches!(
        &p.body[0].kind,
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, r), .. }) if matches!(r.kind, ExprKind::And(_, _))
    ));
    assert!(matches!(
        &p.body[1].kind,
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, r), .. }) if matches!(r.kind, ExprKind::Or(_, _))
    ));
    assert!(matches!(
        &p.body[2].kind,
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, r), .. }) if matches!(r.kind, ExprKind::Coalesce(_, _))
    ));
}

#[test]
fn compound_assign() {
    let p = lower("<?php $x += 5; $s .= \"a\";");
    assert!(matches!(
        &p.body[0].kind,
        StmtKind::Expr(Expr { kind: ExprKind::AssignOp(BinOp::Add, 0, _), .. })
    ));
    assert!(matches!(
        &p.body[1].kind,
        StmtKind::Expr(Expr { kind: ExprKind::AssignOp(BinOp::Concat, _, _), .. })
    ));
}

#[test]
fn casts_and_unary() {
    let p = lower("<?php $a = (int)$x; $b = -$x; $c = !$x; $d = ~$x;");
    let k = |i: usize| match &p.body[i].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, rhs), .. }) => rhs.kind.clone(),
        other => panic!("got {other:?}"),
    };
    assert!(matches!(k(0), ExprKind::Cast(CastKind::Int, _)));
    assert!(matches!(k(1), ExprKind::Unary(UnOp::Neg, _)));
    assert!(matches!(k(2), ExprKind::Unary(UnOp::Not, _)));
    assert!(matches!(k(3), ExprKind::Unary(UnOp::BitNot, _)));
}

#[test]
fn break_continue_levels() {
    // The braced loop body is a `Block`, so the break/continue lives one level down.
    let inner = |body: &[Stmt]| match &body[0].kind {
        StmtKind::Block(inner) => inner[0].kind.clone(),
        other => other.clone(),
    };
    let p = lower("<?php while (1) { break 2; } while (1) { continue; }");
    match &p.body[0].kind {
        StmtKind::While { body, .. } => assert_eq!(inner(body), StmtKind::Break(2)),
        other => panic!("got {other:?}"),
    }
    match &p.body[1].kind {
        StmtKind::While { body, .. } => assert_eq!(inner(body), StmtKind::Continue(1)),
        other => panic!("got {other:?}"),
    }
}

#[test]
fn inline_html() {
    let p = lower("Hello <?php echo 1; ?> World");
    // leading HTML, echo, trailing HTML
    assert!(matches!(&p.body[0].kind, StmtKind::InlineHtml(h) if &**h == b"Hello "));
    assert!(matches!(&p.body[1].kind, StmtKind::Echo(_)));
    assert!(matches!(&p.body.last().unwrap().kind, StmtKind::InlineHtml(h) if &**h == b" World"));
}

#[test]
fn line_numbers_are_one_based() {
    let p = lower("<?php\n$x = 1;\n$y = 2;\n");
    assert_eq!(p.body[0].line, 2);
    assert_eq!(p.body[1].line, 3);
}

#[test]
fn unsupported_foreach_is_reported() {
    match err("<?php foreach ($a as $v) { echo $v; }") {
        LowerError::Unsupported { what, .. } => assert_eq!(what, "statement"),
        other => panic!("got {other:?}"),
    }
}

#[test]
fn unsupported_array_element_target() {
    // Array-element assignment targets arrive in step 7.
    assert!(matches!(
        err("<?php $a[0] = 1;"),
        LowerError::Unsupported { what: "assignment target", .. }
    ));
}

#[test]
fn parse_error_is_reported() {
    assert!(matches!(err("<?php $x = ;"), LowerError::Parse(_)));
}
