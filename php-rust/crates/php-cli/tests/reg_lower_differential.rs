//! A-HE-100-3 (Concilio WP-100, promosso BLOCCANTE dal WP-101): test
//! differenziale PERMANENTE `BinaryAdd ≡ Binary(Add)`.
//!
//! Flag-OFF l'emissione specializza `+` in `Op::BinaryAdd` (H-B2); flag-ON
//! l'emissione resta `Binary(Add)` (e le finestre fondono le forme registro).
//! L'equivalenza dei due opcodi era «vera per costruzione» solo a commento —
//! qui è una fixture eseguita nei DUE modi sul funnel VERO (binario
//! spawnnato), byte-compare di stdout+stderr+exit: overflow, coercizioni
//! numeric-string, union di array, warning Undef, fallback MISS (tipi non
//! int-int), `+=` compreso. «Corretto per fortuna del corpus» ≠ corretto.
use std::process::Command;

/// I casi limite dell'ADD, uno per riga con etichetta: l'output è il
/// giudice, quindi ogni caso stampa qualcosa (valore o eccezione).
const CASES: &[u8] = br#"<?php
function show($label, $fn) {
  try { $v = $fn(); echo $label, "=", var_export($v, true), "\n"; }
  catch (\Throwable $e) { echo $label, "!", get_class($e), ":", $e->getMessage(), "\n"; }
}
show('int_int',        fn() => 2 + 3);
show('overflow_pos',   fn() => PHP_INT_MAX + 1);
show('overflow_neg',   fn() => PHP_INT_MIN + (-1));
show('int_float',      fn() => 1 + 2.5);
show('float_float',    fn() => 0.1 + 0.2);
show('numstr_int',     fn() => "7" + 3);
show('numstr_float',   fn() => "7.5" + 1);
show('numstr_numstr',  fn() => "10" + "1e1");
show('lead_num_str',   function () { return "5 apples" + 1; });
show('bool_int',       fn() => true + 1);
show('null_int',       fn() => null + 1);
show('arr_union',      fn() => [1, 2] + [9, 8, 7]);
show('arr_int_throws', fn() => [1] + 1);
show('str_throws',     fn() => "abc" + 1);
show('obj_throws',     function () { $o = new stdClass; return $o + 1; });
function undef_warn() { return $novar + 1; }
show('undef_warn',     fn() => undef_warn());
function warm($n) { $s = 0; for ($i = 0; $i < $n; $i++) { $s = $s + 1; } return $s; }
show('loop_add',       fn() => warm(1000));
function plus_eq($n) { $s = PHP_INT_MAX - 500; for ($i = 0; $i < $n; $i++) { $s += 1; } return $s; }
show('pluseq_overflow', fn() => plus_eq(1000));
$x = 3; $x += "4"; echo "pluseq_coerce=", var_export($x, true), "\n";
"#;

fn run_mode(reg: &str, file: &std::path::Path) -> (String, String, bool) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_phpr"));
    c.env_remove("PHPR_REG_LOWER");
    c.env_remove("PHPR_DUMP_OPS");
    c.env("PHPR_REG_LOWER", reg);
    let out = c.arg(file).output().expect("spawn phpr");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

#[test]
fn binary_add_and_generic_add_are_byte_equivalent_across_modes() {
    let dir = std::env::temp_dir();
    let file = dir.join(format!("reg-differential-{}.php", std::process::id()));
    std::fs::write(&file, CASES).expect("write cases php");
    let (off_out, off_err, off_ok) = run_mode("0", &file);
    let (on_out, on_err, on_ok) = run_mode("1", &file);
    let _ = std::fs::remove_file(&file);

    // Controllo positivo: la fixture produce davvero i suoi casi (non è un
    // file vuoto che passa per parità — forgia silenziosa).
    for label in ["overflow_pos", "arr_union", "str_throws", "pluseq_overflow"] {
        assert!(
            off_out.contains(label),
            "fixture muta: manca il caso `{label}`\n{off_out}"
        );
    }
    assert_eq!(off_ok, on_ok, "exit status diverge tra i modi");
    assert_eq!(off_out, on_out, "stdout diverge tra flag-off e flag-on");
    assert_eq!(off_err, on_err, "stderr (diags) diverge tra flag-off e flag-on");
}
