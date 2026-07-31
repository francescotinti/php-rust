# WP_SESSION_82.md — S-82.0 "HONEST INSTRUMENTS + MISURE RESIDUE" — ✅ 8 passi Concilio WP-83 + verdict82 PASS

**Data**: 2026-07-31 sera → 2026-08-01 notte
**Scope**: ordine vincolante Concilio WP-83 §Sintesi (8 passi) + misure
residue A/B della leva A-BB6. Modello verificato all'apertura: Fable 5.
**Commit**: 2a883e8 → 12d6b9e → 84b3fb3 → 7fed57d → 693b6b3 → e932925 →
f57b4d0 → f61e877 → 83a1d67 → 1f691df → f22ff0b → 00858ac → 121fde9 →
8feddc1 → e2990b3 → **5b668d0** (tutti su main, pushati).
**Binari**: phpr **c84bb425f007a52d** (nuova baseline parità, stash
additivo `phpr-wp82`; corpus 1418 + refl 290 per NOME IDENTICI, workspace
0 fail — verificato DUE volte); sestetto a matrix e2990b3 (union
318853ca / census+mem per riga matrix; mem-census **b620d64c89abb584**).

## Ordine WP-83 eseguito (p1-p8)

| # | Esito | Commit |
|---|---|---|
| 1 | verdict81 v2 FAIL-CLOSED (campo vuoto=FAIL, steady_n pin, medie R=3 dietro md5-gate, spread per-campo, derivate D1-D6 scriptate, REQ1 pin A-BB30, selftest 2 denti) + gate-measure-cifre (KG-83-3, selftest morde) + MEASURE81 retro-annotato (header v1 FALSO dichiarato, floor condizionale A-BB27, a2 nominata, A-AH27) | 2a883e8 |
| 2 | **Fold one-shot su acquire (A-TH21)**: run_source_with + run_source_with_argv delegano a run_source_probed(probe=false), argv passante; evento acquire_oneshot (A-SK23) + F-oneshot t2b; sweep di CLASSE any-spelling + decoy SplitDrain (A-SK24); lower_source pinnato PER NOME; PARITÀ integrale | 12d6b9e |
| 3 | Denti macchina: DR-1 header riscritto + dente u64 + mutation-test KH83-1 (morde); publish==2 ESATTO + check_order skip commenti (A-TH20); A-MS18 guardia include-hit + contatore; A-PP19 uclog in-band; A-PP20 dente in-cargo (probe diretto entries); A-TH22 drain-sync; A-DL17 etichette; **buco latente sanato: "supersede" fuori vocabolario uc_log** | 12d6b9e+84b3fb3 |
| 4 | Quarantena: quarantine-raws.sh (manifest sha256) + retro-annotazione 13+13; policy ESERCITATA 3 volte in sessione (162+23 raw quarantinati, mai rm) | 12d6b9e |
| 5 | **Retained ONESTO (A-DL15/16)**: bracket net-at-lower → CachedUnit.main_program_net → rw_main_net in-band; 🔴 **scoperta: lo strumento net era MORTO nel build mem-census** (allocatore nudo, contatori (0,0)) → MemCountingMi + note nel CountingAlloc union; selftest algebra ESATTA (both==s1+own2; il primo controllo ingenuo ha scoperto la condivisione REALE del prelude ~1MB) + bracket≥floor + banda 10%; A-AH29 matrix/CI simmetria (il --no-run ha MORSO: lib-test mem-census non LINKAVA → mimalloc dev-dep); A-AH30 driver_sha+campaign | 7fed57d |
| 6 | Fixture: F2 same-key A MACCHINA (stat ns + put==2/hit==1); **F14 VERDE** (classe condizionale+eval-mint+deferred == oracolo ×4 + positivo); **F15: thrash FIFO REALE, main_evicted==1 CONTATO**; A-DS19 trigger (main_publish_decision estratta); A-DS17 double-compile cross-thread + grep-gate hash-iter ==0 | 693b6b3 |
| 7 | **Misure residue** (campagna-4 a e2990b3, 26 run + base 7593d8e + supplemento R): vedi MEASURE82_RESULTS.md — VF no-leak PASS (coppia scalata 0,000MB), VP peak W=10 **DECLARED-WORSE +34,4MB** (voce Concilio WP-84), VC **NULL** (KB-83-3, N→2000/4000), VH twin-pair **PASS** (path misto CHIUSO), VR retained **20,65MB ×W** (rw_bytes FLOOR + rw_main_net misurato), VA autoload **PASS forma forte** (steady a3==0 ESATTO, quota RUN=+56 call/req in b), scan supersede 2ns/key | 5b668d0 |
| 8 | Revert policy: MAI innescata (zero body ≠ oracolo in tutta la sessione) | — |

