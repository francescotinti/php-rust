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
//! Emendato S-103 (Concilio WP-104): braccio `=0` DISCRIMINANTE
//! (A-HE-104-1: prova che l'env viaggia — dump diverso, stdout uguale),
//! controllo positivo fuori-funnel `Z::{prop-init}` nel dump (A-HE-104-2)
//! e residuo `Binary(Add)` pinnato `==1` ESATTO (A-HE-103-1 emendato).

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

/// A-HE-105-1 (Concilio WP-105): conteggio PER CORPO di un op nel dump di
/// PHPR_DUMP_OPS. I corpi aprono con l'intestazione `-- NOME n_slots=… --`;
/// l'op si riconosce ANCORATO come secondo token di una riga-istruzione
/// (`0003 Binary(Add)`), mai come substring nuda (un ipotetico
/// `XBinary(Add)` non conta).
fn per_body_op_counts(dump: &str, op_token: &str) -> std::collections::BTreeMap<String, usize> {
    let mut cur = String::from("<fuori-corpo>");
    let mut map = std::collections::BTreeMap::new();
    for line in dump.lines() {
        if let Some(rest) = line.strip_prefix("-- ") {
            cur = rest.split(" n_slots").next().unwrap_or(rest).trim().to_string();
        } else {
            let mut it = line.split_whitespace();
            let is_instr = matches!(it.next(), Some(ix) if !ix.is_empty() && ix.chars().all(|c| c.is_ascii_digit()));
            if is_instr && it.next() == Some(op_token) {
                *map.entry(cur.clone()).or_insert(0) += 1;
            }
        }
    }
    map
}

#[test]
fn absent_env_subprocess_dump_identical_to_explicit_one() {
    let dir = std::env::temp_dir().join(format!("phpr-absent-eq-one-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let src = dir.join("body_zoo.php");
    std::fs::write(&src, BODY_ZOO).expect("write fixture");

    let (out_absent, dump_absent) = run_arm(&src, None);
    let (out_one, dump_one) = run_arm(&src, Some("1"));
    let (out_zero, dump_zero) = run_arm(&src, Some("0"));
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
    // A-HE-104-2 (Concilio WP-104): il claim «modulo intero» si PROVA —
    // il dump deve contenere il corpo FUORI-FUNNEL dell'inizializzatore
    // di proprieta' (`Z::{prop-init}`), non solo {main}/fn.
    assert!(
        d.contains("Z::{prop-init}"),
        "controllo positivo fuori-funnel: il dump non contiene il corpo \
         `Z::{{prop-init}}` — il «modulo intero» non e' provato\n{d}"
    );
    // A-HE-105-1 (Concilio WP-105, supera il «==1 GLOBALE» di A-HE-103-1):
    // il residuo `Binary(Add)` si giudica PER CORPO, ancorato al token-op —
    // un residuo nuovo esce col NOME del corpo invece di conflazionare tre
    // cause (prelude cambiato / funnel allargato / residuo nuovo) in un
    // conteggio globale.
    let adds = per_body_op_counts(&d, "Binary(Add)");
    let residui: Vec<String> = adds.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    assert_eq!(
        adds.get("Z::{prop-init}").copied().unwrap_or(0),
        1,
        "Binary(Add) atteso ESATTAMENTE 1 dentro Z::{{prop-init}}; residui per corpo: {residui:?}"
    );
    assert_eq!(
        adds.len(),
        1,
        "Binary(Add) residuo FUORI da Z::{{prop-init}} — corpi con residuo: {residui:?}"
    );
    // Tripwire di enumerazione (primo passo verso A-HE-105-3): i corpi
    // dello zoo compaiono TUTTI per NOME nel dump del braccio assente.
    for body in ["{main}", "fn f", "Z::m", "Z::{prop-init}"] {
        assert!(
            d.lines().any(|l| l.strip_prefix("-- ").is_some_and(|r| r.trim_start().starts_with(body))),
            "corpo '{body}' assente dal dump: lo zoo non e' piu' enumerato"
        );
    }
    // A-HE-104-1 (Concilio WP-104): braccio `=0` DISCRIMINANTE — due
    // bracci uguali-per-costruzione non possono fallire per modo; il
    // terzo braccio prova che l'env viaggia: stdout identico ma dump
    // DIVERSO (emissione a pila, zero forme registro).
    assert_eq!(out_zero, out_absent, "stdout `=0` diverso (parita' funzionale rotta)");
    let dz = String::from_utf8_lossy(&dump_zero);
    assert_ne!(
        dump_zero, dump_absent,
        "il dump `=0` e' IDENTICO all'assente: l'env non viaggia — il \
         dente non discrimina il modo"
    );
    assert!(
        !dz.contains("BinaryDst") && !dz.contains("CmpJmpSC") && !dz.contains("BinarySS"),
        "forme registro nel dump `=0`: il modo OFF non e' off\n{dz}"
    );
    // A-HE-105-2 (Concilio WP-105): controllo POSITIVO del braccio `=0` —
    // un env che sotto `=0` uccidesse il dumping intero passerebbe i check
    // negativi sopra. Il dump OFF deve contenere Z::{prop-init} e le forme
    // PILA vive: `BinaryAdd` ancorato > 1. (Rosso di nascita ARCHIVIATO in
    // wp104-harness/denti-rossi/absent-eq-one-he105-2-rosso.txt: la prima
    // attesa contava il generico `Binary(Add)`, ma il loop in pila emette
    // l'op DEDICATO `BinaryAdd` — il generico vive solo fuori-funnel.)
    assert!(
        dz.contains("Z::{prop-init}"),
        "controllo positivo `=0`: manca Z::{{prop-init}} nel dump OFF"
    );
    let n_off: usize = per_body_op_counts(&dz, "BinaryAdd").values().sum();
    assert!(
        n_off > 1,
        "controllo positivo `=0`: attesi BinaryAdd di pila > 1, trovati {n_off}"
    );
    // Il VERDETTO: dump BYTE-identico sul modulo intero.
    assert_eq!(
        dump_absent, dump_one,
        "dump-diff: assente e `=1` NON emettono lo stesso modulo\n--- assente ---\n{}\n--- =1 ---\n{}",
        String::from_utf8_lossy(&dump_absent),
        String::from_utf8_lossy(&dump_one)
    );
}
