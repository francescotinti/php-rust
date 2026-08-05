# COUNCIL_WP100_REVIEWS — Concilio a 9 sedie su S-98.0 + programma S-99.0 (2026-08-05)
# Protocollo a due fasi (regola utente 2026-08-02): fase 1 = verbali INDIPENDENTI (vincolanti);
# fase 2 = note di team tematiche (criterio-rollout, ordine, evidenza); sintesi in coda.



---

# Verbale sedia 1 — Hoare (design linguaggio/runtime Rust, safe-only) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

## 1. Handler `Op::BinaryAdd` (run.rs ~944-975) — forma Rust SANA

Il pattern è corretto: `pop` di rhs (owned), `last()` immutabile per la
guardia con estrazione per copia dei due `i64` (il borrow muore prima di
`last_mut()`), scrittura in-place che sovrascrive un `Zval::Long` — drop
banale, nessun Rc, nessun aliasing possibile. L'overflow promuove sul
valore ORIGINALE (`l as f64 + r as f64`), fedele a `binary_fast` e a Zend.
Il MISS rifà `pop lhs` e rientra INTEGRALE in `binary_value_ab` con lo
stesso ordine di pop del braccio generico («pop rhs then lhs»): diags,
overload GMP/Number ed errori identici per costruzione. Gli `expect` sono
invarianti di compilazione, coerenti con lo stile di casa (CmpJmpConst).
Nessuna refutazione qui.

## 2. Sonda M1 e verdetto «preambolo gratis» — verdetto LEGITTIMO, con due debiti

Il ragionamento cammino-critico batte il conteggio di istruzioni: giusto.
MA:

- **A-HO-100-1 «Fedeltà assoluta della sonda dichiarata»**: main.rs
  promette che la forma A «si valida contro i 6,27 ns/op»; la sonda ne fa
  3,94 — un buco di ~2,3 ns/op che vive esattamente nelle parti NON
  replicate (Zval enum vs i64, drop glue, spill dei caldi). Il verdetto
  regge SOLO grazie al tetto statico anti-hiding (0,55 ⇒ P≤6,7%<10%):
  questo va scritto nel .out — la fedeltà è STRUTTURALE (ASM), non
  assoluta.
- **A-HO-100-2 «Nominare la catena ip come cammino critico»**: la KS «no
  ip locale» preserva lo store→load forwarding di `frames[top].ip` (4-5
  cicli, loop-carried): è il candidato più plausibile a LIMITARE il loop,
  ed è ciò che rende gratis il resto del preambolo. Il verdetto refuta
  H-B1-sotto-KS, non ogni ottimizzazione del preambolo: la clausola di
  riapertura («dato nuovo di cammino critico») deve nominare QUESTA catena.

## 3. REFUTAZIONE CAPITALE — punto 3 dell'ordine S-99.0

**«Criterio pre-registrato in ns/occorrenza derivato da D=6,07» è
illegittimo per le forme registro.** D=6,07 misura il plumbing del
percorso PILA: call con due Zval owned da 24B per valore + pop/push +
match del payload. `BinarySS/SC/Dst` quel plumbing in gran parte NON ce
l'hanno già (binary_fast su BORROW dei slot, zero pop/push, zero
marshalling owned): trasportare 6,07 come àncora produce o una soglia
irraggiungibile o un tetto finto. Il criterio va ri-derivato da un
controfattuale statico NUOVO del corpo di BinarySS (che cosa resta da
togliere: il match su BinOp + la call binary_fast), PRIMA del codice —
**A-HO-100-3**.

## 4. Rollout H-B2 nelle forme registro — rischi di forma

- **A-HO-100-4 «Census dei Binary(Add) residui flag-off»**: `emit_binary`
  specializza, ma ogni sito che chiama `emit(Op::Binary(Add))` DIRETTO
  resta generico in silenzio (buco di perf, non di parità). Il controllo
  positivo copre solo due micro: serve un dente dump-based su fixture più
  larghe (zero `Binary(Add)` flag-off nel {main} e nelle funzioni).
- Ogni specializzazione dentro le forme registro scrive su SLOT, non su
  pila: l'in-place attraverso un `Zval::Ref` corromperebbe l'alias.

## Kill-switch

- **KS-HO-100-1**: il hit-path specializzato nelle forme registro DEVE
  conservare la guardia `Undef|Ref` esistente ed essere equivalente a
  `binary_fast` sugli stessi tag; MISS → funnel owned integrale; VIETATA
  la scrittura in-place su slot senza il deref-funnel.
- **KS-HO-100-2**: al primo cambio di emissione flag-on, il claim
  «flag-on bit-identico» DECADE e non è più citabile; corpus flag-ON per
  NOME + census flag-on ri-pinnati NELLA STESSA sessione, pena VOID della
  misura.

**Firmato**: sedia 1 (Hoare), 2026-08-05.


---

# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

## Perimetro 1 — disciplina di borrow del handler `Op::BinaryAdd` (run.rs 944-975): REGGE

Il handler è NLL-pulito. `pop()` produce un `rhs` POSSEDUTO; la guardia
`match (self.frames[top].stack.last(), &rhs)` copia `(*l, *r)` in `fast:
Option<(i64,i64)>`, quindi il borrow condiviso della pila TERMINA prima di
`last_mut()` — nessuna finestra in cui un ri-borrow invalidi. Il fallback
ri-entra in `binary_value_ab(&mut self, …)` con `lhs`/`rhs` GIÀ mossi fuori
dalla pila: zero borrow pendenti su `self` al confine di ri-entrata. Ordine
dei pop (rhs poi lhs) e panics (`expect`) IDENTICI a `binary_value`
(run.rs 213-216): equivalenza del funnel verificata sul corpo, non sulla
dichiarazione. Il hit-path è verbatim `binary_fast` (Long,Long) Add,
overflow sugli operandi ORIGINALI compreso.

**Riserva strutturale (non capitale)**: tra `pop(rhs)` e la scrittura
in-place la pila è in stato intermedio, non osservabile SOLO perché non c'è
alcun `?`/diag/drop non banale nel mezzo (Long ha drop triviale). È un
invariante di forma, non di tipo: una futura diag inserita lì lo rompe in
silenzio → A-MA-100-3.

## Perimetro 2 — forma B della sonda: il verdetto negativo TRASFERISCE, a fortiori

