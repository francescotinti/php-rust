# WP_SESSION_78_1.md — S-78.1 HARDENING PRE-CENSUS (8/8) + FASE MISURA — 🔴 LEAK RetainSet TROVATO E CHIUSO

**Data**: 2026-07-31 (pomeriggio)
**Scope**: ordine vincolante S-78.1 del Concilio WP-79 (§Sintesi, 8 punti) +
fase misura design78. Modello verificato all'apertura: Fable 5 (regola
post-WP-78).
**Commit**: 146a4c1 → e5d71ac → 13e9f09 → 275c518 → 9bc4e99 → 7b8c3f6 →
a3fefee → 7dd03d1 → 352b63c (tutti su main, pushati).
**Binari**: phpr **ef980f8a** (nuova baseline parità: corpus per NOME PASS,
1418 fail set IDENTICO a GATE72; stash additivo `phpr-wp78`); php-server
union (gemello misura) **5fdc971680d5e6a2**; census build 8f4146db
(solo contatori, KB-78-5).

## S-78.1 eseguita (8/8 punti dell'ordine vincolante)

| # | Esito | Commit |
|---|---|---|
| 1 | Doc purge: 12 mine (il gate KS-MS-2 ne ha trovata UNA in più del verbale, vm/mod.rs:728 "Persistent Vm" maiuscola — grep -i del gate > ricerca case-sensitive); `gate-doc-purge.sh` con esca positiva 7/7, in CI | 146a4c1 |
| 2 | JoinHandle conservati + `shutdown()` = drop senders + join con riga verdetto (KL-78-4); politica panic **FAIL-FAST** (catch_unwind→abort; prima: thread morto in silenzio = 1/N richieste blackholate); trigger gate armato SOLO da env `PHPR_TEST_WORKER_PANIC`; `gate-worker-panic.sh` PASS (SIGABRT 134 + controlli positivo/negativo + join line) | e5d71ac |
| 3 | Grep-gate ricorsivo + pattern `request_end\(` + same-line fix + marker per forma+censimento (1/1); run-gate: BIN dall'artifact json di cargo, PID==SRV, expected rigenerati dall'oracolo con comando, `._*` purgati, git rev nei verdetti; feature-matrix: nm esteso a axum\|tokio\|hyper\|tower, warnings fatali sui TEST target; assert ASSOLUTO in gate_apertura2 | 13e9f09 |
| 4 | Contratto fatal A-TH8: worker = epilogo ESATTO del main (`link_fatal_check` estratto condiviso, `render_fatal`/`handle_uncaught_exception` pub SAPI, exit→200 pulito, exception-handler→200 senza banner); meta.path = path FS (SCRIPT_FILENAME) ⇒ cmp full-body vs oracolo POSSIBILE e PASS su fatal_runtime/fatal_compile/exit; parse = divergenza REGISTRATA (PHPR_DIVERGENCES 3.9, mai claim di parity); `gate-stdout-tandem.sh` (A-PP8: 4 siti, tutti tandem) | 275c518 |
| 5 | 🔴 **LEAK REALE trovato dal gate A-DS8 al primo colpo** (v. sotto) e chiuso: RetainSet PER-RICHIESTA nel worker | 9bc4e99 |
| 6 | Feature `census-instrumentation`: allocatore contatore attorno a mimalloc, fasi a/b/c, retain_len/live_objs/depth con watermark, controlli positivi in cargo (11/11); `--tier0` eseguibile (KG-79.D); igiene A-BB9 (path borrowed, file_s nei branch) | 7b8c3f6 |
| 7 | design78 emendato: meccanismo+osservabile carico chiuso (A-BG9), warm-up ridefinito (picco=intero processo, KG-79.C), vmmap V1/V2 (A-DL8), A/B decomposto + cli-server senza exit-stats dichiarato (KG-79.B), Tier-0 tracciato | 7b8c3f6 |
| 8 | `gate-concurrent.sh` (KH78-1 da contratto A-SK6): W=4, 24 richieste concorrenti, OGNI body == oracolo, sweep panic, join line. **PASS bin=5fdc9716** ⇒ KS-SK-79.3 sbloccato | a3fefee |

## 🔴 IL FINDING: leak RetainSet nel worker (KS-DS-78-4 ha MORSO)

