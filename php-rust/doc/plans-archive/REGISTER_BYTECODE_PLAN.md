# Leva B — Arco bytecode-a-registri (piano multi-sessione, aperto in WP-42)

> Stato: PIANO + census. Nessun codice registri in WP-42 (deliverable da
> handoff). Unica "leva lunga" approvata (WP_SESSION_38); confermata da
> WP-41 (churn operandi = strutturale, nessun chiamante dominante) e dai
> verdetti Gemini post-WP-38/40/41 — con le due correzioni di mira di
> WP_SESSION_41 §verdetti recepite qui: (1) mago è SOLO parser, non si
> tocca; (2) niente "periodo turbolento" su main.

## 1. Evidenza (perché questo arco)

- **WP-41 (attribuzione)**: self drop+clone Zval ~7% della finestra
  GC-heavy SENZA chiamante dominante — è il push/pop/overwrite degli slot
  del modello a stack, sparso su decine di offset del run_loop. Nessuna
  leva locale ≥1% residua (Leva A esaurita WP-33/34, Leva C bocciata su
  A/B WP-41).
- **WP-38**: ridurre il COUNT delle alloc non muove mimalloc; ridurre il
  churn Zval sì.
- **Effetto a valle non contabilizzato dal solo canale operandi**: ogni
  clone/drop di container alimenta `gc_note` (WP-40: 177M note/run, self
  86 campioni = il walk) — meno cloni operandi ⇒ meno traffico GC a
  monte. Il tetto del census (§2) è quindi un TETTO INFERIORE.
- Le fusioni WP-33/34/37 (binary_fast, CmpJmp/CmpJmpConst, IncDecSlot,
  ThisPropGet, ConcatN, simple_call) sono registri ante-litteram: leggono
  slot/const direttamente saltando lo stack. Funzionano (−23,7% micro
  WP-33) ma sono bigrammi enumerati a mano: il modello a registri è la
  loro generalizzazione sistematica.

## 2. Tetto dal census (op-census WP-33, run gruppo media, misurato WP-42)

Fonte: `wp42-harness/census-out/` (media.census 16 blocchi-processo,
aggregate.txt; binario census dedicato, feature `op-census`).

- **Ops totali dispatchate: 743,9M** per run media.
- **Data-movement puro = 30,77% del dispatch (228,9M)**: PushConst 72,3M
  (9,71%) · LoadVar 58,4M (7,85%) · DerefTop 40,5M (5,44%) · Pop 31,5M
  (4,23%) · Dup 13,3M (1,79%) · StoreSlot 11,0M (1,48%) · LoadSlot 1,9M.
  Ognuna di queste è (dispatch + clone/drop di uno Zval sullo stack) =
  esattamente il churn attribuito in WP-41.
- Altri canali dominanti: ThisPropGet 73,6M (9,90%, già IC-cached — il
  costo residuo è il PUSH del risultato) · **Ret 62,6M (8,42%)** con
  bigramma **Ret→DerefTop 40,5M** (valore di ritorno transitato e
  deref-ato via stack → stadio 4 call-ABI) · Sweep 57,5M (7,73%, NON
  target registri) · CmpJmpConst 36,1M · Stringify 31,2M.
- **Bigrammi produce→consume da assorbire negli stadi 2-3**:
  ThisPropGet→CmpJmpConst 29,9M · ThisPropGet→Stringify 29,2M ·
  PushConst→Ret 19,1M · DerefTop→JumpIfFalse 17,6M ·
  ThisMethodCall→ThisPropGet 18,0M · CmpJmpConst→PushConst 16,3M ·
  Stringify→ConcatN 15,4M · PushConst→ThisPropGet 14,8M ·
  Dup→StoreSlot 9,4M + StoreSlot→Pop 9,3M (pattern assegnamento-e-scarta:
  con dst diretto spariscono ENTRAMBI) · PushConst→FetchDim 6,6M.
- **Stima tetto** (incrocio col profilo WP-41: churn drop+clone ~7% CPU
  della finestra GC-heavy + run_loop self ~15%): eliminare ~2/3 del
  data-movement (≈20% del dispatch) + il churn operandi associato + il
  traffico gc_note a monte ⇒ **tetto plausibile ~8-15% del CPU phpr**
  sull'arco completo (stadi 2-4). Da riverificare stadio per stadio:
  ogni stadio ha il suo A/B go/no-go (§5).

## 3. Vincoli non negoziabili (RULEBOOK + decisioni in vigore)

1. **mago non si tocca**: è parser/lexer (`mago_syntax::parser::parse_file`);
   l'allocazione registri vive nel compiler phpr (`compile/` HIR→`Op`) e
   nel `run_loop` (vm/run.rs). (Correzione di mira a Gemini, WP-41.)
2. **Parità a ogni commit su main**: gate per NOME (gate22) verdi a ogni
   step; niente branch lunghi senza gate; dual-mode/opt-in per-funzione
   (§5) per confinare la turbolenza. Zero pass→fail, mai.
3. **Zero `unsafe` nel value core** (RULEBOOK §0). I registri sono slot
   `Zval` normali nel Frame — nessun layout trick.
4. **Zval resta 16 byte** (static assert WP-27); l'arco non tocca il value
   model, solo CHI muove i valori.
5. **Semantica diagnostica invariata**: warning AT the faulting op
   (lezione flush WP-33); l'ordine di valutazione degli operandi non può
   cambiare (RHS-first negli assign composti, WP-14; sequenza di
   coercizioni in Op::Stringify, WP-34).