La sonda dà a B il suo caso MIGLIORE: nel programma noop nessun op caldo
attraversa il confine, e i 16 bracci freddi fanno `continue 'outer` senza
nemmeno ri-prendere `frames` (il macro ignora l'ident). La forma vincolata
dal Concilio WP-99 ha confine a OGNI opcode con metodi `&mut self` — nel
run_loop vero quasi ogni handler caldo (Binary→binary_value, CmpJmp,
IncDecSlot generico) — quindi la B reale è ≤ della B della sonda, che perde
già 0,53 ns/op sotto pressione. Un verdetto negativo su un bracket
FAVOREVOLE trasferisce nella direzione refutante per costruzione. Nessuna
refutazione possibile: la caduta a tavolino di H-B1 sta.

## Perimetro 3 — `bin_op_of` come sinonimo: DUE REFUTAZIONI CAPITALI

**R-1 (capitale, sul programma S-99 punto 3)**: la frase «BinarySS/SC/Dst
hanno lo stesso plumbing generico da togliere» (hb2-addspec.out, ripresa in
NEXT_SESSION) è FALSA sul codice. Il fast-path di `BinarySS` (run.rs
1011-1019) prende ENTRAMBI gli slot per PRESTITO e chiama `binary_fast`,
che è `#[inline(always)]` col braccio (Long,Long) Add già dentro: niente
call, niente marshalling di Zval per valore, niente pop/push. I 6,07
ns/occorrenza di D misurano il plumbing del percorso STACK; nel percorso
registro il residuo rimovibile è il solo carico+match del payload `BinOp`.
Un criterio del rollout «derivato da D=6,07» è quindi miscalibrato per
costruzione: sovrastima il tetto di un fattore ignoto e farà «cadere» o
«passare» la leva sul numero sbagliato.

**R-2 (capitale, latente)**: l'equivalenza BinaryAdd ≡ Binary(Add) vive in
UN punto (`bin_op_of`, reg_lower.rs 189-195) che ha il braccio `_ => None`.
Il dividendo dei match esaustivi conquistato in WP-96 («una variante nuova
di Op non compila») è VACUO proprio qui: BinarySub/BinaryMul del rollout
S-99 compileranno senza toccare `bin_op_of`, e la pipeline mista della
batteria perderà le fusioni in silenzio (divergenza perf-only che nessun
corpus di parità può mordere). La cura enumerabile manca dove serve.

**Nota (non capitale)**: «guardia CONTATA» è stato reinterpretato come
«contata nel tempo totale» su micro sempre-hit; il census conta le
OCCORRENZE di BinaryAdd, non il hit/miss. Il rollout eredita D misurato a
hit-rate 100%.

## Emendamenti

- **A-MA-100-1**: il criterio del punto 3 S-99 NON si eredita da D=6,07:
  si pre-registra un tetto NUOVO misurato sul giudice flag-on (forma
  registro), col controfattuale statico rifatto sul residuo vero (match
  del payload, non call+marshalling).
- **A-MA-100-2**: `bin_op_of` perde il wildcard o guadagna un test pinnato
  che enumera le grafie specializzate: un nuovo opcode aritmetico
  specializzato deve NON COMPILARE (o far fallire la batteria) senza una
  decisione esplicita lì.
- **A-MA-100-3**: commento-invariante + debug_assert nel handler BinaryAdd
  («nessuna operazione fallibile tra pop(rhs) e la scrittura in-place»);
  census con split hit/miss prima di derivare criteri di famiglia.

## Kill-switch

- **KS-MA-100-1**: VOID ogni spedizione di Add-nei-registri il cui
  criterio sia derivato da D=6,07 invece che misurato flag-on.
- **KS-MA-100-2**: VOID la garanzia di pipeline mista della batteria se un
  opcode specializzato nuovo compila senza toccare `bin_op_of` o il suo
  test di enumerazione.


---

# Verbale Sedia 3 — Klabnik (spec, testabilità, matrici, gate) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO
**S-98.0: RIFIUTATO NELLA FORMA** — le misure (M1, D_add=6,07) reggono, ma
due claim di evidenza sono SOVRADICHIARATI (refutazioni 1 e 2). **S-99.0:
EMENDATO** — il punto 4 è sottodimensionato rispetto al tag-space di Add.

## Refutazioni capitali

**R1 — Il «controllo positivo» del funnel controlla il DUMP, non l'OUTPUT.**
`reg_lower_funnel.rs` asserisce solo `off_out == on_out` + tre forme nel
chunk `{main}`: non pinna il valore atteso dello stdout né l'exit status.
Un binario che sbaglia UGUALE nei due modi (o che fatala a runtime DOPO il
dump di compile-time) passa il test. Il claim «controllo positivo sul
{main}» in NEXT_SESSION è quindi più forte del test scritto. Inoltre la
guardia M5 vive in UN solo test: un `cargo test behavioral_parity` filtrato
con `PHPR_REG_LOWER` esportato compila già lowered in `compile()` e applica
il pass DUE volte via `lowered()`, in silenzio.

**R2 — Il gate per NOME è UN BIT per test fallito.** Ho verificato io:
flagoff = flagon = canone wp82, byte-uguali dopo sort (1418) — la catena
NON è induzione, i tre anelli sono confrontati davvero. Ma l'identità del
SET non dice nulla sul CONTENUTO dei 1418 fallimenti: un test che fallisce
con un diff DIVERSO tra flag-off e flag-on è invisibile al gate. Per la
parità di regressione va bene; come **gate di PROMOZIONE è insufficiente**
(risposta a M4: serve il riga-per-riga degli output effettivi dei fail).

**R3 — La matrice smoke di BinaryAdd copre 5 celle di un tag-space ≥12.**
Coperti: overflow positivo, "2"+3, 2.5+1, [1]+[2], concat. Mancano per
NOME (tutti percorso MISS→fallback, dove vive il rischio di ordine
pop/diag): **Ref** (slot alias su `+=`, coda Swap→BinaryAdd), **Bool**,
**Null** (null+1 silente), **Str non-numerica** ("abc"+3: TypeError che
nomina gli operandi IN ORDINE — la stessa classe del fold commutativo
rimosso), `" 2"+3`, string numerica oltre int-range, **overflow negativo**
(PHP_INT_MIN), Long+Double nei DUE ordini, **GMP e BcMath\Number overload**
(A-ST-99-2 lo nomina ma S-99 lo mette in un timebox ½ sessione col resto),
resource+int, undefined-var nel miss path.

**R4 — Il budget cifre alzato DUE volte in sessione (24561→24643→24645).**
Il primo rialzo è forgia legittima (fonti nuove, atto deliberato A-SK61).
Il secondo (+2, «fix decimale») è il gate ri-adattato all'artefatto DOPO
che ha morso: escape hatch usato due volte = inizio di abitudine. Il file
contiene UN numero nudo: l'audit richiede archeologia git.

## Emendamenti
- **A-KL-100-1**: `reg_lower_funnel.rs` pinna lo stdout atteso (letterale)
  e l'exit status; la guardia anti-env M5 sale nell'helper condiviso.
- **A-KL-100-2**: gate di promozione M4 esteso — diff riga-per-riga degli
  output dei 1418 fail flag-off vs flag-on, salvato come evidenza.
- **A-KL-100-3**: matrice fixture Add per NOME (elenco R3) nella batteria
  PRIMA del rollout Sub/Mul; diventa il template della famiglia.
- **A-KL-100-4**: il budget porta il breakdown per fonte nel file stesso;
  un secondo rialzo in-sessione enumera i token nuovi, non solo la causa.
- **A-KL-100-5**: debito B2 (Hejlsberg) in ordine S-99 punto 4 — i 13
  snippet della parità comportamentale rigiocati al funnel VERO (binario
  spawannato); oggi al funnel è provata UNA forma di controllo, zero
  diagnostiche, zero eccezioni, zero funzioni.

## Kill-switch
- **KS-KL-100-1**: promozione flag-on→default VOID senza l'artefatto
  riga-per-riga di A-KL-100-2.
- **KS-KL-100-2**: ogni nuova specializzazione della famiglia (Sub/Mul,
  coda AssignOp) VOID finché la matrice A-KL-100-3 non è nella batteria.
- **KS-KL-100-3**: terzo rialzo del budget in una stessa sessione senza
  enumerazione dei token = gate dichiarato non-mordente, SELFTEST da
  rieseguire prima di ogni altro uso.


---

# Verbale Sedia 4 — Hejlsberg (pipeline di compilazione) — Concilio WP-100

**Oggetto**: report S-98.0 (H-B2 spedita) + programma S-99.0. **Mandato**: refutare.

## VERDETTO
La spedizione H-B2 è coerente sul suo asse (nessun punto in cui un'unità compilata in un modo serve l'altro: `UnitKey.reg_mode` da `enabled()` a vm/mod.rs:16058, `main_chain_fp(reg_mode)` belt-and-braces, `enabled()` OnceLock per-processo condiviso da emissione e pass). Ma il funnel dichiarato «unico» ha DUE gambe già divergenti e il programma S-99.0 porta un criterio mal derivato e un pass con un tombino aperto. **CON EMENDAMENTI VINCOLANTI; refutazioni capitali: sì (3).**

## Refutazioni capitali
**RC-1 — Il contratto del dump è VIOLATO oggi, flag-on.** `compile_hook` (func.rs:325) passa da `compile_body` ⇒ i corpi hook **vengono riscritti** dal pass; `dump_module_ops` li esclude con la clausola «a stage that rewrites those needs to widen this first» — la condizione è violata dallo stage corrente. Corpi riscritti in produzione, invisibili al dump E al battery in-process.

**RC-2 — Il helper `lowered()` NON rispecchia il funnel che dichiara.** Abbassa `prop_init`, ma `compile_prop_init` (func.rs:361) costruisce `Func` a mano SENZA il gate `enabled()` ⇒ in produzione prop_init non è mai lowered. Il battery testa una pipeline che la produzione non esegue (prop_init lowered) e non testa una che esegue (hook lowered). È la «doppia fonte di verità» a mano: ha già derivato in entrambe le direzioni.

**RC-3 — Il criterio del rollout S-99 punto 3 è derivato dal controfattuale SBAGLIATO.** D=6,07 ns/occ è il risparmio sul percorso PILA (call + marshalling 2×Zval + pop/push + Dup elisi). BinarySS/SC/Dst leggono slot e scrivono dst: il pop/push è GIÀ eliso; il rimovibile lì è solo call+match, strettamente minore. Un criterio «derivato da D=6,07» sovrastima e mis-calibra la caduta: va ri-registrato dal controfattuale per-forma.

## Punti del perimetro
**(b) `bin_op_of` (reg_lower.rs:189)**: non è lettera morta — il battery in-process compila flag-off (emette `BinaryAdd`) e applica il pass a mano: quel braccio È il suo unico esercizio. Ma legittima una pipeline mista che la produzione vieta, e **neutralizza un tripwire**: un leakage cross-mode che facesse comparire `BinaryAdd` sotto flag-on verrebbe riscritto in silenzio invece di fallire. Il controllo di sessione (census flag-on ZERO BinaryAdd) era one-shot, non un test permanente. L'equivalenza «BinaryAdd IS Binary(Add) by construction» vive in un commento, non in un test differenziale.

**(c) Siti d'emissione**: tutti e 5 i siti che producono `Op::Binary` passano da `emit_binary` (expr.rs:102,159; assign.rs:960,976,988). Unica emissione diretta: `Binary(NotEq)` (expr.rs:217) — sana, ma la porta non è recintata. Per NOME, portatori di Add FUORI da Op::Binary (funnel generico, dichiarati non mancanti): `FieldAssignOp{op}` (assign.rs:968), `AssignOpPath{op}` (assign.rs:1001); IncDec e coalesce non trasportano BinOp.

**(d) Shape nuove**: Add int-int nelle forme registro non porta `Addr` ⇒ remap e `visit_addrs` reggono senza riscrittura; il pin census «BinaryAdd chiude la tabella» morde consapevolmente. MA `visit_addrs` ha `_ => {}` (reg_lower.rs:74): l'«unica autorità» si difende con un commento, contro la dottrina WP-96 (variante nuova = NON COMPILA). La coda AssignOp in backlog può portare shape con Addr: il tombino va chiuso PRIMA.

## Emendamenti
- **A-HE-100-1**: assert permanente in `reg_lower_funnel.rs`: dump flag-on del `{main}` con ZERO `BinaryAdd` (ripristina il tripwire tolto da bin_op_of).
- **A-HE-100-2**: `visit_addrs` esaustivo (niente `_ => {}`), o dente census-driven che ogni variante Addr-bearing è visitata.
- **A-HE-100-3**: test differenziale permanente `BinaryAdd ≡ Binary(Add)` (output+diags: overflow, coercizioni, fallback MISS) — oggi è un commento.
- **A-HE-100-4**: sanare RC-1/RC-2: allineare dump + `lowered()` al funnel VERO (hook dentro, prop_init o gated o dichiarato fuori), con un test che enumeri i corpi dal Module, non a mano.

## Kill-switch
- **KS-HE-100-1**: promozione flag-on a default VIETATA finché `enabled()` resta lazy senza dente anti-putenv/eager-init — è gate di promozione, non residuo del punto 4.
- **KS-HE-100-2**: nessuna shape registro nuova finché `visit_addrs` non è esaustivo (A-HE-100-2).
- **KS-HE-100-3**: rollout punto 3 VOID se il criterio resta derivato da D=6,07 anziché dal controfattuale ricalcolato sulle forme registro (RC-3).


---

# Verbale Sedia 5 — Bak (microarchitettura) — Concilio WP-100 su S-98.0 / programma S-99.0

**VERDETTO: i due esiti (H-B1 a tavolino; H-B2 spedita) SOPRAVVIVONO, ma NON
per le ragioni scritte nei `.out`** — tre refutazioni capitali sul
ragionamento microarchitetturale, e il punto 3 dell'ordine S-99.0 (criterio
rollout «derivato da D=6,07») è **NON DERIVABILE** così com'è.

