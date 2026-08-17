# VERBALE — sedia MATSAKIS (ownership/aliasing/borrow) — S-151, Fase 1 indipendente

`VERDETTO: CONCORDO CON EMENDAMENTI` sull'impianto A1..A4. La rimozione di
RefCell è sana SOLO se si nomina che cosa la sostituisce: il borrow dinamico
oggi è anche un RILEVATORE (BorrowMutError = allarme); con lo store a indici
lo stesso bug diventa scrittura silenziosa sull'occupante sbagliato dello slot.

## Q3 — cammini re-entranti REALI che oggi passano dal borrow dinamico
Pivot unico: `call_method_sync` (vm/mod.rs:7512) / `enter_object_method`.
1. **__destruct da sweep/collect**: sweep di statement e `collect_cycles_inner`
   (mod.rs:5121–5315) chiamano `call_method_sync(...,"__destruct")` (5315)
   DENTRO il loop sui doomed; il dtor può creare oggetti (store cresce/realloc),
   risuscitare (`global $keep=$this`, test 22401), liberare altri id. Il
   full-scan seed itera `created` con `borrow()` (5168). Con lo store: nessun
   riferimento cache-abile attraverso la chiamata (borrowck lo rifiuta — bene);
   il loop DEVE essere a indici con ricontrollo di vitalità per passo.
2. **__get/__set/hook**: ~30 siti `magic_applies`+`lazy_prop_access`
   (run.rs:667,852,1060,1085,4622–4687,6638; mod.rs:7798–7842,11739–11761,
   13646–13672). Oggi: borrow → release → call. Domani: `&mut vm.objects[id]`
   non attraversa `&mut self` ⇒ re-lookup obbligato DOPO la chiamata; lo slot
   può essere stato liberato/riusato dal magic stesso.
3. **offsetGet/offsetSet (ArrayAccess)**: run.rs:2305–2521,6233–6256;
   mod.rs:11713,13843 — dentro i cammini dim, inclusi i fast-path FD1/RMW.
4. **Iteratori**: `issue_iter_call current/next/valid` (run.rs:3141;
   mod.rs:7901,9816) per passo; `IterState::ObjRefs` itera prop by-ref di un
   oggetto mentre il corpo le muta. Con props inline: uno spill >8 a metà
   iterazione invalida ogni riferimento trattenuto — la disciplina è
   re-fetch-by-index a OGNI passo, mai `&Zval` conservato.
5. **__toString** in conversioni (run.rs:347,547; mod.rs:7760,11240,11332,
   12851) — anche dentro chiavi d'array e concatenazioni.
6. **Hostcall→VM**: usort/array_map/preg_*_callback, stream-wrapper utente,
   session handler, autoload, `Number::__op` bcmath (try_number_binop,
   mod.rs:7536), lazy prop-init thunk (host.rs:2834) e
   `resetAsLazy`→__destruct sincrono (host_reflect.rs:654–675).
Nota positiva: PropIc cache INDICI, non puntatori — sopravvive al realloc
dello store; sinergia reale con A3.

## Il vero buco: refcount esplicito ⇒ Drop manuale
Con `Rc`, clone/drop sono autosufficienti. Con contatore nello store,
`Zval::clone`/`Drop` non raggiungono `vm.objects`: OGNI Vec<Zval> temporaneo,
ogni `?` early-return in host.rs (7,6k righe), ogni unwind di eccezione deve
bilanciare i conti. Dec mancato = dtor mai eseguito (bug SEMANTICO osservabile,
non solo leak); inc mancato = dtor anticipato. Due vittime NOMINATE:
- `impl Drop for PhpArray` (php-types/src/array.rs:503, leva L-RD1 promossa
  S-142): il teardown inline non può decrementare uno store che non vede.