Il primo gate mai scritto sulla superficie di parking (Stogov c.5c: "esercitata
da NESSUN gate") ha smascherato un leak reale: `park_module` parcheggia UN
clone di ogni unit inclusa PER RICHIESTA (pin request-lifetime by design
WP-67 P-2, anche su unit-cache HIT), e il RetainSet è `FrozenVec` append-only
e **write-only** (nessun lookup lo legge mai). Il worker che ne teneva UNO per
la vita del thread trasformava ogni pin in leak permanente: len 1→4 su 4
richieste con un solo require; con WordPress (centinaia di include/req) =
crescita unbounded. **Il vero analogo opcache è la unit cache THREAD-LOCAL**
(WP-63/66, Rc-owned, fingerprint, ways-eviction), che già persiste sul worker.
**Fix**: RetainSet per-richiesta nel worker_loop (la macchineria del main,
WP-68) — leak-freedom per DISTRUZIONE, zero cache nuove (A-BB6 resta
deferito). Doc purgata dalla premessa falsa ("persistent RetainSet" ora
pattern vietato nel gate, censimento a 7); test
`include_units_pinned_per_request_not_leaked` (len()==1 esatto per richiesta).

## Fase misura (design78; dettaglio cifre in `wp78-harness/MEASURE78_RESULTS.md`)

- **Tier-0** (`--tier0`): peak 10,68MB (spread 0,92%), V2 ~4,3M.
- **A/B stesso binario N=100**: axum W1 peak 59,7MB / V2 18,0M vs cli-server
  53,99MB / 13,2-13,3M ⇒ **delta +5,7MB peak, +4,8M residente** (somma
  dichiarata tokio+canale+pool+registry-per-worker+retention, NON decomposta).
  A-BB1 alloc-A/B: NON GIUDICABILE (braccio cli senza contatori né exit-stats,
  KG-79.B) — dichiarato.
- **Prova regina**: V2(N=200) ≡ V2(N=100) = 18,0M ⇒ delta per-richiesta = 0
  (il +0,5M V2−V1 è assestamento che plateaua). Nessun leak.
- **Census per-fase** (--workers 1, R3, righe INTERO-ESATTE): hello a=80.476
  call/13,0MB · b=730/95,6KB · c=0/0; include a=80.513/13,04MB, retain_len=2.
  ⇒ **fase a (lower+compile) = 99% dell'alloc/req**: È il dato frequenza×taglia
  che sblocca il design A-BB6 (cache Module) POST-censimento come da ordine.
  used_n=0 ovunque (KS-DS-78-2 ✓), depth_max=1 in tutti i campioni (KH78-2 ✓).
- **Linearità W** (100 req/worker, R3): V2 = 18,0 / 55,3 / 134,0M per W=1/4/10
  ⇒ base+W·k con k=12,4-13,1M/worker (dominante: copia Registry per worker,
  Leijen c.4). KL-78-2 non scatta. ⚠️ spread del PICCO a W=10 = 12% (186-212MB,
  transitori concorrenti di primo-compile): non mediato, registrato.
- **R-G4**: non commissionato, non eseguito.

## Verifiche trasversali

- workspace `cargo test --release`: **1651/0/1** ×3 (baseline esatta).
- Corpus Zend per NOME ×2 (dopo refactor epilogo: phpr 400f14fb; finale:
  **ef980f8a**): 1418 fail, **set IDENTICO** a GATE72. Cargo.lock invariato
  (A-AH9/KS-AH-78-4 ✓: delta dalla baseline = sola static_assertions).
- Gate battery finale (git 275c518+, bin 5fdc9716): doc-purge · capture-order ·
  stdout-tandem · run-gate (esteso: include/static/fatal) · worker-panic ·
  concurrent KH78-1 · feature-matrix — **tutti PASS**.

## ⭐ Lezioni

1. ⭐⭐ **Un gate nuovo su una superficie mai esercitata morde SUBITO**: due
   finding reali in un giorno (mina doc case-insensitive; leak RetainSet).
   Il valore del gate è proporzionale a quanto la superficie era vergine.
2. ⭐⭐ **Un'arena append-only che nessuno legge è un leak in attesa**: se una
   struttura è write-only, la sua unica funzione è il lifetime — allinearlo
   alla richiesta, non al thread. Il "persistente" va giustificato da un
   LOOKUP, non da un'analogia (l'opcache vero era altrove: unit cache TL).
3. ⭐⭐ **L'identità del binario nel driver di misura non è decorativa**: il
   primo Tier-0 è girato su un binario default lasciato dal build workspace —
   l'hash stampato dal driver l'ha smascherato in 1 riga (KS-AH-78-1).
4. ⭐ **Il self-traffic del census si esclude PER COSTRUZIONE** mettendo il
   logging fuori dalle finestre (s3 prima dell'eprintln → gli alloc del log
   cadono nel gap non conteggiato tra s3 e s0 successivo) — lezione WP-64
   applicata in avanti.
5. ⭐ **`ends_with` sul path del trigger**: quando meta.path è diventato
   filesystem-path, il trigger `== "/__phpr_panic"` sarebbe morto in silenzio
   — i magic-path vanno ancorati alla FINE, mai all'uguaglianza.

## Residui / NON fatto (dichiarato)

- A-BB1 alloc-A/B: braccio cli-server senza contatori (e senza exit-stats
  puliti) — per giudicarlo servono contatori sul path cli o drain pulito.
- Decomposizione del delta +5,7MB (tokio vs pool vs registry) non eseguita.
- Spread 12% del picco W=10: da indagare solo se diventa metrica di verdetto.
- Deferiti WP-79+ invariati: A-TH4 (request_end(self)→CapturedOutput),
  **A-BB6 cache Module (ora CON i dati: 80k call/13MB per richiesta = 99%
  dell'alloc — candidato naturale della prossima leva)**, A-AH5/A-BB4.