## Refutazioni capitali

**R1 — La sonda non misura Δ(A_reale, B).** Nell'ASM della sonda
(`m1-probe-loops.asm`, forma A) `frames.len` vive in **x23 (registro)** e ptr
sta dietro UN load da `[x21,#8]`; nel run_loop vero
(`m1-runloop-preamble.asm`) len e ptr arrivano da **catene di DUE load
dipendenti** (`[sp,#0x2c0]→[x8]`, `[sp,#0x2c8]→[x8]`) che alimentano il madd
dell'indirizzo frame → ldr ip → madd op → ldrb tag. La sonda ha misurato
Δ(A_sonda, B), sistematicamente più piccolo del vero. Il verdetto P<10% regge
SOLO grazie al tetto anti-hiding — vedi R2.

**R2 — Il tetto anti-hiding è auto-contraddittorio.** D_tetto = 1,4 × 11/28 è
una ripartizione per CONTEGGIO di istruzioni: esattamente il metodo che la
STESSA sessione mette nei NON-riproporre. Sopravvive per fortuna aritmetica
(6,7% < 10%); con soglia 8% il verdetto sarebbe stato deciso da un numero
costruito col metodo vietato. Ogni futura P deve venire da un argomento di
CATENA DI DIPENDENZE: il preambolo è fuori dal cammino del VALORE perché la
jump table è servita dal predittore indiretto (il noop a 5 bigrammi fissi è
pasto per un TAGE; i load servono solo alla verifica del retire) — non da
quote di istruzioni.

