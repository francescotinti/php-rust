# WP_SESSION_41 — archivio storico della sessione WP-41

> Convenzione: un file per sessione; il handoff tiene solo l'ultima sintesi.
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> 🚫 **WP-41 (2026-07-23, commit `a24539f` revertato da `17bfbcd` — zero
> delta codice a fine sessione)** — **punto 1 (Leva C, shim gc_note)
> ESEGUITO E BOCCIATO su A/B; punto 2 (attribuzione churn drop/clone Zval)
> ESEGUITO: verdetto "nessuna leva locale ≥1%, il canale è strutturale →
> arco a registri", con una sola mini-leva locale identificata.**

## Punto 1 — shim `#[inline(always)]` sul frontend di gc_note: BOCCIATO

Implementazione (come da handoff): `gc_note` diventa shim inline-always
(census hook + un range-check sul discriminante — `Array|Ref|Closure|Object`
sono varianti CONTIGUE dell'enum, quindi un solo confronto) e il walk va
out-of-line in `gc_note_slow` `#[inline(never)]`; la ricorsione del walk
rientra dal shim così i contatori census restano identici per costruzione.

Parità PROVATA prima della misura:
- gc-census media == baseline WP-40: inserted 47.017.314 / freed 2.085.632 /
  demoted 47.468.514 / collects 1 (roots 50000 freed 1) / dtor 1001 IDENTICI;
  notes 177.051.677 vs .751 = Δ−74 su 177M (nondeterminismo workload noto).
- Probe battery (dtor39, sent_engine, sent_builtin, wp39, edge2/3,
  offset_edge) BYTE-ID vs binario old.
- Dal binario: simbolo `gc_note` SPARITO (inlinato ovunque), `gc_note_slow`
  presente out-of-line — lo shim ha fatto esattamente ciò che doveva.

Esito A/B interleaved stesso-giorno (media group, user CPU dai `.time`),
old = `0a03772`, **4 round**:
| round | old | new |
|---|---|---|
| 1 | 56,12 | 56,56 |
| 2 | 56,23 | 56,43 |
| 3 | 55,92 | 56,26 |
| 4 | 56,04 | 56,45 |
media old 56,08 vs new 56,43 = **new +0,62%, più lento in TUTTI e 4 gli
accoppiamenti** = regressione piccola ma reale, non rumore. Oracle di
giornata 20,84/20,98.

⭐⭐ **Lezione (perché il tetto ~1-1,5% non esisteva)**: i 86 campioni self
di `gc_note` nel riprofilo WP-40 erano il WALK dei **container** (borrow +
match + discesa a strong_count==1) — che lo shim non riduce — non
l'overhead call+match degli scalari. Il risparmio teorico sugli scalari
(~120M call evitate ≈ 0,2-0,3s) è stato più che mangiato dal **bloat
I-cache di ~60 siti inline dentro il run_loop** (stessa fisica di WP-33:
"branch mai-preso nel run_loop = +2,9%"). Corollario: su un canale il cui
self è WORK e non CALL-OVERHEAD, l'inlining del frontend è una leva morta.
⭐ Il metodo ha funzionato come previsto: parità provata PRIMA (census +
probe + simboli), verdetto SOLO dall'A/B interleaved, revert secco a parità
(`17bfbcd`, come SSO in WP-38). Da non riproporre senza dati nuovi.

Gate22/run32 NON eseguiti, deliberatamente: il tree post-revert è
byte-identico a `0a03772` (= stato gated WP-40; `git diff` vuoto), quindi
run31 e tutte le baseline gate restano valide per costruzione.

## Punto 2 — attribuzione per-chiamante churn drop/clone Zval + memmove

Metodo WP-26/39: sample 10s a t=35s (finestra GC/op-heavy) su run media,
binario corrente (= WP-40); due finestre (il sample WP-40 esistente + una
fresca `wp41-harness/gc-out/wp41.sample`). Finestra fresca: main thread
7234 campioni, ~5.700 on-CPU (recvfrom/read esclusi).

Self top-of-stack: clone\<Zval\> 202 + drop\<Zval\> 202 (= ~7% della
finestra), memmove 158, gc_note 102, run_loop self 858.

**Decomposizione per chiamante (lettura DIRETTA dell'albero del sample —
⚠️ un parser a stack sul testo di `sample` si rompe sulle righe della
sezione "Total number in stack": attribuire SOLO dalla sezione albero):**
1. **Churn operandi inline nel run_loop** — i nodi grossi: drop 39
   (`run_loop+162356`), clone 27 (`+59212`), clone 14 (`+99468`) + coda
   lunga di siti da 1-9 campioni sparsi su offset diversi = push/pop/
   overwrite di slot del modello a STACK. Nessun singolo op dominante:
   **strutturale**, il rimedio è l'arco bytecode-a-registri (Leva B).
2. **`recycle_frame`** ~156 in-tree: drop di slots/stack al teardown del
   frame + cascate `Rc::drop_slow`/`Repr` = rilascio semantico vero, poco
   elidibile localmente.
3. **`dim_is_walk` → `silent_get_path`** ~86 in-tree (≈1,5% finestra):
   dentro: `PhpArray::get`, `Zval::clone`, `coerce_key_silent`/
   `Key::from_zstr`, drop del temp. **Unica inefficienza meccanica
   locale**: il walk clona OGNI intermedio (`cur = v` per ogni chiave) e
   clona il leaf anche quando il chiamante è `isset`/`empty` che butta il
   valore (per `??` invece il clone del leaf serve).
4. **memmove 158 self**: frammentato (dyn_prop_name_value 6, concat_n_join
   5, run_loop 6, BTree, gd/webp — quest'ultima quota la paga anche
   l'oracle via libgd). Nessuna leva unica.

**VERDETTO (solo misura, nessuna leva applicata, come da mandato):**
- Il "collo drop/clone Zval" NON ha un chiamante dominante aggredibile:
  è il traffico strutturale del modello a stack → conferma indipendente
  del verdetto in vigore (Gemini post-WP-38/40): la leva vera è l'**arco
  bytecode-a-registri**, multi-sessione, census WP-33 alla mano.
- Mini-leva locale candidata (l'unica): **`silent_get_path` by-borrow** —
  walk iterativo per riferimento (Ref via borrow-guard), clone del SOLO
  leaf e SOLO quando il valore serve (`??`/coalesce; mai per
  isset/empty). Tetto dichiarato **~0,5-1%** (86/5.700 nella finestra
  peggiore). Possibile warm-up WP-42, A/B obbligatorio, aspettative basse.

## Stato a fine sessione

- Codice: IDENTICO a `0a03772` (WP-40 gated). Commit di sessione:
  `a24539f` (shim) + `17bfbcd` (revert) + docs. Cargo test: invariati
  (nessun delta codice).
- Gap: media 56,08/20,91 = **2,68×** (A/B odierno, invariato) · full
  **2,06×** (run31 resta baseline) · footprint **12,0×** (non aggredito).
- Artefatti: `wp41-harness/` (ab-out 4 round, gc-out census+sample,
  build-old/build-census/reprofile41 script).

## 📨 Direttive Gemini post-WP-41 (`20260723_gemini_post_wp41.md`) — verdetti (verificati su codice e dati, 2026-07-23)

- **✅ §1 (diagnosi I-cache sul fallimento Leva C) — CONCORDANTE, con una
  correzione di dettaglio**: la diagnosi è la stessa già a verbale (self di
  gc_note = walk container; bloat da ~60 siti inline). Restano due
  precisazioni: (a) è l'ipotesi più consistente coi dati (fisica WP-33),
  non una misura diretta — nessuno ha contato i miss L1i; (b) "L1
  Instruction Cache tipicamente 32KB" è taglia x86 — sui P-core Apple
  Silicon la L1i è 192KB: il meccanismo plausibile è pressione
  I-cache/BTB/decoder su un run_loop già enorme, non la saturazione di
  32KB. Verdetto invariato: Leva C chiusa.
- **✅ §2 (churn = sentenza sul bytecode a stack) — CONCORDANTE**: è la
  riformulazione del verdetto WP-41; nessuna correzione.
- **✅ §3a (warm-up silent_get_path by-borrow, WP-42) — ACCOLTA** (era già
  il punto 2 del handoff). Precisazione di design verificata sul codice:
  dentro `silent_get_path` non gira MAI codice utente (gli `Object` fanno
  `None`; ArrayAccess è intercettato FUORI, in `dim_aa_leaf`/
  `field_aa_walk`) ⇒ il walk by-borrow è sicuro coi RefCell; servono i
  borrow-guard per le catene `Ref` (oggi la ricorsione crea il guard per
  livello) e la biforcazione va fatta nei CHIAMANTI: exists/truthy
  (isset/empty → mai clone) vs value (`??` → clone del SOLO leaf). Tetto
  resta ~0,5-1%, A/B obbligatorio, abbandonare se flat.
- **⚠️ §3b (Leva B registri) — CONCORDANTE sull'apertura, DUE CORREZIONI
  DI MIRA**: (1) "istruire mago ad allocare i registri" sbaglia strato:
  mago è SOLO parser/lexer (verificato: `mago_syntax::parser::parse_file`
  + AST consumati da `lower/`); l'allocazione slot/registri vive nel
  compiler di phpr (`lower/` → emissione `Op`) e nel run_loop — mago non
  si tocca. (2) Il "periodo turbolento in cui il codice non compilerà o le
  perf peggioreranno" NON è compatibile con le regole del progetto
  (RULEBOOK: gate per nome a ogni commit, parità mai persa): l'arco va
  stadiato con parità a ogni commit (dual-mode dietro flag / lowering
  opt-in per-funzione / branch con gate regolari) — turbolenza confinata,
  mai su main. Concordante su "nessuna ottimizzazione locale mentre il
  cantiere è aperto" (= decisione già in vigore). Resta il prerequisito
  NON citato da Gemini: census WP-33 alla mano per fissare il tetto PRIMA
  di aprire.
