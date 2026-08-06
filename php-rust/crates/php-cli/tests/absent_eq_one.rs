//! A-HE-103-3 (Concilio WP-103, S-102 punto 5): il dente VERO su
//! «assente ≡ `=1`» — in SOTTOPROCESSO, giudicato dal dump-diff.
//!
//! La vecchia metà in-process era `f(x)==f(x)` (R-HE-103-1, copertura
//! fabbricata): compilava due volte con lo stesso bool nello stesso env.
//! Qui i due bracci sono DUE PROCESSI del binario vero con ambienti
//! DIVERSI (env costruito, mai ereditato — apparato A-SK-93..97): uno con
//! `PHPR_REG_LOWER` ASSENTE, uno con `=1`. Giudice: dump BYTE-identico
//! (`PHPR_DUMP_OPS=1`, modulo intero su BODY_ZOO) + stdout identico +
//! CONTROLLO POSITIVO (il dump contiene le forme registro: un dump vuoto o
//! un env mai propagato non può dare un verde).
//! Nota di copertura: il corpus «nei 2 modi» esercita default(assente) e
//! `=0`; la coppia assente↔`=1` end-to-end la esercita SOLO questo dente.

use std::process::Command;

const BODY_ZOO: &str = r#"<?php
class Z {
    const K = 5;
    public $p = self::K + 1;
    public function m($a, $b) { return $a + $b * 2; }
}
function f($x) { $s = 0; for ($i = 0; $i < $x; $i++) { $s = $s + $i; } return $s; }
$z = new Z();
echo f(9), ":", $z->m(3, 4), ":", $z->p, "\n";
"#;

fn run_arm(src_path: &std::path::Path, reg_lower: Option<&str>) -> (Vec<u8>, Vec<u8>) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_phpr"));
    // Ambiente COSTRUITO con lista chiusa (A-SK-93): niente eredita' dal
    // processo di test — l'assenza di PHPR_REG_LOWER e' PER COSTRUZIONE.
    cmd.env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("PHPR_DUMP_OPS", "1")
        .arg(src_path);
    if let Some(v) = reg_lower {
        cmd.env("PHPR_REG_LOWER", v);
    }
    let out = cmd.output().expect("spawn phpr");
    assert!(
        out.status.success(),
        "phpr rc!=0 (reg_lower={reg_lower:?}): stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    (out.stdout, out.stderr)
}

#[test]
fn absent_env_subprocess_dump_identical_to_explicit_one() {
    let dir = std::env::temp_dir().join(format!("phpr-absent-eq-one-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let src = dir.join("body_zoo.php");
    std::fs::write(&src, BODY_ZOO).expect("write fixture");

    let (out_absent, dump_absent) = run_arm(&src, None);
    let (out_one, dump_one) = run_arm(&src, Some("1"));
    let _ = std::fs::remove_dir_all(&dir);

    // Controllo positivo PRIMA del verdetto: il braccio assente e' davvero
    // nel modo default-ON — il dump esiste e mostra le forme registro nel
    // {main}/f (il loop scalare le produce). Senza questa prova, due dump
    // vuoti o due env morti darebbero un verde indistinguibile (recidiva
    // A-PE-102-1: il modo si PROVA, non si presume).
    let d = String::from_utf8_lossy(&dump_absent);
    assert!(
        d.contains("BinaryDst") || d.contains("CmpJmpSC") || d.contains("BinarySS"),
        "controllo positivo: il dump del braccio ASSENTE non mostra forme \
         registro — dump morto o modo non-default?\n{d}"
    );
    // Output del programma identico (parita' funzionale dei due bracci).
    assert_eq!(
        out_absent, out_one,
        "stdout diverso tra assente e `=1` (parita' funzionale rotta)"
    );
    // Il VERDETTO: dump BYTE-identico sul modulo intero.
    assert_eq!(
        dump_absent, dump_one,
        "dump-diff: assente e `=1` NON emettono lo stesso modulo\n--- assente ---\n{}\n--- =1 ---\n{}",
        String::from_utf8_lossy(&dump_absent),
        String::from_utf8_lossy(&dump_one)
    );
}
