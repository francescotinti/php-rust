//! End-to-end: the bytecode VM calling the *real* builtins.
//!
//! This test lives in php-builtins (not php-runtime) because only a crate that
//! depends on both can build a populated `Registry` and feed it to the VM —
//! php-runtime can't depend on php-builtins without a dependency cycle.

use php_builtins::registry;
use php_runtime::compile::compile_program;
use php_runtime::lower_source;
use php_runtime::vm::run_module;

/// Lower → compile → run a snippet through the VM with the real registry.
fn vm(src: &[u8]) -> Vec<u8> {
    let reg = registry();
    let program = lower_source(b"t.php", src).expect("lower");
    let module = compile_program(&program, &reg).expect("compile");
    let out = run_module(&module, &reg);
    assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
    out.stdout
}

#[test]
fn strlen_value_builtin() {
    assert_eq!(vm(b"<?php echo strlen('hello');"), b"5");
}

#[test]
fn count_array_literal() {
    assert_eq!(vm(b"<?php echo count([1, 2, 3]);"), b"3");
}

#[test]
fn var_dump_writes_to_stdout() {
    assert_eq!(vm(b"<?php var_dump(true);"), b"bool(true)\n");
}

#[test]
fn nested_value_builtins() {
    assert_eq!(vm(b"<?php echo strtoupper(implode('-', ['a', 'b', 'c']));"), b"A-B-C");
}

#[test]
fn sort_is_by_reference() {
    assert_eq!(
        vm(b"<?php $a = [3, 1, 2]; sort($a); echo implode(',', $a);"),
        b"1,2,3"
    );
}

#[test]
fn array_push_is_by_reference_variadic() {
    assert_eq!(
        vm(b"<?php $a = [1]; array_push($a, 2, 3); echo implode(',', $a);"),
        b"1,2,3"
    );
}

#[test]
fn builtin_inside_user_function() {
    assert_eq!(
        vm(b"<?php function n($s) { return strlen($s); } echo n('abcd');"),
        b"4"
    );
}
