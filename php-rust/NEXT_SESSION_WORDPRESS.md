# NEXT_SESSION_WORDPRESS.md — S-81.0: LEVA A-BB6 SPEDITA, churn VERDICT PASS → WP-82 (misure residue footprint/CPU/retained)

**Ultima sessione**: S-81.0 (2026-07-31 sera, commit 62cd100…f6e13c3) —
ordine vincolante Concilio WP-82 eseguito passi 1-7: fix meccanici (6fd8aac),
ORM 16/16 + hk 0E/0F su ef90cb19 PRE-leva (7593d8e), driver_sha+porcelain
harness (88ea8ee), **LEVA A-BB6 (57ec7dc)** con tutti i same-commit (epoch
u64, MAIN_CHAIN_FP computato+falsificatore, allowlist vm_new, put dopo
link_fatal_check, bare.php riscritta, M-68.5 superata, A-DS12 §5 in due
classi, A-DS14 scelte pinnate), strumenti §11 (drained_*, SESTETTO
mem-census, RetainedWalk+controllo positivo, stranded_bytes), fixture
F1-F13 TUTTE VERDI, **A/B churn a verdetto MACCHINA: PASS su tutte le
predizioni §10** (`wp81-harness/MEASURE81_RESULTS.md` + verdict81.sh/out).
Punto 8 (revert): mai innescato — zero body ≠ oracolo.
Dettaglio: `sessions/WP_SESSION_81.md`.

## Stato gate

- **phpr (CLI, parità release)**: **f33151fbe383159c** (stash additivo
  `phpr-wp81`; bit mossi dalla leva) — corpus Zend per NOME: **1418
  IDENTICO** + refl 290 IDENTICO a ef90cb19 (`wp81-harness/evidence/`);
  workspace 0 fail.
- **php-server SESTETTO (git 3f32c16, matrix `tree=clean`, archivio
  per-run)**: union **9f9f8d92ff969729** · census **12a8777c8c38fdc4** ·
  **mem-census d5bba760069639e0** (nuova config KS-AH-82-4, lane CI) ·
  census-axum-only 13c429be · axum-only e2183043 · default cee8c63e.
  Battery COMPLETA PASS sul tree leva: run-gate · census-twin (pin
  worker_pool 20→23 NOMINATO) · concurrent · worker-panic · stdout-tandem ·
  capture-order · doc-purge (12 pattern, ora anche fixtures/*.php) · DR-1 ·
  **gate-lever-pins** (A-MS13/A-PP16/KS-PP-82-3/A-TH14) ·
  **gate-lever-fixtures 1+2** (F1-F13).
- **Misura leva (churn)**: `wp81-harness/MEASURE81_RESULTS.md` — hello
  a_calls HIT **80.476→2**, floor bare-HIT=2≤200 (KB-82-3 regge), a1==0,
  a3==0, b/resid INVARIANTI ESATTI, retain 1/1/3/6, idle drift 0/0@60s,
  spread 0,00% ri-derivato in-campagna, driver_sha in ogni header.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (invariate)

1. **Output capture BEFORE request_end()**. 2. **Isolamento = semantica
FPM** (A-DS2). 3. **RetainSet thread-affine PER-RICHIESTA, sigillato**;
la porta vm_new/park_main è ALLOWLISTED (gate-lever-pins). 4. **Panic =
FAIL-FAST**. 5. **Un body ≠ oracolo sulla leva ⇒ git revert, mai
fix-forward** (KS-DS-80-3, mai innescata).

## §WP-82 (prossima sessione) — misure RESIDUE dell'A/B, poi verdetto A-BB6 COMPLETO

I residui sono DICHIARATI in MEASURE81 §Residui — nessuno è chiuso:
1. Pre-flight skill + battery verde + matrix sestetto sul tree di apertura.
2. **Footprint twin** (design79 §11.3): union NON strumentato, V2 steady
   W=1 su N vs 2N con floor vmmap 0,1MB (N scalato: floor ≤ 1/10
   dell'effetto) + **peak a W=num_cpus** (KL-80-2; lo spread 12% W=10 è
   metrica di verdetto). Coppia build-adiacente stessa-sera vs phpr-wp80/
   binari WP-80 se si cita un delta.
3. **CPU slope due-N** (100/200 req, KB-81-5) con risoluzione ex-ante
   dichiarata (KL-82-3) + costo scan supersede (A-DS9, soglia numerica).
4. **Retained ×W**: run mem-census (d5bba760) su hello+include_heavy,
   cifra retained_walk_bytes (etichetta "≥, Const esclusi") → budget ×W
   DICHIARATO (KS-MS-82-2) prima di ogni lettura ulteriore del braccio leva.
5. **Fixture autoload-run** (KB-81-3/KB-82-5) PRIMA di citare "HIT salta
   a3" come verdetto.
6. **Battery-su-HIT col pin ESPLICITO main_hit per richiesta** (KS-SK-82-3).
7. ORM/hk POST-leva build-adiacente se l'A/B li cita (KS-AH-82-3).
8. F4: giudizio del Concilio WP-83 sulla deviazione dichiarata (nessun
   path main-impuro sul tree).

**Kill-switch di rotta (attivi)**: tabella WP-82 (30 KS — verbale
`wp82-harness/COUNCIL_WP82_REVIEWS.md` §Kill-switch) + tabella WP-81 +
mappa design79 + storici. KH82-1 SODDISFATTO (epoch u64 nel commit leva);
KS-AH-82-2 SODDISFATTO (falsificatore in-cargo); KS-PP-82-2/3 SODDISFATTI
(F8c contatori; SplitDrain-primo-return a macchina); KG-82-1 SODDISFATTO
(verdict81.sh); KS-SK-82-4 rispettato (R=3, spread in-campagna).

**NON riproporre**: tutti i NON-riproporre WP-81 restano; in più — cifra
retained citata senza riga matrix mem-census + lane CI (KS-AH-82-4);
campagna con harness toccato mid-run (l'identità del protocollo include
gli script: la campagna si rifà INTERA a un rev); claim CPU derivato dal
floor alloc (KB-82-4 — il floor misurato è 2 call, alloc-invisibile ≠
CPU-invisibile).

**Deferiti WP-83+**: A-TH4 (request_end(self)); A-AH5/A-BB4 (superglobali
reali axum — KS-DS-82-3 attivo: mai fixture $_GET/$_SERVER censuscli↔axum
pre-A-BB4); registry condivisa read-only; spread 12% picco W=10.

---
**Chiusura**: 2026-07-31. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`. ⚖️ Concilio WP-83: verbale in
`wp83-harness/COUNCIL_WP83_REVIEWS.md` (vincolante per WP-82).
