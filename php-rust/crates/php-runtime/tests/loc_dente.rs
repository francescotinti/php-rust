//! S-151 (concilio, sintesi §RATIFICA-A4 + team-struttura §Spec-dente-A4):
//! dente anti-ricrescita sulle RIGHE dei sorgenti .rs. Sede = BATTERIA (la CI
//! locale è specchio tardivo). Misura = `lines().count()` (≡ wc -l), MAI
//! pattern testuali componibili (lezione auto-morso bea7ea3). Regole:
//! - file NUOVO (fuori allowlist): ≤ 2.000 righe;
//! - allowlist (path, cap, motivo) a cap ESATTI odierni (plenaria S-151 §4;
//!   minoritaria Hejlsberg +50 a registro): mai crescere oltre il cap;
//! - anti-slack (KS-E): cap − n ≤ 200, chi snellisce abbassa il cap nello
//!   STESSO commit; cap solo in discesa, salita = diff dichiarato a verbale.

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
fn nessun_sorgente_rs_oltre_cap() {
    const CAP_NUOVI: usize = 2000;
    const SLACK_MAX: usize = 200;
    // Cap = conteggi ODIERNI (S-151, pin s150) verificati con wc -l.
    let allow: &[(&str, usize, &str)] = &[
        // Salita DICHIARATA a verbale (S-153, leva L-BT2 promossa a verdetto
        // A/B rc=0: BtFrame→ZStr in mod.rs +3; pool BT_STATICS + riscrittura
        // ho_debug_backtrace in host.rs +35 — s153-ab-bt2-verdetto.out).
        // Salita DICHIARATA a verbale (S-154, leva L-CE1 vinta all'A/B:
        // resolve_named/resolve_class_autoload via LcKey, +5 righe di
        // commento-leva in mod.rs — s154-ab-ce1b-verdetto.out + arbitrato).
        // S-156: salita DICHIARATA +30 (leva HD2-hostcall: macro a due
        // sezioni + tabella slice) — verbale wp156-harness/s156-promo.
        // S-158: salita DICHIARATA +1 (leva L-RF2 vinta all'A/B: sei nomi
        // __reflect_* spostati vec->slice + riga di commento tranche-2 —
        // s158-refl2-verdetto.out; criterio refl2 p.9, salita pre-dichiarata).
        // S-157: salita DICHIARATA +36 (leva L-AL1 vinta all'A/B: pool
        // guard-key + arg rc-clone in try_autoload, delega
        // resolve_class_autoload_with — s157-al1-verdetto.out + arbitrato).
        // S-159 L-AM1: salita PRE-dichiarata (criterio s159-criterio-am1.md
        // p.8) — mod.rs +31 (push_closure_frame_one), host.rs +22 (fast path
        // array_map); calls.rs (+21) resta fuori allowlist sotto il cap 2000.
        // S-161: salita DICHIARATA +21 (leva L-AL2: fast path loader autoload
        // k=1 in try_autoload — criterio s161-criterio-al2.md p.1).
        // S-162 L-AM2: salita PRE-dichiarata (criterio s162-criterio-am2.md
        // p.1) — mod.rs +16 (push_fn_frame_one), host.rs +18 (fast path
        // array_map string-callable); calls.rs (+38) resta fuori allowlist
        // sotto il cap 2000.
        // S-163 L-AU1: salita PRE-dichiarata (criterio s163-criterio-au1.md
        // p.1) — mod.rs +62 (push_method_frame_one +28, ammissione
        // array-callable in try_autoload +34); calls.rs (+31,
        // call_method_one) resta fuori allowlist sotto il cap 2000.
        ("php-runtime/src/vm/mod.rs", 25909, "monolite VM — bersaglio A2; +62 L-AU1 S-163 PRE-dichiarato"),
        ("php-runtime/src/vm/host.rs", 7726, "hostcall — backlog A2; +18 L-AM2 S-162 PRE-dichiarato"),
        // S-156: salita DICHIARATA +29 (leva HD2-hostcall: braccio
        // CallHostBuiltin, pop diretti ≤4) — verbale wp156-harness/s156-promo.
        // S-165 L-MC1: salita PRE-dichiarata +79 (fast path borrow-IC k≤2 in
        // Op::MethodCall — criterio s165-criterio-mc1.md p.6). Gli
        // unreachable!×2 in calls.rs (az.rev. S-163 #4) sono stati RIMOSSI a
        // verdetto di misura (braccio C null-edit: −5 ns su arrload —
        // s165-criterio-nulledit.md): l'azione si chiude a verbale, non a dente.
        // S-165 L-MC1b: +20 DICHIARATI (forma outline: corpo in
        // methodcall_fast #[inline(never)], cura layout — arbitrato
        // s165-arbitrato-guardie.md; l'arm di run_loop torna a ~10 righe).
        ("php-runtime/src/vm/run.rs", 6914, "run_loop — ULTIMO o mai (A2); +99 L-MC1b S-165 dichiarato"),
        ("php-runtime/tests/eval.rs", 4773, "batteria eval"),
        ("php-builtins/tests/builtins.rs", 4772, "batteria builtins"),
        ("php-runtime/src/lower/mod.rs", 3838, "lowering"),
        ("php-runtime/src/vm/dom.rs", 3641, "ponte DOM"),
        ("php-types/src/big5.rs", 3372, "GENERATO, cap fisso"),
        ("php-builtins/src/string.rs", 2865, "builtins stringhe"),
        ("php-builtins/src/file.rs", 2758, "builtins file"),
        ("php-runtime/src/compile/expr.rs", 2590, "compile expr"),
        ("php-builtins/src/date.rs", 2458, "builtins date"),
        ("php-runtime/src/bytecode.rs", 2306, "bytecode"),
        ("php-runtime/src/preg.rs", 2290, "preg"),
        ("php-types/src/memcensus.rs", 2268, "strumentazione census"),
        ("php-runtime/src/lower/class.rs", 2168, "lowering classi"),
        ("php-runtime/src/vm/arrays.rs", 2167, "array ops"),
        ("php-runtime/src/lsp_check.rs", 2094, "lsp check"),
        ("php-builtins/src/fileinfo.rs", 2083, "builtins fileinfo"),
        ("php-server/src/worker_pool.rs", 2074, "server worker pool"),
        ("php-runtime/src/lower/expr.rs", 2029, "lowering expr"),
    ];
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut files = Vec::new();
    rs_files(crates, &mut files);
    assert!(files.len() > 100, "scansione sospetta: {} file .rs", files.len());
    let mut bad = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let n = src.lines().count();
        let rel = f.strip_prefix(crates).unwrap().to_string_lossy().replace('\\', "/");
        match allow.iter().find(|(p, _, _)| *p == rel) {
            Some((_, cap, why)) => {
                if n > *cap {
                    bad.push(format!(
                        "{rel}: {n} righe > cap {cap} ({why}) — ricrescita: si snellisce, o salita dichiarata a verbale"
                    ));
                } else if cap - n > SLACK_MAX {
                    bad.push(format!(
                        "{rel}: {n} righe, cap {cap} lasco (> {SLACK_MAX}) — abbassare il cap nello STESSO commit"
                    ));
                }
            }
            None if n > CAP_NUOVI => bad.push(format!(
                "{rel}: {n} righe > {CAP_NUOVI} (file fuori allowlist) — si spezza o si dichiara nell'allowlist col motivo"
            )),
            None => {}
        }
    }
    assert!(bad.is_empty(), "dente loc (righe .rs):\n{}", bad.join("\n"));
}
