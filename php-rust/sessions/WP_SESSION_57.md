# WP_SESSION_57 — sessione di QUOTA: frequenza `.=` per sito (canale MORTO sul full) + metro non-biased Fase 3 (arr peak esatto 66,4MB = 4,3% del fisico)

> ⚡ **WP-57 (2026-07-26)** — Sessione di quota, zero modifiche ai binari di
> parità (release = `phpr-wp56` 65466c64… INTATTO, verificato: nessun gate
> necessario). **Ob.1**: op-census esteso (commit `b4e85ca`) a 6 famiglie di
> siti `.=` non-locali — eventi + Σ byte lhs (= volume copia O(n²)) + rhs +
> istogramma log2(lhs). Scoperta compilatore: `$o->p .= x` NON emette
> `Op::PropOpSet` ma `Dup;rhs;Swap;PropGet;Swap;Binary(Concat);PropSet`
> (assign.rs:955-961) ⇒ attribuzione del sito Prop via macchinetta a stati
> nel `record()` (firma Swap→Binary(Concat)→PropSet). Validato ORACLE sul
> probe56: byte ALLA CIFRA (12.797.440.000 = 5000·4999/2·1024 per sito);
> banda di rebuild calibrata **~30GB/s**. **VERDETTO sul FULL (prima run
> census di sempre sul full, 30.472 test)**: ArrElem 30.633 ev/0,34MB ·
> Prop 8.338/0,20MB · Field 9.963/0,16MB · StaticProp/Dyn/ArrayAccess 0 —
> **TOTALE 48.934 eventi, Σ lhs 0,70MB ≈ 23µs** (+~10ms overhead
> per-evento) su ~700s; nessun evento con lhs >1KB; media group: 221 ev /
> 2,5KB. **Il canale O(n²) non-locale è MORTO sul workload: Ob.1c (fuso
> esteso) NON si apre** — il ~500×/evento del probe è reale ma il sito non
> copre MAI il bordo delle stringhe grandi. **Ob.2**: canale arr da
> death-estimator (5,7× over, WP-56) a **live-accounting ESATTO** (commit
> `b23de56`: campo `accounted` + `census_sync()` nei mutatori, Drop
> riconcilia ⇒ zero drift; validato: reached↔live esatti a 128B su 3,5MB) +
> istogramma per-repr `arr_shape` nel walk EOR. **Census57-media
> (memgc57 41a21c65…): riconciliazione PERFETTA (master 63.435==63.435);
> arr peak ESATTO 66,4MB = ~4,3% del peak fisico 1.536MB** (il "39% del
> proxy" del checkpoint WP-55 era l'artefatto 6,0×); str peak 62,3MB;
> unit standing 222,6MB. Popolazione: packed 36,7k/4,2MB (dominante 1-2
> el), hashed 26,7k/9,2MB (21,4k da 1-4 el); overhead fisso 64B/arr ≈ 30%
> del canale. **Ordine tranche 2 coi numeri veri: arena handle ≈
> −10..−20MB peak (~−1% fisico), exact-fit hashed piccoli ≈ −1..−2MB — la
> prossima leva GRANDE di footprint NON è nel canale arr.** Divergenze
> WP-55/56 catalogate (`0b5790f`). ⚠️ Panic census-only a verbale (sotto).

## Ob.1 — dettaglio

- Siti strumentati: ArrElem(AssignOpPath, copre element+nested) ·
  Prop(macchinetta Swap;Concat;PropSet + hook su PropOpSet mai-emesso) ·
  StaticProp(StaticPropOpSet) · StaticPropDyn · Field(FieldAssignOp, 3
  bracci) · ArrayAccess(PathAa::Op). Hook `#[cfg(feature="op-census")]`
  nei handler = compilati VIA nei binari di parità; binario `phpr-op57`
  (dcb589ea…) in `phpr-op-target/` (target separato, pattern WP-53).
- Istogramma full: massa in b0 (lhs vuoto/non-str) e b5-b6 (16-64B);
  coda a b10 (≤1KB). La lezione WP-55 "la banda si realizza al bordo che
  il SITO copre" chiude il fronte: qui il bordo non è mai coperto.
- Attribuzione WP-54 ricalibrata: il "str-copy 12%" era il sito LOCALE
  (chiuso dal fuso WP-55, −2,56% full) + copie generiche, NON questi siti.

## Ob.2 — dettaglio

- `live_estimate(CH_ARR)` ora legge LIVE (esatto); CUM = alloc+adjust
  positivi (convenzione str). Lag del sync ≤ un passo di capacity
  dell'array caldo (probe: 128B su 3,5MB).
- Master media: arr peak 66,4MB · live EOR 9,5MB · cum 2.619MB/5,46M
  alloc; mark a proxy 268MB: arr.live 47,8MB. Subprocess: peak ~2,8MB.
- arr_shape (EOR master): packed b1=22.198 · hashed b1+b2=21.437 ·
  hashed b4 (9-16 el)=3.041/2,1MB · coda ≥b7: 96 arr/1,1MB.

## ⚠️ Panic census-only — INDAGINE COMPLETA (mandato utente, coda di sessione)

Prima run full census: panic del master a ~6 min, `run.rs:478`
`index out of bounds: len 78, index 78` (PC oltre fine `func.ops`).
**Indagine sistematica** (richiesta utente prima della tranche 2):
- **Caccia statica**: l'invariante di compilazione (ogni Func termina con
  `PushConst;Ret`) esclude il fall-through; `Ret` poppa sempre; tutti i
  target jump/catch/finally sono indirizzi COMPILATI (deterministici ⇒
  incompatibili con la non-riproducibilità); lo scheduler dtor applica
  `(top,ip)` immediatamente. Restano solo i meccanismi di park/resume.
