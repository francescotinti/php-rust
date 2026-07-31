# WP_SESSION_81.md — S-81.0 "SEALS & INSTRUMENTS + LEVA A-BB6" — ✅ ordine WP-82 passi 1-7; churn VERDICT PASS; residui footprint/CPU/retained DICHIARATI

**Data**: 2026-07-31 (sera)
**Scope**: ordine vincolante Concilio WP-82 (§Sintesi, 8 passi). Eseguiti
1-7 (l'8 = revert policy: mai innescata — nessun body ≠ oracolo). Modello
verificato all'apertura: Fable 5.
**Commit**: 62cd100 → 6fd8aac → 88ea8ee → 7593d8e → **57ec7dc (LEVA)** →
7400b7f → 781e235 → 85941a1 → 9257dee → 552e6fb → d3437d9 → 2dc11eb →
542af3d → 3f32c16 → f6e13c3 (tutti su main, pushati).
**Binari**: phpr **f33151fbe383159c** (nuova baseline parità, stash
additivo `phpr-wp81`; corpus 1418 + refl 290 per NOME IDENTICI a
ef90cb19); SESTETTO php-server a git 3f32c16: union **9f9f8d92ff969729** ·
census **12a8777c8c38fdc4** · **mem-census d5bba760069639e0** ·
census-axum-only 13c429be · axum-only e2183043 · default cee8c63e.

## Ordine WP-82 eseguito

| # | Esito | Commit |
|---|---|---|
| 0 | Apertura: matrix quintetto sul tree di sessione (hash IDENTICI a 6910767, equivalenza COMANDATA `git diff --stat -- crates/` vuoto, KH82-2) + battery integrale PASS | 62cd100 |
| 1 | Fix meccanici: A-PP15 (mod tests ANCORATO+negativo, anche in gate-dr1) · A-SK14 (pin idle==4 + self-test sintetico 4/5 righe) · A-BG24 ($RUN.summary per-run) · A-MS14 (TRIP_TEST_LOCK) · A-MS15 (a1 open-while-open ⇒ trip + test) · A-MS16 (dispatch-Err VOID dichiarato) · A-DL14 (gross=1 census-global 2 arm) · A-TH15 (OUTSTANDING ri-scoped: watermark>1 = run VOID) · A-TH17 (DR-1 a occorrenze gsub + decoy 2-token) · A-AH24/25 · A-SK15/18 dichiarazioni | 6fd8aac |
| 2 | **A-AH23**: ORM 3484 3E/13F per NOME 16/16 IDENTICO + hk 1665 0E/0F su phpr ef90cb19 PRE-leva (watchdog; evidence in wp81-harness/evidence/) | 7593d8e |
| 3 | **A-AH21**: driver_sha nell'header+raw; porcelain esteso a wp7*/wp8* *.sh/*.pl | 88ea8ee |
| 4 | **LEVA A-BB6** con TUTTI i same-commit: `main_unit_acquire`/`MainLever` nella STESSA unit cache (A-DS5); MAIN_CHAIN_FP COMPUTATO single-binding + falsificatore in-cargo (A-AH22); epoch **u32→u64** (KH82-1); publish `publish_if_armed` UNICO, DOPO link_fatal_check (A-PP16); park_main nel RetainSet (A-DS8 1→2 PER NOME); contatori main_* (UcStats+uc_log union-build); A-DS14 scelte pinnate (ini assert-vuoto ESEGUIBILE; autoload = F10, impossibile pre-Vm); A-DS15 dichiarato al bump; M-68.5 superata DELIBERATAMENTE + frase bandita; bare.php riscritta + doc-purge su fixtures (A-BB22); design79 §3/§5(DUE CLASSI+enumerazione A-DS12)/§9(F13, F-oneshot 3 denti, deviazione F4) emendati; gate-lever-pins.sh (allowlist A-MS13 + ordini + KS-PP-82-3 + A-TH14) e gate-lever-fixtures.sh (F-oneshot 3 denti, F5, F8b, F8c CONTATORI) PASS al primo run | 57ec7dc |
| 5 | Strumenti §11: drained_* ledger + census-drained + test positivo con DRAIN_TEST_LOCK (A-PP14) · SESTETTO mem-census (matrix+CI, debito -D pagato: 8 doc-comment morti) · **RetainedWalk** visited-set per puntatore + walk Program + controllo positivo S1+S2−Sshared ESATTO (KL-82-2, come selftest php-server: i lib-test mem-census non linkano mimalloc) · stranded_bytes_dropped al drop del supersede | 7400b7f/781e235/85941a1 |
| 6 | Fixture: F1 (put==2), F2 (SAME-SIZE **SAME-MTIME** via touch -r: solo il fp discrimina), F3 (symlink=revalidate_path 1), F4 (impure-skip==0, deviazione dichiarata), F6 in-cargo (retain==3), F7 (ordine classi ×3 == oracolo), F9 (supersede VISTO ×5), F10 (mai binding stantio), F11 (__FILE__ == oracolo su HIT), F12 (strict_types HIT ×3), F13 (ordine include ×4 == oracolo + CONTROLLO POSITIVO comparatore) — TUTTE VERDI | 9257dee |
| 7 | **A/B churn**: campagna 26/26 R=3 ENFORCE a UN rev, verdetto MACCHINA (verdict81.sh): **hello a_calls HIT 80.476→2 (−99,997%)**, floor bare-HIT=2≤200 (KB-82-3 regge), a1==0, a3==0, b/resid INVARIANTI ESATTI, retain 1/1/3/6, idle drift 0/0 a 60s (canali ciechi enumerati KL-82-1), spread 0,00% ri-derivato in-campagna; cli 81.613→1.140; corpus 1418 + refl 290 per NOME IDENTICI; workspace 0 fail; battery integrale PASS | f6e13c3 |

## 🔵 Scoperte

1. **Il probe main è quasi alloc-invisibile**: a_calls(HIT)=2 su OGNI
   fixture — canonicalize+stat+hash lavorano fuori dal GlobalAlloc; il
   floor ex-ante ≤200 era 100× conservativo. (Bound su ALLOCAZIONI: la CPU
   resta da giudicare con la slope, A-BB25.)
2. **Il determinismo dei contatori SOPRAVVIVE alla leva**: spread 0,00%
   anche con canonicalize/stat/hash nel path (ri-derivato, non ereditato).
3. 🐛 due morsi di gate DAL VIVO, entrambi pre-merge: A-AH6 (DRAIN_TEST_LOCK
   dead nel test-build axum-only) e -D census (stranded field senza lettore
   in quel perimetro).

## ⭐ Lezioni

1. ⭐⭐ **L'harness è parte dell'IDENTITÀ del protocollo**: scrivere
   verdict81.sh MENTRE la campagna girava ha fatto rifiutare 13 run dal
   porcelain A-AH21 appena introdotto — il tripwire ha morso il suo autore;
   campagna rifatta INTERA a un rev. Mai toccare harness durante una
   campagna.
2. ⭐⭐ **Ogni superficie nuova paga il -D subito**: due warning nati in
   sessione uccisi dai gate nello stesso giro (A-AH15 di nuovo).
3. ⭐⭐ **acquire/publish come API unica + probe un-parametro** tiene i due
   SAPI a wrapper sottili: A-TH14 per costruzione, pin a macchina.
4. ⭐ I lib-test di un instrument che linka simboli dell'allocatore del BIN
   vivono come pub-selftest chiamati dal crate bin.
5. ⭐ Il "floor misurato gratis" di Bak (bare su HIT) ha chiuso in un colpo
   floor ex-post, delta hello−bare e A-BB24.

## Residui / NON fatto (dichiarato, MEASURE81 §Residui)

- **Footprint twin** (V2 N/2N + peak W=num_cpus): verdetto memoria RESIDENTE
  APERTO. - **CPU slope due-N** (+ scan supersede A-DS9): ogni claim CPU =
  ADVISORY. - **Retained ×W**: strumento pronto e controllato, cifra su run
  reale + budget NON prodotti (budget NULLO, KS-MS-82-2). - **Autoload-run
  fixture** (KB-81-3): "HIT salta a3" resta ADVISORY. - **Battery-su-HIT col
  pin ESPLICITO main_hit** (KS-SK-82-3): la census-twin battery GIRA su HIT
  ma senza conteggio pinnato ⇒ formalmente "path misto". - F4: nessun path
  main-impuro esiste (deviazione da giudicare al Concilio WP-83).
- ORM/hk POST-leva non rilanciati (pre-leva 16/16 fatti al passo 2;
  KS-AH-82-3 esige il confronto build-adiacente quando l'A/B li cita).