**R3 — D=6,07 è 2,2–4,3× il SUO tetto statico** ([1,4–2,8] ns pre-registrato
nello stesso `.out`): il meccanismo NON è quantitativamente spiegato.
Candidati non separati: (i) prologo/epilogo di `binary_value_ab`
(callee-saved, ~10 store+load per chiamata); (ii) marshalling — su arm64
AAPCS un composito >16B passa per **puntatore a copia**: ogni Zval è store
24B + load nel callee, e il ritorno `Result` è sret ⇒ 3–4 salti
store→load-forwarding SERIALI (4–5 cicli l'uno) sul cammino del VALORE — è
per questo che il plumbing di chiamata paga dove il preambolo era gratis;
(iii) pop/push del `Vec` (load len, branch, store len, store/load elemento).
6 ns ≈ 19 cicli: plausibile come somma, MAI decomposto. Senza decomposizione
il criterio del rollout è un numero copiato, non derivato.

**R4 (minore) — Il −0,53 della forma B (build 2) è un artefatto della
sonda**: testa di bounds-check DUPLICATA (ingresso + back-edge, righe 39–51
dell'ASM), allineamento/BTB. La lezione corretta è «la sonda ha un pavimento
di rumore ~0,5 ns/op», NON «split-borrow perde». Il NON-riproporre di H-B1
resta valido sul solo P<10%.

## Punto (c): quota di D trasferibile alle forme registro

BinarySS/SC/Dst NON hanno pop/push: la quota (iii) sparisce per costruzione.
Restano (i)+(ii) se anche le forme registro chiamano `binary_value_ab`.
Quota trasferibile: plausibile 50–70%, ma è stima da tetto, **non misura** —
ignota finché R3 non è decomposta.

## Emendamenti

- **A-BA-100-1**: PRIMA del rollout flag-on, UNA build intermedia che
  decompone D (es. `#[inline(always)]` del solo ramo Add nel funnel, tenendo
  pop/push): separa (i)+(ii) da (iii); il criterio delle forme registro si
  pre-registra dalla quota MISURATA.
- **A-BA-100-2**: annotare in `m1-preamble.out` che il tetto anti-hiding è
  count-derived (R2) e che la sonda sotto-misura Δ (R1); la lezione «reload
  gratis» si riscrive «non misurabile sotto ~0,5 ns con questa sonda su
  questo core».
- **A-BA-100-3**: nel collaudo arith flag-off di S-99, profilo a campioni sul
  loop-head: se il preambolo pesa >7% del tempo, M1 si riapre (il noop
  generalizza all'arith solo se i port L1 non saturano — i corpi Zval li
  contendono).

## Kill-switch

- **KS-BA-100-1**: VOID ogni criterio di rollout copiato da D=6,07 senza la
  decomposizione A-BA-100-1; ogni occorrenza (Sub/Mul int-int, forme
  registro) porta il SUO controfattuale statico con la quota pop/push
  sottratta.
- **KS-BA-100-2**: VOID ogni claim della sonda M1 sotto 2× il suo pavimento
  (1,0 ns/op): si pubblica come banda, mai come verso.


---

# Verbale sedia 6 — Pedersen (confini per-richiesta, lifecycle, ambiente) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI — una refutazione capitale sull'ORDINE di S-99

### Refutazione capitale R1 — L'ordine 1→2 di S-99 viola KS-PE-99-1 per costruzione

Il punto 1 (collaudo full+media WordPress col launcher) **usa php-server**: full e media girano dal WP-77 attraverso Axum, e ogni coppia full storica cita il pin server accanto a quello phpr. Ma KS-PE-99-1 dice «VOID **ogni uso/misura** del server prima di restapi+option per NOME sotto env -i» — e il pin 365f4d4069513de3 non è collaudato. Eseguire il punto 1 prima del punto 2 produce un collaudo VOID dal suo stesso kill-switch. **L'ordine va invertito: parità server PRIMA del collaudo WP** (o il punto 1 deve dichiararsi CLI-only, cosa che non è). Il debito è doppio: 832568a7 mai collaudato, 365f4d40 mai collaudato, d45b578 non riproducibile — **due rotazioni di pin senza un solo collaudo**, e il 365f4d40 è nato come *effetto collaterale* di una build, non da un ordine.

### Refutazione R2 — Il canale putenv è APERTO e il dente arriva DOPO il rollout: ordine indifendibile

`putenv()` PHP chiama `std::env::set_var` in-process (`crates/php-builtins/src/file.rs:1277`). `reg_lower::enabled()` è un `OnceLock` **pigro** (`reg_lower.rs:50-53`): il modo del processo è deciso alla PRIMA lettura, che oggi avviene «quando capita» (primo compile: `func.rs:162`, `compile/mod.rs:750`, o costruzione UnitKey `vm/mod.rs:16058`). In CLI il sigillo cade compilando `{main}` prima che lo script possa eseguire putenv — sicuro *per accidente*. Nel server il sigillo dipende dall'ordine di boot (prelude compilato prima dell'accept?): **invariante non pinnata da nessun test**. Peggio: `set_var` con worker concorrenti che leggono env è unsound in Rust. Mettere il dente anti-putenv al punto 4, DOPO le misure flag-on del punto 3 sul percorso di promozione a default, lascia il canale aperto esattamente nella finestra in cui il flag conta di più.

### Refutazione R3 — Il funnel test copre il runner CI solo per DUE variabili, e il pin di parità ora CHURNA

`reg_lower_funnel.rs:18-19` fa `env_remove` di PHPR_REG_LOWER e PHPR_DUMP_OPS prima di `env()`: il caso «flag esportato nell'ambiente del runner» è coperto **per il figlio spawannato**, e M5 (`reg_lower.rs:597-601`) copre la batteria in-process. Bene. Ma (i) è una allowlist di sottrazione a lista chiusa — la lezione WP-96 dice che l'ambiente si COSTRUISCE, non si sottrae: ogni futura PHPR_* che biforca l'emissione riapre il buco in silenzio; (ii) manca il controllo flag-OFF al funnel (nessun dump-assert «zero forme registro» sul binario vero); (iii) il test rilinka il binario di parità a ogni `cargo test` — l'hash del pin cambia per costruzione (4e268c3f ≠ 39e0a359, «stesso sorgente» dichiarato ma non provato da un gate).

### Su (d) — worker con modi diversi nello stesso processo: NO, oggi

`OnceLock` è atomico: un solo valore per processo; `reg_mode` in UnitKey (`vm/mod.rs:15525`) e nel fp (`lower/mod.rs:841`) è cintura-e-bretelle corretta. Ma il commento «fixed per process today» poggia su R2: la fissità è garantita dalla fortuna del primo lettore, non da un sigillo.

## Emendamenti

- **A-PE-100-1**: invertire S-99: (1) parità server restapi+option per NOME sotto env -i sul pin 365f4d40, (2) collaudo WP full+media.
- **A-PE-100-2**: sigillo EAGER di `reg_lower::enabled()` come primo atto dei DUE main (CLI e server) + test che putenv dopo il boot non può flippare il modo — PRIMA del punto 3, non al 4.
- **A-PE-100-3**: registro pin con campo `collaudato: sì/no`; vietata la terza rotazione non collaudata; vietato il pin nato come effetto collaterale.
- **A-PE-100-4**: funnel: aggiungere il braccio flag-OFF con dump-assert «zero forme registro»; il binario di parità si pinna per COMMIT+stash, non per hash churnante.

## Kill-switch

- **KS-PE-100-1**: VOID il collaudo WP di S-99 punto 1 se eseguito prima della parità server sul pin che serve le richieste.
- **KS-PE-100-2**: VOID ogni misura flag-on SERVER e ogni gate di promozione a default finché il sigillo eager + il test anti-putenv non esistono.
- **KS-PE-100-3**: VOID ogni cifra attribuita a un pin server non presente nel registro come `collaudato: sì`.


---

# Verbale Sedia 7 — Leijen (allocatore mimalloc, footprint fisico) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: APPROVATO CON EMENDAMENTI VINCOLANTI — due refutazioni capitali sul mio perimetro.

## Refutazioni capitali

**R1 (capitale) — La «coppia peak al collaudo WP» promessa da S-99 è una forgia
senza strumento.** L'ordine S-99 punto 1 la contiene DAVVERO («coppia peak nello
stesso giro (chiude il debito Leijen)») ma non nomina né `/usr/bin/time -l` né
`vmmap` Physical footprint, né l'env mimalloc: senza lo stesso
`MIMALLOC_PURGE_DELAY=0` e lo stesso strumento della coppia WP-94, il confronto
col riferimento **1901,11 MiB** non è un confronto (RSS di `ps` mente per
regola permanente). Una promessa di misura senza strumento nominato è la classe
di forgia che fallisce in silenzio (lezione WP-96). Aggravante: la riga
⏱ FONDAMENTALI di NEXT_SESSION traccia SOLO full/media **CPU** — il footprint
non è misurato da WP-90/94 e il debito non ha contatore, quindi non scade mai;
«roadmap footprint ferma» + nessun contatore = deriva invisibile mentre due
sessioni (S-97.1, S-98.0) hanno cambiato emissione e binari.