- **Nella famiglia park/resume trovato un BUG REALE DI PARITÀ** (sotto):
  stato `yield_from` stantio — stessa famiglia meccanica (stato residuo
  nei frame ripresi), ma il collegamento a ip==len NON è derivabile dal
  codice: NON dichiarato causa del panic.
- **Riproduzione**: 3 full census aggiuntivi (1×--debug, 1×condizioni
  originali con trappola armata) TUTTI puliti — panic 1/4 run totali,
  non riprodotto. La repro strumentata chiude 30.472 test con
  **2F/86W = profilo run45** (conferma: i 7F della run --debug erano
  artefatti del timing sugli assert query-count).
- **Mitigazione permanente**: trappola diagnostica nei build census
  (`ip>=len` → fn/file/line, coda op, exc_table, stack frame) — se
  ricompare, la diagnosi è automatica. Zero costo parità (cfg).
- **Verdetto onesto**: root cause NON dimostrata; bug adiacente reale
  trovato e chiuso; strumento di cattura in posizione. Resta nel backlog
  con questa evidenza.

## 🔴 FIX ENGINE (parità): `yield_from` stantio dopo throw del delegate catturato (`e9a1679`)

`probe57-yieldfrom-stale.php`: delegate che lancia al primo avvio,
catturato DENTRO il generatore esterno → oracle `caught,10,20,30,done`;
phpr saltava la delega successiva (`caught,30,done`). Causa: nel ramo Gen
`ext.yield_from` è settato PRIMA di `ensure_started` (run.rs:2859→2861) e
nessun percorso d'errore lo puliva: il `yield from` successivo prendeva
il ramo re-entry (poppa il proprio delegate come "sent", vede il sub
stantio Done) ⇒ delega saltata in silenzio. **Fix**: l'unwind che
instrada un handler (catch O finally) nel frame azzera la delegazione
in-flight (semantica PHP: l'espressione yield-from ha lanciato), notando
i riferimenti rilasciati come root (disciplina gc_note). **TDD**:
`generator_yield_from_survives_caught_delegate_throw` rosso→verde.
**GATE57 VERDE**: corpus **1421 IDENTICO** per nome · refl **290
IDENTICO** · ORM **3E/13F IDENTICO** · hk **0E/0F** · cargo **1643/0**.
Probe = oracle. **Stash: `phpr-wp57` (sha256 a5ae7d27…)**; il fail-set
full per nome verrà ri-validato dal full A/B della tranche 2 (WP-58) —
il fix altera solo il caso errore-catturato del `yield from`.

## ⭐ Lezioni

- ⭐⭐ **Un canale ~500× più lento per evento può valere ZERO secondi**:
  la quota è ns/evento × frequenza × distribuzione delle taglie, e la
  frequenza×taglia va misurata sul workload PRIMA di progettare (qui:
  0,70MB totali ⇒ 23µs — nessun design aperto, mezza giornata salvata).
- ⭐⭐ **Il metro decide l'arco**: con l'estimatore la tranche 2 valeva
  −100..−190MB; col live esatto vale −10..−20MB. Stessa leva, stesso
  codice: solo il metro è cambiato. Le tranche future si quotano SOLO su
  live-accounting o walk (l'estimatore live×death-avg è morto).
- ⭐ **Il lowering è parte della mappa dei siti**: un op può esistere nel
  runtime e non essere MAI emesso (`Op::PropOpSet`) — il census per-sito
  deve seguire il compilatore, non l'enum degli op (macchinetta a stati
  sul bigram = pattern riusabile).
- ⭐ Il pattern accounted+sync+Drop dà live-accounting esatto drift-proof
  per costruzione (ogni mutazione passa da `&mut self`; il Drop riconcilia
  ciò che l'ultimo sync non ha visto) — riusabile per obj (unico canale
  ancora death-accounted).
- ⭐ La riconciliazione reached==live_n alla UNITÀ (63.435==63.435) è il
  gate di qualità di uno strumento di memoria: se non torna a zero, il
  metro non è pronto per decidere un arco.

## Prossimo (WP-58)

1. **⚖️ DECISIONE UTENTE (2026-07-26, in sessione): tranche 2 arena
   COMUNQUE** (banda onesta −10..−20MB; l'indagine panic richiesta è
   chiusa col verdetto sopra). Il full A/B della tranche valida anche il
   fail-set del fix yield_from (baseline run33, old = phpr-wp57).
2. Candidato metro: estendere accounted+sync al canale obj (chiude
   l'ultimo death-estimator).
3. Backlog: panic census-only run.rs:478 (trappola armata, evidenza
   sopra); bug isset via `__get` annidato (WP-42).
