# COUNCIL_WP97_REVIEWS — 9 sedie + 3 team + sintesi (protocollo due fasi, token-lean)

Verbali individuali = FONTE VINCOLANTE. Assemblato meccanicamente (cat).

----------------------------------------------------------------------
## FILE: verbali/verbale-1-hoare.md

# Verbale sedia 1 — Hoare (WP-97, su S-95.0 A-ZV2 F1+F2 e programma §WP-96 F3)

Perimetro: design linguaggio/runtime Rust, safe-only. Mandato: refutare.

## VERDETTO: CON EMENDAMENTI

Le bande F1/F2 (47,11% rc, safe 90,21%, ALTA) reggono come misura: la
direzione d'errore dichiarata è rispettata quasi ovunque e il difetto che ho
trovato sposta i conteggi in modo trascurabile. NON regge la premessa del
§WP-96 «F3 emesso solo dove F2 lo consente»: `movable_safe` non è una base di
emissione sound (vedi Refutazione capitale). Il Rust è safe ovunque
(l'`unsafe` di `libc::atexit` è solo nella build di misura); bitset difensivi
(`get` fuori intervallo = falso) corretti.

## Emendamenti

**A-TH-97-1 — Archi eccezionali non devono subire il kill delle def.** In
`analyze` (liveness.rs 427-434) il contributo exc è fuso in `live_out[i]` e
poi `inb` sottrae i `defs`: su un arco eccezionale la def può non essere
avvenuta. Regola sound e prudente: se `exc_edges[i]` non è vuoto, le def di
`i` non uccidono (o si fonde `live_in[handler]` in `inb` DOPO il kill).
Ricontare F1/F2 col fix prima di scrivere l'opcode.

**A-TH-97-2 — Niente `_ => {}` in `effect()` e `renounce()`.** Il wildcard è
un buco di soundness a futura memoria: ogni variante `Op` nuova che legge uno
slot diventa silenziosamente «nessun effetto». Match esaustivo con elenco
esplicito dei no-effect; audit una-tantum delle varianti oggi nel wildcard
(es. `CallBuiltinRefCell`: è in `renounce` per `observes_scope` ma non ha
use/def in `effect()` — verificare che non porti uno slot).

**A-TH-97-3 — Il guard runtime su `Ref` è necessario ma non sufficiente, e la
banda MEDIA obbliga il confronto.** L'osservabilità della vita non è solo
`__destruct`: `WeakReference::get`, finalizzazione risorse (flush/close di
stream), timing GC. Se a inizio F3 si sceglie il nucleo stringhe (rischio
zero), quella è banda MEDIA: per la STESSA regola a tre bande di §P1 va
confrontata col piano B a parità di conto, non adottata in silenzio.

**A-TH-97-4 — Contabilità GC del valore preso.** Oggi la morte del valore
passa da `gc_note(&old)` in `StoreSlot`/`Pop`; un valore spostato che muore
nel consumatore (es. dentro `binary_value_ab`) deve comunque passare da
`gc_note`, o il cycle collector perde radici (lezione WP-72: gli Rc non
raccolgono i cicli).

**A-TH-97-5 — Emissione a compile-time e predizione ri-derivata.** F3 non
deve riusare la cache address-keyed di `zvalcensus` (collisioni da riuso
d'indirizzo accettabili solo in misura); dopo QUALUNQUE fix all'analisi la
predizione di `slot_reads_avoided` per il controllo positivo F4 va
ri-derivata dai contatori nuovi, o il controllo di Bak è vacuo.

## Kill-switch

**KS-TH-97-1**: esiste una variante `Op` che legge uno slot per indice, cade
nel `_ => {}` e non è coperta dalle rinunce F2 → conteggi F1/F2 e ogni F3 su
di essi invalidi.

**KS-TH-97-2**: `TakeSlot` emesso da `movable_safe` senza A-TH-97-1 → il
lavoro F3 è invalido (parità violabile per costruzione).

**KS-TH-97-3**: il fix A-TH-97-1 porta `safe_su_would_take_pct` sotto 60 o la
banda safe fuori dall'ALTA → P2/P3 da ri-derivare, non difendere.

## Refutazioni capitali

**Sì, una — prospettica su F3, non sulle bande.** Controesempio: `$m` vivo;
`echo $m` (fuori da ogni regione protetta); poi `try { builtin_out(..., $m) }
catch { echo $m; }` dove il builtin è abbassato a `CallHostBuiltinOut` con
`out_slot = $m` e lancia (TypeError/ValueError) PRIMA di scrivere l'out.
L'analisi: def di `$m` sottratta anche sul contributo dell'arco exc →
`live_out` dell'`echo` non contiene `$m` → `movable_safe` (load fuori
regione, slot non rinunciato). Con `TakeSlot` il catch vede `Undef` (warning
«Undefined variable»); Zend stampa il valore. Stessa classe: `StoreSlot` con
coercizione typed-ref, `CallHostBuiltinScanf`. La sessione S-95.0 resta
valida (sola misura); il programma §WP-96 è invalido finché A-TH-97-1 non è
applicato e ricontato. Aggiungere il controesempio ai test-trappola del
punto 3 del §WP-96.

----------------------------------------------------------------------
## FILE: verbali/verbale-2-matsakis.md

# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — WP-97

## VERDETTO: CON EMENDAMENTI

Il dataflow F1 è sano: direzione dell'errore scelta (scritture non modellate
sottocontano; letture non modellate = elenco F2), def per-arco su
IterNext/CatchMatch corretti, `StoreGlobal` come uso-mai-def evita il
sovraconteggio giusto. F2 copre i canali statici di condivisione. Ma il
numeratore `would_take_safe_rc` è INQUINATO e la mossa F3 sposta il punto di
morte dei valori fuori dai sentieri che il GC osserva. Non benedico la banda
1,91–2,75% come predizione: è un TETTO.

## Emendamenti