**R2 (capitale) — Il rollout S-99 punto 3 spende un budget che nessuno conta.**
Il tag di dispatch è un byte (`ldrb` nell'ASM di M1) ⇒ tetto duro 256; N_OPS è
salito a **186** (test census aggiornato in S-98.0) e il rollout per famiglia
(Add/Sub/Mul/compare × stack/BinarySS/SC/Dst) è MOLTIPLICATIVO, ma il «dente
N_OPS<256» giace in BACKLOG, cioè DOPO le occorrenze che lo consumano. Inoltre
ogni variante è un corpo caldo in più nel run_loop (lezione WP-33: branch
mai-preso = +2,9%; M1: già a 185 bracci gli indirizzi di frames.len/ptr
SPILLANO) — il canary noop inter-build (0,5% misurato per la prima occorrenza)
non è pre-registrato come obbligo per le successive.

## Refutazioni minori (forma, non sostanza)

**r3** — `hb2-addspec.out` non dichiara né misura l'impatto footprint di H-B2.
Per MIA ispezione di run.rs:944-975 il hit-path è alloc-neutro (pop di `Vec`
+ scrittura in-place; `Zval::Long` senza heap) e il high-water dello stack è
identico al generico (pop+pop+push ≡ pop+write-in-place, picco uguale, mai
superiore); il MISS ricalca il funnel. Ma è ispezione del revisore, non claim
del report: un claim assente non è un claim vero.

**r4** — La sonda M1 pinna la TAGLIA (OpS=48B, FrameS=176B) e la catena critica
(madd frame → ldr ip → madd op), ma non gli OFFSET di ip/func del `Frame` vero
(repr(Rust) non li garantisce) né l'allineamento del frame sulle linee da 64B
(176B ne attraversa 3). La conclusione «reload L1-resident gratis» è robusta
per dati L1, quindi non cade; ma la fedeltà venduta è di taglia, non di layout.

**r5** — `reg_lower::enabled()` sta nella chiave della unit-cache: corretto per
la parità («nessun ibrido»), ma se mai il flag diventasse togglabile nel
processo la cache terrebbe DUE emissioni residenti = footprint doppio del
codice compilato. Oggi è env process-level: va pinnato che resti tale.

## Emendamenti

- **A-LE-100-1**: ordine S-99 punto 1 riscritto con lo strumento: coppia peak
  stessa-sera via `/usr/bin/time -l` (peak fisico; `vmmap` Physical footprint
  come spot-check; MAI RSS di ps), env mimalloc identico a WP-94, cifre accanto
  a 1901,11 MiB. La riga ⏱ FONDAMENTALI acquista il campo «ultima misura
  peak = WP-N (k sessioni fa)».
- **A-LE-100-2**: ogni .out di specializzazione dichiara il profilo di
  allocazione del hit/miss-path (per H-B2: retro-annotare «alloc-neutro per
  costruzione, high-water invariato»).
- **A-LE-100-3**: dente N_OPS≤255 promosso a GATE compile-time PRIMA del
  rollout punto 3, con budget di varianti nominato; canary noop inter-build
  pre-registrato per OGNI occorrenza nuova.
- **A-LE-100-4**: nella sonda m1-probe, controllo positivo `offset_of!` di
  ip/func contro il Frame vero prima di ogni riuso come strumento.

## Kill-switch

- **KS-LE-100-1**: VOID ogni confronto peak non prodotto da `/usr/bin/time -l`
  (o vmmap Physical footprint) stessa-sera sui due lati con env mimalloc
  identico al riferimento.
- **KS-LE-100-2**: VOID la spedizione di una nuova variante `Op` se porta
  N_OPS oltre 255 o se il canary noop devia oltre 3× lo spread.

— Leijen, Sedia 7, 2026-08-05


---

# Verbale Sedia 8 — Stogov (Zend engine, semantica) — Concilio WP-100

## VERDETTO
S-98.0: la FORMA del handler `Op::BinaryAdd` REGGE i miei attacchi semantici.
Il programma S-99.0 NO: il punto 3 (rollout nelle forme registro) porta un
criterio derivato dal percorso SBAGLIATO, e la coda AssignOp è ambigua tra
verdetto `.out` e ordine. **Approvo il report CON EMENDAMENTI; refuto il
criterio del punto 3 dell'ordine.**

## Attacchi condotti (esiti)
(a) **Ref sulla pila**: `Op::LoadVar` (run.rs:636-653) e `Op::LoadGlobal`
(:755-758) passano da `read_slot`, che DEREF-CLONA i `Ref`: nessun `Ref`
raggiunge `BinaryAdd` da questi emettitori. Se mai arrivasse (nessun sito
nell'emissione lo produce), la guardia `(Long,Long)` MISS → pop lhs →
`binary_value_ab` — lo STESSO funnel del vecchio `Binary(Add)` (che pure
non fast-pathava i Ref in `binary_fast`). Identico per costruzione. TIENE.

(b) **Overflow**: run.rs:959-962 = `checked_add`, MISS → `l as f64 + r as
f64` sugli operandi ORIGINALI — verbatim `binary_fast` (:118-121) e
identico a Zend `fast_add_function` (`(double)op1 + (double)op2`). −0.0
non producibile da Long+Long. TIENE.

(c) **GMP/BcMath/__toString**: oggetto ⇒ guardia MISS ⇒ `binary_value_ab`
⇒ `binary_fast` MISS ⇒ `apply_binop_ovl` — braccio identico al vecchio.
Add non è `cmp_op`: il ramo `__toString` (run.rs:335) non è mai toccato,
com'era prima. TIENE.

(d) **Diagnostica**: il fallback passa `(lhs, rhs)` in ORDINE (run.rs:
969-971): "Unsupported operand types: int + array" conserva i nomi. Il
warning undef è emesso da LoadVar PRIMA dell'op (queued, :640-646),
invariato; e con un operando Undef la pila porta Null ⇒ MISS ⇒ funnel
vecchio. Né il vecchio hit di `binary_fast` né il nuovo hit in-place
flushano i diags: parità anche lì. TIENE.

(e) **Siti op=**: expr.rs:90-106 (slot), assign.rs:956-962 (prop, RHS
prima di PropGet come ASSIGN_OBJ_OP), :971-992 (global/superglobal):
`emit_binary` sostituisce 1:1 nella STESSA posizione di sequenza; timing
di __get/__set e degli effetti del RHS invariato. `FieldAssignOp`/
`AssignOpPath` portano il payload `op` e NON passano da `emit_binary`:
fuori perimetro, dichiararlo nel rollout. TIENE (con nota).

## Refutazioni capitali
**R1 — Il criterio del punto 3 è derivato dal percorso sbagliato.**
D=6,07 ns/occ misura il plumbing dello STACK-path (call con due Zval per
valore + pop/push). Ma `BinarySS` (run.rs:1011-1019) già BORROW-a i due
slot dentro `binary_fast` senza marshalling né pop/push: il rimovibile lì
è solo match esterno+Option — classe di costo DIVERSA e plausibilmente
≪6 ns. Un criterio «in ns/occorrenza derivato da D=6,07» ripete l'errore
che S-98.0 stessa ha messo nei NON-riproporre (criterio dal conteggio
senza cammino critico). Va derivato da un controfattuale del percorso
REGISTRO, con la sua soglia.

**R2 — La coda AssignOp è ambigua.** Il verdetto di `hb2-addspec.out`
(:134-136) la elenca fra le «prossime occorrenze della stessa famiglia»;
l'ordine S-99 la tiene in BACKLOG dietro le sette trappole A-ST-99-3 (non
ancora SCRITTE). Due letture possibili = un buco: senza dente, il rollout
punto 3 può inghiottire il fold.

## Emendamenti
- **A-ST-100-1**: prima del punto 3, controfattuale statico del percorso
  registro (che cosa rimuove la specializzazione DENTRO BinarySS/SC/Dst)
  e soglia pre-registrata da QUELLO, non da D=6,07.
- **A-ST-100-2**: le sette fixture-trappola A-ST-99-3 diventano
  PRECONDIZIONE nominata nell'ordine (non prosa di backlog).
- **A-ST-100-3**: fixture in-tree (non smoke una-tantum): `PHP_INT_MIN +
  (-1)`, ordine dei nomi con const-lhs (`3 + $arr`), alias by-ref
  (`$r=&$a; $r+1`), confrontate all'oracle.

## Kill-switch
- **KS-ST-100-1**: ogni braccio specializzato dentro le forme registro
  tiene il MISS = funnel owned ATTUALE (warning Undef, deref Ref,
  `binary_value_ab`), ordine dei diags byte-identico; un braccio che
  tocca i tag Undef/Ref da sé è VOID.
- **KS-ST-100-2**: nessun fold/fusione AssignOp atterra prima che le
  sette trappole passino per NOME su ENTRAMBI i motori.


---

# Verbale Sedia 9 — Gregg (metodologia di misura, attribuzione) — Concilio WP-100, MANDATO INVERSO

**Oggetto giudicato**: S-98.0. Domanda: che cosa sappiamo di phpr oggi che ieri non sapevamo?

## VERDETTO

**APPROVATO CON EMENDAMENTI.** Dal punto di vista dell'OGGETTO è la sessione più densa da settimane: tre numeri nuovi e veri su phpr, di cui uno negativo (il più raro). Ma due refutazioni capitali sul modo in cui quei numeri vengono RACCONTATI.

## (a) Inventario: conoscenza dell'oggetto vs apparato

Conoscenza NUOVA dell'oggetto: (1) **il preambolo di dispatch costa ~0 su questo core OoO** (P=0%, banda [0–6,7%]; la split-borrow perde 0,53 ns/op sotto pressione) — conoscenza negativa solida, chiude un intero asse senza codice; (2) **anche gli op economici costano 6,27 ns contro 1,07** (noop 200M): i corpi dominano ovunque; (3) **D=6,07 ns/occ è il prezzo del plumbing generico di un Binary** — prima attribuzione POSITIVA del fattore ~8; (4) l'ASM del loop head decomposto 11/17. Apparato (utile ma non-oggetto): reg_lower_funnel, bin_op_of, M5, ricostruzione census. Rapporto oggetto/apparato: il migliore da WP-94.

## (b) Igiene

La serie bimodale (build concorrente) è dichiarata nel .out con la ⚠️ e scartata. La coppia add È rimisurata da zero su ENTRAMBI i binari nella stessa finestra: il .out lo dice esplicitamente («COPPIA rimisurata da zero su ENTRAMBI i binari nella stessa finestra», R=5+R=5, baseline dallo stash verificato per hash, deriva inter-build 0,5% via noop, misure riprodotte sul pin finale). La baseline early-window (5,64) e quella rimisurata (5,63) coincidono allo 0,2%: omogeneità DIMOSTRATA, non assunta. Su (b) non refuto nulla.

## (c) Bande — refutazione capitale n.1

`arith` in banda [−3,−5]% per KS-GR-99-2 è TROPPO prudente nel caso specifico: con R=5 c'è **separazione di rango completa** (tutti i 7,50–7,66 sotto tutti i 7,88–7,94; p≈1/252). Il 3×spread sulla gamba peggiore, gonfiato da UN outlier, butta informazione che il rango conserva. Prudenza in direzione sicura, ma un criterio futuro che pescasse da quella banda erediterebbe l'imprecisione.

**La refutazione capitale è l'altra metà**: il titolo «add 14,5→12,1» ha la **gamba oracle STANTIA** (0,39 misurato nella finestra baseline, MAI rimisurato in coppia post-build) — e «arith 18,1→17,1» lo ammette solo in commento. Il progetto ha istituito la regola della coppia stessa-finestra e la viola nel proprio titolo. Il claim pulito è D=6,07 (phpr-vs-phpr); i RAPPORTI sono nominali e vanno etichettati tali. Inoltre str/re/prop/calls flag-off NON rimisurati dopo il cambio di emissione: i criteri di attivazione di H-C/H-D («prop resta sopra 5×») galleggiano su baseline morte.

## (d) Attribuzione — refutazione capitale n.2

D=6,07 è una differenza pulita; la scomposizione call/marshalling/pop-push NON serve prima del rollout: il criterio pre-registrato per occorrenza basta come go/no-go. MA il testo dice «le forme registro flag-on hanno lo stesso plumbing da togliere — è lì che compone col −30,7%»: **ipotesi spacciata per fatto**. BinaryDst/SS/SC non fanno pop/push: la quota rimovibile flag-on è per costruzione DIVERSA, e la lezione di M1 dice che il controfattuale statico può essere nascosto dall'OoO. D=6,07 NON è trasferibile.

## (e) FONDAMENTALI

NEXT_SESSION punto 1 dice «DOVUTO, non rinviabile»: nessuna scusa lì. La crepa è in WP_SESSION_98: «alla prossima occasione utile» — linguaggio che ha già fatto slittare collaudi quattro volte (WP-94 è 4 sessioni fa). E il punto 3 (rollout) è il più attraente: senza un dente, la tentazione di invertire l'ordine è strutturale.

## Emendamenti

- **A-GR-100-1**: ogni rapporto phpr/oracle pubblicato senza gamba oracle stessa-finestra porta l'etichetta obbligatoria «nominale, gamba oracle stantia»; il 14,5→12,1 la riceve retroattivamente.
- **A-GR-100-2**: i criteri di caduta del rollout flag-on si derivano da una misura FLAG-ON del controfattuale; D=6,07 non si ricicla.
- **A-GR-100-3**: KS-GR-99-2 si integra col criterio di rango (separazione completa a R=5 ⇒ stima puntuale con banda, non banda sola).
- **A-GR-100-4**: al collaudo S-99, ri-baseline delle sei categorie flag-off su entrambi i motori stessa-finestra (rianima i criteri H-C/H-D).

## Kill-switch

- **KS-GR-100-1**: in S-99 nessuna spedizione del rollout H-B2 né alcun claim CPU nuovo PRIMA che il collaudo di parità WordPress (punto 1) sia chiuso per NOME; violazione ⇒ VOID.
- **KS-GR-100-2**: VOID ogni occorrenza del rollout il cui criterio è derivato dal D flag-off invece che da misura flag-on.


---

# Team «criterio-rollout» — Concilio WP-100 (relatore)

**Perimetro**: punto 3 dell'ordine S-99 (rollout specializzazione Add nelle forme registro flag-on) e il suo criterio. Fonti vincolanti: verbale-1-hoare, verbale-2-matsakis, verbale-5-bak, verbale-8-stogov. Questa nota riconcilia, non sostituisce.

## Convergenze (tutte e quattro le sedie)

1. **D=6,07 ns/occ NON è ereditabile come àncora per le forme registro.** D misura il plumbing del percorso PILA (call `binary_value_ab` con due Zval owned da 24B, pop/push, marshalling/sret); `BinarySS/SC/Dst` prendono gli slot per PRESTITO e chiamano `binary_fast` `#[inline(always)]`: quel plumbing lì in gran parte non esiste. Un criterio copiato da D è miscalibrato per costruzione (Hoare A-HO-100-3; Matsakis R-1/A-MA-100-1; Bak R3/KS-BA-100-1; Stogov R1/A-ST-100-1).
2. **Il criterio va PRE-REGISTRATO da un controfattuale statico NUOVO del percorso registro**, scritto PRIMA del codice: il residuo rimovibile in BinarySS è solo carico+match del payload `BinOp` (+ eventuale bordo di call), classe di costo plausibilmente ≪6 ns.
3. **MISS = funnel owned ATTUALE, intoccato**: guardia Undef|Ref conservata, deref-funnel, `binary_value_ab`, ordine diags byte-identico; vietato l'in-place su slot attraverso un `Zval::Ref` (KS-HO-100-1 ≡ KS-ST-100-1).
4. La forma del handler `BinaryAdd` pila REGGE (nessuna sedia la refuta); è il CRITERIO del rollout a cadere.

## Conflitti / posizioni per sedia

- **Bak vs Matsakis — compatibili, non identici.** Matsakis (R-1): BinarySS già inlinea `binary_fast`, quindi il residuo è solo il match del payload — argomento STATICO sul codice. Bak (R3/A-BA-100-1): D non è mai stato decomposto (è 2,2–4,3× il suo tetto statico) e chiede una build intermedia di MISURA che separi prologo+marshalling (i)+(ii) da pop/push (iii), quota trasferibile stimata 50–70% ma ignota. Non c'è contraddizione: Matsakis dice DOVE il residuo vive, Bak esige che la sua GRANDEZZA sia misurata, non stimata. La build di Bak resta utile anche se BinarySS non chiama il funnel nel hit-path: spiega D e tara il controfattuale.
- **Chi chiede cosa PRIMA del rollout**: Bak → build di decomposizione di D (A-BA-100-1). Matsakis → tetto nuovo misurato flag-on sul giudice registro (A-MA-100-1) + chiusura wildcard `bin_op_of` (A-MA-100-2, R-2 latente: Sub/Mul compilerebbero senza fondersi, perf-only invisibile al corpus). Hoare → census dump-based dei `Binary(Add)` residui flag-off (A-HO-100-4) + decadenza del claim «flag-on bit-identico» al primo cambio di emissione (KS-HO-100-2). Stogov → sette fixture-trappola A-ST-99-3 come PRECONDIZIONE nominata (A-ST-100-2) + fixture in-tree (A-ST-100-3); coda AssignOp da disambiguare (R2).

## Proposta del team per il punto 3 di S-99 (forma finale)

1. **Pre-misura obbligatoria** (prima di ogni codice): (a) controfattuale statico del corpo BinarySS (residuo = match payload) con banda pre-registrata; (b) build intermedia A-BA-100-1 che decompone D; (c) baseline flag-on del giudice registro.
2. **Criterio**: soglia in ns/occ derivata da (a)+(b), MAI da D=6,07 (KS-MA-100-1, KS-BA-100-1); pubblicata come banda se sotto 2× il pavimento sonda (KS-BA-100-2).
3. **Precondizioni**: test/chiusura `bin_op_of` (KS-MA-100-2); sette trappole A-ST-99-3 scritte e verdi per NOME (KS-ST-100-2); census flag-off (A-HO-100-4).
4. **Gate di spedizione**: MISS=funnel intatto (KS-HO-100-1/KS-ST-100-1); corpus flag-ON per NOME + census ri-pinnati stessa sessione (KS-HO-100-2).

**Priorità**: 1) controfattuale statico registro; 2) chiusura `bin_op_of`; 3) build decomposizione D; 4) trappole A-ST-99-3; 5) solo poi il codice del rollout.


