//! S-145 (az.rev. S-144 #4): dente CI sul funnel `zcell`. La rettifica del
//! revisore S-144 ha stabilito che il tipo di `zcell` vincola i SITI
//! CONVERTITI, non impedisce a un sito nuovo di comporre `Rc::new` con
//! `RefCell::new(zval)` direttamente — la chiusura resta ricerca
//! esaustiva. Questo ratchet la rende meccanica: ogni occorrenza del pattern
//! fuori da `php-types/src/zval.rs` (il funnel) sta nell'allowlist con il suo
//! conteggio ESATTO e il payload dichiarato (Object/Resource/GenState o
//! #[test]: mai Zval). Un sito nuovo fa fallire la batteria: o passa dal
//! funnel (payload Zval) o entra QUI con payload dichiarato non-Zval.

use std::path::{Path, PathBuf};

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn rczval_pattern_resta_nel_funnel() {
    // Pattern composto a pezzi: questo file stesso è nel perimetro della scansione.
    let pat: String = ["Rc::new(", "RefCell::new"].concat();
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    // (path relativo a crates/, conteggio esatto, payload dichiarato)
    let allow: &[(&str, usize, &str)] = &[
        ("php-types/src/zval.rs", usize::MAX, "IL FUNNEL (zcell/zcell_prop + doc + test)"),
        ("php-types/src/array.rs", 1, "#[test] set_returning_displaced_equals_composite"),
        ("php-types/src/object.rs", 1, "payload Object (mint principale)"),
        ("php-runtime/src/vm/coroutines.rs", 1, "payload GenState"),
        ("php-runtime/src/vm/host.rs", 6, "payload Resource"),
        ("php-runtime/src/vm/mod.rs", 8, "payload Object/Resource"),
        ("php-runtime/src/vm/run.rs", 2, "payload Object"),
    ];
    let mut files = Vec::new();
    rs_files(crates, &mut files);
    assert!(files.len() > 100, "scansione sospetta: {} file .rs", files.len());
    let mut bad = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let n = src.matches(&pat).count();
        if n == 0 {
            continue;
        }
        let rel = f.strip_prefix(crates).unwrap().to_string_lossy().replace('\\', "/");
        match allow.iter().find(|(p, _, _)| *p == rel) {
            Some((_, exp, _)) if *exp == usize::MAX || n == *exp => {}
            Some((_, exp, why)) => bad.push(format!(
                "{rel}: {n} occorrenze, attese {exp} ({why}) — sito nuovo? Zval passa da zcell()"
            )),
            None => bad.push(format!(
                "{rel}: {n} occorrenze FUORI allowlist — Zval passa da zcell(); non-Zval si dichiara qui"
            )),
        }
    }
    assert!(bad.is_empty(), "dente rczval-funnel:\n{}", bad.join("\n"));
}