6. **Op::Sweep e il GC restano statement-boundary**: i registri temporanei
   vivono DENTRO lo statement; il loro svuotamento non deve cambiare il
   momento dei distruttori (sentinelle drop-order pinnate PRIMA di ogni
   stadio che tocca il teardown del frame — RULEBOOK §3).
7. **I-cache è il rischio n.1** (fisica WP-33 "+2,9% branch mai-preso",
   WP-41 shim bocciato): ogni stadio SOSTITUISCE forme stack con forme
   registro nei siti caldi — mai aggiungere varianti che convivono a lungo
   con le vecchie nei percorsi caldi. A/B interleaved obbligatorio per
   stadio; revert secco se flat/regressione (metodo WP-38/41).

## 4. Design di massima

- **Registri = estensione del file di slot del Frame.** Il Frame ha già
  `slots` (locals, indirizzati da LoadSlot/StoreSlot/IncDecSlot). Il
  compiler calcola per funzione `max_temps` e li alloca come slot extra
  (`nslots + t`): un "registro" è uno slot temporaneo con indice statico.
  Niente nuovo storage, niente nuova invalidazione GC: il recycle_frame
  e drop_bounded li coprono già.
- **Operand sourcing sugli op caldi, non un secondo ISA.** L'enum `Op` ha
  ~180 varianti: riscriverle tutte è il progetto sbagliato (I-cache,
  §3.7). La forma a registri è un CAMPO OPERANDO sugli op caldi:
  `src: Operand { Temp(u16) | Slot(u16) | Const(u16) | Stack }` e
  `dst: { Stack | Temp(u16) | Slot(u16) }` — il run_loop legge
  l'operando per borrow dallo slot invece di poppare un clone dallo
  stack. Gli op freddi restano stack-based per sempre (il census dice
  quali contano).
- **Lowering in due passi nel compiler**: il pass esistente emette stack-Op
  (invariato, è la semantica di riferimento); un pass di
  register-allocation per-funzione riscrive le sequenze
  produce→consume locali (LoadSlot/PushConst/temporanei lineari) in
  operandi diretti ed elide Push/Pop/Dup morti. Il pass è opt-in
  per-funzione (§5) e DEVE preservare l'ordine di valutazione e i punti
  di flush diagnostici (il refold di un Push oltre un op osservabile è
  vietato).
- **Unit-cache**: il bytecode cacheato porta il flag di modalità; la
  chiave di cache include la modalità finché il dual-mode esiste.
- **Ref (3 canali, WP-33)**: gli slot che possono contenere `Zval::Ref`
  hanno già semantica write-through nei fast-path slot-based esistenti;
  il pass registri riusa le stesse guardie (`has_hints`,
  `scope_private_overrides`-style) — un operando `Slot` con Ref possibile
  passa dalla forma generica.

## 5. Stadiazione (parità a ogni commit; ogni stadio = 1+ sessione)

- **Stadio 0 (WP-42, questa sessione)**: census (§2) + questo piano.
  Nessun codice.
- **Stadio 1 — infrastruttura a parità zero-delta**: `max_temps` nel
  `Func` (=0 ovunque), estensione Frame, tipo `Operand`, pass di
  riscrittura vuoto dietro flag runtime (`PHPR_REG_LOWER`), diff di
  bytecode atteso VUOTO a flag spento; gate22 + A/B "flag spento vs old"
  = rumore zero (guardia contro il costo del solo layout, fisica WP-32).
- **Stadio 2 — Binary/CmpJmp a operandi diretti**: generalizza
  binary_fast/CmpJmpConst: `Binary{l,r,dst}` con sorgenti Slot/Const/Temp;
  il pass riscrive i trigrammi LoadSlot,LoadSlot,Binary. I bigrammi
  enumerati WP-33/34 vengono ASSORBITI (sostituzione, non convivenza).
  A/B per stadio.
- **Stadio 3 — temporanei di espressione**: catene lineari dentro lo
  statement (FetchDim, ConcatN, Stringify, PropGet fast) scrivono/leggono
  Temp; elisione Pop/Dup/Swap. Sentinelle drop-order PRIMA (i temp
  cambiano il possessore dell'ultimo strong count dentro lo statement —
  il momento di rilascio deve restare lo statement-sweep).
- **Stadio 4 — call ABI**: argomenti sorgenti da Temp/Slot senza transito
  stack (`CallArgs`/`simple_call` WP-37 già bypassano parte del giro);
  bind_params legge dagli operandi. Qui vive il grosso del churn
  restante (enter/exit frame).
- **Stadio 5 — consolidamento**: se il census post-stadio-4 dà lo stack
  residuale nei percorsi caldi, rimozione del dual-mode e del pass
  opt-in; altrimenti il dual-mode resta e si chiude l'arco.
- **Go/no-go per stadio**: A/B interleaved ≥4 round; keep solo se new <
  old consistente; flat = si prova lo stadio successivo solo se il census
  lo giustifica, altrimenti chiusura arco (le fusioni esistenti restano).

## 6. Rischi e mitigazioni

| Rischio | Mitigazione |
|---|---|
| I-cache bloat da varianti duplicate (WP-33/41) | sostituzione, non aggiunta; A/B per stadio; revert secco |
| drop-order dei temp ≠ stack (dtor timing) | sentinelle pinnate prima dello stadio 3; Sweep invariato |
| ordine di valutazione/diagnostica alterato dal pass | pass conservativo: mai riordinare oltre op osservabili; probe battery diagnostiche |
| unit-cache incoerente tra modalità | chiave di cache con flag modalità (stadio 1) |
| Ref write-through saltato da un operando diretto | guardie ereditate dai fast-path slot esistenti; corpus refl/orm gate obbligatori |
| effort multi-sessione senza payoff | census PRIMA (tetto); go/no-go per stadio |