---

# Team «ordine» — Concilio WP-100 (relatore)

**Fonti vincolanti**: verbale-6-pedersen.md, verbale-9-gregg.md. Questa nota riconcilia solo l'ORDINE di S-99.

## Convergenze

1. **Nessuno dei due vuole rinviare il collaudo WordPress**: Gregg lo esige come primo deliverable (KS-GR-100-1: nessun rollout né claim CPU prima del punto 1 chiuso per NOME); Pedersen non chiede di spostarlo a un'altra sessione, chiede che non sia VOID per costruzione.
2. **Il rollout (punto 3) è l'ultimo atto in entrambe le letture**: per Pedersen dietro sigillo eager + dente anti-putenv (KS-PE-100-2); per Gregg dietro criteri derivati da misura flag-on, non dal D flag-off (KS-GR-100-2). Le due catene di precondizioni sono compatibili e si sommano.
3. **Le baseline morte vanno rianimate al collaudo, non dopo** (A-GR-100-4 combacia con l'igiene dei pin di A-PE-100-3: tutto ciò che è citato deve essere fresco o etichettato).

## Il conflitto

- **Pedersen (R1)**: il punto 1 USA php-server; il pin 365f4d40 non è collaudato ⇒ per KS-PE-99-1 il collaudo eseguito prima della parità server è VOID dal suo stesso kill-switch. Parità server PRIMA.
- **Gregg (e)**: «alla prossima occasione utile» ha già fatto slittare il collaudo quattro volte; il punto 3 è strutturalmente attraente; la gamba oracle dei rapporti è stantia e va rimisurata al collaudo.

**Risoluzione**: il conflitto è apparente. Pedersen chiede il sigillo eager PRIMA del punto 3 («non al 4»), NON prima del punto 1; e la parità server non è un rinvio del collaudo WP ma la sua precondizione tecnica — si esegue DENTRO il punto 1, stessa sessione. Nessun KS dei due verbali collide con questa lettura.

## Sequenza S-99 proposta

1. **Punto 1a (precondizione dentro il punto 1)**: parità server restapi+option per NOME sotto env -i sul pin 365f4d40; esito nel registro pin come `collaudato: sì` (A-PE-100-1/3; soddisfa KS-PE-99-1 e KS-PE-100-1/3). La ricetta si scripta: servirà di nuovo al passo 4.
2. **Punto 1b**: collaudo WP full+media stessa sessione, immediatamente dopo. Dente anti-slittamento: se 1a fallisce, la sessione ripara la parità — non passa ad altro (KS-GR-100-1 resta armato).
3. **Ri-baseline** delle sei categorie flag-off su ENTRAMBI i motori stessa-finestra (A-GR-100-4); etichetta «nominale, gamba oracle stantia» retroattiva sui rapporti già pubblicati (A-GR-100-1).
4. **Sigillo eager** di `reg_lower::enabled()` nei due main + test anti-putenv + braccio flag-OFF del funnel (A-PE-100-2/4). Il rebuild produce pin nuovi ⇒ ri-parità server per NOME con lo script del passo 1a (nessuna rotazione non collaudata).
5. **Misure flag-on server e criteri di rollout** derivati da misura flag-on (A-GR-100-2), ammessi solo dopo il passo 4 (KS-PE-100-2, KS-GR-100-2).

## KS che restano in vigore

KS-PE-100-1, KS-PE-100-2, KS-PE-100-3, KS-GR-100-1, KS-GR-100-2; KS-PE-99-1 (soddisfatta dal passo 1a); KS-GR-99-2 integrata col criterio di rango (A-GR-100-3).


---

# Team «evidenza» — Concilio WP-100 (relatore)
Fonti: verbale-3-klabnik.md · verbale-4-hejlsberg.md · verbale-7-leijen.md (vincolanti; questa nota riconcilia).

## (a) Convergenze
1. **Le misure di S-98.0 reggono; i CLAIM di evidenza sono più forti dei test scritti.** Klabnik R1: il «controllo positivo» del funnel confronta i dump (`off_out == on_out` + tre forme nel `{main}`) ma non pinna stdout atteso né exit status — un binario che sbaglia UGUALE nei due modi passa. Hejlsberg RC-1/RC-2 rincara: il dump è cieco per costruzione dove serve di più.
2. **Il gate corpus per NOME è un bit per fail** (Klabnik R2, verificato: set 1418 byte-uguale flagoff/flagon/canone): parità di regressione sì, **gate di PROMOZIONE no** — un fail con diff diverso tra i due modi è invisibile. Serve il riga-per-riga degli output dei fail, salvato come evidenza (A-KL-100-2).
3. **Punti ciechi del dump/battery** (Hejlsberg RC-1/RC-2, Klabnik R1 coda): flag-on i corpi **hook** VENGONO riscritti dal pass ma sono esclusi da `dump_module_ops` (clausola violata oggi); `lowered()` abbassa **prop_init** che in produzione (`compile_prop_init`, func.rs:361, senza gate `enabled()`) non è MAI lowered. Il battery testa una pipeline che la produzione non esegue e non testa quella che esegue. Guardia M5 in un solo test; `cargo test` con env esportato applica il pass due volte in silenzio.
4. **Due rialzi di budget cifre nella stessa sessione** (24561→24643→24645, Klabnik R4): il primo è forgia legittima, il secondo è gate ri-adattato all'artefatto dopo il morso. Breakdown per fonte nel file (A-KL-100-4); terzo rialzo senza enumerazione token = gate non-mordente (KS-KL-100-3).
5. **Strumentazione S-99 sottospecificata**: coppia peak senza strumento nominato = forgia che fallisce in silenzio (Leijen R1: `/usr/bin/time -l`, spot-check vmmap Physical, MAI RSS ps, env `MIMALLOC_PURGE_DELAY=0` come WP-94, accanto a 1901,11 MiB); dente **N_OPS≤255** in BACKLOG cioè DOPO le occorrenze che lo consumano, con N_OPS=186 e rollout moltiplicativo (Leijen R2).

## (b) Conflitti per sedia
- **Klabnik vs Hejlsberg sul controfattuale**: Klabnik emenda la matrice smoke Add (R3, 5 celle su tag-space ≥12) tenendo D=6,07 come base; Hejlsberg RC-3 dichiara D=6,07 il controfattuale SBAGLIATO per BinarySS/SC/Dst (pop/push già eliso) → il criterio va ri-registrato per-forma. Componibili: matrice per la correttezza, controfattuale per-forma per il criterio.
- **Hejlsberg vs Klabnik su bin_op_of**: per Hejlsberg neutralizza un tripwire (A-HE-100-1: assert ZERO BinaryAdd flag-on); per Klabnik il rischio vive nella matrice fixture. Non contraddittori.
- **Leijen r3-r5** sono minori dichiarate (retro-annotazioni, offset_of, pin env process-level): nessuna sedia le contesta, ma nessuna le eleva a bloccante.

## (c) Lista MINIMA che blocca i PASS futuri (regola di ammissione: blocca l'oggetto)
1. **Promozione flag-on→default** bloccata da: diff riga-per-riga dei 1418 fail off/on (KS-KL-100-1) + dente anti-putenv/eager-init su `enabled()` (KS-HE-100-1) + sanatoria dump/`lowered()` al funnel vero — hook dentro, prop_init gated o dichiarato fuori (A-HE-100-4) + funnel pinna stdout/exit (A-KL-100-1).
2. **Rollout famiglia (Sub/Mul, punto 3)** bloccato da: gate compile-time N_OPS≤255 + canary noop pre-registrato (KS-LE-100-2/A-LE-100-3) + matrice fixture Add per NOME (KS-KL-100-2) + criterio ri-derivato per-forma (KS-HE-100-3) + tripwire ZERO BinaryAdd flag-on (A-HE-100-1).
3. **Coppia peak S-99**: strumento nominato o confronto VOID (KS-LE-100-1/A-LE-100-1).

## (d) Priorità
P1 sanatoria dump/lowered+funnel (blocca promozione E rollout) · P2 diff riga-per-riga + dente enabled() · P3 gate N_OPS + strumento peak · P4 matrice Add + criterio per-forma.

**BACKLOG per NOME** (non blocca l'oggetto): A-KL-100-4 breakdown budget; A-KL-100-5/debito B2 (13 snippet al funnel vero); A-HE-100-2 `visit_addrs` esaustivo (blocca solo shape Addr-bearing future, KS-HE-100-2); A-HE-100-3 test differenziale BinaryAdd≡Binary(Add); A-LE-100-2 profilo alloc nei .out; A-LE-100-4 offset_of sonda M1; pin env process-level (r5).


---

# Concilio WP-100 — SINTESI DI CONVERGENZA (su S-98.0 e programma S-99.0)

## §FONDAMENTALI (prima di tutto, per regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: NON zero — Gregg
(mandato inverso) la certifica «la sessione più densa di conoscenza-oggetto
da WP-94». Nuovo per NOME: H-B1 chiusa a COSTO ZERO con la sola misura
(P=0%: il preambolo di dispatch è gratis sul core OoO, la split-borrow può
perfino perdere); H-B2 spedita con verdetto numerico (D=6,07 ns/occorrenza
sul giudice `add`, −16,2%); il fattore ~8 del costo per opcode ATTRIBUITO
al plumbing dei corpi; DUE corpus per NOME (flag-off e flag-on) sullo
stesso albero. Il metodo (criterio scritto prima) ha morso per la terza
sessione consecutiva.

**(b) Contatore sessioni-senza-misura**: full/media = WP-94, 4 sessioni fa
— e ora il collaudo è DOVUTO senza scuse (l'emissione flag-off è cambiata).
Peak footprint: nessuna misura da m90 e NESSUNO strumento nominato
nell'ordine (refutazione di Leijen). La gamba ORACLE delle sei categorie
non è mai stata rimisurata dalla baseline S-97.0 (Gregg).

**(c) Rischio d'oggetto più trascurato**: il pin php-server è alla SECONDA
rotazione consecutiva senza collaudo (832568a7 mai collaudato → 365f4d40
mai collaudato) e l'ordine S-99 in bozza gli faceva servire il collaudo
WordPress del punto 1: VOID per costruzione (Pedersen, KS-PE-99-1).

## Verdetti di fase 1 (9/9 CON EMENDAMENTI; Klabnik «rifiutato nella FORMA,
misure valide»; Bak «esiti sopravvivono ma non per le ragioni dichiarate»)

Verbali integrali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali, per convergenza:

1. **D=6,07 NON è ereditabile come criterio del rollout** (SEI sedie
   indipendenti: Hoare, Matsakis, Bak, Stogov, Hejlsberg, Gregg — la
   convergenza più larga mai registrata). È il costo del plumbing del
   percorso PILA (call + marshalling per valore + pop/push); le forme
   registro flag-on NON hanno quel plumbing (BinarySS inlinea già
   `binary_fast` su prestiti: il residuo è il match del payload BinOp).
   Il criterio del punto 3 si deriva da controfattuale/misura DEL percorso
   registro (A-ST-100-1, A-BA-100-1, A-MA-100-1, A-GR-100-2); ogni
   spedizione col criterio ereditato è VOID (KS-MA-100-1, KS-HE-100-3,
   KS-BA-100-1, KS-GR-100-2).
2. **L'ordine S-99 in bozza era VOID al punto 1** (Pedersen): il collaudo
   WordPress passa da php-server, che è il binario NON collaudato.
   Riconciliazione team-ordine (Gregg concorda: il collaudo NON slitta):
   parità server = PRIMA GAMBA del punto 1, stessa sessione, poi il
   collaudo. Sigillo eager di `enabled()` + dente anti-putenv promossi a
   GATE DI PROMOZIONE (KS-HE-100-1, KS-PE-100-2), non residuo.
3. **Evidenza sovradichiarata** (Klabnik+Hejlsberg+Leijen, team-evidenza):
   il controllo positivo del funnel pinna il DUMP ma non stdout/exit; il
   gate corpus per NOME è UN bit per fail (la promozione esige il diff
   riga-per-riga, KS-KL-100-1); dump e `lowered()` sono ciechi sugli hook
   riscritti flag-on e su prop_init (RC di Hejlsberg); la coppia peak
   promessa era SENZA strumento (KS-LE-100-1: solo `/usr/bin/time -l` +
   env mimalloc di WP-94); il byte-tag è a 186/256 senza gate
   (A-LE-100-3); due rialzi di budget in una sessione — il terzo senza
   delibera esplicita = gate non-mordente (KS-KL-100-3).
4. **La sonda M1 va annotata, non ritrattata** (Bak): misura
   Δ(A_sonda, B) con una A più ottimizzata della A reale (i caldi non
   spillano) e il «tetto anti-hiding» usa il metodo count-derived che la
   sessione stessa ha refutato; il verdetto H-B1 REGGE (P sotto soglia su
   tutta la banda) ma A-BA-100-2 impone l'annotazione nel .out e
   KS-BA-100-2 declassa a banda ogni claim della sonda sotto 2× il suo
   pavimento di risoluzione.
5. **«Compone col −30,7%» è un'ipotesi, non un fatto** (Gregg): flag-off e
   flag-on non sono mai stati misurati INSIEME dopo H-B2; i rapporti con
   gamba oracle stantia (14,5→12,1) portano l'etichetta «nominale»
   (A-GR-100-1).

## Ordine DEFINITIVO S-99.0 (regola di ammissione applicata)

1. **Parità server** (prima gamba, precondizione): restapi+option per NOME
   sotto `env -i` sul pin 365f4d4069513de3 + sentinella output-capture.
2. **Collaudo WordPress full+media stessa-sera** (parità per NOME sui due
   lati) **+ coppia peak** con `/usr/bin/time -l` + env mimalloc pinnate da
   WP-94 (A-LE-100-1). Nessun claim CPU nuovo e nessun rollout prima della
   chiusura per NOME (KS-GR-100-1).
3. **Ri-baseline delle sei categorie** su ENTRAMBI i motori, stessa
   finestra (A-GR-100-4): rianima i criteri di H-C/H-D e sana la gamba
   oracle stantia.
4. **Pre-misura del rollout, SOLO misura** (il codice del rollout NON è in
   quest'ordine): controfattuale statico del percorso registro
   (A-ST-100-1) + build intermedia che decompone D in call/marshalling vs
   pop/push (A-BA-100-1) + baseline flag-on; il criterio del rollout nasce
   QUI. Se il timebox regge: sigillo eager + test anti-putenv
   (A-PE-100-2 — gate di promozione).

**Precondizioni per NOME dei passi FUTURI** (non slot di sessione):
promozione flag-on ⇐ diff riga-per-riga (A-KL-100-2) + sigillo
eager/anti-putenv + sanatoria dump/lowered (A-HE-100-4) + stdout pinnato
nel funnel (A-KL-100-1); rollout ⇐ gate N_OPS≤255 (A-LE-100-3) + matrice
fixture Add (A-KL-100-3) + tripwire zero-BinaryAdd flag-on (A-HE-100-1) +
trappole A-ST-99-3 per ogni fusione AssignOp (KS-ST-100-2); shape nuove ⇐
visit_addrs esaustivo (A-HE-100-2). BACKLOG per NOME: test differenziale
BinaryAdd≡Binary(Add) (A-HE-100-3), registro pin `collaudato:`
(A-PE-100-3), braccio flag-OFF nel funnel (A-PE-100-4), fixture in-tree
MIN/const-lhs/by-ref (A-ST-100-3), offset_of nella sonda (A-LE-100-4),
breakdown budget (A-KL-100-4), etichette e bande .out (A-BA-100-2,
A-LE-100-2, A-GR-100-1/3), census Binary(Add) residui (A-HO-100-4),
debito B2 batteria (A-KL-100-5).

L'ordine NON è composto di solo apparato: i punti 1-3 sono misure
dell'oggetto; il punto 4 è la misura che fonda la prossima leva.

## Conflitti registrati

- team-ordine: server-prima (Pedersen) vs collaudo-non-slitti (Gregg) —
  RICONCILIATO: stessa sessione, server come prima gamba del punto 1.
- team-criterio-rollout: build di decomposizione (Bak, misura) vs argomento
  statico (Matsakis, codice) — COMPATIBILI: il luogo del costo si legge dal
  codice, la grandezza si misura; entrambi entrano nel punto 4.
- team-evidenza: matrice-prima (Klabnik) vs controfattuale-prima
  (Hejlsberg) — componibili nelle precondizioni per NOME del rollout.
