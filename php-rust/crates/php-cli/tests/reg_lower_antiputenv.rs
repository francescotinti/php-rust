//! A-PE-100-2 / A-PE-99-1 (Concilio WP-100): il modo register-lowering è
//! deciso dall'AMBIENTE ALLO SPAWN e sigillato eager al bootstrap
//! (`seal_reg_lower_mode`, primo atto dei due main) — `putenv()` da codice
//! PHP non può flipparlo in NESSUNA direzione.
//!
//! Due bracci sul funnel VERO (binario spawnnato, come reg_lower_funnel):
//! - flag ASSENTE allo spawn + `putenv("PHPR_REG_LOWER=1")` prima di un
//!   `include`: l'unità compilata DOPO il putenv non deve avere forme
//!   registro (con controllo positivo che il dump dell'unità esista e
//!   contenga opcodi).
//! - flag PRESENTE allo spawn + `putenv("PHPR_REG_LOWER")` (unset) prima
//!   dell'`include`: l'unità compilata dopo deve avere ANCORA le forme
//!   registro (il sigillo tiene anche nella direzione set→unset).
//!
//! Nota onesta (verbale Pedersen R2): in CLI la prima lettura del flag
//! avveniva comunque alla compile del `{main}`, prima di ogni putenv — il
//! braccio CLI pinna l'INVARIANTE end-to-end come regressione; la finestra
//! davvero aperta era il server, chiusa per COSTRUZIONE dal sigillo nel suo
//! main (stesso `seal_reg_lower_mode`, auditabile staticamente).

use std::process::Command;

/// Corpo foldable: la stessa forma di reg_lower_funnel (BinarySC/CmpJmpSC/
/// BinaryDst attese flag-on nel `{main}` dell'unità inclusa).
const INCLUDED: &[u8] =
    br#"<?php $s=0; for($i=0;$i<100;$i++){ $s += $i*3 - ($i>>2); } echo $s,"\n";"#;

const REG_FORMS: [&str; 4] = ["BinarySC", "CmpJmpSC", "BinaryDst", "BinarySS"];

fn run_case(spawn_flag_on: bool, putenv_stmt: &str) -> (String, String, String) {
    let dir = std::env::temp_dir();
    let tag = format!("{}-{}", std::process::id(), spawn_flag_on);
    let inc = dir.join(format!("antiputenv-inc-{tag}.php"));
    let main = dir.join(format!("antiputenv-main-{tag}.php"));
    std::fs::write(&inc, INCLUDED).expect("write included php");
    std::fs::write(
        &main,
        format!("<?php {putenv_stmt}; include '{}';", inc.display()).into_bytes(),
    )
    .expect("write main php");

    let mut c = Command::new(env!("CARGO_BIN_EXE_phpr"));
    c.env_remove("PHPR_REG_LOWER");
    c.env_remove("PHPR_DUMP_OPS");
    c.env("PHPR_DUMP_OPS", "1");
    if spawn_flag_on {
        c.env("PHPR_REG_LOWER", "1");
    }
    let out = c.arg(&main).output().expect("spawn phpr");
    let inc_name = inc.file_name().unwrap().to_string_lossy().into_owned();
    let _ = std::fs::remove_file(&inc);
    let _ = std::fs::remove_file(&main);
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        inc_name,
    )
}

/// Il chunk di dump dell'unità INCLUSA (compilata DOPO il putenv).
fn included_chunk<'a>(stderr: &'a str, inc_name: &str) -> &'a str {
    stderr
        .split("== unit ")
        .find(|u| u.contains(inc_name))
        .unwrap_or_else(|| {
            panic!("no dump chunk for {inc_name} — PHPR_DUMP_OPS dead?\n{stderr}")
        })
}

#[test]
fn putenv_set_after_boot_cannot_turn_the_pass_on() {
    // Baseline dello stesso modo senza putenv: l'output del programma non
    // deve dipendere dal putenv (parità), e non si pinna un letterale
    // calcolato a mano.
    let (base, _, _) = run_case(false, "");
    let (out, err, inc_name) = run_case(false, "putenv('PHPR_REG_LOWER=1')");
    assert_eq!(out, base, "putenv(set) cambia l'output del programma");
    let chunk = included_chunk(&err, &inc_name);
    // Controllo positivo: il chunk contiene opcodi (il dump morde davvero).
    assert!(
        chunk.contains("CmpJmp") || chunk.contains("Binary"),
        "dump chunk vuoto/insensato per {inc_name}:\n{chunk}"
    );
    for form in REG_FORMS {
        assert!(
            !chunk.contains(form),
            "{form} nell'unità inclusa DOPO putenv(set): il modo è flippato \
             a runtime\n{chunk}"
        );
    }
}

#[test]
fn putenv_unset_after_boot_cannot_turn_the_pass_off() {
    let (base, _, _) = run_case(true, "");
    let (out, err, inc_name) = run_case(true, "putenv('PHPR_REG_LOWER')");
    assert_eq!(out, base, "putenv(unset) cambia l'output del programma");
    let chunk = included_chunk(&err, &inc_name);
    assert!(
        REG_FORMS.iter().any(|f| chunk.contains(f)),
        "nessuna forma registro nell'unità inclusa DOPO putenv(unset) con \
         flag allo spawn: il sigillo non tiene nella direzione set→unset\n{chunk}"
    );
}
