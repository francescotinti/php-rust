# COUNCIL_WP102_REVIEWS — Concilio WP-102 su S-100 e programma S-101 (2026-08-05/06)

Assemblato con cat (token-lean): 9 verbali di fase 1 + 3 note di team + sintesi.

## FILE: verbali/verbale-1-hoare.md

# Verbale Sedia 1 — Tony Hoare (design linguaggio/runtime, safe-only) — Concilio WP-102

**VERDETTO: CONCORDO CON EMENDAMENTI** (una refutazione capitale sul contratto di modo; il flip in sé regge sui suoi gate).

## Refutazioni

**R1 (CAPITALE) — «il modo è un INPUT del funnel» è FALSO in un punto del compilatore.**
`compile/mod.rs:780`, `emit_binary`: `if op == BinOp::Add && !reg_lower::enabled()` consulta il globale di processo, non `ctx.reg_lower`. Conseguenza post-flip: il braccio OFF della batteria (`compile_program_with_mode(false)` in un processo sigillato ON, il default nuovo) emette `Binary(Add)` dove la produzione OFF (`PHPR_REG_LOWER=0`) emette `BinaryAdd`. Il braccio OFF collaudato in-process NON è l'emissione OFF di produzione: due fonti di verità, esattamente la classe che S-100 dichiara eliminata («`lowered()` eliminato, niente premesse ambientali»). La copertura semantica del differenziale non sana la falsità della dichiarazione. Nota aggravante: il doc-comment di `emit_binary` («its windows match `Op::Binary(Add)` and must keep fusing it») è STALE — `bin_op_of` vede attraverso `BinaryAdd` dichiaratamente («the windows fuse both spellings»), quindi la giustificazione del sito non esiste più.

**R2 — A-HO-101-1 (sigillo di tipo) declassato a backlog DOPO un flip che colpisce tutti.** `seal_reg_lower_mode()` è una convenzione: nessun tipo obbliga un main (o un terzo binario futuro) a chiamarla. Prima del flip il processo smemorato cadeva su OFF-collaudato; oggi cade su ON con modo deciso dalla prima compile — lo stato di richiesta (putenv) promosso a configurazione di motore è di nuovo raggiungibile per omissione. Un dente di batteria non compila al posto del chiamante mancante.

**R3 — H-C1 «prestito» sottospecificato sul lato aliasing.** Il profilo stesso dice che l'oracle fa «borrow inline NEGLI HANDLER»: il prestito Zend non sopravvive mai all'handler. Un «prestito» che risiede sullo stack VM attraverso più op è un design DIVERSO e in safe-Rust esige refcount+COW, non un borrow: `$o->x` letto, poi la proprietà riassegnata/unset/`__set`-tata PRIMA del consumo nello stesso statement invalida lo storage. La lista fixture della bozza (hook, `__get`, ref, readonly, visibilità) NON contiene la classe aliasing (scrittura tra lettura e consumo; mutazione via alias visibile attraverso la condivisione).

**R4 — «equivalenza già provata» per BinaryAdd è per TEST, non per costruzione.** La riscrittura post-finestre `Binary(Add)→BinaryAdd` in `lower_func` copre l'intero stream (anche regioni exc); la sua correttezza pende dal differenziale A-HE-100-3, che è una batteria enumerata. Se i due handler VM non condividono lo stesso corpo, ogni edit futuro a UNO dei due riapre l'equivalenza in silenzio.

## Emendamenti

