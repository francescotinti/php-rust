# WP_SESSION_64 — DEBITI DEL CONCILIO CONSEGNATI (fatal+assert+contatori, gate pieno verde) + K-64=A + QUOTA TRANCHE 2 INCHIODATA (~90-100MB counted, CPU MORTO 0,36%) — il −74,7% è ora SPIEGATO

> ⚡ **WP-64 (2026-07-27 mattina, `6c4cf7e`→`0a94329`)** — primo atto =
> debiti di apertura del concilio (sintesi a 9 recepita INTEGRALE in
> `wp64-harness/design64.md` PRIMA di ogni codice); poi decisione utente
> **K-64 = OPZIONE A (tranche 2 stub-elision; axum → WP-65)** e
> l'istruttoria completa della quota (obblighi 1-2 dell'opzione A:
> colonne per-canale + quota predetta-vs-misurata). La LEVA slot_names/
> fn-Vec resta per WP-65 (spec-sketch coi rischi WP-44 in design64 §4-B).

## Debiti di apertura (commit `1b8976a` + RULEBOOK `6c4cf7e`)

- **H1''=K6''**: RULEBOOK §4 emendato alla forma SPEDITA (id
  unit-locali all'emissione, baking AL LINK, id assoluti nel Module
  ritenuto; S2 detto giusto per S-2; sign-off utente nel prompt).
- **M1''** assert anti-collisione pun (`prelude`/`seed-stub`) in
  run_include; **H2''=K-M5** `elide_align_miss` → **FATAL in ogni
  build** (commento fuorviante corretto; KH64-3 armato); **M3''**
  debug_assert basi remap in run_linked (adiacenza ora VERIFICATA, 3
  call-site); **S-1** contatore positivo `elide_align_hit` + dump +
  evento log; **M2''** digest minting runtime nel `unit_chain_fp`
  (unico push site: run_linked); **P-1** `reserved_base` esplicito in
  CachedUnit + check sul hit + doc corretto (moved base = miss fp);
  **H4''** debug_assert mirror compile↔link a entrambi i call-site.
- **Gate pieno gate64-debts.sh (flip)**: cargo 1645/0 · sentinelle 5
  assi BYTE-ID · sentinelle63 VERDI · **KS-S6 alignhit>0** · corpus
  1421 IDENTICO · refl 290 IDENTICO · ORM 3E/13F IDENTICO · hk 0E/0F ·
  reverse-order 2F GEMELLO IDENTICO. P1-a..d VERDI **a lato del gate**
  (esecuzione manuale probe63-p1.sh, hit_cross=2, controllo cache-off
  esercitato dopo il fix detector; K-65.2 Klabnik: la matrice K1''
  integrale — P1 e KE-e DENTRO gate65.sh — resta dovuta alla leva).
- **S-4**: divergenza get_declared_classes catalogata
  (PHPR_DIVERGENCES §3.8(vi), pre-esistente, probe wp64-harness/).

## ⚖️ K-64 (utente, in sessione): OPZIONE A — tranche 2

Axum slitta a WP-65+ (primo atto resta KS-P1 invariato).

## Quota tranche 2 (E1-64: memgc64 `6d6b518b`, census64-full)

- **P64-1 (KS64-1, formulazione emendata dal concilio E-65.1)**:
  version.php net/inc 22.623 = slotnames 12.554 + fnvec 7.608 +
  fnshare 2.536 = **100,3%** e comments 109,4% DENTRO ±10% — la
  colonna "proper inquinata" (Hejlsberg) è RISOLTA sui seed-heavy;
  **modules 35,2% e loader 18,1% = bordo per-bersaglio ESERCITATO**
  (residuo attribuito al dup del proper — canale E6 — senza colonna
  che lo misuri: ATTRIBUZIONE, non riconciliazione; mai "✓" secco).
- **P64-2 (SOPRA banda pre-registrata [35,70]MB: +25% — formula di
  scaling falsificata in ALTO; la quota si tiene per l'attribuzione
  P64-1, non per la predizione)**: counted standing = **slotnames
  51,6MB + fnvec 36,0MB = 87,6MB (17,5% del net_tot 499,9 — replica
  WP-63 alla cifra) + fnshare 12,0MB adiacente**. Quota leva ONESTA
  (E-65.3 Hejlsberg): **64-76MB** — la componente fn_ci (~24MB dei
  36,0 di fnvec) è un hash-index misto seed+proper NON banalmente
  condivisibile; sale a ~100MB SOLO con una forma fn_ci a costo zero.