Chiusura KB-83-2 ad ARITMETICA: spike resid req=11 = 3×self-cost dei probe
__census_global (114/124.572 == 3×38/41.524 ESATTO su 4 fixture, pin in
verdict81.sh §SPIKE). Bound scan Bak: 2ns/key (500× sotto soglia).

## 🔵 Scoperte

1. 🔴 **Strumento net-alloc MORTO nel build mem-census**: la feature compila
   i contatori, ma l'allocatore era mimalloc NUDO — alloc_counters()
   congelato a (0,0); ogni CensusNetWindow in quel binario era silenziosamente
   vacua. Due condizioni di attivazione distinte: allocatore contante +
   env PHPR_MEM_CENSUS (e il dump unitcache vive nel teardown di
   run_module_with_hir ⇒ arm CLI-SERVER, non axum).
2. **Il thrash FIFO di Stogov è reale** (F15: 5 fp su 4 ways evince il main,
   main_evicted==1) — ma sul working-set misurato main_evicted==0.
3. **"HIT salta a3" è ESATTO anche per l'autoload runtime** (l'include
   dell'autoload è esso stesso un HIT; quota RUN = +56 call/req in b).
4. Il peak W=10 paga il retained ×W della leva: +34,4MB ≈ 3,44MB/worker —
   la contropartita di churn −13MB/req. Accettazione = Concilio WP-84.
5. "supersede" era emesso FUORI dal vocabolario chiuso uc_log dal S-79.0.7
   (debug_assert mai attivo in release).

## ⭐ Lezioni

1. ⭐⭐ **Un contatore compilato non è un contatore armato**: feature ≠ env
   ≠ allocatore — lo strumento si prova VIVO (controllo positivo) prima di
   ogni cifra, in OGNI build che la cita.
2. ⭐⭐ **Il porcelain whole-tree del matrix governa la sequenza**: output
   di battery/gate FUORI dal repo; archivi matrix = file NUOVI (add
   esplicito); matrix ULTIMO prima della campagna, senza commit in mezzo.
3. ⭐⭐ **La quarantena ha pagato subito**: 3 campagne abortite → 3 manifest
   sha256, zero rm, contatore void_runs nell'header — la campagna valida è
   falsificabile CONTRO le morte.
4. ⭐⭐ **worktree ≠ workspace**: la root git può non essere la root cargo;
   e il lock vivo con dev-dep nuovi rompe --locked su manifest vecchi
   (--offline + lock copiato = stesse versioni di build, deviazione
   dichiarata).
5. ⭐ Il controllo positivo "ingenuo" che fallisce può essere una SCOPERTA:
   l'algebra S1+S2−Sshared rotta ha rivelato la condivisione del prelude
   tra Program lowered (~1MB), portando al controllo con composizione nota.

## Residui / NON fatto (dichiarati in MEASURE82 §Aperture)

- **VP peak ×W**: DECLARED-WORSE aperto — Concilio WP-84 decide
  (accettare il costo ×W o mitigare: registry condivisa read-only in
  backlog). - **VC slope CPU**: NULL per KB-83-3 — N=2000/4000 in WP-83
  (medie grezze a verbale: lever 150 vs base 7043 µs/req). - ORM/hk perf
  build-adiacente: non misurati, nessun claim (KS-AH-83-4 non innescata).
- A-MS17 (token ZST porta vm_new) + newtype PreludeBinding (A-AH26):
  "al prossimo tocco del file", non toccato.