- **A-HO-102-1**: `emit_binary` prende il modo da `ctx.reg_lower` (o, meglio: emette `BinaryAdd` INCONDIZIONATAMENTE — le finestre fondono entrambe le grafie per dichiarazione propria — e l'`enabled()` residuo nel compile-side scompare del tutto, tripwire invariato). Doc-comment sanato nello stesso commit. Dente: il braccio OFF del funnel dump-hash-uguale alla produzione OFF (`env =0`, processo separato).
- **A-HO-102-2**: A-HO-101-1 promosso da backlog a S-101: testimone ZST (classe VmGate) reso da `seal_reg_lower_mode()` e preteso dal confine di compilazione di produzione — l'omissione del sigillo NON COMPILA.
- **A-HO-102-3**: l'iscrizione di H-C1 dichiari PRIMA quale dei due design è (a) refcount+COW con censimento dei siti di scrittura come prerequisito, oppure (b) fusione PropGet+consumatore in un handler dove il prestito non esce dall'handler (la forma dell'oracle). Fixture aliasing obbligatorie (scrittura/unset/riassegnazione tra lettura e consumo) accanto a quelle già in bozza.
- **A-HO-102-4**: esibire (o creare) il corpo condiviso fra i handler `Binary(Add)` e `BinaryAdd`; il differenziale degrada a cintura di regressione, non a prova.

## Kill-switch

- **KS-HO-102-1**: nessun nuovo lavoro d'emissione (H-C1 compresa) si iscrive finché l'`enabled()` di `emit_binary` non è rimosso o derivato da `ctx`: un funnel col braccio OFF ibrido non può fare da giudice a un cambio d'emissione.
- **KS-HO-102-2**: se H-C1 sceglie il design (a), nessuna riga senza il censimento COW dei siti di scrittura proprietà pubblicato per NOME; se sceglie (b), nessun prestito può attraversare un confine di op sullo stack VM — pena rigetto.

════════════════════════════════════════════
## FILE: verbali/verbale-2-matsakis.md

# Verbale Sedia 2 — Niko Matsakis (ownership/aliasing/borrow) — Concilio WP-102

## VERDETTO

S-100 sui miei temi: **nessuna refutazione capitale**. La bozza §S-101 ha
**una refutazione capitale** (H-C1 così come nominata punta al canale
sbagliato sul micro che la motiva) più una clausola refutata al punto 3.

## Refutazioni

**R1 (CAPITALE) — H-C1 «clone del valore letto» è mal mirata sul suo stesso
micro.** `Zval` è `#[derive(Clone)]` con varianti heap `Rc<…>`
(`php-types/src/zval.rs:14`): il "clone" è un bump di refcount, mai una
copia profonda. Su prop.php le proprietà sono `Long` (Copy: clone
gratuito). Il churn del ciclo di vita Zval viene dal **RICEVITORE**: ogni
`PropGet` fa `obj.deref_clone()` (`run.rs:3394` — bump di `Rc<Object>`),
poi Pop/drop, e ogni drop di `Zval::Object` passa da `gc_note`
(`vm/mod.rs:3898`) che fa un `rc.borrow()` per nota. Il ~27% aggregato
(drop 12,6 + clone 7,6 + gc_note 5,3 + deref_clone 2,1) NON attribuisce
tra ricevitore e valore. Prior art in-tree: `ThisPropGet` (`run.rs:3418`)
già PRESTA il ricevitore sull'IC-hit senza clone. Prima riga di H-C1:
census che decompone i quattro simboli per SITO (ricevitore vs valore vs
Sweep/Pop di traffico), altrimenti si ottimizza il canale minore.

**R2 (clausola §S-101 punto 3) — «l'emissione non cambia ⇒ batteria+corpus
bastano» è FALSA per H-C1.** Un cambio di aliasing/lifetime a runtime è
PIÙ pericoloso di un cambio d'emissione: sposta il momento in cui
`strong_count` tocca 1, cioè l'ORDINE dei `__destruct` e la finestra del
cycle-collector («under-noting delays a destructor», vm/mod.rs:3892). Il
collaudo WP di parità è dovuto anche per H-C1.

**R3 — il profilo come tariffa.** ~27% è la quota dell'INTERO micro, non
il recuperabile: Sweep/Pop droppano anche temporanei che nessun prestito
elimina. L'atteso di H-C1 si calcola dal canale contato (n° bump+note
eliminati/iter × costo per evento misurato), mai da 27%×T.

**R4 — estensione BinaryAdd (reg_lower.rs:291-295): sound, con due bordi.**
La riscrittura è in-place 1:1 (nessun addr/exc_table da rimappare — bene),
`visit_addrs`/`bin_op_of` la classificano esaustivamente, il fallback del
handler rientra nel funnel generico. Ma: (a) il tripwire «flag-on zero
`Binary(Add)`» vive nei test del funnel — i corpi FUORI funnel
(prop_init, const-thunk) legittimamente lo conservano: il tripwire deve
dichiarare il suo perimetro per NOME o decade in silenzio; (b) la
riscrittura tocca anche regioni blocked/park: l'equivalenza lì è provata
solo dal differenziale, che va esteso se mai BinaryAdd divergesse dal
generico su un solo diagnostico.

## Emendamenti

- **A-MA-102-1**: prima di iscrivere H-C1, census per SITO dei quattro
  simboli Zval-lifecycle (ricevitore vs valore vs traffico); l'ipotesi si
  nomina sul canale che il census indica (probabile: prestito del
  RICEVITORE su PropGet, à la ThisPropGet).
- **A-MA-102-2**: lista scritta PRIMA degli hazard di aliasing con una
  fixture ciascuno: `__get`, hook get/set, `&$o->x` (Ref nel slot),
  riassegnazione intra-espressione (`$o->x + ($o->x=…)`), `unset` durante
  la lettura, lazy ghost/proxy, readonly/typed-uninit (`Undef` fatale),
  ordine `__destruct`, ciclo GC con proprietà nel ciclo, e gli op che
  scrivono lo stack IN PLACE (`BinaryAdd` fa `*last_mut()=v`: mai un
  alias di un slot proprietà sullo stack).
- **A-MA-102-3**: nessun borrow-guard `RefCell` può attraversare un
  confine di op o un rientro VM: il sigillo sia di TIPO (scope del borrow
  chiuso prima del push), non una review.
- **A-MA-102-4**: perimetro del tripwire zero-`Binary(Add)` dichiarato
  per NOME (funnel sì, prop_init/thunk no) e asserito via `all_funcs`.

## Kill-switch

- **KS-MA-102-1**: H-C1 non si scrive finché census A-MA-102-1 e fixture
  A-MA-102-2 non sono in-tree e verdi su ENTRAMBI i motori.
- **KS-MA-102-2**: atteso dal canale contato; se < pavimento sonda ⇒
  rinuncia pre-registrata.
- **KS-MA-102-3**: qualunque cambio d'ordine dei distruttori o del
  free-order (corpus per NOME, gate GC) ⇒ reject, senza appello.
- **KS-MA-102-4**: se H-C1 si scrive, coppia WP di parità OBBLIGATORIA
  anche senza cambi d'emissione (R2).

════════════════════════════════════════════
## FILE: verbali/verbale-3-klabnik.md

# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — Concilio WP-102

## VERDETTO

S-100 regge nella sostanza (flip eseguito con gate reali, contratto value-parsed,
denti derivati dal contratto). Ma UNA refutazione è capitale: la motivazione
dichiarata del gate carry-over è FALSA a livello macchina. Tre refutazioni
minori su carve-out, matrice denti e bozza S-101.

## Refutazioni

**R1 (CAPITALE) — «il flip cambia solo la costante» è falso.** Il diff
per-test (A-KL-101-4, evidence 20:41) è stato eseguito sul binario candidato
a2772e62 (albero 9d0d001), non sul pin. La motivazione dichiarata cade sul
`git show fb861e4 --stat`: il flip tocca `compile/func.rs` (compile_body legge
`ctx.reg_lower`), `compile/mod.rs` (+29: `compile_program_with_mode`, entry) e
150 righe di `reg_lower.rs` — è un ricablaggio dell'entry di compilazione, non
una costante. Mitigante reale: sul pin girano corpus 1418 per NOME ×2 modi +
batteria 1735/0 + server bimodale. Buco residuo: il diff BYTE dei chunk FAIL
non è mai stato ri-giudicato sul pin — classe carry-over (WP-94).

**R2 — la carve-out nondet è provata a metà ed esenta troppo.** L'entropia
intra-modo è provata SOLO flag-off (8bdbc200≠70cfe091); la gamba on è
presunta. E l'esenzione è chunk-wide: una divergenza off↔on VERA nel resto
del chunk dei 3 settype (testo warning, ordine) passerebbe sotto il tappeto.
Il «quarto test urla» solo se l'entropia si manifesta nella singola coppia di
run (R=1 per modo): rilevazione probabilistica, non garantita.

**R3 — post-flip il percorso di produzione (flag ASSENTE) è giudicato da UN
bit.** Corpus, funnel e trappole girano tutti con valori ESPLICITI; il
braccio absent di antiputenv verifica solo `any(REG_FORMS)==DEFAULT_ON`.
Nessun dente asserisce absent ≡ `=1` a parità di DUMP. Il tappeto-tautologia
è però evitato: `mode_contract_default_is_on_post_flip` (reg_lower.rs:990)
pinna il letterale. Ma antiputenv.rs:108 cita ancora il dente MORTO
`mode_contract_default_is_off_pre_flip`: un commento che indica un tripwire
inesistente è un gate di carta.

**R4 — bozza S-101, punto 3 refutato.** «L'emissione non cambia ⇒ batteria +
corpus bastano» confonde parità d'emissione con parità di RUNTIME: H-C1
(clone→prestito) cambia il ciclo di vita Zval — timing dei `__destruct`,
refcount osservabili, aliasing — la classe di bug che SOLO il collaudo WP ha
storicamente trovato (leak WP-78). Inoltre la matrice fixture (hook, __get,
ref, readonly, visibilità) è aperta: mancano per NOME `&$o->p` (alias byref),
timing distruttori, weakref/GC, coercizione typed. E nessuna banda numerica è
pre-registrata per la ri-baseline (punto 1).

## Emendamenti

- **A-KL-102-1**: ri-eseguire `s100-corpus-diff.sh` sul PIN (le due passate
  costano; la definizione operativa esiste già). L'evidenza di un gate nasce
  dall'albero giudicato.
- **A-KL-102-2**: carve-out → normalizzazione per TOKEN (strip del solo hex
  `random_bytes`, byte-diff del resto del chunk) + prova d'entropia
  intra-modo ANCHE flag-on.
- **A-KL-102-3**: dente absent ≡ `=1` a parità di dump-hash; sanare il
  commento morto in antiputenv.rs:108.
- **A-KL-102-4**: matrice fixture H-C1 chiusa per NOME PRIMA del codice,
  inclusi alias byref, timing `__destruct`, weakref/GC, typed coercion.

## KS (criteri vincolanti)

- **KS-KL-102-1**: nessun gate futuro cita evidenza prodotta su un albero
  diverso dal promosso senza allegare il diff macchina dei crates runtime.
- **KS-KL-102-2**: H-C1 si iscrive col SOFFITTO pre-registrato: il profilo dà
  clone 7,6% + deref 2,1% (+ quota drop/gc_note) ⇒ guadagno max ~27% ⇒ prop
  ≥ ~9× anche a successo pieno. H-C1 NON è la cura del 12,4→3: dichiararlo.
- **KS-KL-102-3**: WP pair di parità NON derogabile per H-C1 (cambia il
  runtime, non l'emissione); rilevazione nondet con R=2 intra-modo sul
  fail-set.

════════════════════════════════════════════
## FILE: verbali/verbale-4-hejlsberg.md

# Verbale sedia 4 — Hejlsberg (pipeline di compilazione, unit-cache/modes, dedup) — Concilio WP-102

## VERDETTO

S-100 ha eseguito il flip con disciplina, ma il claim di testa — «il modo è un
INPUT del funnel, batteria SENZA premesse ambientali» — era FALSO al momento
della chiusura: REFUTAZIONE CAPITALE, indipendentemente confermata (vedi R1).
La bozza §S-101 è ammissibile dal mio seggio SOLO con i denti sotto.

## Refutazioni

**R1 (CAPITALE) — un sito ambientale residuo dentro il funnel.** Nel commit
fb861e4, `emit_binary` (mod.rs:780) leggeva `reg_lower::enabled()` — il
globale OnceLock — a UNA chiamata di distanza dal campo `ctx.reg_lower`
creato apposta per eliminarlo (`FnCompiler` porta già `ctx`, riga 598).
Conseguenza: il braccio OFF in-process (`compile_mode(src,false)`) sotto
batteria default-ON emetteva `Binary(Add)` generico, una emissione che la
produzione flag-off NON produce mai (produce `BinaryAdd`, H-B2). Il claim
«lowered() eliminato, il braccio dei test è il funnel VERO» valeva per il
braccio ON soltanto. Nota di concordanza: in-flight ho trovato nel working
tree l'edit NON COMMESSO A-HO-102-1 (sedia 1) che sposta la lettura su
`self.ctx.reg_lower` — la refutazione resta a verbale su S-100 come spedita.

**R2 — il fix senza dente è una regressione futura garantita.** Nessun dente
in-process asserisce che il braccio OFF contenga `Op::BinaryAdd`: la
violazione R1 era invisibile alla batteria (e `BinaryAdd` NON è in
`is_reg_form`). Peggio: il commento di `stage2v3_flag_off_emits_no_register_forms`
(reg_lower.rs:936) dichiara «né BinaryAdd» — non asserito, e col fix di
sedia 1 diventa FALSO (l'emissione OFF lo contiene per H-B2). Commento ≠ dente.

**R3 — il tripwire «zero Binary(Add) flag-on» è falso a perimetro-modulo.**
La fixture del progetto stesso lo refuta: `public $d = self::K + 1`
(BODY_ZOO) produce un prop-init FUORI dal pass che flag-on SHIPPA un
`Binary(Add)` generico (emesso da `emit_binary`, mai riscritto). Il funnel
test lo scampa solo perché il grep è scoped a `{main}`/`fn probe`; il
commento a reg_lower.rs:289 («l'emissione non contiene MAI») non dichiara il
perimetro. Corollario: il handler generico `Binary` resta CALDO flag-on.

**R4 — l'esaustività di `all_funcs` è 2/3.** Il commento (reg_lower.rs:622)
promette «un campo nuovo di Module O DI CompiledClass NON COMPILA» — ma
`CompiledClass` NON è destrutturata (dot-access su 4 campi): `enum_cases`,
`attributes`, `abstract_sigs` e ogni campo futuro con corpi sfuggono in
silenzio all'assert flag-off «nessuna forma registro OVUNQUE».

**R5 — A-HE-101-3 non è solo aperto: oggi è INESERCITABILE.**
`UnitKey.reg_mode` è ri-letto dal globale (vm/mod.rs:16058) che è
OnceLock-costante intra-processo: il discriminante di chiave non può MAI
differire in vivo, quindi il «controllo positivo» non ha un percorso
pubblico che lo eserciti. Il modo ora vive in TRE posti (OnceLock, ctx,
UnitKey) tenuti d'accordo per convenzione, non per costruzione.

**R6 — dump e ereditarietà: duplicazione non pinnata.** `prop_info` è
flattened parent-first: l'hook del padre si ridumpa sotto OGNI figlio.
Deterministico (sort per nome) ma nessuna fixture con ereditarietà lo
dichiara; un census-da-dump double-conta i corpi ereditati.

## Emendamenti

- **A-HE-102-1**: dente in-process che pinna il braccio OFF con `BinaryAdd`
  PRESENTE e il braccio ON senza `Binary(Add)` (via `compile_mode`); sanare
  il commento di stage2v3_flag_off.
- **A-HE-102-2**: alla rotazione la batteria cargo gira nei DUE modi
  espliciti (`=0` e `=1`) — la purezza del funnel si prova eseguendo.
- **A-HE-102-3**: destrutturare `CompiledClass` in `all_funcs` (match
  esaustivo vero).
- **A-HE-102-4**: perimetro del tripwire dichiarato per NOME + controesempio
  prop-init (`self::K+1`) pinnato come eccezione ATTESA flag-on.
- **A-HE-102-5**: stampare il modo NEL `Module`; `UnitKey.reg_mode` derivato
  da lì; dente a chiave costruita a mano (chiude A-HE-101-3).
- **A-HE-102-6**: fixture con ereditarietà per il dump; politica di dedup
  dichiarata (o duplicazione pinnata).

## Keystones

- **KS-HE-102-1**: nessuna iscrizione/promozione H-C1 prima della batteria
  nei due modi espliciti nella stessa rotazione.
- **KS-HE-102-2**: A-HO-102-1 non si committa senza A-HE-102-1 — il fix
  entra solo col dente che l'avrebbe morso.

════════════════════════════════════════════
## FILE: verbali/verbale-5-bak.md

# Verbale Sedia 5 — Lars Bak (microarchitettura, path caldi, alloc-rate) — Concilio WP-102

## VERDETTO

S-100 è metodologicamente la sessione migliore dell'arco (flip collaudato nei due modi, decisioni con misura). Ma la prima misura H-C **nomina il meccanismo sbagliato con strumenti ciechi**, e la bozza §S-101 rischia di scrivere H-C1 contro un nemico che sul giudice potrebbe non esistere. Refutazioni capitali: **sì**.

## Refutazioni

**R1 (capitale) — Il meccanismo H-C1 è nominato da visibilità asimmetrica su un carico di SOLI INTERI.** `prop.php` muove int (`$o->x = $o->y + 1`). Il clone di uno Zval intero non dovrebbe allocare né toccare refcount: se `drop_in_place<Zval>` 12,6% + `gc_note` 5,3% compaiono su un carico scalare, il costo NON è "clone vs borrow di dati refcounted" ma drop-glue, controllo di discriminante e write-barrier chiamati incondizionatamente su scalari. La cura "prestito/refcount al posto del clone" presuppone traffico refcount; su int potrebbe AGGIUNGERLO dove oggi non c'è. Intanto l'oracle nasconde il SUO traffico refcount inline negli handler SPEC ("nessun simbolo visibile" = cecità, non assenza). Il meccanismo va ri-nominato dopo un census alloc/refcount, non dedotto dai simboli out-of-line.

**R2 — La decomposizione 2,0×6,2 è un'identità aritmetica, non due leve.** ns/op è una MEDIA su specie eterogenee: i 9 op di puro traffico (Sweep, Pop, Swap, doppioni Load) alzano il conteggio e ABBASSANO meccanicamente il costo/op medio. Eliminare traffico peggiorerebbe la "gamba costo"; le due gambe non sono indipendenti e "domina il costo/op" è un artefatto del framing. E "~9-10 ns quasi invariante di categoria" poggia su n=2 (prop, arith residuo): due punti non fanno un invariante.

**R3 — Il profilo co-equale con run_loop al 50% inlined non giudica.** Il 27% Zval conta solo simboli out-of-line: ogni clone/drop inlinato dentro run_loop è invisibile ⇒ 27% è un PAVIMENTO. Simmetricamente, dentro quel 50% non si distingue dispatch da corpi handler da fast-path clone: attribuire il residuo al dispatch è non supportato. Qualsiasi post-misura di H-C1 giudicata contro questa baseline sfocata è un verdetto sfocato.

**R4 — L=12,9 ns/occ non trasporta, e la sessione stessa lo prova.** Un solo shape, 200M iter, BTB perfettamente addestrata, ~190 ns/iter di contesto: sull'occorrenza residua VERA di add.php la contro-misura dà −5,6 ns/occ — la "banda" si è già dimezzata cambiando vicinato di dispatch. La decisione di estendere resta valida (frequenza ≠ 0, equivalenza provata); il NUMERO no: chiamarlo "coerente" con D=6,27 a fattore 2 di distanza è generoso.

**R5 — H-C1 non chiude H-C nemmeno a successo pieno, e la bozza non lo dice.** Rimuovendo TUTTO il 27%: 5,22×0,73=3,81 s ⇒ ~9,1× — ancora 3× sopra l'obiettivo X≤3. Il tetto atteso va scritto PRIMA, o il verdetto post-misura sarà negoziato dopo.

## Emendamenti

- **A-BA-102-1**: census alloc/iter e refcount-ops/iter su prop.php nei DUE motori (allocatore contatore o stats mimalloc) PRIMA di iscrivere H-C1; se alloc/iter≈0, il meccanismo si ri-nomina (drop-glue/discriminante su scalari).
- **A-BA-102-2**: profilo inline-aware (stack con inlining ricostruito, o build `inline(never)` mirata CON perturbazione misurata) per aprire il 50% di run_loop: quote dispatch/handler/clone-inlinato per NOME.
- **A-BA-102-3**: fixture H-C1 per SPECIE di valore (int, string condivisa, array), criterio per specie — la cura giusta per string può essere quella sbagliata per int.
- **A-BA-102-4**: L=12,9 non entra in alcun registro come coefficiente riusabile; ogni shape futuro porta il SUO micro.

## Kill-switch

- **KS-BA-102-1**: H-C1 NON si iscrive senza tetto atteso pre-registrato in ns/iter (≤ pavimento 27% + quota inline da A-BA-102-2) e la dichiarazione esplicita che il successo pieno lascia prop ~9×.
- **KS-BA-102-2**: la ri-baseline sei categorie si pubblica con ns/op per specie (traffico vs lavoro), mai solo il prodotto conteggio×costo: l'identità non è una decomposizione causale.

════════════════════════════════════════════
## FILE: verbali/verbale-6-pedersen.md

# Verbale SEDIA 6 — Pedersen (confine per-richiesta, lifecycle server, ambiente) — Concilio WP-102

## VERDETTO

Il flip S-100 nel merito CLI regge; il **«sì — GRADUATO» del pin server f2ab0636 sovradichiara** e la parità bimodale server ha un buco di controllo positivo. **Una refutazione capitale** (R1).

## Refutazioni

**R1 (CAPITALE) — parità server bimodale senza controllo positivo di modo.** Nei due bracci l'esito ATTESO è identico (fails=0 off e on): `fails=0 ×2` è indistinguibile da «PHPR_REG_LOWER non è mai arrivato al processo server e i due bracci erano entrambi default-on». Il funnel CLI asserisce dump-hash DIVERSI tra i bracci; per il server non esiste l'analogo: nessuna riga di log del modo effettivo, nessuna interrogazione, `srv.log` scritto e mai giudicato. La parità bimodale server è oggi un gate che non può fallire per la causa che dichiara di sorvegliare.

**R2 — gradazione mal nominata.** La voce «server-HTTP option+restapi via phpr CLI ✓» accredita al pin **php-server** una gamba che quel binario non esegue: `run_group` in `s100-parity-server.sh` usa solo `$PHPR`/`$ORACLE`. Solo la gamba A (sentinella) esercita f2ab0636. In più l'hash phpr in gamba B è «registrato, non gate»: la gamba può girare su un phpr non-pin. Il WP VERO dietro HTTP resta non esercitato — la gamba B è CLI phpunit.

**R3 — che cosa la sentinella estesa NON vede.** (a) 16 richieste ÷ 4 per endpoint: i leak LENTI (100+ richieste) restano fuori; (b) la byte-identità dell'output non misura la memoria: un leak di retention stile WP-78 che non altera l'output passa — nessun campione RSS/footprint del processo server; (c) burst concorrente solo su `p1`, nessuna prova che ENTRAMBI i workers abbiano servito (nessun worker-id); (d) `j1.php` è «restapi-shaped», non restapi; (e) status HTTP mai asserito.

**R4 — putenv impotente solo su REG_LOWER (census eseguito in seduta, chiude la parte enumerativa di A-PE-101-2).** `PHPR_STUB_ELISION` e `PHPR_UNIT_CACHE` (`vm/mod.rs:15718/15727`) sono LAZY (`get_or_init`), a grammatica APERTA (`map_or(true, |v| v != "0")` — presence-quirk della stessa classe del vecchio `=0`-accende), NON sigillate: un `putenv` prima del primo uso decide il modo dell'INTERO processo server — azione per-richiesta che muta stato di processo, violazione del confine. Anche `PHPR_REQ_NS` lazy in `worker_pool.rs:155`; `PHPR_MEM_CENSUS` riletto a runtime (apparato, ma osservabile).

**R5 — deployment reale.** Il modo viene dall'env allo spawn, ma solo i launcher lo costruiscono: nessun `--build-info` (backlog A-HO-101-4/A-PE-101-5 ancora aperto) né log di startup dichiara il default del binario deployato; lo spawn fuori-launcher (shell utente, launchd) non è collaudato da nessun atto.

**Su §S-101**: «l'emissione non cambia ⇒ batteria+corpus bastano» esclude il server — ma H-C1 tocca clone/drop/gc_note del ciclo di vita Zval, ESATTAMENTE la classe di retention per-richiesta di WP-78: se H-C1 si scrive, la sentinella (e l'endurance sotto) è DOVUTA.

## Emendamenti

- **A-PE-102-1**: il server dichiari il modo effettivo (log startup + interrogabile); i launcher lo ASSERISCANO per braccio.
- **A-PE-102-2**: sigillo eager + value-parse a lista chiusa per `PHPR_STUB_ELISION` e `PHPR_UNIT_CACHE`; elenco R4 = base per chiudere A-PE-101-2 per NOME.
- **A-PE-102-3**: sentinella di ENDURANCE — N≥100 richieste su endpoint allocante + RSS/footprint del server inizio/fine, banda calibrata sul rumore MISURATO.
- **A-PE-102-4**: una gamba WP VERA servita da php-server via HTTP prima di scrivere «server-HTTP» in una gradazione.
- **A-PE-102-5**: PIN_REGISTRY — ogni gradazione nomini il BINARIO che la esegue; hash phpr in gamba B diventa gate.
- **A-PE-102-6**: se H-C1 si scrive in S-101, sentinella estesa + endurance nel suo ordine di collaudo, non opzionali.

## Kill-switch

- **KS-PE-102-1**: nessuna cifra «server modo X» senza asserzione del modo EFFETTIVO nel processo server.
- **KS-PE-102-2**: nessun nuovo `PHPR_*` dietro `OnceInit` senza grammatica a lista chiusa + braccio anti-putenv.

════════════════════════════════════════════
## FILE: verbali/verbale-7-leijen.md

# Verbale sedia 7 — Daan Leijen (allocatore, footprint fisico, layout) — Concilio WP-102

## VERDETTO

S-100 regge sui gate di parità; la **contabilità del footprint NO**. Due
refutazioni capitali. La bozza §S-101 punto 4 va riformulata prima
dell'esecuzione.

## REFUTAZIONI

**R1 (capitale) — «cross-albero» è un'etichetta non provata.** L'R=2
intra-sera (0,5%) prova solo che il +95 MiB della gamba OFF **non è rumore
di stasera**; non prova che sia dell'ALBERO. Tra WP-99 e S-100 è cambiato
anche l'AMBIENTE (corpus WP, uploads, stato MySQL, macchina/disco a ~13G).
La contro-prova che discrimina esiste ed è economica: **un solo full run
del pin S-99 sigillato (stash `phpr-s99-sigillo`) stasera-stessa accanto
al pin S-100**. Senza questo A/B, un bisect (≥5 full run, con pin storici
che sappiamo non sempre riproducibili — WP-94) rischia di inseguire un
fantasma d'ambiente.

**R2 (capitale) — il peak del DEFAULT è un numero del CANDIDATO.** La
coppia WP è girata su a2772e62; il pin promosso 725a2ffad763bbc4 nasce
DOPO, col flip fb861e4 che tocca il funnel di produzione
(`ProgramCtx.reg_lower`, `compile_program_with_mode`, `lowered()`
eliminato). Il corpus per NOME non gatea il footprint: una regressione di
peak passa la parità. Eppure la rotazione pubblica «full peak ora 1929,0
MiB nel modo default»: è lo stato di un binario **mai misurato**.

**R3 — la voce aperta è mal nominata: non è «gamba OFF».** ON è a
1929,0 vs 1892,56 di WP-99 = **+36,5 MiB**; OFF +95÷106. La crescita
tocca ENTRAMBE le gambe; l'ON ne mostra meno forse perché il lowering ne
maschera una parte. Attribuire «OFF +95» pre-seleziona la spiegazione.

**R4 — «in storia (+1,9%)» per la gamba ON è indulgente**: la storia
dichiarata è −0,45% su WP-94→99; +1,9% è ~4× quel movimento, benedetto
senza banda.

**R5 — banda onesta sul full-peak**: la ±2% simmetrica era doppiamente
sbagliata — sotto il rumore (+14,6% intra-sera oracle) e simmetrica per
un gate anti-regressione. Il peak è una statistica di MASSIMO (coda
destra pesante): R=2 non è una calibrazione. E il rumore è **per-motore**
(phpr 0,5%, oracle 14,6%): una banda unica per i due motori è vuota.

## EMENDAMENTI

- **A-LE-102-1**: prima mossa dell'attribuzione = **A/B stessa-sera dei
  due pin sigillati (S-99 vs S-100), stesso ambiente**; solo se il delta
  si riproduce ⇒ **census allocatore PRIMA del bisect** (vmmap physical
  footprint per fase + stats mimalloc, `MIMALLOC_PURGE_DELAY=0`): il
  census dà il CANALE, il bisect solo il commit, a 5× il costo.
- **A-LE-102-2**: la voce si rinomina «crescita d'albero: OFF +95 / ON
  +36,5 MiB»; l'attribuzione misura entrambe le gambe.
- **A-LE-102-3**: calibrazione del rumore PRIMA di ogni banda futura sul
  full-peak: R≥5 sul pin, per motore; statistica = **mediana** (lezione
  WP-91), spread pubblicato; banda **unilaterale** (solo tetto
  anti-regressione), larghezza ≥ spread misurato.
- **A-LE-102-4**: ogni numero di peak pubblicato in rotazione porta
  l'HASH del binario misurato; smoke peak media sul pin promosso SUBITO
  (costa ~1 min), full alla prossima coppia.

## KILL-SWITCH

- **KS-LE-102-1**: se l'A/B dei due pin non riproduce il +95 (delta <
  2× lo spread intra-sera), la voce si riclassifica AMBIENTE e il bisect
  è VIETATO.
- **KS-LE-102-2**: banda sul full-peak senza spread R≥5 pubblicato prima
  = VOID.
- **KS-LE-102-3**: se lo smoke peak sul pin promosso esce dallo spread
  calibrato del candidato, il «1929,0» si RITIRA dalla rotazione e il
  flip si ricollauda sul footprint.

════════════════════════════════════════════
## FILE: verbali/verbale-8-stogov.md

# Verbale Sedia 8 — Dmitry Stogov (Zend engine, semantica PHP) — Concilio WP-102

**VERDETTO: S-100 regge sul flip; REFUTAZIONI PUNTUALI su §3.12 (catalogata male
in tre punti) e sulla FORMA di H-C1 (il "prestito" non è la semantica Zend).
Nessuna refutazione capitale.**

## Refutazioni (con verifica su fonte C 8.5.7 e oracle, stasera)

1. **§3.12 REFUTATA COME SCRITTA — tre difetti.** (a) È **mode-dependent**:
   in weak il ref va a 0; in **strict** lo stato RESTA 1 ma il TypeError
   catturato CAMBIA in `Cannot assign null to reference held by property
   T::$i of type int` (verificato sull'oracle brew). La voce cataloga solo il
   weak e dichiara "messaggio a parità" — in strict è falso. (b) Il commento
   della fixture (b) dice "regime strict del FILE ASSEGNANTE" ma il file NON
   ha `declare(strict_types=1)`: collauda il weak raccontandosi strict.
   (c) **Sotto-scopata**: anche il typed-**prop** diretto (`$t2->i += "abc"`
   → 0, verificato; `zend_binary_assign_op_typed_prop` ha la stessa forma).
   **Meccanismo VERIFICATO** (non più "plausibilmente"): il path d'errore di
   `add` fa `ZVAL_UNDEF(result)` quando `result != op1`
   (`convert_op1_op2_long`, zend_operators.c); `zend_binary_assign_op_typed_ref`
   opera su `z_copy`; in weak `zend_verify_ref_assignable_zval` →
   `i_zend_verify_type_assignable_zval` = −1 → `zend_parse_arg_long_weak` su
   UNDEF (tipo 0 < IS_TRUE, ≠ IS_NULL) → **0 senza warning**. "Azzera" è un
   artefatto di un UNDEF che attraversa la coercizione debole; phpr
   "conserva" coincide con lo STATO Zend strict e col principio "op fallito ⇒
   nessuna scrittura", non col messaggio strict.
2. **§3.11**: il warning oracle nasce dal fetch CV **RW** (`zval_undefined_cv`),
   non dall'AssignOp: la cura appartiene alla famiglia fetch-undef (stessa di
   §3(c)), non a un cerotto sul compound; solo così l'ordine
   rhs→warning→errore-op diventa collaudabile una volta per tutte.
3. **H-C1 refutata NELLA FORMA "prestito/refcount al posto del clone"**: Zend
   `FETCH_OBJ_R` non presta — fa `ZVAL_COPY_DEREF`: copia di valore 16-byte +
   ADDREF **solo se refcounted**. Sul giudice prop.php i valori sono INT:
   l'oracle non fa NESSUN refcount, eppure phpr spende ~27% in
   clone/drop/gc_note — il ciclo di vita Zval costa anche sui **scalari**
   (gc_note su Int?). La COPIA vera in Zend non avviene mai in lettura: gli
   array sono CoW, la separazione scatta in scrittura.
4. **Il borrow nudo è indifendibile** in almeno tre casi PHP: mutazione
   interlacciata nello statement (`$o->a + $o->mut()`: il valore osservabile
   è PRE-mutazione — l'addref lo garantisce, un prestito dello slot no);
   `__get`/hook/lazy (il valore è un temporaneo, non c'è slot); slot
   contenente ref (serve deref).
5. **Census**: verificato che il corpo del loop è IDENTICO pre/post optimizer
   (9 op a 0x10000 e 0x20000) ⇒ la decomposizione 2,0×6,2 non soffre del
   disallineamento census/cronometro su QUESTO giudice; il criterio però va
   scritto prima di riusare la tavola su calls.php, dove l'optimizer fonde.
6. **Tetto pre-registrato**: azzerare TUTTO il ciclo di vita Zval porta prop
   da 12,4× a ~9×: H-C1 **non** chiude H-C (il 6,2×/op resta dispatch+handler
   generici vs SPEC/TAILCALL). Vietato presentarla come la cura.

## Emendamenti

- **A-ST-102-1**: riscrivere §3.12 (meccanismo verificato; scopo esteso al
  typed-prop; tabella weak/strict con messaggio sostituito-e-chained in
  strict); fixture (b): twin strict + twin typed-prop; correggere il commento.
- **A-ST-102-2**: fix §3.11 nominato nella famiglia fetch-undef (CV e prop),
  giudice = trap (e) per l'ordine osservabile.
- **A-ST-102-3**: controfattuale H-C1 in due stadi: (i) censimento delle
  SPECIE di Zval che attraversano gc_note/drop nel giudice; (ii) bypass
  bookkeeping per non-refcounted (semantica-neutra); solo dopo, addref per i
  refcounted — mai borrow nudo.
- **A-ST-102-4**: fixture H-C1 obbligatorie PRIMA del codice: mutazione
  interlacciata, `__get`/hook/lazy, ref-in-slot, warning undef-prop,
  destruct-timing (riusare la trap f).
- **A-ST-102-5**: tavola census×costo su calls.php: census nei DUE regimi
  optimizer prima di fissare il denominatore.

## Kill-switch

- **KS-ST-102-1**: nessuna riga H-C1 senza le fixture A-ST-102-4 verdi
  sull'oracle, attese scritte PRIMA.
- **KS-ST-102-2**: se il censimento mostra gc_note su non-refcounted, il
  bypass scalari si misura DA SOLO prima di ogni schema refcount.
- **KS-ST-102-3**: la ri-baseline sei categorie (p.1 bozza) precede ogni
  cifra H-C1: rapporti citati senza quella misura non fanno fede.

════════════════════════════════════════════
## FILE: verbali/verbale-9-gregg.md

# Verbale Sedia 9 — Brendan Gregg (metodologia di misura, attribuzione) — Concilio WP-102, mandato inverso

## §FONDAMENTALI

**(a) Avanzamento dell'oggetto in S-100 — misure vere, per nome.** Sessione
ricca sull'oggetto: (1) H-B2-sotto-flip decisa CON misura — isolante
dump-verificato L=12,9 ns/occ ≥ pavimento 1,0, estensione eseguita,
contro-misura L'∈[−1,0], giudice add on 3,25 netto (−31% vs off); (2) H-C
prima misura completa — prop on 12,4× decomposto in conteggio 2,0× (census
opcache 9 vs 18 op/iter) × costo/op 6,2× (1,56 vs 9,67 ns/op), profilo
co-equale con simboli per NOME (~27% ciclo vita Zval), candidata H-C1
nominata senza scrivere righe; (3) coppia WP nei due modi stessa-sera con
bande pre-registrate (CPU 1,008/1,004; peak 0,981/0,965); (4) fatto di
strumento NUOVO: oracle full-peak +14,6% intra-sera. Il flip è stato
giudicato da misure, non da fede.

**(b) Contatore sessioni-senza-misura**: **0** (riga ⏱ di NEXT_SESSION:
ultima full/media = WP-100, questa sessione; campagna oggetto = S-100).

**(c) Rischio d'oggetto più trascurato**: la baseline del giudice è oggi
ETEROGENEA — prop e add misurate in modo default, le altre quattro
categorie ferme a S-99 flag-off. Se H-C1 si iscrive prima della
ri-baseline sei-categorie (punto 1 della bozza), il suo criterio nasce da
numeri di due regimi diversi. Secondo rischio: +95 MiB full-peak OFF
cross-albero senza attribuzione — la gamba di rollback invecchia al buio.

## VERDETTO

S-100 è una sessione d'oggetto piena: flip promosso con gate misurati,
H-C decomposta e meccanismo NOMINATO. **Nessuna refutazione capitale.**
Tre refutazioni di metodo e quattro emendamenti.

## Refutazioni (non capitali)

**R1 — Il "costo/op ~9-10 ns quasi invariante" non è un invariante: sono
DUE punti** (prop 9,67; residuo arith 9,9). Usarlo come tariffa predittiva
ripeterebbe l'errore 57/43. Osservazione legittima, tariffa no.

**R2 — Il 27% ciclo-vita-Zval è un PAVIMENTO, non una quota**: il profilo
è top-of-stack e run_loop assorbe il 50% come "dispatch+handler inline" —
massa NON attribuita che può contenere altri clone/drop inlinati.
Simmetricamente, "zero simboli alloc" sull'oracle prova invisibilità alla
granularità dei simboli, non assenza di costo refcount. Il criterio di
H-C1 non può derivare la soglia dal 27%.

**R3 — La sanity ±5% della decomposizione è VACUA**: 12,4 = 2,0 × 6,2 è
esatta per costruzione (entrambi i fattori derivano dagli stessi T e
census: il prodotto ricostruisce il rapporto per identità algebrica).
Manca la sanity INDIPENDENTE: il census è STATICO (op/iter dal dump); la
conferma che 18 op/iter vengano davvero eseguiti esige il contatore
DINAMICO (feature op-census, esiste da WP-33..38) — oppure un micro
variante con corpo diverso che riproduca ~9,7 ns/op.

Sul metodo co-equale in sé: sano. Il campionamento è A TEMPO (4 s per
lato, durate 4,7 vs 5,2 s comparabili): i 300M vs 30M iter non introducono
bias di campionamento — a regime i campioni pesano il tempo, non le
iterazioni. Il difetto è solo l'attribuzione inline (R2).

## Emendamenti

- **A-GR-102-1**: prima del giudizio su H-C1, validare il census statico
  (18 vs 9 op/iter) col contatore dinamico op-census sul lato phpr.
- **A-GR-102-2**: decomporre il 50% di run_loop — riprofilo con
  attribuzione inline-aware (frame pointers/debug info o `#[inline(never)]`
  temporaneo sui candidati) per dare a H-C1 una BANDA di meccanismo.
- **A-GR-102-3**: bande sul full-peak d'ora in poi ASIMMETRICHE e
  calibrate sul rumore misurato (+14,6% intra-sera oracle); i gate
  anti-regressione sono a UN lato.
- **A-GR-102-4**: la ri-baseline sei-categorie in modo default (bozza §S-101
  punto 1) è PREREQUISITO del criterio di H-C1, non passo parallelo.

## KS

- **KS-GR-102-1**: la soglia di caduta di H-C1 nasce dal SUO micro
  isolante ≥ pavimento sonda — mai dal 27% del profilo né dalla "tariffa"
  9-10 ns/op.
- **KS-GR-102-2**: nessuna banda su metrica di peak più stretta del rumore
  run-to-run MISURATO dello strumento nella stessa finestra; violazione ⇒
  gate VOID.

## Sulla bozza §S-101

L'ordine È misura-first (ri-baseline al punto 1, H-C1 iscritta col
controfattuale al 2, apparato timeboxato al 4-5): approvato con gli
emendamenti sopra, in particolare A-GR-102-4 che rende vincolante la
precedenza del punto 1.

════════════════════════════════════════════
## FILE: verbali/team-flip-residuo.md

# Team «flip-residuo» — Concilio WP-102, fase 2

**Relatore**: sedia 1 (Hoare). **Team**: 1 Hoare, 4 Hejlsberg, 3 Klabnik.
**Fonti**: verbale-1-hoare.md, verbale-4-hejlsberg.md, verbale-3-klabnik.md (restano VINCOLANTI; questa nota non li sostituisce).

## Nota di sessione (fatti accaduti DOPO la fase 1)

1. Il fix di `emit_binary` (lettura da `ctx.reg_lower` al posto di `reg_lower::enabled()`, **A-HO-102-1**) è **GIÀ APPLICATO**; i denti mirati sono **verdi**. Batteria completa **in corso** al momento della stesura.
2. Il corpus-diff e il corpus-gate verranno **RI-ESEGUITI sul PIN NUOVO post-fix**: la stessa ri-esecuzione soddisfa sia **A-KL-102-1** (evidenza sull'albero giudicato) sia il collaudo del fix stesso.
3. Le refutazioni R1 restano a verbale su S-100 *come spedita* (concordanza esplicita di sedia 4): il fix in-flight non le sana retroattivamente.

## Convergenze (con numeri emendamento)

**C1 — Refutazione CAPITALE condivisa: sito ambientale residuo in `emit_binary`** (Hoare R1 ≡ Hejlsberg R1, trovata indipendentemente). Il claim S-100 «il modo è un INPUT del funnel, niente premesse ambientali» era FALSO alla chiusura: `compile/mod.rs:780` consultava il globale OnceLock a una chiamata di distanza dal campo `ctx.reg_lower`. Conseguenza concorde: il braccio OFF in-process sotto batteria default-ON non era l'emissione OFF di produzione (due fonti di verità). Rimedio: **A-HO-102-1** (applicato, v. nota §1) + doc-comment stale sanato nello stesso commit.

**C2 — Il fix senza dente è regressione garantita** (Hejlsberg R2/**A-HE-102-1** + dente richiesto in A-HO-102-1 + Klabnik KS-KL-102-1 in spirito). La violazione era invisibile alla batteria; commento ≠ dente. **KS-HE-102-2**: A-HO-102-1 non si committa senza A-HE-102-1 (braccio OFF con `BinaryAdd` PRESENTE, braccio ON senza `Binary(Add)`; sanare il commento di `stage2v3_flag_off`, reg_lower.rs:936, che col fix diventa FALSO).

**C3 — La purezza del funnel si prova ESEGUENDO nei due modi espliciti** (Hejlsberg **A-HE-102-2**; Hoare: dente dump-hash braccio OFF ≡ produzione OFF `env =0` processo separato; Klabnik **A-KL-102-3**: dente absent ≡ `=1` a parità di dump-hash). Tre formulazioni dello stesso principio: nessun braccio della matrice modi resta presunto.

**C4 — Commenti-gate morti o stale da sanare per NOME** (Hoare: doc-comment `emit_binary`; Hejlsberg: stage2v3 reg_lower.rs:936 e reg_lower.rs:289 senza perimetro; Klabnik: antiputenv.rs:108 cita dente MORTO). Un commento che indica un tripwire inesistente è un gate di carta.

**C5 — H-C1 non si iscrive senza precondizioni dure** (tutte e tre le sedie, da angoli diversi): Hoare **A-HO-102-3**/**A-HO-102-4**/**KS-HO-102-1**/**KS-HO-102-2** (scelta di design (a) refcount+COW vs (b) fusione in-handler PRIMA dell'iscrizione; fixture aliasing; corpo condiviso dei due handler Add); Klabnik R4/**A-KL-102-4**/**KS-KL-102-2**/**KS-KL-102-3** (matrice fixture chiusa per NOME inclusi alias byref, timing `__destruct`, weakref/GC, typed coercion; soffitto pre-registrato ~27% ⇒ H-C1 NON è la cura del 12,4→3; WP pair non derogabile); Hejlsberg **KS-HE-102-1** (niente H-C1 prima della batteria due-modi nella stessa rotazione). Convergenza piena: parità d'emissione ≠ parità di RUNTIME (Klabnik R4, leak WP-78 come precedente).

**C6 — Evidenza solo dall'albero giudicato** (Klabnik R1 CAPITALE + **A-KL-102-1** + **KS-KL-102-1**; nessuna sedia dissente). Soddisfatta dalla ri-esecuzione sul pin nuovo (nota §2).

## Conflitti (posizione di ciascuna sedia)

**K1 — Forma del fix `emit_binary`.**
- *Hoare*: preferenza dichiarata per la variante FORTE — emettere `BinaryAdd` **incondizionatamente** (le finestre fondono entrambe le grafie per dichiarazione propria), facendo sparire del tutto l'`enabled()` compile-side. Nota del relatore: la variante forte sanerebbe anche il controesempio prop-init di Hejlsberg R3.
- *Hejlsberg*: concordanza registrata sulla variante `ctx.reg_lower` (quella applicata), ma vincolata al dente (KS-HE-102-2); in più pretende perimetro del tripwire dichiarato + controesempio `self::K+1` pinnato come eccezione ATTESA (**A-HE-102-4**), che con la variante applicata resta necessario.
- *Klabnik*: nessuna posizione sulla forma.
- Stato: il fix applicato è la variante debole; la variante forte resta proposta aperta per S-101 (assorbirebbe A-HE-102-4).

**K2 — Rimedio alla molteplicità del modo (OnceLock + ctx + UnitKey, «tre posti tenuti d'accordo per convenzione»).**
- *Hoare* (**A-HO-102-2**): sigillo di TIPO — testimone ZST reso da `seal_reg_lower_mode()` e preteso dal confine di compilazione: l'omissione NON COMPILA. Promozione da backlog a **S-101**.
- *Hejlsberg* (R5 + **A-HE-102-5**): modo stampato NEL `Module`, `UnitKey.reg_mode` derivato da lì, dente a chiave costruita a mano (A-HE-101-3 oggi INESERCITABILE).
- *Klabnik*: non si pronuncia sul meccanismo.
- Conflitto reale: non sul merito (complementari) ma sulla **priorità** — Hoare lo vuole in S-101, Hejlsberg non ne fa un keystone. Il team NON compone d'ufficio: entrambe le voci restano a verbale.

**K3 — Ordine delle precondizioni H-C1.** Hoare antepone la scelta di design (KS-HO-102-2); Klabnik antepone matrice fixture + soffitto pre-registrato (A-KL-102-4, KS-KL-102-2); Hejlsberg antepone la batteria due-modi (KS-HE-102-1). Nessuna contraddizione di merito: sono TRE gate cumulativi; il conflitto è solo su quale citare come primo. Il team li registra come congiunzione (tutti e tre necessari).

## Priorità proposte per l'ordine S-101 (perimetro flip-residuo)

Regola applicata: **apparato in ordine solo se blocca l'oggetto**; il resto = backlog per NOME.

### BLOCCANTE (in ordine)

1. **Chiusura del fix A-HO-102-1 secondo KS-HE-102-2**: commit del fix (già applicato e verde sui denti mirati) SOLO insieme ad **A-HE-102-1** (dente OFF-con-BinaryAdd / ON-senza-Binary(Add) via `compile_mode`) e alla sanatoria dei commenti C4 (stage2v3, doc-comment emit_binary, antiputenv.rs:108). Attesa: esito batteria completa in corso.
2. **A-KL-102-1** — corpus-diff + corpus-gate RI-ESEGUITI sul PIN NUOVO post-fix (già programmato, nota §2). Nessun gate futuro cita evidenza pre-pin (KS-KL-102-1). Chiude anche il buco carry-over dei chunk FAIL (byte-diff mai ri-giudicato sul pin).
3. **A-HE-102-2** — batteria cargo nei DUE modi espliciti (`=0` e `=1`) alla rotazione. È il cancello di KS-HE-102-1: senza di essa H-C1 non si iscrive.
4. **A-KL-102-3** — dente absent ≡ `=1` a parità di dump-hash (post-flip il percorso di produzione è giudicato da UN bit: va pinnato per costruzione, non per letterale).
5. **Gate d'iscrizione H-C1** (blocca H-C1, non il residuo flip; da consegnare all'ordine S-101 come precondizione cumulativa C5): scelta di design (a)/(b) dichiarata PRIMA + matrice fixture chiusa per NOME (aliasing Hoare + byref/`__destruct`/weakref/typed Klabnik) + soffitto pre-registrato KS-KL-102-2 + WP pair KS-KL-102-3.

### BACKLOG per NOME (apparato: non blocca l'oggetto)

- **A-HO-102-2** — testimone ZST del sigillo (Hoare lo vuole BLOCCANTE in S-101: dissenso registrato in K2; il relatore lo colloca qui perché non blocca né il fix né la ri-esecuzione sul pin).
- **A-HE-102-5** — modo nel `Module` + `UnitKey.reg_mode` derivato + dente a chiave manuale (chiude A-HE-101-3).
- **A-HE-102-3** — destrutturare `CompiledClass` in `all_funcs` (esaustività 3/3).
- **A-HE-102-4** — perimetro tripwire per NOME + controesempio prop-init pinnato (decade se S-101 adotta la variante forte di K1).
- **A-HE-102-6** — fixture ereditarietà dump + politica dedup dichiarata.
- **A-KL-102-2** — carve-out nondet per TOKEN + prova entropia intra-modo ANCHE flag-on.
- **A-HO-102-4** — corpo condiviso handler `Binary(Add)`/`BinaryAdd` (il differenziale degrada a cintura di regressione). Diventa bloccante SOLO all'iscrizione di lavoro d'emissione ulteriore (KS-HO-102-1, oggi soddisfatto dal fix).
- Variante forte di `emit_binary` (K1, Hoare) — proposta aperta, da giudicare in S-101.

════════════════════════════════════════════
## FILE: verbali/team-hc-canale.md

# Team «hc-canale» — Concilio WP-102, fase 2

**Sedie**: 5 Bak · 2 Matsakis · 8 Stogov — Relatore: hc-canale
**Fonti vincolanti**: verbale-5-bak.md · verbale-2-matsakis.md · verbale-8-stogov.md
**Tema**: candidata H-C1 («prestito al posto del clone su PropGet»)

---

## 1. Verdetto di team

**H-C1 nella forma «prestito/refcount al posto del clone del valore letto» è
REFUTATA da tutte e tre le sedie e NON si iscrive all'ordine S-101 così
nominata.** Le tre refutazioni convergono sul fatto centrale: sul giudice
prop.php i valori sono `Long` (Copy), il clone del valore non alloca e non
tocca refcount — eppure phpr spende ~27% nel ciclo di vita Zval. Quindi il
meccanismo che costa NON è quello che H-C1 nomina, e la cura proposta
(prestito) potrebbe AGGIUNGERE traffico dove oggi non c'è (Bak R1), mira al
canale minore ignorando il RICEVITORE (Matsakis R1), e non corrisponde alla
semantica Zend, che non presta mai: `ZVAL_COPY_DEREF` = copia 16-byte +
ADDREF solo se refcounted (Stogov p.3).

Ulteriore vincolo condiviso (Bak R5 ≡ Stogov p.6): **anche a successo pieno
(tutto il 27% azzerato) prop resta ~9×** — H-C1 non chiude H-C e il tetto va
scritto PRIMA nell'ordine, mai negoziato dopo.

## 2. Riformulazione condivisa — H-C1 è SOSTITUITA da una forma a due stadi

Adottiamo la forma a due stadi di Stogov (A-ST-102-3), che sussume la
ri-nominazione per census di Bak (A-BA-102-1) e l'attribuzione per sito di
Matsakis (A-MA-102-1). Le nuove ipotesi, tutte SUBORDINATE al census di §3:

- **H-C1a «bypass scalari»** — se il census mostra gc_note/drop-bookkeeping
  su Zval NON-refcounted (Int ecc.), bypass del bookkeeping per i
  non-refcounted. Semantica-neutra per costruzione; si misura **DA SOLA**
  prima di ogni schema refcount (KS-ST-102-2).
- **H-C1b «canale ricevitore»** — se il census attribuisce il churn al
  ricevitore (`obj.deref_clone()` = bump `Rc<Object>` + drop→gc_note per
  ogni PropGet), estensione del prior art in-tree `ThisPropGet`
  (run.rs:3418) che sull'IC-hit già evita il clone del ricevitore. Sigillo
  di TIPO per ogni prestito: nessun borrow-guard attraversa un confine di
  op o un rientro VM (A-MA-102-3).
- **H-C1c «copy+addref condizionale»** — solo per valori refcounted e solo
  dopo (i) e (ii): la forma Zend, mai borrow nudo dello slot valore
  (indifendibile su mutazione interlacciata, `__get`/hook/lazy,
  ref-in-slot — Stogov p.4).

L'atteso di ciascuna riga si calcola **dal canale contato** (n° eventi
eliminati/iter × costo per evento misurato), MAI da quota%×T (Matsakis R3):
il 27% è la quota dell'intero micro, non il recuperabile — Sweep/Pop
droppano anche temporanei che nessun prestito elimina.

## 3. Ordine di misura — PRIMA di ogni riga di codice

1. **Ri-baseline sei categorie** (KS-ST-102-3): nessuna cifra H-C1 fa fede
   senza; pubblicata con ns/op per specie — traffico vs lavoro — mai solo
   il prodotto conteggio×costo (KS-BA-102-2).
2. **Census dinamico su prop.php, DUE motori**, decomposto su tre assi:
   - **specie** di Zval (Int / string condivisa / array / object) che
     attraversa clone/drop/gc_note (Bak A-BA-102-1: alloc/iter e
     refcount-ops/iter; se alloc/iter≈0 il meccanismo si ri-nomina);
   - **sito** (ricevitore vs valore vs traffico Sweep/Pop) per i quattro
     simboli drop 12,6 + clone 7,6 + gc_note 5,3 + deref_clone 2,1
     (Matsakis A-MA-102-1);
   - **canale** (alloc, refcount-bump, gc_note-call).
   Le tre predizioni di sedia si PRE-REGISTRANO come attese rivali:
   Bak = drop-glue/discriminante incondizionato su scalari; Matsakis =
   ricevitore Rc + gc_note; Stogov = gc_note su non-refcounted. Il census
   arbitra per nome.
3. **Profilo inline-aware** (A-BA-102-2) per aprire il 50% di run_loop:
   la post-misura di qualunque H-C1x giudicata contro la baseline sfocata
   attuale è un verdetto sfocato (Bak R3).
4. **Tetto pre-registrato nell'ordine**: successo pieno ⇒ prop ~9,1×,
   ancora >3× sopra X≤3 (KS-BA-102-1); vietato presentare H-C1x come "la
   cura" di H-C (Stogov p.6).

## 4. Fixture semantiche OBBLIGATORIE (in-tree e verdi su ENTRAMBI i motori, attese scritte PRIMA — KS-MA-102-1, KS-ST-102-1)

Unione delle liste A-MA-102-2, A-ST-102-4, A-BA-102-3:

- **Per specie di valore** (Bak): int, string condivisa, array — criterio
  di successo PER SPECIE (la cura giusta per string può essere sbagliata
  per int).
- **Aliasing/osservabilità** (Matsakis ∪ Stogov): mutazione interlacciata
  nello statement (`$o->a + $o->mut()` — valore osservabile PRE-mutazione,
  l'addref lo garantisce, un prestito no); `__get` e hook get/set (valore =
  temporaneo, non c'è slot); lazy ghost/proxy; `&$o->x` (Ref nello slot,
  serve deref); riassegnazione intra-espressione (`$o->x + ($o->x=…)`);
  `unset` durante la lettura; readonly/typed-uninit (`Undef` fatale);
  warning undef-prop; op che scrivono lo stack IN PLACE (`BinaryAdd` fa
  `*last_mut()=v`: mai alias di uno slot proprietà sullo stack).
- **Ciclo di vita** (Matsakis R2 + Stogov): ordine `__destruct`
  (destruct-timing, riusare la trap f); ciclo GC con proprietà nel ciclo;
  finestra del cycle-collector («under-noting delays a destructor»).

**Collaudo di parità WP obbligatorio anche senza cambi d'emissione**
(KS-MA-102-4): un cambio di aliasing a runtime sposta il momento in cui
`strong_count` tocca 1, cioè l'ordine dei distruttori — «l'emissione non
cambia ⇒ batteria+corpus bastano» è FALSA per questa famiglia (Matsakis R2).
Qualunque cambio d'ordine distruttori/free-order ⇒ reject senza appello
(KS-MA-102-3).

## 5. Conflitti residui (posizione per sedia)

1. **Prestito del ricevitore: ammesso o no?** — Matsakis: sì, à la
   `ThisPropGet`, purché sigillato dal TIPO (scope del borrow chiuso prima
   del push, mai attraverso confine di op). Stogov: «il borrow nudo è
   indifendibile» e la forma Zend è addref, mai prestito — ma i suoi tre
   controesempi (p.4) colpiscono il borrow dello SLOT VALORE, non il
   ricevitore. Bak: neutrale sul canale, esige che il census lo nomini
   prima. **Composizione di team**: H-C1b resta iscrivibile come prestito
   del ricevitore SOLO con sigillo di tipo A-MA-102-3 E fixture
   destruct-timing/interlacciata verdi; se una fixture mostra osservabilità
   dell'ordine dei drop del ricevitore, si ripiega su addref (forma
   Stogov). Il borrow dello slot valore resta VIETATO per unanimità.
2. **Nome del meccanismo dominante** — tre predizioni rivali (§3.2): non è
   un conflitto da negoziare ma da arbitrare col census; le predizioni
   pre-registrate impediscono la ri-narrazione post-hoc.
3. **Grado di refutazione** — Bak la classifica capitale (strumenti ciechi
   + nemico forse inesistente), Matsakis capitale sulla mira, Stogov
   puntuale sulla forma. Irrilevante per l'ordine: l'esito operativo
   (nessuna riga senza census+fixture) è identico e unanime.

## 6. Priorità per l'ordine S-101

1. **P1** — Ri-baseline sei categorie (prerequisito di ogni cifra).
2. **P2** — Census dinamico specie×sito×canale su prop.php, due motori,
   con le tre predizioni pre-registrate; in coda il profilo inline-aware.
3. **P3** — Fixture semantiche §4 in-tree, verdi su entrambi i motori.
4. **P4** — H-C1a (bypass scalari) SE il census la nomina: misurata da
   sola, semantica-neutra, primo codice candidato.
5. **P5** — H-C1b (ricevitore) SE il census la nomina: sigillo di tipo +
   coppia WP di parità obbligatoria; ripiego addref se le fixture mordono.
6. **P6** — H-C1c solo dopo P4/P5, mai borrow nudo dello slot.
7. **Fuori tema ma vincolante per l'ordine** (dai verbali): riscrittura
   §3.12 e fix §3.11 in famiglia fetch-undef (A-ST-102-1/2); perimetro del
   tripwire zero-`Binary(Add)` per NOME (A-MA-102-4); L=12,9 non è
   coefficiente riusabile (A-BA-102-4).

**Kill-switch di team** (unione, tutti attivi): KS-BA-102-1/2,
KS-MA-102-1/2/3/4, KS-ST-102-1/2/3.

════════════════════════════════════════════
## FILE: verbali/team-misura-server.md

# Team «misura-server» — Concilio WP-102, fase 2

Relatore: team sedie 7 (Leijen), 9 (Gregg), 6 (Pedersen). Fonte vincolante = verbali individuali
(`verbale-7-leijen.md`, `verbale-9-gregg.md`, `verbale-6-pedersen.md`); questo file compone, non sostituisce.

**Fatto post-fase-1 registrato**: il mode-probe del server (A-PE-102-1 — dump dell'unità nel log del
server, atteso dal contratto) è GIÀ implementato nella sentinella estesa e verrà esercitato nel
ri-collaudo del pin nuovo. La refutazione capitale R1-Pedersen ha quindi già la sua cura in codice:
resta da ESERCITARLA (i launcher devono ASSERIRE il modo per braccio, non solo loggarlo) prima di
riaccreditare qualunque cifra «server modo X» (KS-PE-102-1).

## Convergenze

1. **Bande peak: asimmetriche, calibrate, mai sotto il rumore.** Leijen (R5, A-LE-102-3,
   KS-LE-102-2) e Gregg (A-GR-102-3, KS-GR-102-2) convergono per NOME: gate anti-regressione a UN
   lato; larghezza ≥ rumore run-to-run MISURATO dello strumento nella stessa finestra, PER MOTORE
   (phpr 0,5% vs oracle +14,6% intra-sera: una banda unica è vuota); banda senza calibrazione
   pubblicata = VOID. Si adotta la forma più severa (Leijen): R≥5 sul pin, statistica = mediana
   (lezione WP-91), spread pubblicato — compatibile e inclusiva di KS-GR-102-2.
2. **Nessuna cifra senza il binario che l'ha prodotta.** Leijen R2/A-LE-102-4 (il «full peak 1929,0
   default» è lo stato di un binario MAI misurato: la coppia girò su a2772e62, il pin promosso nasce
   dopo il flip fb861e4; il corpus per NOME non gatea il footprint) e Pedersen R2/A-PE-102-5 (la
   gradazione «server-HTTP ✓» accredita a f2ab0636 una gamba eseguita da phpr CLI; hash gamba B
   registrato-non-gate). Stessa refutazione in due domini: ogni numero pubblicato porta l'HASH del
   binario misurato; ogni gradazione nomina il binario che la esegue.
3. **Gate che non può fallire per la causa che sorveglia = vacuo.** Pedersen R1 (fails=0 ×2
   indistinguibile da env mai arrivato) e Leijen R2 (parità che benedice il footprint) sono la stessa
   lezione: il controllo positivo del canale è parte del gate. Il mode-probe già implementato è la
   risposta al primo; lo smoke peak sul pin promosso (A-LE-102-4, ~1 min) al secondo.
4. **La gamba OFF non deve invecchiare al buio.** Gregg §FONDAMENTALI(c) nomina il +95 MiB come
   secondo rischio d'oggetto trascurato; Leijen ne fa l'oggetto di R1/R3. Voce rinominata
   «**crescita d'albero: OFF +95 / ON +36,5 MiB**» (A-LE-102-2): la crescita tocca ENTRAMBE le
   gambe; «gamba OFF» pre-selezionava la spiegazione.
5. **H-C1 ha prerequisiti nominati, non opzionali.** Gregg A-GR-102-4 (ri-baseline sei-categorie
   in modo default PRIMA del criterio: la baseline oggi è eterogenea, due regimi) + A-GR-102-1
   (census statico 18 vs 9 op/iter validato col contatore DINAMICO op-census) + KS-GR-102-1 (soglia
   dal SUO micro isolante, mai dal 27% né dalla «tariffa» 9-10 ns/op); Pedersen A-PE-102-6 (H-C1
   tocca clone/drop/gc_note = la classe di retention WP-78 ⇒ sentinella estesa + endurance nel suo
   ordine di collaudo).

## Conflitti (posizione per sedia)

- **Verdetto su S-100.** Gregg: sessione d'oggetto piena, NESSUNA refutazione capitale (flip
  giudicato da misure). Leijen: DUE capitali (etichetta «cross-albero» non provata; peak del default
  = numero del candidato). Pedersen: UNA capitale (parità server bimodale senza controllo positivo).
  Non è conflitto di merito — domini disgiunti (oggetto vs contabilità footprint vs confine server) —
  ma i tre verdetti restano distinti e vincolanti ciascuno nel suo dominio.
- **Calibro del rumore peak.** Gregg calibra sulla cifra esistente (+14,6% intra-sera oracle);
  Leijen esige spread R≥5 fresco prima di OGNI banda futura (senza = VOID) e nota che il peak è una
  statistica di MASSIMO (coda destra: R=2 non calibra). Risoluzione di team: forma Leijen (più
  severa) come regola, cifra Gregg come stima provvisoria MAI usabile da sola come banda.
- **Ordine della bozza §S-101.** Gregg APPROVA l'ordine (misura-first, ri-baseline al punto 1) con
  emendamenti; Leijen chiede di RIFORMULARE il punto 4 (attribuzione) prima dell'esecuzione: A/B
  pin sigillati stessa-sera PRIMA del bisect, census allocatore prima del bisect (il census dà il
  CANALE, il bisect solo il commit a 5× il costo). Composizione: l'ordine regge, il punto 4 si
  riscrive nella forma A-LE-102-1 — nessuna incompatibilità residua.
- **Sigillo eager dei due env lazy (Pedersen R4).** Nessuna sedia si oppone, ma è apparato che non
  blocca la ri-baseline: il team lo colloca al prossimo tocco del pin server (vedi priorità), non
  nell'ordine bloccante — Pedersen lo accetta per costruzione (KS-PE-102-2 vincola i NUOVI `PHPR_*`,
  non impone la data del sigillo sugli esistenti).

## Priorità per l'ordine S-101

**BLOCCANTI (per NOME, in quest'ordine):**

1. **Ri-baseline sei-categorie in modo default** (bozza punto 1) — prerequisito del criterio H-C1
   (A-GR-102-4 vincolante).
2. **Attribuzione «crescita d'albero» riformulata** (ex punto 4): prima mossa = **A/B stessa-sera
   dei due pin sigillati S-99 (`phpr-s99-sigillo`) vs S-100, stesso ambiente** (A-LE-102-1); se il
   delta si riproduce ⇒ census allocatore (vmmap per fase + stats mimalloc,
   `MIMALLOC_PURGE_DELAY=0`) PRIMA del bisect. **KS-LE-102-1: delta < 2× spread intra-sera ⇒ voce
   riclassificata AMBIENTE, bisect VIETATO.** Entrambe le gambe si misurano (A-LE-102-2).
3. **Calibrazione rumore full-peak** R≥5 per motore, mediana + spread pubblicato, bande d'ora in
   poi unilaterali (A-LE-102-3 + A-GR-102-3). KS: banda senza spread pubblicato = VOID
   (KS-LE-102-2, KS-GR-102-2).
4. **Smoke peak media sul pin promosso 725a2ffa SUBITO** (~1 min; full alla prossima coppia); se
   fuori dallo spread calibrato del candidato ⇒ il «1929,0» si RITIRA dalla rotazione e il flip si
   ricollauda sul footprint (A-LE-102-4, KS-LE-102-3). Ogni peak in rotazione porta l'hash del
   binario.
5. **Ri-collaudo del pin nuovo col mode-probe** (A-PE-102-1, GIÀ implementato): i launcher
   ASSERISCONO il modo effettivo per braccio (non solo log); hash phpr in gamba B promosso a GATE
   (A-PE-102-5). KS-PE-102-1: nessuna cifra «server modo X» senza asserzione del modo effettivo.
6. **Se H-C1 si scrive in S-101** (condizionale ma non opzionale una volta attivato): (a) census
   dinamico op-census valida il 18 vs 9 (A-GR-102-1); (b) soglia di caduta dal micro isolante ≥
   pavimento sonda (KS-GR-102-1); (c) sentinella estesa + **endurance N≥100 richieste su endpoint
   allocante + RSS/footprint server inizio/fine con banda calibrata** nel suo ordine di collaudo
   (A-PE-102-3 + A-PE-102-6).

**BACKLOG per NOME (apparato-solo-se-blocca):**

- **A-PE-102-2** — sigillo eager + value-parse a lista chiusa per `PHPR_STUB_ELISION` e
  `PHPR_UNIT_CACHE` (vm/mod.rs:15718/15727) + `PHPR_REQ_NS` (worker_pool.rs:155); chiude
  A-PE-101-2 per NOME. Si attiva al prossimo tocco/ri-pin del server (cambia il binario ⇒ nuovo
  pin comunque).
- **A-PE-102-4** — una gamba WP VERA servita da php-server via HTTP; fino ad allora la gradazione
  NON scrive «server-HTTP» per la gamba B (che resta CLI phpunit, correttamente nominata).
- **A-GR-102-2** — riprofilo inline-aware del 50% run_loop (frame pointers o `#[inline(never)]`
  temporaneo) per dare a H-C1 una BANDA di meccanismo; diventa bloccante solo al momento del
  GIUDIZIO di H-C1.
- **R5-Pedersen / `--build-info`** — resta nel backlog aperto A-HO-101-4/A-PE-101-5 (spawn
  fuori-launcher non collaudato); nessun nuovo apparato finché non blocca una cifra pubblicata.
- **Residui sentinella (R3-Pedersen)** — worker-id nel burst, status HTTP asserito, restapi vera al
  posto di «restapi-shaped»: si compongono nell'endurance quando A-PE-102-3 si attiva, non come
  atti separati.

**Regola apparato-solo-se-blocca applicata**: endurance, riprofilo inline-aware e sigillo eager
sono NOMINATI con la loro condizione di attivazione; nessuno entra nell'ordine bloccante finché la
condizione non scatta.

════════════════════════════════════════════
## FILE: verbali/SYNTHESIS.md

# Concilio WP-102 — SINTESI DI CONVERGENZA (su S-100 e programma S-101)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: Gregg (mandato
inverso) dà PASS pieno — «sessione d'oggetto piena». Nuovo per NOME: il
FLIP è ESEGUITO E MISURATO (flag-on default, corpus 1418×2 per NOME sul
pin, diff per-test ZERO, server bimodale, coppia WP nei 2 modi con bande);
H-B2 chiusa con misura (estensione, L=12,9→[−1,0]; add on −31%); H-C
DECOMPOSTA (12,4 = conteggio 2,0 × costo/op 6,2) con profilo co-equale e
simboli per NOME; due divergenze semantiche nuove a catalogo.

**(b) Contatore sessioni-senza-misura**: full/media = **WP-100 = QUESTA
sessione (0)**; giudice: prop e add freschi post-flip (le altre quattro
categorie da ri-baseline in modo default — primo atto S-101).

**(c) Rischio d'oggetto più trascurato**: la crescita d'albero del peak
(voce RINOMINATA dal concilio: **OFF +95 / ON +36,5 MiB** vs WP-99, «cross-
albero» era una conclusione non ancora provata contro l'ambiente) è senza
attribuzione; e il costo/op quasi-invariante ~9-10 ns resta senza
decomposizione DENTRO run_loop (il 50% del profilo è un simbolo solo).

## Verdetti di fase 1 (9/9: nessun MI OPPONGO; capitali per convergenza)

Verbali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali:

1. **`emit_binary` leggeva `enabled()` globale, non `ctx.reg_lower`**
   (Hoare A-HO-102-1 + Hejlsberg, convergenti indipendenti; mod.rs:779):
   «il modo è un INPUT del funnel» era falsificato da un sito residuo — il
   braccio OFF in-process dei test emetteva `Binary(Add)` sotto default ON
   invece dell'emissione di produzione OFF. **SALDATA IN SESSIONE**: fix a
   `self.ctx.reg_lower`, denti mirati verdi, batteria 1735/0, pin ruotati
   (b618e3a).
2. **«Il flip cambia solo la costante» è falso** (Klabnik): fb861e4 ricabla
   l'entry (ProgramCtx) — l'evidenza del diff per-test va giudicata SUL
   PIN (A-KL-102-1). **IN SALDO IN SESSIONE**: corpus-diff ri-eseguito sul
   pin post-fix (copre anche il collaudo del fix n.1).
3. **La parità bimodale server non provava il modo effettivo** (Pedersen
   A-PE-102-1): fails=0 nei due bracci è indistinguibile da env mai
   propagato. **SALDATA IN SESSIONE**: mode-probe nella sentinella (dump
   dell'unità nel log del server, atteso dal contratto), esercitata nel
   ri-collaudo del pin.
4. **H-C1 «prestito al posto del clone» punta al canale sbagliato**
   (Bak + Matsakis + Stogov, convergenti): su prop.php i valori sono Long
   (copy senza alloc) — il churn nominato dal profilo è il RICEVITORE
   (Rc dell'oggetto + gc_note), prior art `ThisPropGet`; Zend fa
   copy+addref CONDIZIONALE, mai borrow nudo. **RIFORMULATA dal team
   hc-canale** (sotto), il borrow nudo dello slot valore è VIETATO
   all'unanimità.
5. **«Cross-albero» non provato + peak pubblicato per un binario mai
   misurato** (Leijen): l'attribuzione esige A/B dei pin stessa-sera PRIMA
   del bisect; ogni cifra peak nomina l'HASH del binario che l'ha
   prodotta. Voce rinominata: «crescita d'albero OFF+95 / ON+36,5».
6. Metodo (Gregg, non capitali): la «tariffa» 9-10 ns/op viene da 2 punti
   (arith, prop) — banda, mai coefficiente; il 27% Zval è un PAVIMENTO
   (run_loop inlinea altro ciclo-vita); la sanity 2,0×6,2=12,4 è esatta
   per costruzione — serve il census dinamico a validare lo statico.

## Riformulazione H-C1 (team hc-canale, VINCOLANTE per S-101)

H-C1 sostituita da FORMA A STADI, ognuno col suo controfattuale e criterio:
- **H-C1a**: bypass del bookkeeping per SCALARI (niente gc_note/clone-
  contabile per Long/Double/Bool/Null) — misurabile da sola.
- **H-C1b**: prestito/addref del RICEVITORE à la `ThisPropGet` (sigillo di
  tipo sul borrow; fixture aliasing/hook/__get/ref/readonly PRIMA).
- **H-C1c**: copy+addref condizionale stile Zend per i valori refcounted.
Prerequisiti di misura PRIMA di ogni riga: ri-baseline sei categorie in
modo default; census dinamico specie×sito×canale su prop.php nei DUE motori
con TRE predizioni pre-registrate; TETTO scritto nell'ordine: il successo
pieno di H-C1a-c lascia ~9× (KS-BA-102-1/KS-KL-102-2) — H-C1 NON chiude
H-C da sola, chi presenta il contrario è refutato in anticipo.

## Ordine DEFINITIVO S-101 (regola di ammissione applicata)

1. **Ri-baseline sei categorie IN MODO DEFAULT** sui due motori, stessa
   finestra (i numeri che giudicano tutto il resto).
2. **Census dinamico specie×sito×canale** su prop.php (due motori,
   predizioni pre-registrate) + decomposizione inline-aware del 50%
   run_loop (A-BA-102-2/A-GR-102-2): il census DINAMICO valida lo statico.
3. **H-C1a → b → c** nella forma a stadi (fixture semantiche PRIMA,
   criterio per stadio, tetto dichiarato); gate cumulativi ad ogni stadio:
   batteria + corpus 2 modi + diff per-test; WP pair di parità NON
   derogabile se cambia il runtime (KS-KL-102-3/KS-MA-102-4).
4. **Attribuzione crescita d'albero**: A/B pin S-99↔S-100 stessa-sera
   (prima del bisect) + census allocatore; bande peak UNILATERALI
   calibrate sul rumore misurato per-motore (R≥5, mediana, spread
   pubblicato; banda < rumore ⇒ VOID, KS-GR-102-2/KS-LE).
5. **Denti residui di rotazione** (piccoli, bloccano la fiducia nei gate):
   dente in-process BinaryAdd sul braccio OFF (A-HE-102-1); dente
   absent≡`=1` (A-KL-102-3); smoke peak sul pin quando una cifra peak
   entra nel registro (A-LE-102-4).
6. (timebox) H-D prima misura: stessa tavola su calls.php col census
   bi-regime (A-ST-102-5).

**BACKLOG per NOME** (non slot di sessione): A-HO-102-2 (sigillo ZST),
A-HO-102-4 (corpo condiviso BinaryAdd≡Binary(Add)), A-HE-102-2 (batteria
nei 2 modi espliciti), A-HE-102-3 (destructuring CompiledClass),
A-HE-102-5 (modo nel Module — chiude A-HE-101-3), A-HE-102-6 (dump
ereditarietà), A-KL-102-2 (carve-out per token + prova entropia flag-on),
A-PE-102-2/3/4 (sigillo STUB_ELISION/UNIT_CACHE; endurance N≥100; WP vero
via HTTP), A-ST-102-1/2 (riscrivere §3.12 mode/typed-prop col meccanismo C
verificato; §3.11 famiglia fetch-undef), A-BA-102-3 (fixture per specie),
A-GR-102-3 (bande asimmetriche), --build-info (A-HO-101-4/A-PE-101-5).

## Conflitti registrati

- forma del fix emit_binary (Hoare: variante forte incondizionata; il
  fix applicato usa ctx — registrato, non bloccante).
- priorità del sigillo ZST (Hoare: S-101; relatore team flip-residuo:
  backlog — resta BACKLOG per regola di ammissione).
- prestito del ricevitore (Matsakis: borrow col sigillo di tipo; Stogov:
  addref; team: si decide con le fixture — chi morde vince).
- verdetti divergenti su S-100 (Gregg 0 capitali, Leijen 2, Pedersen 1):
  domini disgiunti, nessuna contraddizione di merito.

════════════════════════════════════════════
