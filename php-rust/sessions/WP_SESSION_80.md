# WP_SESSION_80.md — S-80.0 "IDENTITY & CHANNEL REPAIR" (9 punti Concilio WP-81) + misura R≥3 + DR-1 — ✅ punti 1-8 integrali; leva = WP-81

**Data**: 2026-07-31 (pomeriggio/sera)
**Scope**: ordine vincolante S-80.0 del Concilio WP-81 (§Sintesi, 9 punti,
non rinegoziato). Eseguiti i punti 1-8; il punto 9 (implementazione leva
A-BB6 + fixture + A/B) è per contratto "SOLO POI" = prossima sessione.
Modello verificato all'apertura: Fable 5 (regola post-WP-78).
**Commit**: a1dee58 → 42e95a5 → 3f737e1 → 1305327 → 6910767 → 7ddb6bc
(tutti su main, pushati).
**Binari**: phpr **ef90cb19b0cf93ea** (nuova baseline parità, stash additivo
`phpr-wp80`; bit mossi dai fix warning A-AH15, corpus per NOME IDENTICO);
quintetto php-server a git 6910767: union **5260f50b991d8cb7** · census
**5c9c6eec481d5133** · census-axum-only 4c6264 · axum-only 29a62b00 ·
default 2d5257e3 (feature-matrix.log con `tree=clean`, archivio per-run).

## S-80.0 eseguita (punti 1-8 dell'ordine vincolante)

| # | Esito | Commit |
|---|---|---|
| 0 | Raw di misura WP-78/79 TRACCIATI nel repo (KG-81-1; vmmap gzippati) — il tree può finalmente essere pulito al matrix gate | a1dee58 |
| 1 | **Identità GIT/tree**: matrix gate rifiuta tree sporco (porcelain vuoto, A-AH14) + archivio per-run `matrix-archive/` (A-SK11) + QUINTETTO (quinta config census-axum-only pinnata, A-AH16) + lint -D warnings cross-crate php-runtime/php-cli (A-AH15); driver confronta `git=` matrix↔HEAD (KG-81-2) + source-porcelain-clean + match esatto `^bin\[row\] ` + copia matrix per-run | 42e95a5 |
| 2 | **Driver**: boot-probe fuori canale (fix off-by-one A-BG19; raw storici ricontati: census.r1 111=1+10+100 con la regola vecchia) · census modes rifiutati a W>1 (KB-81-1) · idle window anche su censuscli + protocollo IDLE_SECS=60 (A-BG20/KL-81-3) · check campi tripwire (a3_trip==0, inflight_max≤1; campo assente su build enforced = FAIL) | 42e95a5 |
| 3 | **Canale**: drain RAII su OGNI uscita di execute_with_retain (A-PP11 — F8 non inquina più) · assert a1≤a FATAL (A-TH12) · A1Window/A3Window RAII !Send compile-time + token LIFO + provider-guard + SPLIT_TRIP→`a3_trip=` (A-MS7/A-MS11/KS-MS-81-1/2) · **OUTSTANDING dec-post-send** = osservabile closed-sequential verdict-grade (A-TH9/KH81-1) · tripwire d==0⇒panic con test negativi che MORDONO (A-TH10/KH81-2) · gross=1 in-band (A-DL10) · census-global sul cli arm · SAFETY catch_unwind corretta (A-TH11) + pattern doc-purge | 3f737e1 |
| 4 | **Sigillo A-MS3** (KS-MS-81-3): `execute_request(reg, meta)` costruisce il RetainSet DENTRO; la forma `&RetainSet` è privata di modulo (i gate test la introspettano, nessun caller esterno può riusarla) | 3f737e1 |
| 5 | **Gate**: marker census PER-FILE sulla forma cfg (39 siti, 7 file; raw==form; file non pinnati = FAIL) (A-AH18/KS-SK-81-3) · concurrent: soglie derivate stampate + esca ramo FAIL + body W=1 verificati + verdict "overlap≥2" (A-SK9/KS-SK-81-1) · worker-panic: near-miss ARMATA (A-PP12) · capture-order: censimento letture post-request_end pinnato (worker_pool==2) + `#[cfg(test)]` ancorato (A-PP10/KS-PP-81-3) · CI a quintetto + lint/test census cross-crate + axum-only --no-run (A-AH15/16/17) | 1305327 |
| 6 | **Misura di riferimento R≥3** (`wp80-harness/MEASURE80_RESULTS.md`, sostituisce design79 §1 ADVISORY): 26 run ENFORCED, spread 0,0%; hello axum: a=80.476 call/13,0MB di cui **a1=74.288/10.825.612 IDENTICO su ogni fixture/run**; a3=0 steady; inflight_max=1 su 2.860 righe; **idle drift=0 su entrambi gli arm anche a 60s**; floor non-compile ex-ante ≤200 call (KB-81-2) | 7ddb6bc |
| 7 | **design79 emendato stesso commit della misura** (KS-AH-81-2): §1→MEASURE80 · A-TH14 · MAIN_CHAIN_FP computato+input enumerati · pin coppia (Module,Program) · KS-MS-80-2 su park-EVENTI (F6==3) · budget ×W · DR-1 risultato nel §5 · F8b/F8c/F10/F11/F12/F-probe/F-oneshot · KS-AH-80-4 v2 su UNA quantità · predizioni resid+b · strumento retained con regola Rc-shared · floor numerici · CPU slope due-N · M-68.5 same-commit · bound autoload-run | 7ddb6bc |
| 8 | **DR-1 CHIUSO a verdetto MACCHINA** (`wp80-harness/gate-dr1-module-immut.sh` PASS): Program graph 0 mutability (A-PP13); Module = 3 siti IC allowlisted; **IC epoch-guarded** — bump_ic_epoch() in Vm::new invalida tutto a ogni richiesta (forma reset-con-contatore KH81-3), test invalidazione verdi in CI; residui dichiarati (wrap u32 ADVISORY, confine Zval-const) | 6910767 |

