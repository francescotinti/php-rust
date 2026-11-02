//! Enum members see the enum as their class: `__CLASS__`/`__METHOD__`, the
//! defining class of closures and `self::m(...)` inside enum methods
//! (Zend/tests/enum/__class__.phpt, enum/__method__.phpt). `lower_enum` now sets
//! `cur_class` the way `lower_class` does (PR #4, 2026-10-24). Expected strings
//! are the oracle's (php 8.5.7 CLI) byte for byte. Own file: tests/eval.rs sits
//! exactly at its `loc_dente` cap (REGOLE §8).
use php_runtime::run_source;

fn out(src: &str) -> String {
    let o = run_source(b"t.php", src.as_bytes()).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    String::from_utf8(o.stdout).expect("utf8")
}

#[test]
fn magic_class_method_in_enum() {
    assert_eq!(
        out("<?php enum E { case A; function m(){ echo __CLASS__,'|',__FUNCTION__,'|',__METHOD__; } } E::A->m();"),
        "E|m|E::m"
    );
    assert_eq!(
        out("<?php namespace N; enum E { case A; static function s(){ echo __METHOD__; } } E::s();"),
        "N\\E::s"
    );
    assert_eq!(
        out("<?php enum E { case A; function m(){ return function(){ echo __FUNCTION__,'|',__CLASS__; }; } } (E::A->m())();"),
        "{closure:E::m():1}|E"
    );
    assert_eq!(
        out("<?php enum E { case A; static function s(){ echo 's'; } static function f(){ return self::s(...); } } (E::f())();"),
        "s"
    );
}