- invariante ricevitore WP-102 (run.rs:648: «il __destruct sincrono vede
  sempre ≥1 handle posseduto»): oggi vale per costruzione via Rc; con gli id
  va RIPROVATA (l'id sullo stack Rust non tiene vivo niente da sé).
Disciplina che regge: coda di decrementi thread-local (push-only, niente
borrow) drenata dallo sweep di statement GIÀ esistente ed EAGER-audited
(mod.rs:3956 «always buffer, sweep at END of statement») — il timing dei
distruttori resta quello già collaudato. Senza questa coda nel criterio A3,
A3 è irricevibile per me.

## Q1 — sequenza
A1 prima di tutto: sì (A3 indecidibile senza numeri). MA: (i) il census va
chiavato per CANALE/funzione semantica, MAI per file:riga, o A2 lo invalida;
(ii) A2 integrale (4–6 sessioni) fa invecchiare il census su codice che si
muove (l'inliner flippa: lezione WP-104). Emendo verso l'interleaving:
A2 limitato al perimetro che A3 toccherà (object_store, exec/ops_objects,
cuciture di run_loop); dom/reflect/system solo se bloccano.

## Q2 — gate per tranche
Sostituto della byte-identità (vietata): per una tranche move-only, disasm di
run_loop (istr/bl-count) INVARIATO o delta d'inlining DICHIARATO, + batteria
+ corpus 1412×2 per NOME + fixture bilaterali + micro R=5 a sola-regressione.
Coppia WP: dovuta a ogni pin nuovo (regola utente) ⇒ UN pin per SESSIONE
(tranche accorpate), una coppia per pin — non per tranche. Partizione:
foglie fredde prima (host_reflect, dom, system), dispatch di run_loop ULTIMO
e minimo; commit move-only (mai move+rewrite nello stesso commit),
`pub(super)`, firme intatte, `#[inline]` preservati.

## Q3c — numeri che il census DEVE produrre (decidibilità, non utilità)
1. CHURN 32% SPACCATO per specie: Rc-Object vs Rc-Array vs ZStr vs
   Rc-RefCell-Ref, clone e drop, su ORM (non solo suite).
2. Canale borrow/borrow_mut contato a sé (è nella partizione: bene) e
   spaccato Object vs Ref.
3. dec-a-zero vs dec-a-non-zero (dimensiona il cammino dtor-schedule).
4. Frequenza fughe re-entranti (magic/hook/dtor/offsetGet/iter) per 1k op
   su ORM: dimensiona il costo del re-lookup.
5. **Distribuzione #props per istanza su ORM**: le entity Doctrine superano
   spesso 8 — se la mediana ORM è >8, inline-8 non paga proprio dove serve.
6. Riconciliazione col TETTO movimenti 1,27 s ≈ 3,4%.
Sulla tensione (obbligo di pronuncia): il pilastro 1 di Gemini («azzerare il
traffico di movimento») vale ≤3,4% del gap ORM per il NOSTRO tetto — il caso
di A3 deve reggersi sugli ALTRI canali (borrow-check, indirection/località,
alloc/oggetto, inline props), ciascuno prezzato; «30–45%»/«−35%» restano
cifre non firmate. MAI ObjectId Copy-senza-refcount: concordo col vincolo,
e aggiungo che l'id va GENERAZIONALE (index+gen): WeakReference/WeakMap non
devono risuscitare il nuovo occupante di uno slot riusato; `spl_object_id`
espone il solo indice (il riuso degli handle è anche di PHP). Precedente
interno già sano: risorse pdo/gd/xslt/tidy sono GIÀ side-table a u32.

## Q4 — dente
Sede: BATTERIA (la CI ha 3 giorni di ritardo — dato fresco; il pre-commit è
solo consultivo). Forma: allowlist per FILE con conteggio ATTUALE come tetto,
ratchet monotono decrescente a ogni tranche A2, file NUOVI ≤2.000. Il
conteggio esplicito per nome evita l'auto-morso da pattern (bea7ea3).

## Q5 — ciò che manca (invalida se trascurato)
1. **Zval::Ref(Rc<RefCell<Zval>>)** (php-types/src/zval.rs:30): il piano parla
   solo di oggetti; i reference PHP restano una cella a borrow dinamico SUL
   cammino di assegnazione. Un A3 che ignora Ref lascia RefCell nel motore.
2. Closure (`Rc<Closure>`) catturano Zval contati: chi decrementa al drop
   della closure? Stessa coda — va nel criterio.
3. Aliasing a due oggetti (`__clone` copia props, `==` ricorsivo): borrowck
   rifiuta due `&mut` nello stesso Vec ⇒ `get_disjoint_mut`/copy-out, da
   prezzare.
4. Gregg inverso: BT1 insegna che UNA hostcall infedele valeva 1,2× su ORM —
   prima di 4–6 sessioni di chirurgia, il census (testa hostcall +
   none.other 94,6M) può nascondere altre leve BT1-class più economiche.

## Kill-switch pre-registrabili
- K-M1: census A1 con (borrow+refcount+alloc-oggetto) < 15% del tempo ORM
  ⇒ A3 retrocede, si spende la testa hostcall.
- K-M2: census con canale singolo-NOME ≥5% ORM ⇒ leva di fedeltà PRIMA di A3.
- K-M3: tranche A2 con delta disasm non dichiarabile o micro fuori banda ⇒
  revert della tranche, non emenda del gate.
- K-M4: prototipo coda-decrementi che cambia UN output di fixture
  dtor-order ⇒ stop A3, ridisegno prima di ogni riga promossa.
- K-M5: mediana #props ORM > 8 ⇒ inline-8 fuori dal criterio A3 (o soglia
  rifondata dal numero).
