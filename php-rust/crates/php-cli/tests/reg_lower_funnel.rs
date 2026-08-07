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

fn run_phpr(envs: &[(&str, &str)], file: &std::path::Path) -> (String, String, bool) {
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
        out.status.success(),
    )
}

#[test]
fn flag_on_folds_the_main_toplevel_and_matches_flag_off_output() {
    // The arith_small shape twice: a top-level tight loop in `{main}` AND la
    // stessa forma dentro una FUNZIONE — la probe fuori dal `{main}`
    // (A-HE-101-4: il punto cieco del dump erano proprio i corpi non-main).
    // La catena a TRE add `$t+$j+$j+$j` lascia il sito CENTRALE a lhs-stack
    // col risultato in pila (il primo fonde in BinarySS, l'ultimo in
    // BinaryDst): flag-on deve diventare `BinaryAdd` (estensione
    // A-MA-101-3), mai restare `Binary(Add)` generico.
    let src = br#"<?php
function probe($n){ $t=0; for($j=0;$j<$n;$j++){ $t = $t + $j + $j + $j; $t -= 2*$j; $t += $j*3 - ($j>>2); } return $t; }
$s=0; for($i=0;$i<1000;$i++){ $s += $i*3 - ($i>>2); } echo $s,"\n"; echo probe(1000),"\n";"#;
    let dir = std::env::temp_dir();
    let file = dir.join(format!("reg-funnel-{}.php", std::process::id()));
    std::fs::write(&file, src.as_slice()).expect("write test php");

    // Bracci con valore ESPLICITO della lista chiusa (contratto di modo
    // S-100): un braccio scritto sulla sola assenza girerebbe nel modo NUOVO
    // dopo il flip del default — falso verde stesso-modo (R1 Hejlsberg).
    // ENTRAMBI i bracci col dump: la bit-identità/diversità d'emissione si
    // giudica SOLO dal diff dei dump, mai dal cronometro (KS-HE-101-4).
    let (off_out, off_err, off_ok) =
        run_phpr(&[("PHPR_REG_LOWER", "0"), ("PHPR_DUMP_OPS", "1")], &file);
    let (on_out, on_err, on_ok) =
        run_phpr(&[("PHPR_REG_LOWER", "1"), ("PHPR_DUMP_OPS", "1")], &file);
    let _ = std::fs::remove_file(&file);

    // A-KL-100-1: stdout ATTESO pinnato (derivato, non copiato da un run) ed
    // exit status asseriti — un binario che sbaglia UGUALE nei due modi non
    // passa più per parità.
    assert!(off_ok, "flag-off exit non-zero");
    assert!(on_ok, "flag-on exit non-zero");
    let main_sum: i64 = (0..1000i64).map(|i| i * 3 - (i >> 2)).sum();
    let probe_sum: i64 = (0..1000i64).map(|j| j * 4 - (j >> 2)).sum();
    // probe: t += 3j; t -= 2j; t += 3j - (j>>2)  =>  Σ(4j - (j>>2)) — come sopra.
    let expected = format!("{main_sum}\n{probe_sum}\n");
    assert_eq!(off_out, expected, "flag-off stdout != atteso");
    assert_eq!(on_out, expected, "flag-on stdout != atteso");

    // KS-HE-101-1: i due bracci devono provare emissione DIVERSA sulla probe
    // — hash dei due dump pubblicati nel log del test, poi il diff.
    let h = |s: &str| {
        use std::hash::{Hash, Hasher};
        let mut hs = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hs);
        hs.finish()
    };
    eprintln!("dump-hash off={:016x} on={:016x}", h(&off_err), h(&on_err));
    assert_ne!(
        off_err, on_err,
        "i dump dei due bracci sono IDENTICI: falso verde stesso-modo"
    );

    let fname = file.file_name().unwrap().to_string_lossy().into_owned();
    let chunk_of = |err: &str, label: &str| -> String {
        let unit = err
            .split("== unit ")
            .find(|u| u.split_whitespace().next().is_some_and(|p| p.ends_with(&fname)))
            .unwrap_or_else(|| panic!("no dump chunk for {fname} — PHPR_DUMP_OPS dead?\n{err}"));
        unit.split(&format!("-- {label} "))
            .nth(1)
            .and_then(|r| r.split("\n-- ").next())
            .unwrap_or_else(|| panic!("no body `{label}` in unit dump\n{unit}"))
            .to_owned()
    };

    // Positive control flag-on: `{main}` AND the probe function both show the
    // fused register forms (il compare del loop è slot-const nel main →
    // CmpJmpSC, slot-slot nella probe `$j<$n` → CmpJmpSS). EMENDAMENTO
    // DICHIARATO S-106 (leva H-A1): il `+=` fonde nella RMW-su-slot
    // `BinarySTDst`; BinaryDst RI-COLLOCATO nella probe (catena lhs-stack).
    // EMENDAMENTO DICHIARATO S-107 (lotto superistruzioni): l'albero
    // `$i*3 - ($i>>2)` fonde le DUE BinarySC e la Sub in `BinarySCSC`
    // (l'attesa BinarySC standalone è SOSTITUITA — un contains("BinarySC")
    // resterebbe verde per SOTTOSTRINGA di BinarySCSC, quindi il controllo
    // nomina la forma INTERA), e il trigramma IncDecSlot;Pop;Jump del
    // back-edge fonde in `IncDecSlotJmp` in entrambi i corpi.
    for (label, cmp_form, dst_form) in [
        ("{main}", "CmpJmpSC", "BinarySTDst"),
        ("fn probe", "CmpJmpSS", "BinaryDst"),
    ] {
        let chunk = chunk_of(&on_err, label);
        for form in ["BinarySCSC", "IncDecSlotJmp", cmp_form, dst_form] {
            assert!(
                chunk.contains(form),
                "no {form} in the `{label}` dump: the pass did not rewrite it\n{chunk}"
            );
        }
        // A-HE-100-1 ri-derivato sull'estensione A-MA-101-3: flag-on ogni
        // add è forma FUSA oppure `BinaryAdd` — un `Binary(Add)` generico
        // residuo è il tripwire (finestre o estensione morte).
        assert!(
            !chunk.contains("Binary(Add)"),
            "Binary(Add) generico nel dump flag-on di `{label}`\n{chunk}"
        );
    }
    // Controllo positivo dell'estensione: il sito lhs-stack della catena
    // `$t+$j+$j` nella probe deve mostrare la forma specializzata flag-on.
    assert!(
        chunk_of(&on_err, "fn probe").contains("BinaryAdd"),
        "nessun BinaryAdd nel dump flag-on della probe: l'estensione \
         A-MA-101-3 non morde\n{}",
        chunk_of(&on_err, "fn probe")
    );
    // Braccio OFF, controllo positivo speculare: emissione stack-based con la
    // specializzazione H-B2 (`BinaryAdd`) e ZERO forme registro.
    for label in ["{main}", "fn probe"] {
        let chunk = chunk_of(&off_err, label);
        assert!(
            chunk.contains("BinaryAdd"),
            "flag-off senza BinaryAdd in `{label}`: la specializzazione H-B2 è sparita\n{chunk}"
        );
        for form in [
            "BinarySC", "CmpJmpSC", "BinaryDst", "BinarySS", "CmpJmpSS", "BinarySTDst",
            // S-107: le forme del lotto non esistono flag-off.
            "BinarySCSC", "BinaryTC", "IncDecSlotPop", "IncDecSlotJmp",
            "PropGetSlot", "PropSetPop", "StringifySlot",
        ] {
            assert!(
                !chunk.contains(form),
                "{form} nel dump flag-off di `{label}`: il modo OFF non è off\n{chunk}"
            );
        }
    }
}
