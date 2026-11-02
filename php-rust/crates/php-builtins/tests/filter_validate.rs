//! `filter_var()` validators ported from ext/filter/logical_filters.c:
//! FILTER_VALIDATE_EMAIL (`php_filter_validate_email`; PCRE patterns byte-identical
//! to php-src `regexp0`/`regexp1`) and FILTER_VALIDATE_DOMAIN
//! (`php_filter_validate_domain_ex`). Contributed in PR #7 (fixes #6); expected
//! strings are the oracle's (php 8.5 CLI, re-checked on 8.5.7) byte for byte.
//! Own file: tests/builtins.rs sits exactly at its `loc_dente` cap (REGOLE §8).
use php_builtins::registry;
use php_runtime::vm::run_source_with;

fn out(src: &str) -> String {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    String::from_utf8(o.stdout).expect("utf8")
}

#[test]
fn filter_validate_email() {
    assert_eq!(
        out("<?php foreach (['ada@example.com', 'first.last+tag@sub.example.co.uk', 'user@[127.0.0.1]', 'user@[IPv6:2001:db8::1]',
                 'not-an-email', '', 'a@b', ' ada@example.com', 'a..b@example.com', 'ada@example.com.', 'ada@-example.com',
                 '\"quoted local\"@example.com', str_repeat('a', 65) . '@example.com', 'a@' . str_repeat('b', 64) . '.com',
                 str_repeat('a', 64) . '@' . str_repeat('b.', 127) . 'com'] as $e) {
                 echo var_export(filter_var($e, FILTER_VALIDATE_EMAIL) !== false, true), ' ';
             }
             var_dump(filter_var('ünicode@example.com', FILTER_VALIDATE_EMAIL));
             var_dump(filter_var('ünicode@example.com', FILTER_VALIDATE_EMAIL, FILTER_FLAG_EMAIL_UNICODE));
             var_dump(filter_var('x', FILTER_VALIDATE_EMAIL, FILTER_NULL_ON_FAILURE));
             var_dump(filter_var('x', FILTER_VALIDATE_EMAIL, ['options' => ['default' => 'fallback']]));"),
        "true true true true false false false false false false false false false false false bool(false)\n\
         string(20) \"ünicode@example.com\"\nNULL\nstring(8) \"fallback\"\n"
    );
}

#[test]
fn filter_validate_domain() {
    assert_eq!(
        out("<?php foreach (['example.com', 'example.com.', '', '.', '.example.com', 'exa..mple.com', '-example.com',
                 'example-.com', 'ex_ample.com', '1.2.3.4', str_repeat('a', 63) . '.com', str_repeat('a', 64) . '.com',
                 str_repeat('a.', 126) . 'a.', str_repeat('a.', 126) . 'ab', 'exa mple', 'a-b.c-d'] as $d) {
                 echo (int) (filter_var($d, FILTER_VALIDATE_DOMAIN) !== false), (int) (filter_var($d, FILTER_VALIDATE_DOMAIN, FILTER_FLAG_HOSTNAME) !== false), ' ';
             }
             var_dump(filter_var('a..b', FILTER_VALIDATE_DOMAIN, FILTER_NULL_ON_FAILURE));
             var_dump(filter_var('a..b', FILTER_VALIDATE_DOMAIN, ['options' => ['default' => 'd']]));"),
        "11 11 10 00 00 00 10 10 10 11 11 00 11 00 10 11 NULL\nstring(1) \"d\"\n"
    );
}