**A-MS-97-1 (numeratore Ref non misurato).** Uno slot che a runtime regge
`Zval::Ref` senza op marcante nel bytecode della funzione — lato interno di
`use(&$x)`, slot del main aliasato da `global` dentro una callee — passa i
predicati statici F2 ed entra in `would_take_safe_rc` (zvalcensus.rs:111
ricorre nel Ref e conta l'interno), ma per il guard di §WP-96 punto 1 NON è
takeable (si de-referenzia = clone). La frazione è ignota. Prima della
decisione di perimetro F3: aggiungere `WOULD_TAKE_SAFE_REF` (un `matches!`)
e un run di rimisura; la banda P3 va derivata su safe−ref, non su safe.

**A-MS-97-2 (guard a WHITELIST, non blacklist).** «Ref si de-referenzia»
è una blacklist: lascia takeable oggetti E array — e un array può contenere
oggetti, quindi l'anticipo di `__destruct` non è confinato al tipo Object.
Il guard runtime sia `matches!(cell, Zval::Str(_))` per la prima spedizione
(rischio distruttori zero per costruzione, banda onesta = `_str`
0,84–1,21%, che per §P1 impone il confronto col piano B, non l'abbandono).
Il perimetro F2 intero solo dopo, con corpus distruttori-ordine dedicato.

**A-MS-97-3 (il drop deve restare sui sentieri notati).** Oggi l'ultimo
handle in uno slot muore su `StoreSlot` (→ `gc_note(&old)`, run.rs:632) o
al teardown del frame. `mem::replace(_, Undef)` sposta quella morte dentro
il consumo inline (binary_value_ab, Pop già nota, gli altri?): se il
sentiero di consumo non registra la radice, i cicli il cui ultimo root
muore lì non vengono MAI raccolti. F3 deve provare per elenco che ogni
consumatore di un valore preso passa dalla stessa disciplina `gc_note`, o
instradarlo. Ownership trasferita ≠ lifecycle osservato.

**A-MS-97-4 (chiave cache = identità, non contenuto).** La chiave
(ptr Func, ptr ops, len) assume indipendenza dei due riusi d'indirizzo, ma
i due blocchi vengono liberati INSIEME e riallocati insieme: con mimalloc e
size-class uguali (eval ripetuto, per-request nel server) il riuso è
correlato, non «coincidenza doppia». Inoltre la chiave non vede mutazioni
in place di `ops` (quickening/IC futuri). Accettabile in sola misura;
VIETATO portare questo keying in F3 — l'emissione sia compile-time, e si
documenti l'invariante «Func mai mosso, ops mai riscritto».

**A-MS-97-5 (due buchi di elenco).** (a) `CallBuiltinRefCell` in
`effect()` cade nel ramo vuoto e in `renounce()` è solo name-check: se il
cell può sopravvivere alla call, lo slot è condiviso e non rinunciato —
provare che ogni sito è preceduto da PushRef/MakeRef marcante, o marcarlo.
(b) design95 elenca `debug_zval_refcount` fra gli osservatori;
`observes_scope` ha 7 nomi e non lo include — riconciliare.

## Kill-switch

**KS-MS-97-1**: divergenza per NOME sui test-trappola (`use(&$x)` dopo il
take, ordine distruttori, `compact`) o sul corpus → si ritira l'emissione,
non si difende.
**KS-MS-97-2**: in F4 `slot_reads_avoided` sotto la predizione F2 oltre la
frazione Ref misurata (A-MS-97-1) → meccanismo non agito, banda da
ri-derivare prima di rivendicare qualunque Δ.

## Refutazioni capitali

**Una**: `guadagno_cpu_atteso_safe_pct` letto come predizione P3 è
REFUTATO — il numeratore contiene esecuzioni Ref non-takeable in quantità
non misurata; è un maggiorante, e firmare P3 su un maggiorante viola la
regola «predizione derivata da una misura». Il programma F3 resta in piedi
sugli emendamenti sopra.

----------------------------------------------------------------------
## FILE: verbali/verbale-3-klabnik.md

# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — WP-97

## VERDETTO

**PASS CON RISERVE VINCOLANTI** su S-95.0 (F1+F2 sola misura) e sul
programma §WP-96/F3. La decisione di banda ALTA regge: ho riverificato le
derivazioni dei raw (47,11%→2,12/3,06; 42,33%→1,91/2,75; 18,65%→0,84/1,21 —
aritmetica corretta) e la conclusione è robusta fino a un canale ≈2,8%
(ALTA cade solo se il canale reale è < 1,3/0,4711). Identity committati
(`f1-out/f1.identity`: sha census + sha parità + head + epoch), raw
per-processo in-repo, append multi-processo documentato, env letta solo a
exit. MA la matrice `effect()` ha UN buco reale e ZERO test a macchina.

## Emendamenti

- **A-SK-97-1 (buco di matrice)**: `Op::NewAnonDeferred` ri-valuta gli
  argomenti del costruttore «nel bridged scope del chiamante»
  (bytecode.rs §deferred): legge i locali per nome a runtime, come `eval`.
  Non è in `effect()` (cade nel `_ => {}`) né in `renounce()` (che copre
  solo Eval/Include/dyn/observes_scope/generatori). Va aggiunto al
  whole-renounce (con `DeclareDeferred` prudenziale) e F2 va ricontata
  PRIMA di F3 (delta atteso ~0, ma il conto si fa, non si presume).
- **A-SK-97-2 (decadimento silenzioso)**: eliminare il `_ => {}` di
  `effect()`: match esaustivo con elenco esplicito dei no-effect, così
  ogni variante FUTURA dell'enum `Op` con effetti su slot rompe la
  compilazione della build census invece di venire classificata
  no-effect in silenzio. L'invariante di testata oggi non è presidiata
  da nulla.
- **A-SK-97-3 (smoke → test)**: gli smoke di S-95.0 sono stati manuali e
  NON committati (nessun `#[cfg(test)]` in liveness.rs, nessun
  riferimento nei test d'integrazione). Il negativo che ha morso
  (slot vivo sul back-edge non contato) è oggi irripetibile a macchina.
  Prima o nello stesso commit di F3: fixture con conteggi `would_take*`
  ESATTI attesi + i negativi (back-edge, `global`, `use(&$x)`,
  `compact`) come test cargo/phpt.
- **A-SK-97-4 (grade misto nei raw)**: i `.out` sono grade=VERDICT ma i
  campi `guadagno_cpu_atteso_*` derivano dal canale 4,5–6,5% che è una
  STIMA di prof95 non riproducibile dal raw. Marcare il grade per campo
  (conteggi=VERDICT, guadagni=DERIVED-ESTIMATE), pinnare la derivazione
  del canale in un raw proprio, scrivere il margine di robustezza (2,8%).
  I 42 eventi di divergenza dal before restano un ignoto nominato: se in
  F4 crescono d'ordine, non è più «rumore».

## Kill-switch

- **KS-SK-97-1**: se S-96.0 spedisce F3 (commit che CAMBIA l'emissione e
  il binario) con il canale env di git ancora aperto, i PASS di parità F3
  NON sono verdict-grade. La clausola timebox «si spedisce l'oggetto e
  l'apparato torna in coda» NON si applica ai gate di un commit che tocca
  l'emissione: lì A-SK-93..97 (denti T27-T30) va chiusa prima, o i gate
  si dichiarano provisional per NOME.
- **KS-SK-97-2**: se il handler `TakeSlot` non guarda il TIPO a runtime
  (un `Zval::Ref` si de-referenzia, mai si sposta — il lato interno delle
  closure by-ref è INVISIBILE alla `Func` del callee, `param_by_ref` non
  lo copre), o se il riconteggio post A-SK-97-1 porta
  `safe_su_would_take_pct` sotto 60, F3 si ferma e la banda si ri-deriva.

## Refutazioni capitali

**Nessuna sul verdetto di banda** (ALTA sopravvive ai margini). Refutate
però due affermazioni: (1) la testata di liveness.rs — «le uniche letture
fuori modello sono ESATTAMENTE quelle del perimetro F2» — è FALSA:
`NewAnonDeferred` è fuori modello e fuori perimetro; (2) «gli smoke
bastano come controllo positivo/negativo» — no: un controllo che non può
più mordere non è un controllo, è un ricordo. L'apparato A-SK-93..97 è
correttamente prioritizzato e NON retro-blocca S-95.0 (sola misura, sha
di parità pinnato indipendentemente dall'env del gate) — ma vedi
KS-SK-97-1 per F3.

----------------------------------------------------------------------
## FILE: verbali/verbale-4-hejlsberg.md

# Verbale — Sedia 4 (Hejlsberg) — WP-97
Perimetro: compilatori incrementali, interning/dedup, collocazione delle analisi.

## VERDETTO
S-95.0 (F1+F2 in sola misura) regge nel mio perimetro: la direzione d'errore è dichiarata, i conteggi sono deterministici, il binario di parità è invariato. Il programma §WP-96 (F3) invece **NON è eseguibile come scritto**: due assunti sono refutati sotto. F3 ammissibile SOLO sotto gli emendamenti.

## Emendamenti

**A-AH-97-1 (collocazione).** L'analisi va nel COMPILATORE, all'emissione della funzione, PRIMA dell'inserimento nella unit cache (A-BB6): la riscrittura `LoadSlot→TakeSlot` deve stare nella unit cache stessa, così include ripetuti la pagano UNA volta per unit, non per inclusione. Nessuna struttura di analisi sopravvive a runtime: il risultato si consuma nell'emissione e si butta. `eval` paga per compilazione (corpi piccoli; coperto da A-AH-97-2).

**A-AH-97-2 (budget di compile-time).** `analyze` è un punto fisso full-sweep con `Bits::new`+`clone` per op per iterazione e `exc_edges` O(regione×op): accettabile in misura, non in compilazione su ~migliaia di funzioni WordPress. Obbligo: (a) bailout per funzioni sopra soglia `ops × slot_words` — il bail = rinuncia totale (si tiene il clone), correttezza intatta; la soglia DERIVATA dalla distribuzione `sites_total` per funzione del census, non scelta (NON-riproporre «soglia non derivata»); (b) buffer preallocati/worklist al posto del full-sweep; (c) giudice = oracle compile-side `--list-tests` (WP-59), coppia prima/dopo nello stesso commit.

**A-AH-97-3 (identità ≠ puntatore).** La chiave `(addr Func, addr ops, len)` di `zvalcensus.rs` è tollerabile solo perché il conteggio è advisory. Se una qualunque cache d'analisi sopravvive in F3, la chiave deve essere identità STRUTTURALE (unit-id + indice funzione, o hash del contenuto ops), mai indirizzi: con unit cache TL, eval e riuso dell'allocatore, l'ABA su indirizzo attribuirebbe `movable` di una funzione a un'altra — in F3 non è un sovraconteggio, è un take sbagliato.

**A-AH-97-4 (layout di Op).** `size_of::<Op>() == 48` come static assert/test NELLO STESSO commit dell'opcode: una variante nuova è layout-free solo finché il payload sta nel massimo esistente E il conteggio varianti non attraversa la soglia del discriminante. In alternativa flag: MAI un campo nuovo dentro `LoadSlot`; se flag, un bit rubato nell'operando slot (slot < 2³¹) o bitmap laterale per-funzione (1 bit/op, fuori dall'array caldo).

**A-AH-97-5 (banda al netto).** La banda F4 (1,91–2,75% safe) è LORDA: non sottrae il costo dispatch del corpo nuovo. Serve una build A/A con handler compilato ma mai emesso per quotare la penalità alla WP-38 prima di leggere il Δ.

## Kill-switch

**KS-AH-97-1**: `size_of::<Op>() != 48` dopo F3 → commit respinto (footprint per worker).
**KS-AH-97-2**: oracle `--list-tests` compile-side peggiora oltre il budget deliberato → niente ship, prima bailout/worklist.
**KS-AH-97-3**: qualunque cache keyed-by-pointer nel percorso F3 di release → reject senza discussione.

## Refutazioni capitali

**RC-1.** design95 chiude con «questa NON aggiunge opcode al percorso caldo, ne cambia uno esistente»; §WP-96 dice «l'opcode TakeSlot». Le due frasi non stanno insieme: o corpo handler nuovo (lezione WP-43: il costo è il NUMERO di corpi caldi) o branch in un arm esistente (WP-38: +2,9% da un branch mai preso). La banda predetta ignora entrambi i costi: com'è scritta, la predizione P3 per F4 è irricevibile finché non è al netto (A-AH-97-5).

**RC-2.** Riusare il meccanismo F1 tal quale (analisi lazy a runtime, cache a puntatori) in F3 è refutato in capite: puntatore non è identità di funzione, e ciò che in misura era una collisione innocua in emissione diventa corruzione semantica silenziosa. F3 esiste solo come analisi di compilazione (A-AH-97-1/3).

----------------------------------------------------------------------
## FILE: verbali/verbale-5-bak.md

# Verbale sedia 5 — Bak (VM: alloc-rate, path caldi, corpi handler) — WP-97

## VERDETTO

**NON REFUTATO nel merito; PROCEDI CON EMENDAMENTI VINCOLANTI.** F1+F2 sono
lavoro come lo pretendo: contatori esatti, determinismo dimostrato, negativo
che morde, direzione dell'errore scelta. Ma l'aritmetica delle bande è
un'ESTREMO SUPERIORE spacciato per stima centrale, e il §WP-96 contiene una
frase falsa sul mio perimetro.

**Le bande.** La moltiplicazione frazione×canale è aritmeticamente esatta
(verificata cifra per cifra nei due .out) ma il fattore 4,5–6,5% ha due
difetti: (a) è SCREEN R=1 (la mia consulenza lo dichiara), quindi
`grade=VERDICT` vale per i CONTEGGI, non per le righe `guadagno_cpu_*` — la
banda ALTA è una decisione SCREEN-grade; (b) il canale «Zval clone+drop
attribuibile a run_loop/binary_value_ab» include i drop FINALI (la free vera)
e il traffico Zval non-slot dei 185 bracci: un take evita solo la COPPIA
transiente inc/dec, non la distruzione finale, che avviene comunque una volta.
Quindi la banda è un tetto, e il margine sopra la soglia ALTA è 0,61pp.
Robustezza: anche con canale gonfiato 2×, safe cade in MEDIA (0,95%) e str
resta sopra l'abbandono — la decisione «si prosegue» regge; l'etichetta ALTA
no.

**Il corpo caldo.** «Non aggiunge opcode al percorso caldo, ne cambia uno
esistente» (design95 §finale) è FALSO per F3: `TakeSlot` è un braccio NUOVO
di `run_loop`, eseguito ~22,7M volte (perimetro safe). La lezione WP-39..44
impone il tetto, non lo slogan.

## Emendamenti (A-LB-97-n)

- **A-LB-97-1 (tetto corpi caldi)**: Δ netto bracci CALDI ≤ 0. `TakeSlot`
  sostituisce 1:1 una `LoadSlot`/`LoadVar` (op-census: totale dispatchato
  INVARIANTE, quota TakeSlot = quota sottratta ai due bracci). Corpo di
  `TakeSlot` ≤ corpo di `LoadSlot` (move + store Undef + guard Ref; niente
  altro). `nm -S run_loop`: taglia predetta PRIMA del commit, misurata dopo;
  se sfora la predizione, si outlinea (O1) PRIMA di rivendicare.
- **A-LB-97-2 (controllo positivo F4, specifica completa)**: tre contatori,
  non uno — `takes_executed`, `ref_fallbacks`, e l'identità
  `takes_executed + ref_fallbacks = would_take_safe` dinamico (tolleranza =
  rumore suite, ~decine su 22,7M, come in nota-determinismo). Il numero
  predetto si SCRIVE prima di F4. Dichiarare che conteggio (build census) e
  cronometro (build parità) vivono su BINARI DIVERSI, stessa suite.
- **A-LB-97-3 (falsificatore mancante in P3)**: guadagno F4 SOTTO la banda
  min del perimetro scelto (oltre lo spread A/A) = modello del canale
  falsificato; va NOMINATO e il canale ri-derivato (split transiente/finale),
  non assorbito in silenzio.
- **A-LB-97-4 (decisione perimetro)**: la scelta F2-intero vs nucleo _str si
  prende A INIZIO F3 col conto dei raw; se si sceglie F2-intero, i test
  trappola distruttori sono BLOCCANTI nello stesso commit.

## Kill-switch (KS-LB-97-n)

- **KS-LB-97-1**: identità A-LB-97-2 violata → nessuna lettura di tempo è
  valida; F4 si ferma lì.
- **KS-LB-97-2**: `ref_fallbacks` > 5% di `would_take_safe` → il perimetro
  statico perde; bande da ri-derivare prima di allargare l'emissione.
- **KS-LB-97-3**: qualunque divergenza d'ordine `__destruct` sui gate →
  ripiego IMMEDIATO al nucleo _str (banda MEDIA), non un fix inseguito.
- **KS-LB-97-4**: op-census non invariante o taglia run_loop oltre predizione
  senza outline compensativo → la leva non è provata, Δ tempo non rivendicabile.

## Refutazioni capitali

**Una**: la frase «non aggiunge opcode al percorso caldo» è refutata — F3
aggiunge un braccio caldo e deve pagare il tetto WP-39..44 (A-LB-97-1). La
consulenza Bak è stata usata correttamente su tetto dispatch e
contatore-prima-dell'orologio; NON sul conteggio dei corpi. I conteggi
F1/F2 e la decisione di proseguire NON sono refutati.

----------------------------------------------------------------------
## FILE: verbali/verbale-6-pedersen.md

# Verbale sedia 6 — Pedersen (WP-97)
Perimetro: confine per-richiesta/per-test, lifecycle, identità dei misurati.
Oggetto: S-95.0 (A-ZV2 F1+F2 sola misura) + §WP-96 (F3 TakeSlot).

## VERDETTO
**RESPINTO IN PARTE.** I conteggi F1/F2 reggono come misure (somme macchina
verificate: f1-out/f2-out `.sum` = trascrizioni `.out`, F1↔F2 identici al
contatore). REFUTATA la provenance di F2: il run parte alle 10:00:08 con
`f2.identity head=fb0599ba53ab`, ma `zvalcensus-f2.out` dichiara «HEAD
ee3f551» — commit creato alle **10:00:22, 14 secondi DOPO l'avvio del run**.
Il binario 2a321e3b è stato costruito da un albero NON committato; nulla lo
lega a ee3f551. L'identità in banda esiste ma **contraddice l'header
pubblicato** e non basta: manca il porcelain (stato dirty), manca il re-hash
post-run, manca ogni identità della suite misurata oltre ai conteggi.

La somma su 16 processi NON è un'identità ben definita: è una popolazione
EMERGENTE («chi ha ereditato la env, ha contato, ed è uscito via `exit()`»).
`atexit` non scatta su morte per segnale/`_exit`; un figlio morto sparisce
in silenzio dalla somma; la riga raw non porta pid; il summer perl SCARTA
in silenzio le righe malformate (append concorrente ⇒ rischio riga
spezzata). Nessun dente pretende N=16: il 16 è osservato, mai asserito.

«Determinismo pieno fra i due run»: la coppia identica è reale e
machine-verified, ma è **N=1** e convive col rumore di 42 eventi vs il
before della stessa mattina, la cui sorgente non è nominata. Osservazione
forte, generalizzazione indebita.

L'esito-suite «IDENTICO» è parità di CONTEGGI (762/1912/52), in violazione
della regola di progetto «gate per NOME, mai solo conteggio».

**Binding rule output-capture-before-request_end**: F3 non la viola
strutturalmente — `TakeSlot` ANTICIPA i drop (il valore muore all'operazione
consumante), mai li posticipa oltre `request_end()`; l'output di un
`__destruct` anticipato resta dentro la finestra di cattura. Il rischio
reale è fratello, non figlio: l'ordine dei distruttori è semantica per-test
osservabile, e il perimetro F2 intero (oggetti/array) lo espone; il nucleo
`_str` è a rischio zero per costruzione. Le 5 trappole elencate sono un
elenco finito contro una classe semantica: servono il caso per-richiesta e
un kill-switch, non solo test.

## Emendamenti
- **A-PP-97-1**: `.identity` DEVE includere `git status --porcelain`
  (vuoto o hash del diff) + re-hash del binario census a FINE run; gli
  header dei `.out` si trascrivono DALL'identity, mai a memoria. Correggere
  `zvalcensus-f2.out`: head reale fb0599b, contenuto ee3f551 non dimostrato.
- **A-PP-97-2**: riga raw con `pid=`/`ppid=`; il runner ASSERISCE il numero
  atteso di processi (16) — mismatch = FAIL; il summer conta e FALLISCE su
  righe scartate/malformate.
- **A-PP-97-3**: identità della suite per NOME (diff dei nomi/esiti tra
  run), non per conteggio — la regola vale anche per la suite di misura.
- **A-PP-97-4**: le trappole F3 includano un caso PER-RICHIESTA
  (php-server): `__destruct` che emette output a ridosso di
  `request_end()`, verificando la cattura invariata; più generatore sospeso
  attraverso il confine di richiesta.
- **A-PP-97-5**: declassare «determinismo pieno» a «coppia identica, N=1»;
  nominare la sorgente dei 42 eventi PRIMA di usare il determinismo come
  premessa di F4.

## Kill-switch
- **KS-PP-97-1**: in F3, QUALUNQUE divergenza di ordine dei distruttori su
  qualunque gate (corpus/refl/ORM/hk/battery61) ⇒ restrizione automatica al
  nucleo `_str` nella stessa sessione — nessun dibattito.
- **KS-PP-97-2**: in F4, `slot_reads_avoided` fuori dalla quantità predetta
  da `would_take_safe` (tolleranza nominata ex-ante) ⇒ il Δ tempo NON è
  attribuibile: nessuna rivendicazione.
- **KS-PP-97-3**: conteggio processi ≠ atteso in un run census/F4 ⇒ run
  INVALIDO, si ripete; mai sommare popolazioni diverse.

## Refutazioni capitali
**SÌ, una**: la provenance F2 — l'header «HEAD ee3f551» è falsificato
dall'identity in banda (fb0599b) e dalla cronologia dei commit; build da
albero non committato, irriproducibile come dichiarata.

----------------------------------------------------------------------
## FILE: verbali/verbale-7-leijen.md

# Verbale sedia 7 — Leijen (allocatore mimalloc v3, footprint fisico) — WP-97

Oggetto: S-95.0 (A-ZV2 F1+F2, sola misura) e §WP-96 (F3 TakeSlot, F4 coppia).

## VERDETTO

**NON REFUTATO nel merito; prosecuzione CONDIZIONATA agli emendamenti.**
Il censimento F1/F2 è pulito dal mio perimetro: conteggi esatti, deterministici,
binario di parità invariato, strumentazione confinata dietro feature. Ma la
sessione tratta TakeSlot come leva *puramente* CPU, e questo è falso dal lato
allocatore: la mossa NON elimina allocazioni, però **sposta il momento
dell'ultimo drop** (free anticipati al termine dell'operazione invece che alla
riscrittura dello slot) e **consegna valori con rc=1** a valle. Entrambi i
canali toccano footprint e pattern di riuso mimalloc, in entrambe le direzioni.
Con il regresso media footprint 3,381× APERTO e NON attribuito (WP-94), spedire
F3 senza una predizione footprint contaminerebbe l'attribuzione futura: il solo
cronometro NON basta.

## Emendamenti

- **A-DL-97-1 (predizione footprint per F4, obbligatoria).** F4 registra anche
  il peak fisico (`/usr/bin/time -l`) della stessa coppia media, con predizione
  ex-ante FIRMATA prima del run: Δfootprint atteso debolmente ≤0 (free
  anticipati + separazioni CoW evitate); una predizione nulla è comunque una
  predizione. Qualunque AUMENTO oltre lo spread A/A è falsificatore nominato e
  va ATTRIBUITO (TakeSlot vs regresso aperto), mai sommato al trend.
- **A-DL-97-2 (canale CoW non contato, da nominare ex-ante).** Un valore mosso
  arriva con rc=1: può evitare separazioni copy-on-write di array e abilitare
  il riuso in-place di PhpStr growable (WP-57) — canale che il censimento NON
  conta. Se il guadagno F4 supera la banda P3 (falsificatore "doppio della
  banda"), questa è la prima candidata: va scritta PRIMA, o il sovra-guadagno
  resta un effetto non capito. Nota a favore del nucleo `_str` nella decisione
  di perimetro F3: rischio distruttori zero E canale di riuso stringhe massimo.
- **A-DL-97-3 (modello di costo per-evento).** La banda 4,5–6,5% assume costo
  medio uniforme per lettura rc, ma `Zval::drop` (7,20%) include i drop
  DEALLOCANTI, che TakeSlot non elimina: li anticipa soltanto. Se il controllo
  positivo (`slot_reads_avoided`) centra la predizione ma il Δt manca la banda
  per difetto, il verdetto è «modello di costo sbagliato», non «leva fallita» —
  la reazione va decisa prima di misurare.
- **A-DL-97-4 (churn di purge).** Con `MIMALLOC_PURGE_DELAY=0` i free
  anticipati possono indurre purge/ricommit ravvicinati (madvise churn, page
  fault). F4 usa lo stesso env del riferimento; se il Δt PEGGIORA, si guarda il
  canale purge prima di incolpare l'opcode.

## Kill-switch

- **KS-DL-97-1**: MAI leggere un footprint da una build `zval-census`:
  l'analisi ritiene `movable`/`movable_safe` per funzione e alloca nel punto
  fisso (clone di bitset per arco per iterazione). Footprint SOLO dal binario
  di parità.
- **KS-DL-97-2**: la lettura footprint di F4 (R=1, gruppo media) è grado
  **SCREEN**, non verdict: serve da tripwire e da ancora di attribuzione, NON
  chiude il regresso 3,381×. Resta integro il vincolo WP-96: qualunque claim di
  picco e la leva arene per-file esigono α RI-DERIVATO su mimalloc v3 con
  PURGE_DELAY=0.

## Refutazioni capitali

**NESSUNA.** La coppia F4 proposta rivendica TEMPO, non picco: non viola il
vincolo R=1-è-SCREEN — lo violerebbe solo un claim footprint senza
ri-derivazione, che KS-DL-97-2 preclude. F1/F2 restano valide come misura del
meccanismo; gli emendamenti aggiungono i canali allocatore che il censimento,
per costruzione, non vede.

----------------------------------------------------------------------
## FILE: verbali/verbale-8-stogov.md

# Verbale sedia 8 — Stogov (engine Zend / fedeltà oracle) — WP-97

## VERDETTO

**F1+F2 in sola misura: metodologia corretta** (liveness prima del dispatch,
come da mia consulenza §2; direzione d'errore dichiarata; determinismo al
contatore). **MA il §WP-96 pone F3 come scelta libera fra due opzioni pari, e
non lo è: la banda ALTA (take su tutto il perimetro F2, oggetti/array inclusi)
è REFUTATA come infedele all'oracle.** Zend non consuma MAI un CV — solo
TMP/VAR — esattamente perché svuotare lo slot anticipa la morte del valore.
L'anticipo è osservabile anche SENZA `__destruct` dichiarato: riuso di
`spl_object_id`/`spl_object_hash`, scadenza anticipata di `WeakReference`/
`WeakMap`, side-effect di chiusura risorse (flock, tmpfile), `__destruct`
ereditati. Quindi anche la "via intermedia" per assenza di dtor dichiarato è
pre-refutata. **La scelta fedele: emissione su `movable_safe` (perimetro F2
intero) + guard di TIPO a runtime nel handler — si sposta SOLO `Str`
(Long/Null/Bool/Double sono già copy-free); Ref si de-referenzia; oggetti/
array/risorse restano sul percorso clone byte-identico.** Guadagno atteso =
banda MEDIA (0,84–1,21%), non ALTA: P3 va RI-derivata, non difesa. Il guard
runtime è comunque obbligatorio per il Ref (punto 1 del §WP-96): estenderlo al
tipo costa un confronto, non un branch nuovo nel percorso comune.

**Warning "Undefined variable"**: sì, preservabile, ma per contratto:
`TakeSlot` deve replicare ESATTAMENTE il ramo Undef di `LoadVar` — warning
PRIMA di produrre il valore, stesso testo, stessi modi di soppressione
(isset/coalesce), slot lasciato Undef, Null prodotto. Il warning rientra nella
VM via `set_error_handler` (mia consulenza §5): serve un phpt apposito.

## Emendamenti

- **A-DS-97-1**: F3 = TakeSlot emesso su `movable_safe`, handler con gate di
  tipo: move solo `Str`; Ref→deref+clone; ogni altro tipo→clone identico a
  oggi. Nessuna eccezione "oggetto senza dtor".
- **A-DS-97-2**: P3 per F4 ri-derivata ex-ante alla banda str
  (`guadagno_cpu_atteso_str_pct_min/_max`); un risultato in banda ALTA è
  "effetto non capito" (design95 §P3) e va nominato, non rivendicato.
- **A-DS-97-3**: contratto Undef di TakeSlot (sopra) + phpt: variabile
  indefinita in sito movibile con `set_error_handler` che rientra.
- **A-DS-97-4**: `renounce()` copre solo gli osservatori chiamati DENTRO la
  funzione; il canale re-entrante (warning→error handler→`debug_backtrace`
  con args, `getTrace()`) può osservare i PARAMETRI. O si escludono i param
  slot dal take, o si prova che phpr materializza gli args del backtrace
  indipendentemente dagli slot.
- **A-DS-97-5**: guardia di esaustività su `effect()`: il `_ => {}` tratta
  come fall-through ogni op non modellato — innocuo in misura,
  correttezza-critico in F3 se un op porta un indirizzo di salto non
  matchato. Test che enumera ogni variante di `Op` con target. Audit di
  `CallBuiltinRefCell` (in `renounce` solo per nome, mai marcato per slot).
- **A-DS-97-6** (backlog per NOME, NON F3): estensione array-di-scalari solo
  con bit "no-destructible" mantenuto su `PhpArray`, misura e gate propri.

## Kill-switch

- **KS-DS-97-1**: qualunque divergenza dtor/weakref/object-id nei gate →
  la classe di siti torna al clone; mai "fix forward" che allarga.
- **KS-DS-97-2**: guadagno F4 oltre il doppio della banda str → stop e
  nominare l'effetto prima di rivendicare.
- **KS-DS-97-3**: guardia A-DS-97-5 trova un op non matchato → si ricontano
  F1/F2 PRIMA di emettere.

## Refutazioni capitali

1. **"Perimetro F2 intero = banda ALTA" come opzione ammissibile per F3**:
   infedele (CV mai consumati in Zend; morte anticipata osservabile).
2. **"Oggetti senza `__destruct` sono sicuri"**: falso — spl_object_id,
   WeakReference, risorse.

Consulenza S-95.0: usata correttamente (§2 ordine, §3/§4 a backlog); il §5
(re-entrancy dei diagnostici) NON è ancora entrato nei predicati → A-DS-97-4.

----------------------------------------------------------------------
## FILE: verbali/verbale-9-gregg.md

# Verbale sedia 9 — Gregg (metodologia di misura e attribuzione) — WP-97

## VERDETTO

**APPROVATO CON DECLASSAMENTO DI GRADO.** La decisione di prosecuzione
(strada lunga F2→F3→F4) regge; la banda P3 NO, non al grado dichiarato.

1. **Il canale 4,5–6,5% NON regge come moltiplicatore VERDICT.** Deriva da
   prof95-media.out, che si auto-dichiara SCREEN R=1 («serve a ORDINARE le
   leve, non a pinnare cifre»). L'aritmetica è consistente (7,20×0,542 +
   2,85×0,581 ≈ 5,56%, dentro la banda), ma soffre di tre debolezze non
   quantificate: (a) attribuzione a livello di simbolo sotto inlining
   release — i clone/drop INLINED spariscono dai simboli `Zval::clone/drop`
   e finiscono direttamente nel chiamante, quindi la quota è distorta in
   direzione ignota; (b) R=1: nessuno spread del canale stesso; (c) le
   percentuali dei chiamanti vengono da stack invertiti campionati, non da
   conteggi. **La banda P3 va dichiarata SCREEN finché F4 non la misura.**
2. **Grado del prodotto = il fattore più debole.** frazione (VERDICT,
   conteggi esatti, determinismo riprodotto) × canale (SCREEN) = **SCREEN**.
   zvalcensus-f1.out e f2.out marchiano `grade=VERDICT` in testa a file che
   contengono righe `guadagno_cpu_atteso_*` derivate dal canale SCREEN:
   contaminazione di grado nel raw stesso.
3. **Sensibilità che salva la decisione**: canale sovrastimato 2× →
   floor safe 1,91%→0,95% = banda MEDIA, che comunque preferisce la strada
   lunga a parità di conto. La PROSECUZIONE è robusta all'errore del
   canale; la PREDIZIONE no. Distinguere le due cose per iscritto.
4. **Asimmetria del modello di costo**: la predizione conta il lavoro
   RISPARMIATO (coppie inc/dec) ma non il lavoro AGGIUNTO da F3 (guard di
   tipo a runtime, store di `Undef`). Un Δ sotto banda con
   `slot_reads_avoided` al valore predetto = il prezzo del guard, e va
   nominato, non trattato come rumore.

## Emendamenti

- **A-BG-97-1**: annotare nei due raw (o in un companion) che le righe
  `guadagno_cpu_atteso_*` sono grado SCREEN; il `grade=VERDICT` di testa
  copre SOLO i conteggi.
- **A-BG-97-2**: prima di F4, sanity-check ns/evento (regola WP-53/54 «i
  conteggi non sono secondi»): 22,67M coppie rc safe × costo plausibile
  per coppia confrontato con la user CPU del media group — se il canale
  implicito esce fuori da 4,5–6,5%, ri-derivare la banda.
- **A-BG-97-3**: F4 dichiari R e il tetto di spread A/A PRIMA del run; il
  confronto è «Δ vs spread», con esito UNDECIDED se lo spread copre il
  floor della banda.
- **A-BG-97-4**: specificare che `slot_reads_avoided` si legge da un run
  SEPARATO con feature census sullo stesso HEAD: il controllo positivo
  non condivide mai il binario col cronometro.
- **A-BG-97-5**: se F3 sceglie il nucleo stringhe, la P3 si RI-deriva dai
  numeri `_str` (0,84–1,21%, banda MEDIA) prima dell'opcode — non si
  eredita la banda safe.

## Kill-switch

- **KS-BG-97-1**: spread A/A ≥ 1,9% (floor della banda safe) → F4 non può
  emettere verdetto; fermarsi e alzare R o ridurre il rumore.
- **KS-BG-97-2**: `slot_reads_avoided` devia oltre una tolleranza
  dichiarata ex-ante dai 22.674.665 di F2 → qualsiasi Δ è NON attribuito.
- **KS-BG-97-3**: Δ oltre 2× la banda (falsificatore P3 esistente) O sotto
  il floor con controllo positivo verde → profilo di coppia obbligatorio
  prima di qualunque rivendicazione.

## Refutazioni capitali

**Sì, una (di grado, non di merito)**: la banda ALTA è presentata a valle
di file `grade=VERDICT` ma è un prodotto SCREEN — refuto l'etichetta, non
la prosecuzione, che sopravvive perfino a un errore 2× del canale.

OGGETTO: sì, la sessione ha avanzato l'oggetto — oggi sappiamo, con conteggi esatti e deterministici, che il 47,11% delle letture rc di phpr è un ultimo uso, che la prudenza ne lascia il 90,21%, e che i Ref a runtime sfuggono all'analisi statica: fatti nuovi, falsificabili da F4.

----------------------------------------------------------------------
## FILE: verbali/team-engine.md

# Team engine (Hoare · Matsakis · Stogov) — nota di riconciliazione WP-97

Verdetti: 3× CON EMENDAMENTI. F1+F2 in sola misura reggono per tutte le sedie; nessuna benedice il §WP-96 così com'è.

## Convergenze

1. **Arco eccezionale non sound**: la def sottratta anche sul contributo exc rende `movable_safe` una base di emissione invalida — A-TH-97-1 (con controesempio capitale e KS-TH-97-2/3); Stogov KS-DS-97-3 impone il riconteggio F1/F2 prima di emettere.
2. **Esaustività di `effect()`/`renounce()`**: il `_ => {}` è un buco a futura memoria; audit di `CallBuiltinRefCell` chiesto da tutte e tre — A-TH-97-2, A-DS-97-5, A-MS-97-5(a); più il gap `debug_zval_refcount` (A-MS-97-5b) e il canale re-entrante backtrace/args (A-DS-97-4).
3. **Guard runtime a WHITELIST, prima spedizione solo `Str`**: A-MS-97-2 e A-DS-97-1 coincidono (move solo Str; Ref→deref+clone; resto→clone identico). Refutazioni convergenti: banda ALTA infedele (Stogov cap. 1-2: CV mai consumati in Zend; "oggetto senza dtor" non è sicuro — spl_object_id, WeakReference, risorse; idem Hoare A-TH-97-3).
4. **P3 va RI-derivata, non difesa**: il numeratore safe è un maggiorante (refutazione capitale Matsakis) — A-MS-97-1 (`WOULD_TAKE_SAFE_REF`, banda su safe−ref), A-DS-97-2 (banda str), KS-TH-97-3.
5. **Destino del valore preso**: ogni consumatore di un valore mosso deve passare da `gc_note` o i cicli perdono radici — A-TH-97-4 ≡ A-MS-97-3.
6. **Emissione compile-time, mai cache address-keyed**: A-TH-97-5 ≡ A-MS-97-4 (riuso correlato con mimalloc; invariante «Func mai mosso, ops mai riscritto»).
7. **Test-trappola per NOME**: controesempio Hoare (try/catch su builtin-out), KS-MS-97-1 (`use(&$x)`, ordine dtor, `compact`), A-DS-97-3 (phpt Undef con `set_error_handler` rientrante).

## Conflitti

- **Destino del perimetro F2 intero**: Matsakis (A-MS-97-2) lo ammette DOPO, con corpus distruttori-ordine dedicato; Stogov lo pre-refuta per sempre come F3 (array-di-scalari = leva separata per NOME, A-DS-97-6); Hoare non decide ma esige il confronto banda MEDIA vs piano B per §P1 (A-TH-97-3), non l'adozione silenziosa.
- **Sufficienza del guard**: per Stogov il gate di tipo nel handler È la scelta fedele; per Hoare il guard è «necessario ma non sufficiente» (WeakReference, flush risorse, timing GC restano da coprire lato analisi).

## Priorità proposte per WP-96

1. Fix A-TH-97-1 + riconteggio F1/F2 (verifica KS-TH-97-3).
2. Match esaustivi + audit `CallBuiltinRefCell` + guardia op-target (A-TH-97-2/A-DS-97-5/A-MS-97-5).
3. Rimisura `WOULD_TAKE_SAFE_REF` e ri-derivazione P3 su banda str/safe−ref (A-MS-97-1/A-DS-97-2).
4. TakeSlot compile-time su movable_safe con whitelist Str + contratto Undef + disciplina gc_note (A-DS-97-1/A-MS-97-2/A-DS-97-3/A-TH-97-4/A-MS-97-3/A-TH-97-5).
5. Test-trappola per NOME (punto 7 sopra) + chiusura gap renounce (A-DS-97-4/A-MS-97-5b).
6. Confronto banda MEDIA vs piano B per §P1 prima di rivendicare (A-TH-97-3).

----------------------------------------------------------------------
## FILE: verbali/team-misura.md

# Team-misura (Bak · Leijen · Gregg) — nota di riconciliazione WP-97

## Convergenze

1. **Nessuna refutazione di merito: F1/F2 e la prosecuzione reggono.** Conteggi esatti, deterministici, negativo che morde (Bak, Leijen); la decisione sopravvive perfino a un canale sovrastimato 2× (Gregg §3, Bak "robustezza": safe cade in MEDIA 0,95% ma la strada lunga resta preferita).
2. **La banda P3 è SCREEN, non VERDICT.** Il canale viene da prof95-media.out (R=1, auto-dichiarato SCREEN); grado del prodotto = fattore più debole (Gregg §1-2 + refutazione di grado; Bak: banda ALTA è decisione SCREEN-grade; Leijen KS-DL-97-2 sul lato footprint). Contaminazione nei raw: `grade=VERDICT` copre SOLO i conteggi → A-BG-97-1.
3. **Il canale è un tetto: i drop DEALLOCANTI non sono eliminati, solo anticipati.** TakeSlot evita la coppia transiente inc/dec, non la free finale (Bak §bande; Leijen A-DL-97-3; Gregg §4 asimmetria guard/store Undef). Δ sotto banda con controllo positivo verde = «modello di costo sbagliato», da nominare ex-ante (A-DL-97-3, A-LB-97-3, KS-BG-97-3).
4. **Protocollo F4 comune**: controllo positivo su binario census SEPARATO dal cronometro, stesso HEAD (A-LB-97-2 ≡ A-BG-97-4); predizione SCRITTA prima del run; R e tetto spread A/A dichiarati prima, esito UNDECIDED se lo spread copre il floor (A-BG-97-3, KS-BG-97-1); identità `takes+fallbacks=would_take` (A-LB-97-2, KS-LB-97-1/2, KS-BG-97-2); ns/evento prima di F4 (A-BG-97-2, regola WP-53/54).
5. **Nucleo `_str` favorito nella decisione di perimetro F3**: rischio distruttori zero e canale riuso stringhe massimo (Leijen A-DL-97-2; Bak A-LB-97-4/KS-LB-97-3; Gregg A-BG-97-5: banda RI-derivata 0,84–1,21%, non ereditata).

## Conflitti

- **Enfasi CPU vs footprint**: Leijen esige che F4 misuri ANCHE il peak fisico con predizione firmata (A-DL-97-1, canale CoW A-DL-97-2, churn purge A-DL-97-4); Bak e Gregg trattano F4 come misura di tempo. Non contraddittorio ma additivo: costo di protocollo extra da accettare o motivatamente rifiutare.
- **Corpo caldo nuovo**: solo Bak refuta la frase di design95 e impone il tetto Δ bracci caldi ≤ 0 con predizione `nm -S` (A-LB-97-1, KS-LB-97-4); Leijen/Gregg non lo toccano.
- **Reazione al falsificatore alto**: Gregg → profilo di coppia obbligatorio (KS-BG-97-3); Leijen → prima candidata il canale CoW non contato (A-DL-97-2). Ordine di attribuzione da fissare.

## Priorità proposte per WP-96

1. Declassare la banda P3 a SCREEN nei raw/companion (A-BG-97-1) prima di ogni uso.
2. Decisione perimetro F3 a inizio sessione col conto dei raw; se `_str`, ri-derivare la banda (A-LB-97-4 + A-BG-97-5).
3. F3 sotto tetto corpi caldi + op-census invariante (A-LB-97-1).
4. F4: protocollo completo — identità contatori, binari separati, spread A/A ex-ante, ns/evento, predizione footprint firmata (A-LB-97-2, A-BG-97-2/3/4, A-DL-97-1).
5. Falsificatori nominati prima di misurare: sotto-banda e sopra-2×banda con reazioni pre-decise (A-LB-97-3, A-DL-97-2/3, KS-BG-97-3).

----------------------------------------------------------------------
## FILE: verbali/team-catena.md

# Team-catena (Klabnik · Hejlsberg · Pedersen) — nota di riconciliazione WP-97

## Convergenze
1. **Provenienza come precondizione dei gate**: KS-SK-97-1 (canale env-git A-SK-93..97 / denti T27-T30 chiuso PRIMA di spedire F3, altrimenti gate provisional per NOME) e A-PP-97-1 (identity con `git status --porcelain` + re-hash post-run; header dei `.out` trascritti dall'identity, mai a memoria) sono la stessa tesi su due livelli: nessun PASS di parità è verdict-grade senza catena env+albero pulita.
2. **Smoke → test committati**: A-SK-97-3 (fixture con conteggi `would_take*` esatti + negativi back-edge/`global`/`use(&$x)`/`compact`) e A-PP-97-4 (caso per-richiesta: `__destruct` a ridosso di `request_end()`, generatore sospeso oltre il confine) — un controllo non ripetibile a macchina non è un controllo.
3. **Invarianti presidiate a compile-time**: A-SK-97-2 (match esaustivo di `effect()`, via il `_ => {}`) e A-AH-97-4 (static assert `size_of::<Op>() == 48` nello stesso commit dell'opcode) condividono il principio: la variante futura deve rompere la build, non decadere in silenzio.
4. **F3 = analisi di compilazione, identità strutturale**: A-AH-97-1/3 e RC-2 (mai cache keyed-by-pointer: ABA = take sbagliato; KS-AH-97-3 reject) con KS-SK-97-2 (TakeSlot guarda il TIPO: `Zval::Ref` si de-referenzia) e KS-PP-97-1 (divergenza ordine distruttori ⇒ restrizione a `_str`).
5. **Banda F4 non usabile com'è**: RC-1/A-AH-97-5 (banda LORDA, serve A/A per il costo dispatch), A-SK-97-4 (grade per campo: conteggi VERDICT, guadagni DERIVED-ESTIMATE; margine 2,8%), A-PP-97-5 (42 eventi da nominare prima di F4).
6. **Suite per NOME**: A-PP-97-3 riafferma la regola di progetto; coerente col gate-per-nome di Klabnik.

## Conflitti
- **Validità di F2 oggi**: Klabnik PASS con riserve (aritmetica e banda ALTA robuste, identity committati); Pedersen RESPINTO IN PARTE — provenance F2 REFUTATA (header «HEAD ee3f551» contraddetto da fb0599b, build da albero non committato, irriproducibile). Hejlsberg non contesta la misura ma refuta F3 come scritto.
- **Determinismo**: Klabnik lo tratta come robusto con ignoto nominato; Pedersen lo declassa a «coppia identica, N=1» (A-PP-97-5).
- **Riparazione F2**: Klabnik → riconteggio dopo A-SK-97-1 (buco `NewAnonDeferred`); Pedersen → riparazione della provenance/header. Complementari ma con oggetto diverso.

## Priorità proposte per WP-96
1. Provenance: A-PP-97-1 (correggere `zvalcensus-f2.out`) + riconteggio F2 post A-SK-97-1, PRIMA di F3.
2. A-SK-97-2 + A-SK-97-3/A-PP-97-4 (match esaustivo; smoke promossi a test, incluso caso per-richiesta).
3. F3 solo come da A-AH-97-1/2/3, con KS-AH-97-3, KS-SK-97-2, KS-PP-97-1 armati.
4. KS-SK-97-1: chiudere A-SK-93..97 o dichiarare i gate F3 provisional per NOME.
5. Prima di F4: A-AH-97-5 (A/A), A-SK-97-4, A-PP-97-5; robustezza summer A-PP-97-2 e suite per nome A-PP-97-3.

----------------------------------------------------------------------
## FILE: verbali/SYNTHESIS.md

# SINTESI DI CONVERGENZA — Concilio WP-97 (su report S-95.0 + programma WP-96)

## §FONDAMENTALI (in testa per regola utente 2026-08-03)

(a) **Avanzamento dell'OGGETTO in S-95.0**: fatti NUOVI e falsificabili sul
meccanismo di phpr — la frazione di letture di slot che sono ultimi usi, il
costo del perimetro conservativo, la scoperta che uno slot può reggere un
`Ref` a runtime invisibile alla rinuncia statica (cifre nei raw
`wp95-harness/zvalcensus-f1.out`/`-f2.out`, contatori esatti). Gregg
(mandato inverso): «OGGETTO avanzato». Nessun cronometro in sessione: il
cronometro è F4.
(b) **Contatore sessioni-senza-misura**: ultima misura full/media = WP-94
(1 sessione fa) · ultima campagna sull'oggetto footprint = m90 (5 sessioni
fa). Il metro è fresco; la rotta corrente è CPU-VM.
(c) **Rischio d'oggetto più trascurato**: costruire F3 su un moltiplicatore
NON rimisurato — il valore del canale in §P1 viene da un profilo R=1: ogni
banda derivata è SCREEN finché F4 non la misura. Secondo rischio: l'apparato
env-git (A-SK-93..97) ora BLOCCA l'oggetto (KS-SK-97-1: senza ambiente
COSTRUITO i PASS di parità di F3 non sono verdict-grade) — entra nell'ordine
di WP-96 per la regola di ammissione, in timebox.

## Verdetti (9 sedie, NESSUNA benedizione)

Hoare CON EMENDAMENTI · Matsakis CON EMENDAMENTI · Klabnik PASS CON RISERVE
VINCOLANTI · Hejlsberg: F3 come scritto NON eseguibile, S-95.0 regge · Bak
PROCEDI CON EMENDAMENTI · Pedersen RESPINTO IN PARTE · Leijen NON REFUTATO,
prosecuzione CONDIZIONATA · Stogov REFUTA la scelta «banda ALTA» · Gregg
APPROVATO CON DECLASSAMENTO.

## Refutazioni capitali (riprodotte nei verbali, fonte vincolante)

1. **Hoare A-TH-97-1**: `movable_safe` è INSOUND per emissione — la def è
   sottratta anche sul contributo dell'arco eccezionale (un catch potrebbe
   vedere `Undef`). Bande F1/F2 dichiarate salve; F3 BLOCCATA finché il
   transfer non è corretto e i conteggi rifatti (KS-TH-97-2/3).
2. **Stogov**: in Zend i CV non si consumano MAI; la morte anticipata è
   osservabile anche senza `__destruct` (spl_object_id, WeakReference,
   risorse). L'F3 fedele = move SOLO Str ⇒ banda MEDIA, P3 ri-derivata.
3. **Bak**: «non aggiunge opcode al percorso caldo» è FALSO — `TakeSlot` è
   un braccio nuovo e paga il tetto WP-39..44; banda ALTA = SCREEN-grade.
4. **Gregg (grado)**: righe `guadagno_*` marchiate VERDICT ma il canale è
   SCREEN — il prodotto eredita il fattore più debole. [APPLICATO IN
   CHIUSURA: `grade-per-campo` nei due raw.]
5. **Pedersen (provenienza)**: header del raw F2 dichiarava un HEAD nato
   DOPO l'avvio del run (build da albero non committato). [APPLICATO IN
   CHIUSURA: header trascritto dall'identity, corrispondenza dichiarata-non-
   provata; «determinismo pieno» declassato a riproduzione N=1.]
6. **Hejlsberg**: F3 col riuso dell'analisi lazy/pointer-key sarebbe
   corruzione semantica — l'analisi va nel COMPILATORE, identità
   strutturale, mai puntatori; banda P3 lorda del corpo caldo nuovo.

## Ordine WP-96 emendato (recepito in NEXT_SESSION §WP-96)

1. Apparato A-SK-93..97 in timebox (ora precondizione del grado di parità).
2. Fix soundness (A-TH-97-1) + match esaustivi senza wildcard (A-TH-97-2 ≡
   A-SK-97-2) + varianti mancanti (A-SK-97-1 NewAnonDeferred, A-DS-97-5 /
   A-MS-97-5 CallBuiltinRefCell + debug_zval_refcount) + contatore
   WOULD_TAKE_SAFE_REF (A-MS-97-1) → RICONTEGGIO F1/F2; se P2 scende sotto
   la soglia, stop e confronto piano B (KS-TH-97-3).
3. Perimetro: whitelist Str-first (A-MS-97-2 ≡ A-DS-97-1); P3 RI-DERIVATA
   sul nucleo; banda attesa MEDIA ⇒ confronto ESPLICITO col piano B
   (A-TH-97-3) al NETTO del corpo caldo (tetto A-LB-97-1, nm -S predetto).
4. Se la strada lunga vince il confronto: emissione SOLO compile-time
   (A-TH-97-5 ≡ A-AH-97-1, identità strutturale A-AH-97-3, assert taglia Op
   A-AH-97-4), contratto Undef+warning (A-DS-97-2), gc_note sul valore
   preso (A-TH-97-4 ≡ A-MS-97-3), trappole come test COMMITTATI (A-SK-97-3
   ≡ A-PP-97-4), gate completi nello stesso commit.
5. F4: census su binario separato, controllo positivo a tre contatori
   (A-LB-97-2), coppia A/A con tetto spread ex-ante (A-BG-97-3), sanity
   ns/evento (A-BG-97-2), predizione footprint firmata (A-DL-97-1; peak
   R=1 resta SCREEN, KS-DL-97-2), suite per NOME (A-PP-97-3).

Conflitti registrati (non appianati): ammissibilità futura del perimetro
F2-intero (Matsakis sì-dopo, Stogov pre-refutato, Hoare via confronto §P1);
sufficienza del guard di tipo (Stogov sì, Hoare no); obbligo del peak in F4
(Leijen sì, altri cronometro-first). Fonte vincolante: i verbali individuali.

