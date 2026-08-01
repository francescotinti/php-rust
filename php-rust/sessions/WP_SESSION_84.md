# WP_SESSION_84.md — S-84.0 "SIGILLI, ORACOLI E LA MISURA CHE DECIDE" — ✅ 8 punti Concilio WP-85 + VERDICT84 PASS + ×W deciso PER-THREAD

**Data**: 2026-08-01 giorno
**Scope**: ordine vincolante Concilio WP-85 §Sintesi (8 punti) eseguito.
Modello verificato all'apertura: Fable 5.
**Commit**: 746bc24 → 20961dc → eddca96 → 937e79d → 3e85318 → **9c2f946**
(tutti su main, pushati).
**Binari**: phpr **c4448075401dee5f** (nuova baseline parità, stash
additivo `phpr-wp84`; bit mossi da A-MS25 lifetime-bound + A-MS26
cfg-gate + A-MS28 put-path + probe A-DL24; corpus 1418 + refl 290 per
NOME IDENTICI in battery); campagna a matrix 937e79d; union
d440c3411c12401a · census d70b86d0502ea7e7 · mem-census 85fd009f66e7d3e4.

## Ordine WP-85 eseguito (p1-p8)

| # | Esito | Commit |
|---|---|---|
| 1 | Retro-correzioni: già eseguite in chiusura S-83.0 (A-BG34/A-BB37/VA VOID) | — |
| 2 | **Sigillo VmGate v2**: `VmGate<'gate>(PhantomData)` lifetime-bound (A-MS25/A-TH27) — mint 1 = `RetainSet::production_gate` PRIVATO (borrow della RetainSet di richiesta), mint 2 = `vm_gate(&self) -> VmGate<'_>` (muore con l'acquire), mint 3 `vm_gate_probe` dietro `cfg(any(test, feature="vm-gate-probe"))` (A-MS26/A-TH28: feature accesa SOLO dal dev-dep di php-server per i target test — nei build campagna/parità il simbolo NON ESISTE, KH85-1 chiuso da rustc). Belt 1b in forma v2 + anti-alias sweep + anti-pin v1; **sezione 1c** = pin produttore `main_program: Some` ==1 body-scoped in `main_publish_ticket` + sweep workspace (A-TH31/KH85-3). 3 denti morsi su copie mutate | 746bc24 |
| 3 | **battery-equivalence v2**: A-SK32 (OUT legale SOLO con `git=$BREV` nel summary + `rev=$BREV` nel .done; `^FAIL <name>` rifiuta SEMPRE), A-SK33 (ledger canonico pinnato nello script, tracked, refuse se divergente da HEAD), A-AH36 (manifest per-gate trasclusi: census-twin dichiara run-gate.sh+fixtures/, doc-purge fixtures/, run-gates la loro .php). 4 morsi dimostrati, incluso (iv) su finestra storica reale via dir-prefix | 20961dc |
| 4 | **Denti**: A-DS26 test a INIEZIONE (main-tagged nella corsia include via accesso diretto, evizione forzata, contatore+evento; **F16** in fixtures2 lo esegue ARMATO) · A-DS28 sig() esteso (parent/interfaces/prop_defaults/consts/static_props/prop_init/static_vars) + mutante 2 (prop default) MORDE · A-MS28 unit_cache_put RACCOGLI-POI-EMETTI (uc_stat/uc_log e TUTTI i drop fuori dal borrow_mut — KS-MS-85-4 strutturale) · A-PP26 `uc_entry_count()` totale + delta==0 in a_pp20 + gemello positivo · A-PP28 `w=` in-band su reqns · A-PP29 · A-TH29 CLASSRE split 5/9 · A-SK35 occorrenze(C1)==1 · A-DS27 header pair-wise · base-arm A-AH35 (provenance-strip=VOID, morso su lock sintetici) + A-TH30 (resolver==2 pinnato, pruned-set nominato 6° arg, diff integrale su file) | eddca96 |
| 5a | **Strumento A-DL24**: `memcensus_unitcache_main_rows(thr)` — righe per-THREAD al teardown worker (thr=/arm=/alloc_id in-band, raccogli-poi-emetti); fixture bisezione VW a path canonici 383/384 ESATTI; measure84-campaign/verdict84/battery-84pre; ledger A-DS29; gate-cifre + dente A-DL26 bytes-first (morso su doc doctored) | 937e79d |
| 5b | **CAMPAGNA + VERDICT84 PASS** (battery 15/15 per NOME a HEAD, F16 verde al primo giro in battery): vedi `wp84-harness/MEASURE84_RESULTS.md` | 3e85318+9c2f946 |
| 6 | A-PP18: NON ingaggiata, dichiarata (nessuna riconciliazione Δglobal a W>1; A-PP24 onorata) | — |
| 7 | Delibera peak: input COMPLETI (VDL24 + rerun VP) — decisione al Concilio WP-86 | — |
| 8 | ROADMAP: non ripresa (campagna ha esaurito la sessione, dichiarato) | — |

## Misure (verdict84.out, fail-closed)

- **VDL24 — PER-THREAD (KL-85-1)**: thr0 E thr1 net(ord1) = 7.349.977 B
  = 7,01 MiB IDENTICI (e identici all'ord1 di campagna-83) — il secondo
  worker RIPAGA il primo lower: il residuo one-time è di THREAD
  (prelude/interner thread-local). **La formula ×W REGGE: rw_budget × W**
  (20.648.477 B = 19,69 MiB per worker).
- **VP — pin A-BB34 MOSSO (nominato)**: 228.278.272/239.878.144/
  240.287.744 B = 217,7/228,8/229,2 MiB vs pin 232±1 (S-82.0 era
  232/232/232 spread 0%); r1 sotto di 12.009.472 B. Delibera WP-86 sui
  valori nuovi.
- **VA2 — VOID WP-83 SANATO (KS-DS-85-1)**: oracle full-body ledgerato
  PRIMA della finestra (5 righe PASS a 937e79d); a3==0; +4,0 register /
  ≤+52,0 include-HIT ora verdict-grade.
- **VW2 — piecewise CONFERMATO, sito NOMINATO**: std `run_path_with_cstr`
  MAX_STACK_ALLOCATION=384 (heap CString(len+1)), 2 syscall path in
  `main_unit_key`; bisezione ESATTA 383→2,0/766 · 384→4,0/1538 · hello
  98→2,0/196; modello 2×len RITIRATO (A-BB38 pin armato).

## 🔵 Scoperte

1. **Le closures NON vivono in `m.functions`**: compilano nella tabella
   separata `m.closures` (index space di `Program::closures`,
   `Op::MakeClosure`) — l'ipotesi di Stogov (verificarle in m.functions)
   era FALSA nella sede, vera nella sostanza: erano un punto cieco del
   comparatore; ora coperte lì, mutante 3 morde.
2. **La soglia VW è la costante di std, non nostra**: 383→2 call/766 B,
   384→4 call/1538 B — la firma esatta dello stack-buffer 384 di
   `run_path_with_cstr` con doppio fallback heap (canonicalize+metadata).
3. **Il residuo 7,35MB è PER-THREAD**: la domanda di Leijen ("di chi è il
   residuo?") ha risposta di macchina — ogni worker lo ripaga; il gap col
   marginale fisico 3,44 MB/worker resta la domanda A-DL27 (allocatore).
4. **Il pin identità peak non è sopravvissuto alla partizione**: 232±1
   spread-0 → 217,7–229,2 con spread tornato; un pin numerico
   cross-sessione invecchia anche quando è un PEAK (classe della lezione
   WP-72 sui pin numerici).

## ⭐ Lezioni

1. ⭐⭐ **Il lifetime è la metà mancante della capability**: la ZST
   provava CHI può mintare, il borrow prova PER QUANTO — solo insieme il
   token smette di essere bancabile (KS-MS-85-1 chiusa a compile-time).
2. ⭐⭐ **Un gate nuovo va fatto mordere sul proprio stesso output**: il
   dente A-DL26 ha respinto 4 righe del MIO MEASURE84 prima del PASS —
   un gate che non ha mai morso il suo autore non è ancora un gate.
3. ⭐⭐ **La verifica per NOME batte l'ipotesi di sede**: le closures
   erano dichiarate "in m.functions" dal verbale; il test per NOME ha
   trovato la sede vera (m.closures) E il punto cieco del comparatore.
4. ⭐ Il cfg-gate è il sigillo più forte del pin: il probe assente dal
   binario chiude l'alias-hole meglio di qualunque regex (KH85-1).
5. ⭐ Round-robin come strumento di misura: a W=2 sequenziale, la coppia
   (probe-request, request-2) atterra ESATTAMENTE su (w0, w1) — il
   protocollo per-thread non ha avuto bisogno di affinità esplicita.

## Residui / NON fatto (dichiarati, per NOME)

- **Delibera peak ×W**: input completi, pin VP mosso — Concilio WP-86.
- **A-DL27**: addendi rw_bytes−bytes_counted opachi; gap 7,01 MiB
  (net/thread) ↔ 3,44 MB/worker (fisico) da scomporre.
- **A-PP18**: aperta, non ingaggiata (A-PP24).
- **A-PP27**: fixture head-segment PRIMA del prossimo twin-pair (nessun
  segmento nuovo in S-84.0).
- **A-MS27**: partizione-per-TIPO vera (CachedMain/CachedInclude) resta
  backlog nominato; "per costruzione" si scrive ancora "per disciplina
  di routing + tripwire" (KS-MS-85-3) — ma ora il tripwire HA morso
  (A-DS26/F16).
- **Slope futura**: A-BG33/A-BG35 da cablare nel prossimo verdict slope
  (A-PP28 già nel binario).
- **ROADMAP** ([[php-rust-todo-master]]): non ripresa, primo candidato
  WP-85(sessione).