- **P64-3 FALSIFICATA — CANALE CPU MORTO**: map 1,66s + remap 1,15s
  all-proc = **0,36% del full** ≪ tetto 6,1%; confermato da B4 sample
  su PARITÀ (unit_remap_elided ≈0,08% dei campioni). KB11: la
  motivazione CPU della tranche 2 CADE; resta la sola footprint. Il
  lower 32,9s (4,2%) resta "causa non attribuita" (B1).
- **P64-4 (KS64-3 — vizio K4'' a verbale, sanato)**: Δcoda =
  1.021,7−382,5 = **639,2MB vs ~623 = +2,6%** (±15%); ⚠️ il letto
  pre-registrato era il join per-path e la sessione l'aveva sostituito
  con l'aggregato (E-65.2/KS65-2: un letto non si sostituisce
  post-hoc). Il concilio ha eseguito il join e la sessione l'ha
  DEPOSITATO (`wp64-harness/coda-join-top40.txt`: 31 path comuni, 0
  crescite, n62==n64 ovunque, Δ visibile 941,3MB) ⇒ **il −74,7% si
  dice SPIEGATO** (totale+bersagli+coda, col letto giusto).
- **P64-5 ✓**: sc 135-169/unit ⇒ statics NON O(seed) (domanda chiusa).
- **L-64a con falsificazione (terminologia emendata L-65.1)**: al
  segmento-peak del FULL committed/used=**1,007** (slack 13,2MB; bin
  4-16K DENSI; copertura strumento 87,9% del phys) — il c/u≈2 vive
  sul drain lt (~12MB assoluti), non al peak ⇒ quota **page-slack** al
  peak ≈0 (L-64c ridimensionata). Il **bin-rounding** requested<block
  è INVISIBILE a c/u e resta VIVO (~×1,15, a FAVORE della leva).

## Catena orchestrate64 (stessa-mattina)

Media: CPU **2,57×** (20,92/53,85u — replica) · footprint **3,01×**
(393,7/1.186,9MB, DENTRO banda 2,9-3,2×, 2ª coppia verso ≥3+3).
Census64-full: letto valido (0E/2F/86W/73S), nessun panic.
**Coppia full run51** (new=debiti vs old=phpr-wp63): fail-set **88
BYTE-ID ×2 = run33** · CPU 788,19 vs 800,07u (−1,5%, spread — non si
cita) · peak 2,035 vs 2,049GB (−0,7%, rumore). Debiti = delta zero.

## Stash e harness

Release = **phpr-wp64 (`522e0f6174039cff…`)** additivo accanto a wp63;
census **phpr-memgc64 (`6d6b518b…`)**. Harness: `wp64-harness/
{design64.md, COUNCIL_WP64_REVIEWS.md, gate64-debts.sh (+KS-S6),
probe64-s4.php(+inc), build-memgc64.sh, census64-listtests.sh,
orchestrate64.sh, watch-sample64.sh}` + out-dir (gate-out-debts/,
census-out/, eve-out/, sample-out/). probe63-p1.sh EMENDATO (detector
hit-eventi).

## ⭐ Lezioni

- ⭐⭐ **Mai cronometrare una finestra che contiene il proprio
  logging**: il remap_ns 4,57s del lt era l'uc_log per-evento (open
  file × 14.104 alignhit) DENTRO il walk cronometrato — senza log:
  42ns/entry, canale morto. Observer-effect auto-inflitto smascherato
  dal letto full (log spento).
- ⭐⭐ **I nomi degli eventi di log sono API dei probe**: `alignhit`
  conteneva "hit" come sottostringa e il detector cache-off di
  probe63-p1 (`grep -q "hit"`) degradava SILENZIOSAMENTE il proprio
  controllo byte-id. Fix al detector (`hit (intra|cross)`); rename
  dell'evento RITIRATO (avrebbe biforcato lo stash dal binario
  appaiato per una stringa di log).
- ⭐⭐ **Le colonne dirette sul modulo ritenuto battono le finestre
  net**: slotnames+fnvec+fnshare riconcilia il net/inc di version al
  100,3% — la "proper inquinata" era decomponibile con 30 righe di
  census, non con nuove ipotesi.
- ⭐ (EMENDATA dal concilio, E-65.2/KS65-2): l'aggregato Σ_coda NON
  sostituisce un letto per-path PRE-REGISTRATO — la versione "niente
  join necessario" era la stessa retorica di KS63-1; il join top-40
  era dovuto, il concilio l'ha eseguito e ora è depositato.
- ⭐ daemonize.pl vuole `<log> <cmd>` e la out-dir DEVE esistere prima.

## Prossimo (WP-65) — vedi NEXT_SESSION §WP-65