## Verifiche trasversali (tree finale)

- Battery server INTEGRALE PASS a git 1305327 (sorgenti crates == 6910767):
  matrix quintetto `tree=clean` · run-gate union 5260f50b · census-twin
  5c9c6eec (marker per-FILE) · concurrent overlap≥2 (0,92s vs W1 3,4s +
  esca) · worker-panic 3 fasi + near-miss · stdout-tandem 6/6 ·
  capture-order (censimento nuovo) · doc-purge 10/10.
- workspace `cargo test --release`: **1652/0**; census tests 17/17 (3
  nuovi) + alloc_census 3/3 (nuovi, ora in CI).
- **Corpus Zend per NOME: 1418 IDENTICO** + refl 290 IDENTICO a phpr
  ef90cb19 (`wp80-harness/evidence/`).
- Campagna misura: 26/26 run ENFORCED senza un solo FAIL.

## 🔵 Scoperte

1. **La coppia (depth, inflight) chiude davvero KH78-2**: inflight_max=1 su
   tutte le 2.860 righe census — il claim closed-sequential ora è
   verdict-grade, non ADVISORY.
2. **Il canale idle è MUTO**: drift 0 call/0 B su entrambi gli arm, anche a
   60s — le diff di fase non assorbono alcun rumore cross-thread.
3. **I contatori census sono DETERMINISTICI**: R=3 byte-identico su ogni
   canale (spread 0,0%) — la coppia build-adiacente resta necessaria solo
   per footprint/CPU, non per il churn.
4. 🐛 **Off-by-one di ritorno nel sommario idle**: il boot-probe
   fuori-canale (fix A-BG19) emette ANCH'ESSO una riga census-global → 4
   righe probe; l'awk head-anchored leggeva boot→p1 (=warm-up, 860k call)
   come "self-cost". Raw corretti, sommario ricomputato, fix tail-3 nello
   stesso commit dei risultati.

## ⭐ Lezioni

1. ⭐⭐ **Un fix che sposta una sorgente di righe cambia la CARDINALITÀ di
   tutti i consumer a valle**: il boot-probe spostato fuori dal canale
   census è entrato nel canale census-global — ogni parser posizionale
   (NR==1..3) va ri-ancorato quando cambia chi scrive nel log.
2. ⭐⭐ **La coda non è l'in-flight**: un watermark con dec-al-pickup misura
   la CODA; l'osservabile del meccanismo closed-sequential è il contatore
   OUTSTANDING con dec-DOPO-il-send della risposta (una richiesta è "nel
   server" finché il client non ha la risposta).
3. ⭐⭐ **Un tripwire vale solo col test negativo che lo vede mordere**: il
   test anti-wrap passava CON la regressione (aritmetica modulare
   auto-annullante, dimostrazione Hoare); i due `#[should_panic]` sui
   d==0 sono il falsificatore vero.
4. ⭐⭐ **Il lint -D warnings per-crate scopre il debito del perimetro**: 6
   warning php-runtime vivevano da sempre sotto il -D scoped a php-server
   (le dipendenze restano warning) — estendere il perimetro = pagare il
   debito PRIMA che la misura ci giri sopra.
5. ⭐ **Il "tree pulito" si compra tracciando i raw**: finché i raw di
   misura vivevano untracked, il clean-tree gate era inapplicabile; KG-81-1
   e KS-AH-81-1 si soddisfano INSIEME o nessuno dei due.

## Residui / NON fatto (dichiarato)

- **Punto 9 (leva A-BB6)**: NON iniziato per contratto ("SOLO POI") —
  design79 emendato è il contratto d'implementazione; DR-1 e misura di
  riferimento sono chiusi, i sigilli pre-leva (A-MS3, KS-AH-81-2,
  KS-PP-81-2) sono nel type system o nel design.
- Walk del Program + `stranded_bytes_dropped` + fixture autoload-run
  (KB-81-3) + budget numerico ×W: dichiarati in design79 §7/§11, da
  implementare nella sessione leva.
- ORM/hk gate non rilanciati (invariati: nessun cambio engine non
  feature-gated oltre ai fix warning — il corpus per NOME copre; stessa
  motivazione di WP-79).
- Deferiti invariati: A-TH4, A-AH5/A-BB4 (superglobali axum), registry
  condivisa read-only.
