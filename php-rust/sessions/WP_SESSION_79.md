# WP_SESSION_79.md — S-79.0 PRE-LEVER HARDENING (8/8) + design79 A-BB6 — ✅ ordine Concilio WP-80 eseguito integrale

**Data**: 2026-07-31 (sera)
**Scope**: ordine vincolante S-79.0 del Concilio WP-80 (§Sintesi, 8 punti,
non rinegoziato) + design leva A-BB6 SOLO dopo i punti 1-7. Modello
verificato all'apertura: Fable 5 (regola post-WP-78).
**Commit**: ff22a2e → 2528cfd → 31ddc30 → 83c8dea → 075947e → 5966d56 →
36d62df (tutti su main, pushati).
**Binari**: phpr **a76acc1668d03c86** (nuova baseline parità, stash additivo
`phpr-wp79`; corpus per NOME PASS 1418 set IDENTICO a GATE72); quaterna
php-server a git 36d62df: union **0405077667de3f2d** · census
**80f88ea7efcb3c4a** · axum-only d010e291 · default 8d6c7778 (tutte nel
feature-matrix.log della stessa build, A-AH10).

## S-79.0 eseguita (8/8 punti dell'ordine vincolante)

| # | Esito | Commit |
|---|---|---|
| 1 | Depth **inc-PRIMA-di-send** (undo su send fallita) + test anti-wrap (200 cicli, bound sul watermark) + `reset_depth_stats()` test-only + `DEPTH_TEST_LOCK` (i contatori sono process-global) — KH80-4 sbloccato: l'osservabile KH78-2 non può più sotto-contare | ff22a2e |
| 2 | Doc purge: frase falsa di `request_end` corretta (il survivor è la unit cache TL, MAI il RetainSet) + 2 pattern nuovi nel gate (9/9 esca); SAFETY inline su AssertUnwindSafe | 2528cfd |
| 3 | **Census split a1/a2/a3**: modulo `alloc_census` in php-runtime (feature nuova, snapshot provider iniettato via OnceLock), bracket su lower_prelude/prefisso compile/run_include (finestre a3 RAII annidabili nette delle interne); riga census estesa (a1/a2=a−a1/a3 + **resid** = gap s3→s0, riconciliazione denominatore A-BB11); controlli positivi in-cargo (a1>0 sempre; **a3>0 su MISS, ==0 su HIT** — il discriminatore di A-BB6; canale c mosso da teardown-allocante, KB-80-5); fixture 2ª `include_heavy.php` + 5 lib (byte-identica all'oracle al 1° colpo); probe idle `/__census_global` | 31ddc30 |
| 4 | **Quaterna d'identità**: census 4ª riga del feature-matrix sotto -D warnings + in CI (build E test); 🔵 **scoperto: `-D warnings` via cargo rustc CAMBIA i bit** (07e6fd9d vs 69760159 stessa sorgente) ⇒ hash loggati dal path operatore (cargo build) + assert on-disk==bin[union] a fine gate (A-AH12); `gate-census-twin.sh` (A-AH11: marker census pinnato 18/11/2 + full battery vs oracolo sul binario census); run-gate parametrico FEATURES; **driver ENFORCE mode-aware** (hash==riga matrix o rifiuta; righe==N; depth>1 ⇒ exit 1; tier0 N/A-by-construction) | 31ddc30 |
| 5 | Gate hardening: **overlap PROVATO senza strumentazione** (8×400ms su W=4 in 0,92-0,94s vs 3,40-3,43s su W=1 — il controllo discrimina, KS-SK-80-1); stdout-tandem verbi estesi (assegnamento, write!, mem::swap/take, clear) + tandem-di-SCRITTURA + NSITES==6 pinnato; marker capture-order pinnato per POSIZIONE (is_empty + cfg(test)); worker-panic RC≠134 ⇒ FAIL + "all 2 workers joined" pinnato; **panic-hook abort globale in modalità axum** (forma forte di A-MS4): panic nel dispatcher ⇒ abort 134 (Phase C nuova; prima: task tokio morto, richiesta appesa) | 83c8dea |
| 6 | Test in-cargo a **forma worker_loop** (RetainSet fresco per richiesta in tutte le famiglie); **riga `census-cli:` sul path cli-server** (A-BG16 ⇒ A-BB1 giudicabile): php-cli feature weak-dep, bracket engine-window in run_php; driver modo `censuscli` | 075947e |
| 7 | **Stranded keys CHIUSE** (KS-DS-80-1): supersede-per-path alla put della unit cache + contatori `stranded_keys_superseded/entries_dropped` (riga tag=unitcache; il grouping dump-time diventa controllo NEGATIVO ≡0) + evento uc_log; test edit-workload sul path REALE (3 edit size-distinti ⇒ 1 sola chiave residua + contatore ≥2) | 5966d56 |
| 8 | **design79.md**: leva A-BB6 = STESSA unit cache TL estesa al MAIN (A-DS5; M-68.5 superato deliberatamente, audit M-67.2 da rifare), fingerprint mai mtime, pin nel RetainSet (KS-PP-80-2), **DR-1 audit IC/immutabilità bloccante**, 9 fixture A-DS6, predizione ex-ante (a_calls hello <4.000, soglia calo ≥90% KS-AH-80-4), A/B coppia churn+retained + peak W=num_cpus + CPU + corpus per NOME, revert mai fix-forward | 36d62df |

## 🔵 Il dato che ridimensiona (e conferma) A-BB6

Census-cli smoke su include_gate (R=1, dichiarato — la misura R≥3 è il primo
atto della fase misura): finestra engine 83.834 call / **13,36MB**, di cui
**a1 prelude = 74.288 call / 10,83MB (~81%)**, a3 include-compile COLD 2.156
call / 198KB (→0 su HIT), a2 main ~2,3MB. Le 3 sedie avevano ragione: il
"99% fase a" era ~4/5 PRELUDE. La leva però resta intera: la forma fedele
cacha la UNIT MAIN, che CONTIENE il prelude compilato ⇒ un HIT salta a1+a2
insieme (~13MB/req). Dimensionamento su ASSOLUTI (KB-80-1 rispettato).

## Verifiche trasversali (tree finale 36d62df)

- workspace `cargo test --release`: **1652/0/1** (baseline 1651 + test
  supersede nuovo).
- **Corpus Zend per NOME: PASS — 1418 fail, set IDENTICO a GATE72** (phpr
  a76acc16; unica modifica engine non feature-gated = supersede-per-path).
- Battery server RILANCIATA sul tree finale: feature-matrix PASS (quaterna)
  · run-gate union PASS (04050776) · census-twin PASS (80f88ea7) ·
  worker-panic PASS (3 fasi, entrambi gli abort exit=134) · concurrent PASS
  (overlap 0,917s / W1 3,429s) · doc-purge 9/9 · capture-order (marker 1/1
  + posizione) · stdout-tandem 6/6.
- php-server test: 9/9 (axum) + 14/14 (census).

## ⭐ Lezioni

1. ⭐⭐ **`-D warnings` cambia i BIT del binario**: `cargo rustc -- -D
   warnings` e `cargo build` sulla stessa sorgente producono hash diversi —
   un log d'identità deve hashare il binario COME LO COSTRUISCE l'operatore,
   o l'ENFORCE respinge ogni build corretta (A-AH12 misurato, non presunto).
2. ⭐⭐ **L'osservabile di sovrapposizione può essere il wall-clock**: 8×400ms
   in 0,92s su W=4 vs 3,43s su W=1 — prova di concorrenza SENZA strumentare
   il gemello, col controllo discriminante che uccide la fixture che non
   dorme.
3. ⭐⭐ **Il fail-fast va nel panic-HOOK, non nel catch**: un panic nel task
   axum era inghiottito da tokio (richiesta appesa, processo vivo) — l'unico
   posto che copre TUTTI i thread è il hook globale; il catch_unwind resta
   come cintura.
4. ⭐⭐ **Un canale aggregato si decompone PRIMA di progettarci sopra**: lo
   split a1/a2/a3 ha ribaltato l'attribuzione (81% prelude) senza uccidere
   la leva — ma ora il design predice sul sotto-canale giusto e la soglia
   ex-ante (≥90%) è falsificabile.
5. ⭐ **Contatori globali nei test = lock condiviso o PASS ereditati**: il
   watermark depth è process-global; senza `DEPTH_TEST_LOCK`+reset un test
   può passare col traffico di un altro.

## Residui / NON fatto (dichiarato)

- Misura census R≥3 (hello + include_heavy, entrambe le arm axum e
  censuscli) NON eseguita: è il PRIMO atto della fase misura pre-A/B
  (design79 §1/§10) — lo smoke R=1 è dichiarato tale.
- A/B della leva A-BB6: non iniziato (il design è il contratto).
- DR-1 (audit IC/immutabilità Module): primo atto dell'implementazione.
- ORM/hk gate non rilanciati (il supersede scatta solo su file EDITATI tra
  compile della stessa path nello stesso processo — workload assente in
  quei gate; il corpus per NOME copre l'engine).
- Deferiti invariati: A-TH4 (request_end(self)→CapturedOutput), A-AH5/A-BB4,
  registry condivisa read-only.
