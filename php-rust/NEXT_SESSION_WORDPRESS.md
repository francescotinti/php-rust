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

## ⚖️ Concilio WP-83 ESEGUITO (2026-07-31, verbali VINCOLANTI): `wp83-harness/COUNCIL_WP83_REVIEWS.md`

**9× CONCORDO CON EMENDAMENTI, 0 opposizioni.** La leva è giudicata REALE:
clone-on-stack borrowck-enforced, put-dopo-link su entrambi i path, purezza
del main verificata al lowering, e **a_calls(HIT)=2 PROVATA dall'aritmetica
dei raw** (Bak: a_bytes==2×len(path canonico) ESATTO; Gregg: md5 r1==r2==r3
su tutte le 8 coppie). REFUTATI: **strumenti di verdetto non fail-closed**
(field() vuoto passa, P5-P7 solo-r1, percentuali a mano nell'header
"scriptato" — A-SK19/20, A-BG26/27/28), **il one-shot NON passa dall'acquire**
(run_source_with_argv duplica lower+compile: F-oneshot t2 vacua, A-TH21),
**u64 chiuso nel codice non nella macchina** (gate-dr1 dice ancora u32,
A-TH19/KH83-1), **ledger drained non esatto** (manca il resid del fatal,
A-PP18), **raw VOID rimossi senza quarantena** (A-AH27), **retained MAIN =
FLOOR non budget** (add_program shallow 1-2 ordini, selftest non lo esercita
— A-DL15/16 + battery mem-census A-AH29), **F13 insufficiente per la classe
2** (serve F14: condizionali+eval-mint+deferred, A-DS18), **eviction thrash
main/include stessa key** (FIFO senza refresh: main_evicted + F15, A-DS20),
**sigillo di TIPO sulla porta vm_new** (token ZST, A-MS17; newtype per il
fp, A-AH26). Ordine vincolante S-82.0 = verbale §Sintesi (8 passi) + 30 KS
nuovi (KH83 · KS-MS-83 · KS-SK-83 · KS-AH-83 · KB-83 · KS-PP-83 · KL-83 ·
KS-DS-83 · KG-83).

## §WP-82 (prossima sessione) — S-82.0 "honest instruments" poi misure RESIDUE

**Ordine = Concilio WP-83 §Sintesi (8 passi, non rinegoziare)**: 1. strumenti
di verdetto fail-closed (A-SK19/20, A-BG26/27/28, KG-83-3, MEASURE81
retro-annotato) · 2. fold one-shot su acquire (A-TH21 + A-SK23/24) ·
3. denti macchina sui sigilli (A-TH19 pin u64, A-TH20, A-MS18/19/20, A-AH26,
A-PP19/20/21, A-DL17/18, A-TH22) · 4. quarantena raw (A-AH27, KH83-2) ·
5. strumento retained onesto PRIMA del budget (A-DL15/16, A-AH29/30) ·
6. fixture nuove (F14 A-DS18 · F15+main_evicted A-DS20 · F2 same-key A-SK21
· trigger F4 A-DS19 · A-DS16/17 · spike resid req=11 NOMINATO KB-83-2) ·
7. SOLO POI misure residue: footprint twin V2 N/2N + peak W=num_cpus (floor
A-DL19) · CPU slope N=1000/2000 (soglie Bak: ≤×1,05+25µs/req; scan ≤1µs/key)
· battery-su-HIT TWIN-PAIR (Pedersen Q5/A-SK25) · autoload-run (KB-82-5) ·
ORM/hk perf SOLO build-adiacente a 7593d8e (KS-AH-83-4) · budget ×W solo
dopo il passo 5 (KL-83-1) · 8. revert policy KS-DS-80-3 invariata.
F4: deviazione ACCETTATA da Stogov col trigger test-only richiesto (A-DS19).

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
