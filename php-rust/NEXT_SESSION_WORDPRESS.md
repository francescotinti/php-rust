# NEXT_SESSION_WORDPRESS.md — S-84.0: SIGILLI+ORACOLI+MISURA-CHE-DECIDE (verdict84 PASS, ×W PER-THREAD, pin peak MOSSO) → WP-85(sessione)

**Ultima sessione**: S-84.0 (2026-08-01, commit 746bc24…9c2f946) — gli 8
punti del Concilio WP-85 eseguiti: sigillo VmGate v2 LIFETIME-BOUND +
probe cfg-gated (KH85-1 chiuso da rustc), battery-equivalence v2
(OUT↔BREV nel codice, ledger canonico, manifest trasclusi), denti A-DS26
(iniezione: il tripwire main_evicted HA MORSO, F16 armato in battery) /
A-DS28 (sig esteso; SCOPERTA: closures in `m.closures`, non m.functions) /
A-MS28 (put-path raccogli-poi-emetti, drop fuori dal borrow) / A-TH29 /
A-SK35 / A-PP26/28/29 / base-arm A-AH35+A-TH30, **campagna measure84 +
VERDICT84 PASS**: **VDL24 PER-THREAD** (×W regge), **VP pin MOSSO**
(nominato), **VA SANATA** (oracle ledgerato A-DS29), **VW piecewise
CONFERMATO** al sito NOMINATO (std run_path_with_cstr 384). Dettaglio:
`sessions/WP_SESSION_84.md` + `wp84-harness/MEASURE84_RESULTS.md`.

## Stato gate

- **phpr (CLI, parità release)**: **c4448075401dee5f** (stash additivo
  `phpr-wp84`; bit mossi da A-MS25/A-MS26/A-MS28/probe A-DL24) — corpus
  Zend per NOME: **1418 IDENTICO** + refl 290 IDENTICO (battery-84pre
  parity-full, `wp84-battery-out/` fuori repo); workspace test exit 0.
- **php-server**: battery-84pre **15/15 per NOME a 937e79d** (nessuna
  equivalenza usata: battery VERA a HEAD); matrix rigenerata in battery
  (prima voce), campagna stesso-chain senza commit intermedi; union
  d440c3411c12401a · census d70b86d0502ea7e7 · mem-census
  85fd009f66e7d3e4. Gate lever: pins (v2 + 1c produttore) / fixtures /
  fixtures2 (F14b+A-SK35, F15/F15b+A-DS27, **F16 nuovo**) tutti PASS.
- **Misure**: `wp84-harness/MEASURE84_RESULTS.md` + verdict84.out —
  VDL24 PER-THREAD (7.349.977 B = 7,01 MiB su ENTRAMBI i thread ⇒
  budget 20.648.477 B = 19,69 MiB × W REGGE, KL-85-1) · VP pin MOSSO
  (228.278.272/239.878.144/240.287.744 B vs 232±1 MiB — delibera WP-86)
  · VA2 PASS con oracle (KS-DS-85-1 sollevata: +4,0 / ≤+52,0 ora
  verdict-grade) · VW2 piecewise pinnato (383→2/766 · 384→4/1538,
  A-BB38 armato). Ledger A-DS29: 5 righe PASS a 937e79d.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (invariate)

1. **Output capture BEFORE request_end()**. 2. **Isolamento = semantica
FPM** (A-DS2). 3. **RetainSet thread-affine PER-RICHIESTA, sigillato**;
porta vm_new/park_main = token `VmGate<'gate>` LIFETIME-BOUND (v2:
rustc giudica CHI minta e QUANTO vive il token; probe cfg-gated fuori
dai build campagna — KS-MS-85-1/KH85-1 chiuse). 4. **Panic =
FAIL-FAST**. 5. **Un body ≠ oracolo sulla leva ⇒ git revert, mai
fix-forward** (KS-DS-80-3, mai innescata). 6. **Mai rm di raw:
quarantena + manifest** (KS-AH-83-2). 7. **Cifre memoria BYTES-FIRST**
(A-DL26/KL-85-2, gate-cifre morde dal MEASURE84 in poi).

## §WP-85(sessione) — prossimo lavoro

1. **Delibera peak ×W** (Concilio WP-86, input completi): VDL24 dice
   PER-THREAD; pin VP mosso a 217,7–229,2 MiB (spread tornato) — la
   delibera decide accettazione del costo ×W e l'eventuale via registry
   (SOLO con A-BB35 budget ex-ante + riapertura ESPLICITA KH81-3,
   KH84-4; frazione condivisibile walk 69,1% / footprint 66,1%).
2. **A-DL27**: scomporre gli addendi opachi (3.057.613 B) e il gap
   7,01 MiB per-thread ↔ 3,44 MB/worker fisico.
3. **A-MS27** (backlog nominato): partizione-per-TIPO vera
   (CachedMain/CachedInclude) — "per costruzione" si scrive ancora "per
   disciplina + tripwire" (KS-MS-85-3), ma il tripwire ora HA morso.
4. **Slope futura** (quando servirà): A-BG33 (base per-request via
   curl -w) + A-BG35 (regime nel verdict) — A-PP28 già nel binario.
5. **A-PP27** PRIMA di ogni nuovo segmento twin-pair (KS-PP-85-3).
6. Poi: **ripresa ROADMAP da [[php-rust-todo-master]]**. Deferred
   invariati: A-TH4; A-AH5/A-BB4 superglobali axum (KS-DS-82-3); ORM/hk
   perf solo build-adiacente (KS-AH-83-4).

**Kill-switch di rotta (ereditati WP-85, attivi)**: KH85-1/2/3,
KS-MS-85-1..4, KS-SK-85-1..3, KS-AH-85-1..3, KB-85-1..3, KS-PP-85-1..3,
KL-85-1..3, KS-DS-85-1..3, KG-85-1/2 — tabella in
`wp85-harness/COUNCIL_WP85_REVIEWS.md`. Stato S-84.0: KL-85-1 ✓
(A-DL24 eseguita), KB-85-2 ✓ (rerun A-BB34 eseguito), KS-DS-85-1 ✓
(oracle ledgerato), KS-DS-85-2 ✓ (A-DS26 verde), KH85-1 ✓ (cfg-gate),
KS-MS-85-1 ✓ (lifetime-bound), KS-SK-85-1/2 ✓ (equivalenza v2 — non
usata in campagna: battery vera a HEAD), KB-85-3 ✓ (bisezione esatta),
KS-PP-85-1 ✓ (guardia 0 righe), KS-SK-85-3 ✓ (A-SK34 in verdict84).

**NON riproporre**: tutti i NON-riproporre WP-83/84 restano; in più —
modello VW 2×len (RITIRATO: solo piecewise con soglia 384); pin peak
232±1 come identità corrente (il pin è MOSSO: i valori vivi sono quelli
di verdict84); citare "rustc è il giudice" senza la coppia
lifetime+cfg-gate.

---
**Chiusura**: 2026-08-01. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
