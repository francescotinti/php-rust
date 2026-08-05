//! A-PE-100-2 / A-PE-99-1 (Concilio WP-100): il modo register-lowering è
//! deciso dall'AMBIENTE ALLO SPAWN e sigillato eager al bootstrap
//! (`seal_reg_lower_mode`, primo atto dei due main) — `putenv()` da codice
//! PHP non può flipparlo in NESSUNA direzione.
//!
//! Bracci sul funnel VERO (binario spawnnato, come reg_lower_funnel),
//! RI-DERIVATI sul contratto di modo S-100 (grafia value-parsed, lista
//! chiusa: assente=>DEFAULT_ON, `=1`=>on, `=0`=>off, altro=>default+warning
//! — KS-MA-101-1, A-PE-101-4). Ogni braccio di MODO usa un valore ESPLICITO
//! della lista chiusa allo spawn (mai la sola assenza: un braccio scritto
//! sull'assenza collauderebbe silenziosamente il modo NUOVO dopo il flip
//! del default — Pedersen R2; il braccio `absent` si deriva dal contratto).
//! putenv() deve essere impotente in TUTTE le direzioni: set, unset, e
//! l'opt-out `=0` introdotto dal contratto.
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

fn run_case(spawn: Option<&str>, tag: &str, putenv_stmt: &str) -> (String, String, String) {
    let dir = std::env::temp_dir();
    let tag = format!("{}-{tag}", std::process::id());
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
    if let Some(v) = spawn {
        c.env("PHPR_REG_LOWER", v);
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
///
/// Morso Klabnik R1 (Concilio WP-101, CONFERMATO a macchina): il path
/// dell'unità inclusa appare come COSTANTE (`cst… Str("…/inc.php")`)
/// dentro il chunk del `{main}`, e un match a substring aggancia il chunk
/// SBAGLIATO (first-match sul main). Il chunk giusto è quello il cui
/// HEADER è il path: dopo `split("== unit ")` ogni chunk INIZIA col path
/// della propria unit.
fn included_chunk<'a>(stderr: &'a str, inc_name: &str) -> &'a str {
    stderr
        .split("== unit ")
        .find(|u| {
            u.split_whitespace()
                .next()
                .is_some_and(|p| p.ends_with(inc_name))
        })
        .unwrap_or_else(|| {
            panic!("no dump chunk HEADED by {inc_name} — PHPR_DUMP_OPS dead?\n{stderr}")
        })
}

/// `=0` è l'opt-out del contratto — PRIMA del contratto ACCENDEVA il pass
/// (`is_some()`). Braccio flip-proof: non dipende dal default.
#[test]
fn explicit_zero_is_off_and_putenv_set_cannot_turn_the_pass_on() {
    // Baseline dello stesso modo senza putenv: l'output del programma non
    // deve dipendere dal putenv (parità), e non si pinna un letterale
    // calcolato a mano.
    let (base, _, _) = run_case(Some("0"), "zero", "");
    let (out, err, inc_name) = run_case(Some("0"), "zero", "putenv('PHPR_REG_LOWER=1')");
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
            "{form} nell'unità inclusa con `=0` allo spawn e putenv(set): \
             contratto value-parsed o sigillo rotti\n{chunk}"
        );
    }
}

/// Flag ASSENTE: l'emissione segue il DEFAULT NOMINATO del contratto
/// (braccio derivato, si ri-deriva da solo al flip; il tripwire FORTE del
/// flip è il test unitario `mode_contract_default_is_off_pre_flip`), e
/// putenv verso il modo opposto resta impotente.
#[test]
fn absent_flag_follows_the_named_default_and_putenv_cannot_move_it() {
    let (base, base_err, base_inc) = run_case(None, "absent", "");
    let base_has = {
        let c = included_chunk(&base_err, &base_inc);
        REG_FORMS.iter().any(|f| c.contains(f))
    };
    assert_eq!(
        base_has,
        php_runtime::REG_LOWER_DEFAULT_ON,
        "l'emissione con flag assente non segue il default nominato del contratto"
    );
    let flip_stmt = if php_runtime::REG_LOWER_DEFAULT_ON {
        "putenv('PHPR_REG_LOWER=0')"
    } else {
        "putenv('PHPR_REG_LOWER=1')"
    };
    let (out, err, inc_name) = run_case(None, "absent", flip_stmt);
    assert_eq!(out, base, "putenv cambia l'output del programma");
    let chunk = included_chunk(&err, &inc_name);
    assert_eq!(
        REG_FORMS.iter().any(|f| chunk.contains(f)),
        base_has,
        "putenv ha mosso il modo di DEFAULT dopo il boot\n{chunk}"
    );
}

#[test]
fn putenv_unset_after_boot_cannot_turn_the_pass_off() {
    let (base, _, _) = run_case(Some("1"), "unset", "");
    let (out, err, inc_name) = run_case(Some("1"), "unset", "putenv('PHPR_REG_LOWER')");
    assert_eq!(out, base, "putenv(unset) cambia l'output del programma");
    let chunk = included_chunk(&err, &inc_name);
    assert!(
        REG_FORMS.iter().any(|f| chunk.contains(f)),
        "nessuna forma registro nell'unità inclusa DOPO putenv(unset) con \
         flag allo spawn: il sigillo non tiene nella direzione set→unset\n{chunk}"
    );
}

/// La direzione APERTA dal contratto: l'opt-out `=0` scritto via putenv
/// dopo il boot deve essere impotente quanto l'unset.
#[test]
fn putenv_zero_after_boot_cannot_turn_the_pass_off() {
    let (base, _, _) = run_case(Some("1"), "pzero", "");
    let (out, err, inc_name) = run_case(Some("1"), "pzero", "putenv('PHPR_REG_LOWER=0')");
    assert_eq!(out, base, "putenv(=0) cambia l'output del programma");
    let chunk = included_chunk(&err, &inc_name);
    assert!(
        REG_FORMS.iter().any(|f| chunk.contains(f)),
        "nessuna forma registro nell'unità inclusa DOPO putenv('=0') con \
         `=1` allo spawn: il sigillo non tiene nella direzione set→zero\n{chunk}"
    );
}

/// Valore fuori grammatica: fallback al default MAI silenzioso (warning su
/// stderr che nomina la grammatica) e stessa emissione del braccio assente.
#[test]
fn out_of_grammar_value_is_default_plus_loud_warning() {
    let (base, base_err, base_inc) = run_case(None, "junkbase", "");
    let base_has = {
        let c = included_chunk(&base_err, &base_inc);
        REG_FORMS.iter().any(|f| c.contains(f))
    };
    let (out, err, inc_name) = run_case(Some("junk"), "junk", "");
    assert_eq!(out, base, "un valore fuori grammatica cambia l'output del programma");
    assert!(
        err.contains("fuori grammatica"),
        "nessun warning per PHPR_REG_LOWER=junk: fallback silenzioso\n{err}"
    );
    let chunk = included_chunk(&err, &inc_name);
    assert_eq!(
        REG_FORMS.iter().any(|f| chunk.contains(f)),
        base_has,
        "un valore fuori grammatica non cade sul default del contratto\n{chunk}"
    );
}
