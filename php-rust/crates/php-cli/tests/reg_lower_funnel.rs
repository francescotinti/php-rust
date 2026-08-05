//! B1 (Concilio WP-99, team-giudice A-HE-99-1 + A-KL-99-3): the register
//! pass battery must exercise the PRODUCTION funnel — the env flag read at
//! process start and the pass applied at its real pipeline point (PRE the
//! WP-65 slot_names cession) — and positively assert fused forms in the
//! `{main}` top-level, the leg the in-process battery could not see.
//!
//! An in-process test cannot toggle `reg_lower::enabled()` (process-wide
//! `OnceLock`), so this spawns the real CLI binary: what production runs is
//! what the test judges. Doubles as M3 (A-KL-99-2), the flag-ON smoke with
//! a positive dump control: a pass that silently stops rewriting fails
//! HERE, not three sessions later in a measurement.

use std::process::Command;

fn run_phpr(envs: &[(&str, &str)], file: &std::path::Path) -> (String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_phpr"));
    // Start from a known flag state regardless of the caller's shell.
    c.env_remove("PHPR_REG_LOWER");
    c.env_remove("PHPR_DUMP_OPS");
    for (k, v) in envs {
        c.env(k, v);
    }
    let out = c.arg(file).output().expect("spawn phpr");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn flag_on_folds_the_main_toplevel_and_matches_flag_off_output() {
    // The arith_small shape: a top-level tight loop, all slots in `{main}`.
    let src = br#"<?php $s=0; for($i=0;$i<1000;$i++){ $s += $i*3 - ($i>>2); } echo $s,"\n";"#;
    let dir = std::env::temp_dir();
    let file = dir.join(format!("reg-funnel-{}.php", std::process::id()));
    std::fs::write(&file, src.as_slice()).expect("write test php");

    let (off_out, _) = run_phpr(&[], &file);
    let (on_out, on_err) =
        run_phpr(&[("PHPR_REG_LOWER", "1"), ("PHPR_DUMP_OPS", "1")], &file);
    let _ = std::fs::remove_file(&file);

    // Parity first: the flag must not change the program's output.
    assert_eq!(off_out, on_out, "flag-on output diverges from flag-off");

    // Positive control on the USER unit's dump chunk: the `{main}` body
    // must show the fused register forms (BinarySC for `$i*3`/`$i>>2`,
    // CmpJmpSC for the loop compare, BinaryDst for the `+=` tail).
    let fname = file.file_name().unwrap().to_string_lossy().into_owned();
    let chunk = on_err
        .split("== unit ")
        .find(|u| u.contains(&fname))
        .unwrap_or_else(|| panic!("no dump chunk for {fname} — PHPR_DUMP_OPS dead?\n{on_err}"));
    for form in ["BinarySC", "CmpJmpSC", "BinaryDst"] {
        assert!(
            chunk.contains(form),
            "no {form} in the {{main}} dump of {fname}: the pass did not \
             rewrite the top-level\n{chunk}"
        );
    }
}
