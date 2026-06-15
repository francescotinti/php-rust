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
fn array_literal_keyed_and_keyless() {
    let p = lower(r#"<?php $a = [1, 'k' => 2, 3];"#);
    match &p.body[0].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, rhs), .. }) => match &rhs.kind {
            ExprKind::Array(elems) => {
                assert_eq!(elems.len(), 3);
                assert!(elems[0].key.is_none());
                assert!(elems[1].key.is_some());
                assert!(elems[2].key.is_none());
            }
            other => panic!("got {other:?}"),
        },
        other => panic!("got {other:?}"),
    }
    // `array(...)` lowers identically to `[...]`.
    let p = lower(r#"<?php $a = array(1, 2);"#);
    assert!(matches!(
        &p.body[0].kind,
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, r), .. }) if matches!(r.kind, ExprKind::Array(_))
    ));
}

#[test]
fn array_element_assignment_targets() {
    // `$a[k] = v`, `$a[] = v`, and nested writes lower to Place-based assigns.
    let p = lower("<?php $a[0] = 1; $a[] = 2; $a['x']['y'] = 3;");
    match &p.body[0].kind {
        StmtKind::Expr(Expr { kind: ExprKind::AssignPlace(place, _), .. }) => {
            assert_eq!(place.base, PlaceBase::Local(0));
            assert!(matches!(place.steps[..], [PlaceStep::Index(_)]));
        }
        other => panic!("got {other:?}"),
    }
    match &p.body[1].kind {
        StmtKind::Expr(Expr { kind: ExprKind::AssignPlace(place, _), .. }) => {
            assert!(matches!(place.steps[..], [PlaceStep::Append]));
        }
        other => panic!("got {other:?}"),
    }
    match &p.body[2].kind {
        StmtKind::Expr(Expr { kind: ExprKind::AssignPlace(place, _), .. }) => {
            assert_eq!(place.steps.len(), 2);
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn foreach_value_and_key_value() {
    let p = lower("<?php foreach ($a as $v) { echo $v; } foreach ($a as $k => $v) { echo $k; }");
    assert!(matches!(
        &p.body[0].kind,
        StmtKind::Foreach { key: None, .. }
    ));
    assert!(matches!(
        &p.body[1].kind,
        StmtKind::Foreach { key: Some(_), .. }
    ));
}

#[test]
fn switch_and_match_lower() {
    let p = lower("<?php switch ($x) { case 1: echo 'a'; break; default: echo 'b'; }");
    match &p.body[0].kind {
        StmtKind::Switch { cases, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(cases[0].test.is_some());
            assert!(cases[1].test.is_none());
        }
        other => panic!("got {other:?}"),
    }
    let p = lower("<?php $r = match ($x) { 1, 2 => 'a', default => 'b' };");
    match &p.body[0].kind {
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, rhs), .. }) => match &rhs.kind {
            ExprKind::Match { arms, .. } => {
                assert_eq!(arms[0].conditions.len(), 2);
                assert!(arms[1].conditions.is_empty()); // default arm
            }
            other => panic!("got {other:?}"),
        },
        other => panic!("got {other:?}"),
    }
}

#[test]
fn isset_empty_unset_lower() {
    let p = lower("<?php $b = isset($a[0]); $c = empty($a); unset($a[1]);");
    assert!(matches!(
        &p.body[0].kind,
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, r), .. }) if matches!(r.kind, ExprKind::Isset(_))
    ));
    assert!(matches!(
        &p.body[1].kind,
        StmtKind::Expr(Expr { kind: ExprKind::Assign(_, r), .. }) if matches!(r.kind, ExprKind::Empty(_))
    ));
    assert!(matches!(&p.body[2].kind, StmtKind::Unset(_)));
}

#[test]
fn parse_error_is_reported() {
    assert!(matches!(err("<?php $x = ;"), LowerError::Parse(_)));
}

// --- step 8: user-defined functions ---

#[test]
fn function_declaration_is_hoisted_into_table() {
    let p = lower("<?php function f($a, $b = 1) { return $a + $b; } echo f(2);");
    // The declaration produces no runtime statement; only the echo remains.
    assert_eq!(p.body.len(), 1);
    assert!(matches!(&p.body[0].kind, StmtKind::Echo(_)));
    // The function is registered with its own local slot table. The prelude's
    // global functions (step 35 procedural date API) precede the user's, so find
    // it by name rather than by a fixed index.
    let f = p
        .functions
        .iter()
        .find(|f| &*f.name == b"f")
        .expect("user function `f` is hoisted");
    assert_eq!(&*f.name, b"f");
    assert_eq!(f.params.len(), 2);
    assert!(f.params[0].default.is_none());
    assert!(f.params[1].default.is_some());
    // Params occupy the first slots of the function's local frame.
    assert_eq!(f.slots.len(), 2);
    assert_eq!(&*f.slots[0], b"a");
    assert_eq!(&*f.slots[1], b"b");
}

#[test]
fn by_reference_param_lowers_with_flag() {
    // By-reference parameters are supported from step 11b: the flag is recorded
    // on the lowered `Param`.
    let p = lower("<?php function f(&$a) { } f($x);");
    // Find the user function by name: the prelude's global functions (step 35)
    // occupy the leading indices.
    let f = p
        .functions
        .iter()
        .find(|f| &*f.name == b"f")
        .expect("user function `f` is hoisted");
    assert!(f.params[0].by_ref);
}

#[test]
fn variadic_param_lowers_with_flag() {
    // Variadic params are supported since step 38-5: the last param carries
    // `variadic: true`.
    let p = lower("<?php function f($a, ...$rest) { } f(1, 2, 3);");
    // Prelude global functions are hoisted first (step 35), so find `f` by name.
    let f = p
        .functions
        .iter()
        .find(|f| f.name.as_ref() == b"f")
        .expect("function f");
    assert_eq!(f.params.len(), 2);
    assert!(!f.params[0].variadic);
    assert!(f.params[1].variadic);
}

#[test]
fn conditional_function_declaration_is_unsupported() {
    // A function defined inside a branch is not hoisted; lowering reports it
    // rather than silently registering it unconditionally.
    assert!(matches!(
        err("<?php if (true) { function f() {} }"),
        LowerError::Unsupported { .. }
    ));
}
