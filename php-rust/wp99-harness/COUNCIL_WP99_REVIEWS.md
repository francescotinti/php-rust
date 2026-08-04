# COUNCIL_WP99_REVIEWS — Concilio a 9 sedie su S-97.0+S-97.1 e programma S-98.0
# Protocollo a due fasi (regola 2026-08-02); verbali individuali = fonte VINCOLANTE.

# Concilio WP-99 — SINTESI DI CONVERGENZA (su S-97.0+S-97.1 e programma S-98.0)

## §FONDAMENTALI (prima di tutto, per regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: NON zero — è la sessione
più densa di misure dell'oggetto da molte settimane. Nuovo per NOME: il
divario `arith` è decomposto in fattori esatti; H-A1 eseguita fino in fondo
con verdetto numerico (19→11 opcode/iter, −30,7%); il costo per opcode
residuo è pinnato (9,87 ns contro 1,23 dell'oracle) e la sua NON-uniformità
confermata tre volte; due ipotesi (H1, H-A1) sono cadute su criterio, una
(H-A2) è spedita. Il metodo nuovo (criterio scritto prima) ha morso due
volte in due sessioni: funziona.

**(b) Contatore sessioni-senza-misura**: ultima full/media WordPress =
WP-94, 3 sessioni fa (per REGOLA della spina: WordPress è collaudo di
parità, non cronometro — l'emissione flag-off non è cambiata). Il micro,
che ORA è il cronometro dell'oggetto, ha girato due volte in coppia R=3.

**(c) Rischio d'oggetto più trascurato**: la parità del SERVER — il binario
php-server è stato ricostruito con H-A2 dentro (incondizionata) e nessun
gate server è girato (Pedersen: parità DOVUTA, non facoltativa; pin ruotato
832568a72b925dd1 senza verifica). Secondo: la roadmap footprint resta ferma
(Leijen: coppia peak dovuta al prossimo collaudo WP).

## Verdetti di fase 1 (9/9 CON EMENDAMENTI, nessun MI OPPONGO)

Verbali integrali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali, per convergenza:

1. **La forma letterale di H-B1 è FALSA** (Hoare+Matsakis, indipendenti):
   «frame in registro, ricaricato solo a call/ret/throw» ignora che gli
   archi di ri-entrata (gc_note, flush_diags, __toString, dtor) attraversano
   quasi ogni handler e che ogni diag legge `frames[top].ip`. Forma safe
   possibile: loop interno su split-borrow di campo, confine = ogni opcode
   che chiama metodi `&mut self`, `ip` come campo vero mai copiato.
2. **Il tetto di H-B1 è ~1,4 ns/op (−17%, banda 8–27%)** (Bak+Gregg,
   indipendenti, dallo stesso dato ha2-sweep: il dispatch noop costa ~1,4
   ns). Il fattore ~8 residuo vive nei CORPI degli handler ⇒ **H-B1
   declassata a sotto-passo, H-B2 (specializzazione per tipo) promossa ad
   asse**. Bak: il criterio −40% di H-A1 era irraggiungibile a tavolino con
   dati già posseduti (la scala uniforme dava −42%) — il criterio del
   prossimo passo si deriva dal TETTO, non dal desiderio.
3. **Il PASS del gate era parzialmente vacuo** (Klabnik): WP_SESSION_97
   registrato judge=no (= escluso) mentre il report ne vantava il PASS; i
   tre .out di S-97.0 senza riga. RIPARATO in chiusura (WP_SESSION_97
   judge=yes → il gate ha subito morso una cifra irrisolta, corretta;
   esclusioni dichiarate per i .out). Resta a S-98.0: delibera su 94/95 e
   ri-collaudo dello strumento census (il pin 9,87 ns ne dipende).
4. **Il harness di test applica il pass in un punto di pipeline diverso
   dalla produzione** (Hejlsberg): `lowered()` gira post-cessione WP-65 —
   batteria potenzialmente vacua sui 13 snippet top-level. La vacuità è di
   COPERTURA TEST, non di produzione (il census CLI ha foldato il {main}:
   19→11 col dump che mostra le forme). Chiusura: assert positivo di forme
   registro nel {main} lowered + batteria spostata sul funnel vero.
5. **Il confine del flag è attaccabile da PHP** (Pedersen): putenv→set_var
   può decidere il modo del worker prima della prima lettura del OnceLock.
   Cura: lettura EAGER all'avvio + dente anti-putenv + fail-loud su
   unwrap_or(b"").
6. **«I confronti non lanciano» è falso in generale** (Stogov): opCoerce
   branda Left/Right; il mirror Lt↔Gt vive su tre contingenze non
   dichiarate — vanno pinnate in property-test di antisimmetria + fixture
   GMP/Number PRIMA di estendere i fold. Le sette fixture-trappola
   dell'eventuale fold AssignOp si scrivono PRIMA del fold.
7. **Igiene di misura** (Gregg): str/re non si dichiarano «rumore» senza
   banda (Δ0,04 < spread 0,08 va scritto così); deriva inter-build
   dichiarata; claim < 3× spread = nulli.

## Ordine proposto per S-98.0 (regola di ammissione applicata)

1. **M1 (misura, ~mezz'ora, PRIMA di ogni codice)**: micro noop 200M +
   census noop/prop/calls + ASM del preambolo corrente; predizione P
   scritta nel .out (P = 19·D/156,6 per arith). Se P < 10%: H-B1 cade a
   tavolino e si passa a H-B2 senza scriverla.
2. **Decisione H-B1 dal numero**: se sopravvive, forma = split-borrow del
   team forma-hb1, criterio = max(P/2, 0,7 ns/op) con caduta se arith
   flag-off > 7,2 s; KS: no unsafe/raw ptr, no mem::take, no ip locale.
3. **H-B2 (asse)**: specializzare UN opcode (Binary Add int-int deciso a
   compilazione) e misurare `arith`; guardia contata (KS-ST-99-3).
4. **Debiti bloccanti ammessi** (apparato che blocca, timebox ½ sessione
   in tutto): parità server restapi+option per NOME sotto env -i (blocca
   OGNI uso del server col binario nuovo — pin attuale NON verificato);
   smoke flag-ON con controllo positivo dump nel CI di sessione; assert
   {main} della batteria (chiude la vacuità di copertura).
5. **A BACKLOG per NOME** (non bloccano l'oggetto): delibera manifest
   94/95; property-test antisimmetria mirror + fixture GMP/Number; flag
   eager + dente anti-putenv; dente N_OPS<256; coppia peak al prossimo
   collaudo WP; bande str/re nel prossimo run-micro; fold coda AssignOp
   (dopo le sette trappole di Stogov).

L'ordine NON è composto di solo apparato: i punti 1-3 sono l'oggetto.

## Conflitti registrati (nessuno di sostanza)

- forma-hb1: sonda-per-addizione (Matsakis) vs sola predizione (Hoare) —
  risolto dall'ordine: M1 è la sonda, la predizione la accompagna.
- team-giudice: B1-prima-di-B2 vs tetto-misura: B2-asse — riconciliato al
  punto 2: B1 sopravvive SOLO se M1 le dà un numero sopra soglia.


════════════════════════════════════════════
## FILE: verbali/verbale-1-hoare.md

# Verbale Sedia 1 — Hoare (design linguaggio/runtime Rust, safe-only) — Concilio WP-99

## VERDETTO: CON EMENDAMENTI

Su S-97.1 in sé non ho refutazioni: la disciplina del criterio scritto ha morso
(−30,7% < −40% → abbandono senza negoziare), e la rimozione del fold commutativo
const-lhs è la decisione giusta presa per la ragione giusta. Le verifiche del mio
perimetro, sul codice:

1. **`reg_store_slot` vs `Op::StoreSlot` (run.rs 655–672 vs 244–257): replica
   esatta.** Il pop/push del ramo typed_refs di StoreSlot è un puro transito del
   valore verso `typed_ref_assign`; tra push e secondo pop non accade nulla, e
   entrambi i path ri-indicizzano `slots[i]` FRESCO dopo la coercizione. Flusso
   del valore, ordine `store_slot`→`gc_note`, stato pila al momento di un
   eventuale `Err` (profondità d'ancora identica): equivalenti. Non refutabile.
2. **`reg_load_slot` su Undef: momento del warning corretto.** Stessa coda
   `self.diags` di `Op::LoadVar` (anch'esso «queued; flushed at the next emit
   point»), stesso punto di flush a valle, stesso ordine l-poi-r nel funnel
   (anche `$u+$u` → due warning, come la sequenza originale).
3. **Bitrot: parzialmente presidiato.** La batteria v3 ESEGUE il codice lowerato
   (`assert_eq!(run(&m), run(&lm))`, via `lower_func` diretto: gira a ogni
   `cargo test` senza dipendere dall'env). Ma vedi A-HO-99-2/3: due rami sono
   scoperti.

## Refutazione capitale (una): R-HO-99-CAP — il confine di H-B1 come scritto è FALSO

«Frame in registro, ricaricato SOLO ai confini call/ret/throw»: l'insieme dei
confini è sottostimato di un ordine. Sono archi di ri-entrata nel VM, presenti
dentro quasi ogni handler: `gc_note` (→ `__destruct`), `flush_diags`
(→ `set_error_handler`), `binary_value_ab` (→ `__toString`, overload GMP/BcMath,
init_trigger lazy), `typed_ref_assign`, `to_bool` su oggetti. In Rust SAFE un
`&mut Frame` cache-ato attraverso questi archi NON COMPILA — quindi niente UB di
aliasing, ma la premessa «ricarico solo a call/ret/throw» è irrealizzabile come
scritta: o si ricarica a OGNI arco (e il risparmio va ristimato su quella
frequenza), o si estrae il frame da `self.frames` (`mem::take`) accecando GC
root-scan e backtrace durante la ri-entrata. La forma safe onesta è lo
split-borrow strutturale (`Vm` = pila frame + resto; handler su
`(&mut Frame, &mut VmRest)`), dove il borrow checker FORZA la resa del frame a
ogni arco: la refutazione colpisce il testo dell'ipotesi, non l'obiettivo.

## Emendamenti

- **A-HO-99-1**: in `reg_load_slot`, `debug_assert` che `unit_slot_name` sia
  byte-uguale al name-const del LoadVar foldato (oggi il contratto WP-65 è
  fidato, non verificato; il ramo Undef è freddo, costo nullo).
- **A-HO-99-2**: snippet in batteria con typed property legata by-ref a un
  locale scritto da una forma `*Dst`: il ramo typed_refs di `reg_store_slot` è
  oggi NON esercitato da alcun test (messaggio di coercizione e TypeError
  byte-identici all'oracle).
- **A-HO-99-3**: test flag-on di attribuzione di RIGA su espressione multilinea
  che lancia TypeError (`$a\n+ $b`): l'op fuso siede alla posizione della
  finestra; il corpus flag-on non è mai stato diffato, la riga può slittare.
- **A-HO-99-4**: il criterio di H-B1 va scritto come NUMERO con predizione di
  canale PRIMA del codice (ns attesi da 4 indicizzazioni + 2 `len()`): «sotto
  il rumore della coppia R=3» è ~0,5% su 7,83 s — un predicato quasi vacuo
  contro un fattore 8, della famiglia già vietata («soddisfatto dal proprio
  testo»).
- **A-HO-99-5**: prima del design, ENUMERARE l'insieme vero degli archi di
  ri-entrata (lista sopra) e adottarlo come confine; realizzazione via
  split-borrow, mai via indice/puntatore cache-ato.

## Kill-switch

- **KS-HO-99-1**: se H-B1 introduce `unsafe`, raw pointer o cache di
  `slots.as_mut_ptr()`/`NonNull<Frame>`, la sessione si ferma (safe-only).
- **KS-HO-99-2**: se richiede `mem::take` del frame fuori da `self.frames`
  attraverso un arco di ri-entrata, H-B1 cade (GC/backtrace ciechi).
- **KS-HO-99-3**: nessuna promozione futura del flag-on a strada di parità
  senza corpus 1418 per NOME eseguito flag-ON (il verde attuale è flag-off).

════════════════════════════════════════════
## FILE: verbali/verbale-2-matsakis.md

# Verbale Sedia 2 — Matsakis (ownership/aliasing/borrow) — Concilio WP-99

**Oggetto**: report S-97.1 (H-A1 caduta, codice dormiente) + programma H-B1.

## VERDETTO: CON EMENDAMENTI

S-97.1: nulla da refutare sul piano borrow del codice spedito. H-B1 **come
formulata è irrefutabilmente NON scrivibile in Rust safe**: va vincolata (A-MA-99-1)
e sottoposta a una sonda preventiva (A-MA-99-2), altrimenti MI OPPONGO.

## Analisi del codice (run.rs)

**Fast-path a borrow (BinarySS/SSDst/CmpJmpSS ~979-1077).** Il pattern è sano:
`fr = &self.frames[top]`, due borrow condivisi sugli slot, `binary_fast` ritorna
`Option<Zval>` **posseduto** → per NLL il borrow muore al `break 'r v` e il push
successivo su `self.frames[top].stack` è libero. Non è fragile per caso: è il
COMPILATORE a garantirlo — se un refactoring facesse ritornare a `binary_fast`
un prestito (`&Zval`, `Cow`) o le passasse `&mut self`, NON compila. Il
canale che invece il borrow-checker **permette in silenzio**: passare a
`binary_fast` un `&mut self.diags` per split di campo (frames shared + diags
mut su campi distinti COMPILA) e farle emettere warning — divergendo dal
funnel del miss-path (doppio diag / ordine). Da sbarrare per contratto:
KS-MA-99-1. Alias `l == r` ($x+$x): due borrow condivisi, legale e corretto.
Nota minore: BinarySC/SCDst/CmpJmpSC materializzano `cv` PRIMA del fast-path
anche sul hit — un bump Rc per dispatch sui const ZStr (commentato deliberato;
su `arith` i const sono Long, costo nullo).

**reg_store_slot (244-257) vs StoreSlot (655-672).** Stesso ordine essenziale:
entrambi clonano l'Rc della cella (per staccare il prestito da `self.frames`
prima di `typed_ref_assign(&mut self)`) → coercizione → `store_slot` →
`gc_note(old)`. La differenza è il **cammino d'errore**: StoreSlot fa
pop→coerce(`?`)→push→pop, quindi su `Err` il valore è già stato POPPATO e
consumato; in reg_store_slot il valore non è mai stato sulla pila. Profondità
di pila diverse all'unwind ⇒ se il catch tronca la pila, può cambiare
l'ORDINE dei drop dei temporanei abbandonati (dtor osservabili in PHP). Oggi
flag-only, zero rischio parità; se mai promosso: fixture nominata in
A-MA-99-3.

## H-B1 — refutazione della forma, proposta della forma vincolata

1. **La formula letterale è impossibile in safe**: un `&mut Frame` (prestito
   di `self.frames`) tenuto attraverso il match mentre gli handler chiamano
   metodi `&mut self` è E0499 da manuale. `mem::take` del frame: VIETATO —
   gli osservatori ambientali cross-frame (backtrace, `current_frame_args`,
   `var_dyn_read`) vedrebbero un frame fantoccio (unsoundness semantica, non
   di memoria). Puntatori raw: fuori policy. `split_last_mut` per-opcode:
   è l'indicizzazione attuale sotto altro nome.
2. **I confini dichiarati (call/ret/throw) sono SBAGLIATI**: `cur_line(top)`
   legge `frames[top].ip` per la riga dei diag, e i warning nascono DENTRO
   gli handler (binary_value_ab). Un `ip` locale ricaricato "solo ai confini"
   sposta le righe dei messaggi in silenzio. KS-MA-99-2.
3. **Forma SAFE realistica (A-MA-99-1)**: loop interno su split di campo —
   `let fr = self.frames.last_mut()` + campi read-only, vivo ATTRAVERSO le
   iterazioni; `fr.ip` resta il campo vero (niente staleness); qualunque
   opcode che richieda un metodo `&mut self` (funnel, gc_note, chiamate) =
   confine: break al loop esterno. Costo onesto: il set caldo va reso
   method-free (gc_note/typed_refs oggi sono metodi ⇒ o si esce, o si
   rifattorizza a funzioni libere su campi splittati). Guardia profondità
   dove `frames` cresce: corretta (i frame crescono solo ai call).
4. **Refuto il conto dei guadagni**: 4 bounds check preveduti + 2 `len()`
   sono rami quasi-gratis su un colosso da ~9,9 ns/opcode; e il beneficio
   copre SOLO il sottoinsieme fast-path — proprio S-97.1 mostra che il costo
   sta negli opcode che entrano nel funnel `&mut self`. Prima di scrivere:
   **sonda per ADDIZIONE** (A-MA-99-2), due indicizzazioni ridondanti in più
   per opcode dietro flag; se `arith` non sale oltre il rumore R=3, togliere
   le quattro non può dare un calo netto ⇒ H-B1 cade senza essere scritta.

## Emendamenti

- **A-MA-99-1**: H-B1 riformulata come "loop interno su split-borrow di campo,
  confine = ogni opcode che richiede un metodo `&mut self`", non "&mut Frame
  attraverso il match".
- **A-MA-99-2**: sonda per addizione (+2 indicizzazioni/opcode dietro flag)
  PRIMA di implementare; criterio numerico scritto in apertura.
- **A-MA-99-3**: fixture nominata (non urgente finché flag-only): TypeError da
  typed-ref dentro un catch con temporaneo dotato di distruttore sulla pila —
  ordine dei drop StoreSlot vs reg_store_slot.

## Kill-switch

- **KS-MA-99-1**: `binary_fast` resta `fn(BinOp, &Zval, &Zval) -> Option<Zval>`
  — pura, senza canale diags né accesso a self. Ogni allargamento di firma =
  gate rosso (divergenza fast-path/funnel che il borrow-checker NON vede).
- **KS-MA-99-2**: vietato cachare `ip` in un locale staccato dal frame con
  write-back "ai confini": o `ip` resta il campo del frame mutato attraverso
  il prestito vivo, o flush prima di OGNI sito che può emettere diag/errore.
- **KS-MA-99-3**: vietati `mem::take` del frame e puntatori raw nel run_loop
  (safe-only); qualunque forma che renda il top-frame invisibile agli
  osservatori cross-frame è respinta a priori.

## Refutazioni capitali

**Sì, due**: (1) la formulazione letterale di H-B1 non è scrivibile in Rust
safe e il suo insieme di confini è sbagliato (ogni sito di diag è un confine
per l'osservabilità di `ip`); (2) il conto dei guadagni attribuisce al
preambolo un costo mai misurato — la sonda per addizione deve precedere il
codice.

════════════════════════════════════════════
## FILE: verbali/verbale-3-klabnik.md

# Verbale sedia 3 — Klabnik (spec, testabilità, matrici, gate) — Concilio WP-99

## VERDETTO: CONCORDO CON EMENDAMENTI

Il verdetto H-A1 (caduta sul criterio scritto) è ben evidenziato e la parità
flag-off è solida. Rifiuto però lo STATO che il report attribuisce a due cose:
il gate cifre e il pin 9,87 ns.

## Refutazioni capitali: SÌ (due)

**R1 — «Gate cifre --all PASS» prova meno di quanto afferma.** Le 4 righe
manifest nuove violano la carta del manifest stesso (blocco A-SK-71, righe
37–44): `WP_SESSION_97.md` — l'ULTIMA sessione, doc di rotazione ATTIVO — è
registrata `judge=no`, mentre WP_SESSION_94 e 95, sessioni CHIUSE, restano
`judge=yes`. La regola è violata in entrambe le direzioni: le cifre della
sessione attiva sono NON giudicate. Inoltre i tre `.out` gemelli di S-97.0
(`micro-baseline.out`, `arith-decomposition.out`, `ha2-sweep.out` — stessa
classe, stessa directory, stessa cifra-corpus) non hanno riga ALCUNA: o la
classe esige la riga (e allora il dente bidirezionale non ha morso: FAIL
latente), o non la esige (e allora le tre righe nuove sono decorative). PASS
sì, ma su un perimetro incoerente con la propria carta.

**R2 — Il pin che «decide la prossima mossa» (9,87 ns, −30,7%) proviene da uno
strumento non collaudato.** È misurato flag-ON, e la parità flag-ON sul corpus
per NOME fu fatta in WP-44 su un albero di luglio; da allora: liveness
riclassificata, `reg_load_slot` seed-aware, fold const-lhs rimosso, elisione
Sweep H-A2. Su QUESTO albero il flag-on è attestato da 13 snippet. Un numero di
grado VERDICT che orienta il programma non può poggiare su uno strumento mai
ri-collaudato.

## Matrice di fold e buchi dei TEST

La matrice implementata è coerente con la spec del module-doc, con tre riserve:
(a) `[LoadVar,PushConst,CmpJmp]` non è gestita — regge sull'invariante NON
scritto «slot-vs-const compare ⇒ sempre CmpJmpConst all'emissione», mai
asserito; (b) il non-fold Spaceship const-first (`3 <=> $x`) non ha alcun test
(nel battery `5 <=> 3` è const-const); (c) la ragione stessa del fold rimosso —
l'ORDINE degli operandi in "Unsupported operand types" — non ha una fixture che
faccia scattare il TypeError nei due ordini. Mancano inoltre: jump-target a
METÀ finestra (guardia `blocked` esiste, zero test strutturali), finestra su
linee sorgente MISTE (guardia `lines` mai esercitata, parità del numero di
riga del warning non provata), indici >u16::MAX (guardie = rami morti), e il
check del battery sugli `Addr` accetta QUALUNQUE indirizzo in range — solo la
parità dinamica può smascherare un remap sbagliato-ma-in-range, con un solo
snippet try/catch. Infine: `lowered()` nei test enumera i corpi a mano
«rispecchiando il funnel», ma nulla pinna che i due insiemi coincidano (i
corpi dei property hook?).

## Il dormiente ha un gate? Solo a metà

`cargo test` esercita `lower_func` e i 7 handler via `run(&lm)` — ma bypassa
ESATTAMENTE le cuciture dove il dormiente marcirà: il funnel `enabled()`, la
chiave `reg_mode` della unit-cache, il corpus. E
`stage2v3_flag_off_emits_no_register_forms` si auto-disattiva in silenzio se
`PHPR_REG_LOWER` è esportata: un dente spegnibile dalla cosa che sorveglia.

## Programma H-B1

Il criterio «scende in modo netto … sotto il rumore della coppia» è un criterio
di CADUTA, non di successo: col rumore a ~0,5% quasi ogni effetto reale passa.
H-A1 cadde a −30,7% contro 40; H-B1 così scritto vincerebbe a −2%. Il numero
va derivato dall'obiettivo: ≤3× su arith ⇒ ~1,4 ns/op complessivi contro 8,24.

## Emendamenti

- **A-KL-99-1**: una delibera che ripari il manifest: WP_SESSION_97 `judge=yes`
  (o carta emendata), 94/95 declassate, riga (o esclusione dichiarata di
  classe) per i tre `.out` di S-97.0.
- **A-KL-99-2**: smoke flag-ON nel CI di sessione: `PHPR_REG_LOWER=1` su
  arith_small + controllo positivo `PHPR_DUMP_OPS` (il dump DEVE contenere
  `BinarySSDst`) — la lezione del `tail` codificata come dente, non come prosa.
- **A-KL-99-3**: completare la batteria: target a metà finestra (assert
  strutturale), finestra multi-linea + parità della riga del warning,
  Spaceship const-first non-fold, TypeError operand-order nei due ordini,
  guardia u16 esercitata; più un test che pinna lowered() ≡ funnel di
  produzione sull'insieme dei corpi.
- **A-KL-99-4**: gli `.out` devono autodescriversi: `micro-ha1-{on,off}.out`
  sono copie byte-identiche dell'header S-97.0 e NON registrano lo stato del
  flag — la provenienza vive solo nel nome del file.
- **A-KL-99-5**: il test flag-off non deve auto-skipparsi in silenzio: se
  l'ambiente inverte la premessa, FALLIRE rumorosamente.
- **A-KL-99-6**: il criterio di H-B1 si scrive in ns/op derivato
  dall'obiettivo 3×, non dal rumore.

## Kill-switch

- **KS-KL-99-1**: nessun futuro `.out` di grado VERDICT può pubblicare un
  numero flag-ON senza citare una parità corpus per NOME flag-ON eseguita
  sullo STESSO albero.
- **KS-KL-99-2**: se alla chiusura di S-98 il manifest ha ancora l'ultima
  WP_SESSION `judge=no` (o i tre `.out` di S-97.0 senza delibera), la riga
  «gate cifre PASS» non si scrive nel report di chiusura.

════════════════════════════════════════════
## FILE: verbali/verbale-4-hejlsberg.md

# Verbale Sedia 4 — Hejlsberg (compilatori incrementali, interning, pipeline)

## VERDETTO: CON EMENDAMENTI

S-97.1 è metodologicamente pulita (criterio scritto prima, onorato; parità per
NOME; controllo positivo del flag). Le refutazioni colpiscono la BATTERIA di
parità, il POSTO del residuo AssignOp e il testo del programma H-B*.

## Refutazioni capitali

**RC-1 — La batteria v3 gira a un punto di pipeline DIVERSO dalla produzione,
e per `{main}` è plausibilmente VACUA.** In produzione `lower_func` gira dentro
`compile_body` (func.rs:163-165), PRIMA della cessione WP-65; poi
`compile_program_impl` fa `main.seed_slots = len; main.slot_names =
Box::default()` (mod.rs:326-327). Il harness di test `lowered()`
(reg_lower.rs:360-382) applica il pass all'output di `compile_program` — DOPO
la cessione — dove `{main}.slot_names` è VUOTO: `fold_slot` (riga 85, egualità
col nome) non può matchare alcun `LoadVar` di `{main}`. I 13 snippet della
batteria sono TUTTI script top-level: per i loro corpi `{main}` il pass testato
è con ogni probabilità l'identità. La sessione ha appena imparato «pretendere
la prova positiva che il flag ha morso» (tail/binario stantio) e la sua stessa
batteria non la pretende: `stage2v3_rewrites_hot_windows` asserisce forme fuse
solo in `fn f`, mai in `{main}`. Finché manca il controllo positivo, le gambe
`{main}` della batteria non sono evidenza.

**RC-2 — «Il conteggio è quasi chiuso (11 vs 7)» confonde lo strumento col
motore.** 11 è flag-ON, dormiente; la strada di parità resta a 19 (fattore
2,7×). La scala H-B2/H-C/H-D si attiva «dopo H-A1 e H-B1», ma H-A1 è caduta e
il suo effetto non raggiungerà MAI il binario spedito senza una decisione di
promozione che il programma non nomina. Il ladder condiziona su uno stato
irraggiungibile per costruzione.

**RC-3 — Il residuo AssignOp nel pass è il posto SBAGLIATO; e il quesito
sull'ordine è mal posto.** (a) Ordine: `thread_jumps` è strettamente in-place
(nessun op inserito/rimosso), quindi l'insieme delle finestre è essenzialmente
invariante all'ordine — spostare il pass prima del threading non allunga
nulla. Il limitatore vero è la guardia UNA-RIGA (`f.lines[j] == line`,
reg_lower.rs:189): i micro sono loop a riga singola, il codice WordPress è
formattato multi-riga — la resa del pass su codice reale è NON misurata e
plausibilmente molto sotto il 42% del micro. (b) Il fold di `LoadSlot` non è
bloccato da una necessità ma da un'EREDITÀ: la regola «mai foldato» esiste per
la parità del warning di `LoadVar`, e `LoadSlot` è silente — non c'è warning
da risintetizzare. Le guardie NECESSARIE sono altre: semantica INTEGRALE di
`StoreSlot` (write-through dei ref, gc_note) e NIENTE store se `Binary` lancia
(ordine osservabile via TypeError/distruttori). (c) Il precedente è già in
albero: `ConcatAssignSlot` (expr.rs:99, WP-55) è ESATTAMENTE la forma fusa
slot-diretta di AssignOp, emessa flag-off nel percorso di parità. Il residuo
va lì — emissione, non `fuse_window`: beneficia la parità (19→~15), è immune
alla guardia di riga, e paga il collaudo WordPress per regola n.2.

**RC-4 — Costo di compilazione del pass: NON misurato.** O(n) ma ~4 traversate
più rebuild completo con `clone()` di ogni op anche a zero finestre; ammortato
dall'unit-cache (`reg_mode` in chiave), accettabile SOLO finché flag-on resta
strumento di misura. L'oracle compile-side esiste (`--list-tests`, WP-64) e
non è stato usato.

## Emendamenti

- **A-HE-99-1**: controllo positivo nella batteria: asserire ≥1 forma fusa in
  `{main}` con il pass invocato al punto di pipeline di PRODUZIONE (env-flag
  in-process o funnel reale), non su modulo post-cessione.
- **A-HE-99-2**: il residuo AssignOp si progetta come fratello di
  `ConcatAssignSlot` all'emissione flag-off (guardie: StoreSlot integrale,
  no-store-on-throw), con collaudo WordPress; NON come quarto fold del pass.
- **A-HE-99-3**: sequenziare rispetto a H-B1: se l'emissione fusa entra, la
  baseline 8,24 ns flag-off si muove — pinnare prima/dopo esplicitamente; e
  riscrivere le condizioni di attivazione H-B2/H-C/H-D senza il riferimento a
  H-A1 dormiente (nominare la decisione promozione/scarto).

## Kill-switch

- **KS-HE-99-1**: nessuna forma registro va flag-off senza coppia compile-side
  `--list-tests` E collaudo di parità WordPress; violazione → reject.
- **KS-HE-99-2**: ogni fusione AssignOp all'emissione porta PRIMA del merge un
  test del percorso di lancio (TypeError dal Binary ⇒ slot NON scritto,
  osservabile via distruttore).
- **KS-HE-99-3**: finché A-HE-99-1 non è verde, le gambe `{main}` della
  batteria v3 non contano come evidenza di parità del pass.

*Refutazioni capitali: sì (RC-1..RC-4). Indipendenza: nessun verbale altrui letto.*

════════════════════════════════════════════
## FILE: verbali/verbale-5-bak.md

# Verbale sedia 5 — Bak (V8/HotSpot: dispatch, code-cache, path caldi, alloc-rate)

**Oggetto**: report S-97.1 (H-A1 caduta) + programma H-B1.
**VERDETTO: CON EMENDAMENTI.**

## 1. La caduta di H-A1 NON è spiegata dai corpi caldi — è aritmetica dei costi (refutazione capitale n.1)

Conto dai `.out`: a costo UNIFORME, 19→11 op/iter darebbe 11/19 × 7,83 = 4,53 s
= **−42,1%**, appena sopra la soglia. Il −40% richiesto presupponeva quindi che
gli 8 opcode rimossi (LoadVar/PushConst/Dup/Pop — i più economici) costassero
quanto l'opcode MEDIO. Ma la non-uniformità era già MISURATA in ha2-sweep
(Sweep noop ≈ 1/5 del medio), stessa sessione che ha scritto il criterio.
**Il criterio era irraggiungibile per la fisica già nota quando fu scritto: la
caduta è un verdetto sul criterio, non sul meccanismo.** Il marginale reale
dei rimossi è (7,83−5,43)/(8×50M) = **6,0 ns per opcode rimosso** — SOPRA la
stima cheap-op (~2-3 ns): H-A1 ha reso più del suo modello, perché i corpi
sostitutivi (fast-path a borrow) sono più economici dei pop/push che
rimpiazzano. La regola n.3 fu applicata bene; la taratura no.

Sul modello WP-44: **non si applica al micro**. Su `arith` il working set
eseguito flag-on è ~10 corpi contro ~11 flag-off — non cresce; tutto sta in
L1I e il target predictor impara la sequenza fissa di 11 opcode. Il +1% di
WP-44 era pressione I-cache/BTB dell'AGGREGATO megamorfo. Quindi una forma
che SOSTITUISCE invece di affiancare **non renderebbe di più sul micro**
(stesso working set eseguito); il suo claim appartiene al giudice aggregato,
e cambierebbe l'emissione di parità — prematuro.

## 2. I 7 arm morti flag-off: WP-33 NON si applica

Il +2,9% di WP-33 era un `if bool` VALUTATO a ogni dispatch. Un arm di match
mai dispatchato è codice freddo dietro una entry di jump-table: zero
istruzioni per tick; costa solo se sposta il layout dei handler caldi.
Evidenza: 7,88 (ha2) → 7,83 (ha1-off) = **−0,6%, segno OPPOSTO a un costo**,
dentro lo spread R=3. Bound onesto: costo ≤ ~1% (binari di HEAD diversi:
è un bound, non una prova di zero). Non rilitigarlo.

## 3. Il tetto di H-B1 è GIÀ MISURATO (refutazione capitale n.2)

ha2-sweep è il controfattuale: un dispatch noop completo (preambolo: guardia
`frames.len`, 3-4 indicizzazioni bound-checked, fetch `ops[ip]`, store
`ip+1`, match, handler-vuoto) costa al margine (7,95−7,88)/50M = **1,4 ns**.
Dunque il PREAMBOLO vale ≤ 1,4 degli 8,24 ns/op: **H-B1 in scope
«preambolo» ha un tetto di −17% sul costo per opcode** — non può «scendere
in modo netto» oltre, e non tocca gli ~8,5 ns che vivono DENTRO i corpi
(catene `self → frames → frame → slots/stack` per operando, push su
`Vec<Zval>`, lavoro Zval — è lì il fattore 8 contro il frame-in-registro di
Ignition/Zend). Un criterio che pretenda più del tetto ripete l'errore del
−40%. Lo scope «frame in registro ANCHE dentro i corpi» non è limitato dal
1,4 ns, ma in Rust safe collide con `&mut self` nei handler: costo
architetturale da dichiarare prima.

## Emendamenti

- **A-BA-99-1**: il criterio di H-B1 si DERIVA dal controfattuale misurato,
  non da una cifra tonda. Scope preambolo: cade se il risparmio < 0,7 ns/op
  (metà del tetto), cioè `arith` flag-off netto > 7,2 s sulla coppia R=3
  stessa-sera (8,24 → ≤ 7,55 ns/op), con spread < metà del delta.
- **A-BA-99-2**: prima di scrivere, dichiarare lo SCOPE (solo-preambolo vs
  frame-esteso) e la predizione-misurata: conteggio statico dei re-borrow
  `self.frames[top]` eliminati per opcode del residuo arith.
- **A-BA-99-3**: ritirare la tariffa «corpi caldi» come obiezione ai micro;
  vale solo per il giudice aggregato (WP-44 resta valido lì).
- **A-BA-99-4**: H-B2 va preparata sulla stessa tavola di decomposizione:
  gli 8,5 ns residui sono corpo-interni, non preambolo.

## Kill-switch

- **KS-BA-99-1**: se il criterio scritto in apertura S-98.0 eccede il tetto
  dello scope dichiarato (1,4 ns/op per il preambolo), il criterio è INVALIDO
  e va riscritto prima di toccare codice.
- **KS-BA-99-2**: ogni misura futura flag-off su `arith` che regredisca >2%
  vs 7,83 a ricetta identica sospende le ipotesi nuove finché il layout
  (arm morti/inlining) non è escluso con una coppia stessa-sera.

## Refutazioni capitali

1. **Il criterio −40% era irraggiungibile con dati già posseduti** (uniforme
   = −42,1%; non-uniformità nota da ha2): H-A1 è caduta su una taratura, non
   sul meccanismo.
2. **H-B1-preambolo ha un tetto misurato di 1,4 ns/op (−17%)**: presentarla
   come l'asse che chiude il fattore ~8 per opcode è refutato dai numeri del
   progetto stesso; il fattore vive nei corpi (pointer-chasing + traffico di
   pila Zval), cioè H-B1-esteso/H-B2.

════════════════════════════════════════════
## FILE: verbali/verbale-6-pedersen.md

# Verbale Sedia 6 — Pedersen (Concilio WP-99)

Perimetro: confine per-richiesta/test, lifecycle, igiene di stato fra run.
Oggetto: report S-97.1 + programma H-B1. Mandato: refutare.

## VERDETTO: CON EMENDAMENTI

## Refutazioni capitali

**R1 — «Il flag è fisso per processo» è un COMMENTO, non un invariante.**
`reg_lower::enabled()` (reg_lower.rs:50-53) è un OnceLock letto alla PRIMA
chiamata; `putenv()` PHP (php-builtins/src/file.rs:1267-1284) chiama
`std::env::set_var` process-wide, e il suo stesso doc dice «safe under
per-process --isolate» — il server è esattamente il caso NON isolato. Su un
worker long-lived in cui la prima compile non è ancora avvenuta, una
RICHIESTA che fa `putenv("PHPR_REG_LOWER=1")` decide il modo dell'intero
processo per sempre: stato di richiesta promosso a configurazione di motore.
Nessun dente lo copre; l'ordinamento «la prelude compila prima» è un claim
mai pinnato.

**R2 — «Parità server da riverificare al primo uso» SOTTOSTIMA il debito.**
Non è igiene facoltativa: l'emissione flag-off È cambiata (l'elisione Sweep
di H-A2 è INCONDIZIONATA, non dietro il flag), quindi per la regola n.2 il
collaudo di parità sull'emissione nuova è dovuto, e lato server non è MAI
girato. Il binario 832568a72b925dd1 non è «lo stesso motore ricompilato».

**R3 — Handoff incoerente sul pin server.** NEXT_SESSION §Stato gate porta
ancora `php-server: f8f4295a1dcdb627`; WP_SESSION_97 dichiara la rotazione a
`832568a72b925dd1`. Due fonti di verità divergenti sulla riga che il
pre-flight della prossima sessione leggerà — violazione diretta della regola
single-source.

**R4 — I contatori NON distinguono le popolazioni.** `UnitKey.reg_mode`
separa correttamente le cache, ma UcStats/uc_log non portano il modo: un
mode-miss è indistinguibile da un `miss_cold` genuino, e un log condiviso da
due processi con env diversi mescola due popolazioni senza marcatore.

## Verificato (non refutato)

Il canale Ref del bridge (WP-33): il fast-path scarta `Ref` verso il funnel;
`reg_load_slot` (run.rs:228-240) ha lo STESSO predicato di `LoadVar`
(run.rs:636-653) — warning solo su `Undef` ESTERNO, un Ref che regge Undef
non avvisa in nessuno dei due, `read_slot` identico; testo byte-identico per
costruzione (fold solo se name==slot_names[slot] a pass-time, seed ceduto
DOPO il pass, contratto fp-guarded). Il re-lower deferred su unità già
cedute degrada chiuso (slot_names vuoto ⇒ nessun fold). MA:
`.unwrap_or(b"")` in reg_load_slot emette «Undefined variable $» (nome
vuoto) se il contratto è mai violato — silenziosamente sbagliato, contro
correct-or-absent e contro la disciplina fail-loud di `seed_prefix_breach`.

## Emendamenti

- **A-PE-99-1**: leggere il flag EAGER a un confine nominato (bootstrap
  processo/Vm), non lazy alla prima compile; test cargo che
  `putenv("PHPR_REG_LOWER=1")` da codice PHP non flippa il modo (dump
  bytecode invariato).
- **A-PE-99-2**: campo `reg=` nel vocabolario uc_log (o header one-shot per
  file); un A/B on/off è valido SOLO come due processi con env fissato allo
  spawn e registrato nel log.
- **A-PE-99-3**: `.unwrap_or(b"")` in reg_load_slot → fail-loud (stessa
  classe di seed_prefix_breach), mai un warning col nome sbagliato.
- **A-PE-99-4**: correggere SUBITO il pin server in NEXT_SESSION (R3).
- **A-PE-99-5 (H-B1, da scrivere nel criterio PRIMA del codice)**: «frame in
  registro, ricaricato ai confini call/ret/throw» è una lista INCOMPLETA:
  vanno enumerati TUTTI i siti dove `self.frames` può riallocare o mutare
  mid-opcode — rientranza builtin→VM (`call_method_sync`), `__toString` nel
  funnel di `binary_value_ab`, distruttori/GC che eseguono codice PHP. Un
  confine dimenticato è UB silenzioso, non un rallentamento.

## Kill-switch

- **KS-PE-99-1**: qualunque uso o misura del server su 832568a72b925dd1
  senza PRIMA restapi 3508 per NOME + option 413 per NOME sotto il launcher
  `env -i` a lista chiusa (PHPR_REG_LOWER ASSENTE dalla lista) e sentinella
  del binding output-capture verde = **VOID**.
- **KS-PE-99-2**: qualunque campagna che confronti flag-on/flag-off dentro
  UN processo = VOID by construction; e finché A-PE-99-1 non è chiuso, ogni
  run server deve conservare il log raw dell'ambiente allo spawn.

════════════════════════════════════════════
## FILE: verbali/verbale-7-leijen.md

# Verbale sedia 7 — Leijen (allocatore, footprint, layout) — Concilio WP-99

**Oggetto**: report S-97.1 (H-A1 caduta, 7 varianti dormienti dietro `PHPR_REG_LOWER`) e programma H-B1.
**VERDETTO: CONCORDO CON EMENDAMENTI.**

## 1. La premessa della domanda è REFUTATA dall'evidenza (a favore del report)

Il sospetto «`with_capacity(n)` senza `shrink_to_fit` ⇒ capacity ritenuta per unità» è
FALSO per ordinamento verificato: `lower_func` gira dentro `compile_body`
(`compile/func.rs:163`), mentre il funnel WP-48 `m.shrink()` gira a valle, a fine
`compile_program` (`compile/mod.rs:451`), e `Func::shrink`
(`bytecode.rs:1495-1497`) fa `ops.shrink_to_fit()` + `lines.shrink_to_fit()`.
L'eccesso (n−m)×48B ops + (n−m)×taglia`Line` è **transiente compile-side**, non
ritenuto. Il parallelismo `lines`/`ops` (una linea per finestra, indicizzata alla
testa) è preservato. Due caveat pinnati, non refutazioni: (a) `Module::shrink`
salta i `Func` Rc-condivisi (`Rc::get_mut`) — coperto dal commento «shrunk dal
modulo proprietario», ma un chiamante FUTURO di `lower_func` fuori dal funnel di
`compile_program` riterrebbe lo slack in silenzio; (b) sotto mimalloc lo
shrink recupera a granularità di size-class, già prezzato in WP-48.

## 2. Flag-off: zero byte di dati, ma «zero-delta» va PERIMETRATO

Le 7 varianti non allargano `Op`: pin **doppio e indipendente** verificato
(`==48` in `reg_lower.rs:571`; `≤48` + `Frame ≤176` in `vm/mod.rs:22051`).
Discriminant invariato — ma il census è a 185 righe: +7 porta l'enum a ~192
varianti, e il confine 256 (discriminant u8) ha ormai headroom ~60 senza alcun
dente che lo guardi. Il costo flag-off residuo è **codice**: 7 corpi handler nel
run_loop (lezione WP-38: branch mai-preso = +2,9%; caveat WP-98: non è tariffa).
Il tempo è coerente (7,83 vs 7,88 di ha2) — ma «flag-off zero-delta» è provato
per TEMPO e PARITÀ, **non per footprint**: taglia binario/icache non riportate,
e il footprint non è misurato da m90.

## 3. REFUTAZIONE CAPITALE: −42% di opcode ≠ −42% di byte

Il 19→11 è un conteggio di **dispatch dinamici sul micro `arith`**; i byte di
`ops` Vec sono **statici** e la frazione di finestre fondibili su codice
WordPress generico è ignota (i vincoli: sorgenti LoadVar/PushConst, stessa
linea, nessun target nel mezzo — su codice reale morde molto meno che su un
loop aritmetico). Chi presentasse lo stream più corto come leva di footprint
commette un errore di categoria. Plafone: anche azzerando TUTTE le `ops` Vec
ritenute si recupera solo la massa che la mappa WP-58/59 prezza — contro un
peak di 1901,11 MiB una promozione flag-on deve reggersi **sulla CPU
soltanto**; il guadagno footprint va misurato, mai dedotto dal census.

## 4. Il rischio che il mio perimetro deve nominare

La roadmap footprint è FERMA e non misurata da m90 — ma l'emissione **flag-off è
già cambiata** (H-A2, Sweep eliso, S-97.0) e il binario è cresciuto di 7
handler, senza alcuna coppia peak. La deriva si accumula non misurata. H-B1 è
attesa footprint-neutrale (niente layout, niente allocazioni): se il peak
cambiasse, è un segnale non richiesto da investigare, non da festeggiare.

## Emendamenti

- **A-LE-99-1**: dente sul discriminant: assert `N_OPS < 256` (census già
  esporta `N_OPS`) accanto al pin 48B — il confine u8 non si attraversa in silenzio.
- **A-LE-99-2**: al prossimo collaudo di parità WordPress (dovuto: l'emissione è
  cambiata), acquisire la coppia peak `/usr/bin/time -l` **nello stesso run** —
  zero run aggiuntivi, m90 si aggiorna gratis.
- **A-LE-99-3**: una riga di commento in `lower_func`: «lo slack di
  `with_capacity(n)` è recuperato dal funnel WP-48 a valle
  (`compile/mod.rs:451`); un chiamante fuori da `compile_program` lo ritiene».

## Kill-switch

- **KS-LE-99-1**: promozione di `PHPR_REG_LOWER` a baseline SENZA coppia peak
  stessa-sera (`/usr/bin/time -l`, entrambe le gambe) ⇒ **respinta**.
- **KS-LE-99-2**: se un pin 48B scatta o `N_OPS` ≥ 256 ⇒ stop e ridisegno del
  layout PRIMA di qualsiasi misura CPU.

## Refutazioni capitali

Sì, una (§3): «lo stream −42% è un guadagno di footprint» — conteggio dinamico
di micro spacciato per byte statici di carico reale; qualunque claim futuro in
quella forma è respinto in assenza di misura. (La premessa «shrink assente» del
mio stesso mandato è invece refutata dall'evidenza, §1: il report non ha il
difetto ipotizzato.)

════════════════════════════════════════════
## FILE: verbali/verbale-8-stogov.md

# Verbale sedia 8 — Stogov (Zend/opcache, semantica engine) — Concilio WP-99

## VERDETTO: CONCORDO CON EMENDAMENTI

Ho attaccato il mirror con casi concreti (overload GMP/BcMath, __toString, stringa-numero, array, warning/error ordering) e NON ho prodotto un controesempio vivo. Ma il mirror sopravvive per TRE fatti contingenti, nessuno dei quali è dichiarato come invariante dove il mirror vive — e il programma H-B1/H-B2 è sotto-specificato sulla soundness di ri-entranza.

## Refutazioni capitali

**RC-1 — «i confronti non emettono errori operando-tipizzati» (reg_lower.rs:24-25, doc di `mirror_cmp`) è FALSO come claim di engine.** Il confronto overloadato PUÒ lanciare: GMP `__cmp` → `_oparg` → ValueError (prelude_gmp.php:143); BcMath `opCoerce` lancia con branding POSIZIONALE — `"$side string operand cannot be converted…"`, side ∈ {Left, Right} (prelude_bcmath.php:285). Il mirror resta sano oggi solo perché: (a) per Number il gate `operand_cmp_ok` (vm/mod.rs:18164, `bc_str_wellformed`) devia le stringhe malformate al braccio *uncomparable*, che è simmetrico per costruzione; (b) il messaggio di GMP per caso non nomina la posizione; (c) il dispatch è a NOME ESATTO (`overload_receiver`, vm/mod.rs:18116-22), quindi una sottoclasse di GMP (classe NON final, prelude_gmp.php:14 — e in PHP vero è sottoclassabile CON operatori) non può ridefinire `__cmp` e rompere l'antisimmetria. Tre contingenze ≠ un invariante: la prossima modifica al prelude rompe il mirror in silenzio. Il vero invariante portante è: **antisimmetria di __cmp + errori position-free sul percorso compare** — va scritto e pinnato.

**RC-2 — «Eq-family unchanged» è impreciso**: `bin_dst` SCAMBIA gli operandi anche per Eq (const a rhs). È sano solo perché `Const` è scalar-only (bytecode.rs:103-112: nessuna variante array): il compare ricorsivo array==array con ordine di visita scambiato, e l'ordine di `realize_full` sui lazy, sono irraggiungibili DA COSTANTE. Se un domani il pool consts accoglie array (constant folding dei literal), la porta si riapre senza che alcun test scatti.

**Risposte puntuali al mandato**: mirror Lt↔Gt su IEEE regge anche con NaN (Lt(a,b)≡Gt(b,a), entrambi false); __toString: regola simmetrica, una sola chiamata in entrambi gli ordini; GMP float const troncato (`_oparg` (int)$v) identicamente nei due ordini. Warning «Undefined variable» risintetizzato: `reg_load_slot` (run.rs:228-240) ACCODA su `self.diags` esattamente come `Op::LoadVar` (run.rs:636-654, push, nessun raise); la materializzazione anticipata della const in BinarySC è pura; a op fuso che lancia (DivisionByZeroError) i diags accodati rendono prima del fatal in entrambi i modi → ordine coincidente. Residuo: la fonte del NOME diverge (const pool vs `unit_slot_name` seed-aware) — il contratto WP-65 «byte-identico per contratto» è asserito da commento, esercitato da NESSUNA fixture flag-on sul percorso unit-cache/{main} linkato.

## Emendamenti

- **A-ST-99-1**: fixture flag-on con slot Undef in finestra fusa ATTRAVERSO il {main} linkato dalla unit-cache (o `debug_assert!` in `reg_load_slot` che il nome risolto eguagli il nome foldato). Il contratto di byte-identità non deve vivere in un commento.
- **A-ST-99-2**: property-test di coerenza del mirror: `eval(op,a,b) == eval(mirror(op),b,a)` su binary_fast E sul funnel pieno, matrice tag × {NaN, ±0.0, i64::MAX±1, " 10", "1e1", stringhe malformate}; più snippet batteria con GMP/Number (`3<$g`, `"abc"<$g`, `"1e3"<$n`) asserendo flag-on≡flag-off INCLUSI i messaggi d'eccezione.
- **A-ST-99-3 (coda AssignOp, PRIMA di qualunque fold futuro)**: fixture scritte in anticipo per (a) timing RHS-first: `$x += ($x=5)` e mutazione by-ref nel rhs; (b) typed-ref: errore di coercizione PRIMA di ogni scrittura, su `module.strict` del file assegnante; (c) non-sovrapposizione con `ConcatAssignSlot` per `.=`; (d) overflow int→float senza wrap nel fast path fuso; (e) warning undef-lhs DOPO gli effetti del rhs e PRIMA dell'errore dell'op; (f) timing `gc_note`/__destruct del valore spiazzato; (g) il branding posizionale Left/Right di `__op` (RC-1) prova che la POSIZIONE è osservabile negli errori arith overloadati: il fold AssignOp non deve MAI commutare.

## Kill-switch

- **KS-ST-99-1**: qualsiasi allargamento di `overload_receiver` da nome-esatto a instanceof (necessario per la fedeltà all'oracle sulle sottoclassi GMP) NON può spedire insieme al mirror attivo sui compare overloadati senza escludere il mirror o pinnare l'antisimmetria: le due modifiche indipendenti = unsound silenzioso.
- **KS-ST-99-2 (H-B1)**: «ricaricare il frame SOLO ai confini call/ret/throw» è una cura ENUMERABILE contro un attacco NON enumerabile (S-96.0): la ri-entranza non è sintattica — `binary_value_ab` ri-entra via __toString/__cmp/__op (`call_method_sync`), `realize_full`, error handler, distruttori da `gc_note`; `self.frames` può riallocare. La disciplina di reload va imposta PER TIPO/borrow (guard invalidata da ogni chiamata `&mut self`), non per lista; se serve caching unsafe di puntatori senza guard, H-B1 cade su soundness prima di misurare. (La disciplina SAVE_OPLINE di Zend è storicamente una fabbrica di bug.)
- **KS-ST-99-3 (H-B2)**: una specializzazione int-int decisa a COMPILAZIONE è unsound in PHP (tipi dinamici: overflow int→float a metà loop). L'op specializzato deve guardare i tag con fallback generico, e il criterio deve contare il costo della guardia nel bersaglio ns/opcode — altrimenti la misura mente.

════════════════════════════════════════════
## FILE: verbali/verbale-9-gregg.md

# Verbale Sedia 9 — Gregg (metodologia di misura e attribuzione + mandato inverso)

## VERDETTO: CON EMENDAMENTI

La coppia stessa-sera R=3 sullo stesso binario è il metodo giusto e il −30,7%
su `arith` (2,40 s contro spread 0,04) è fuori rumore di due ordini. L'abbandono
sul criterio scritto è esemplare. Ma la sessione ha già in mano il dato che
LIMITA H-B1 e non ne ha tratto l'inferenza.

## Refutazione capitale (sì)

**«Il divario vive nel costo per opcode → quindi H-B1 preambolo» è un
non-sequitur refutato dai dati della sessione stessa.** `ha2-sweep.out` dà il
costo MARGINALE di uno Sweep noop: 0,07 s / 50M dispatch = **1,4 ns** — e uno
Sweep noop paga per intero il preambolo (fetch, bounds check, len). Quindi 1,4 ns
è un TETTO sull'overhead uniforme per dispatch. Su `arith` flag-off:
19 × 1,4 = 26,6 ns su 156,6 ns/iter = **tetto ~17%** (banda ~8–27%: il 0,07 s è
appena 2× lo spread). Un tetto del 17-27% non chiude un fattore 8 per opcode:
gli ~8,5 ns residui del residuo stanno nei CORPI (discriminazione di tipo, lavoro
Zval) — cioè H-B2, non H-B1. La lezione «il costo non è uniforme» dice la stessa
cosa e la sessione l'ha scritta senza applicarla all'ordine delle ipotesi.

## Risposte alle domande di perimetro

- **Collaterali**: sorretti. prop 6,27→5,50 (Δ 0,77 s vs spread 0,04-0,05, ~15×),
  calls 4,02→3,39 (Δ 0,63 vs 0,05), arr Δ 0,17 vs 0,01. Ma sono numeri SENZA
  attribuzione: nessun census flag-on/off su prop/calls — quanti Binary/CmpJmp
  per iterazione spiegano il −12,3%? Un calo senza meccanismo contato non è
  conoscenza dell'oggetto.
- **«str/re = rumore»**: classificazione DISONESTA per omissione. str: Δ 0,04 s
  con spread 0,08 sul lato off → NON RISOLTO, banda ~[−11%, +4%] — che è diverso
  da «zero». re: Δ 0,02 s con risoluzione di `time -p` 0,01 s → al limite dello
  strumento. Va scritta la banda, non l'etichetta.
- **7,83 vs 7,88/7,95**: binari DIVERSI (0dd98eb vs 2f6c1a/d5ce86e); la deriva
  inter-build (0,6%, stessa scala dello spread) NON è nominata in
  `ha1-registers.out` («coerente con 7.88» la usa come conferma senza dichiarare
  il caveat). Qui è innocua — anzi è la prova timing dello zero-delta flag-off —
  ma va nominata come termine.

## Ricetta ESATTA per attribuire il preambolo PRIMA di H-B1 (misura M1, zero codice VM)

1. `noop.php`: `for($i=0;$i<200000000;$i++){}` — census → ops/iter (attesi
   ~4-5: IncDecSlot, CmpJmpSC, Jump, Sweep[, Pop]); tempo con la ricetta
   run-micro.sh → **D = ns/dispatch dei soli op economici** con braccio di leva
   4 miliardi di dispatch (statistica stretta, non il Δ 0,07 s di ha2).
2. ASM del binario CORRENTE: contare i bounds check/len davvero emessi nel
   run_loop (prof95-media §PREAMBOLO è di un'ALTRA build; LLVM può già eliderne).
3. Predizione scritta nel .out PRIMA del codice: **P = 19·D/156,6 ns** (quota
   massima di `arith` flag-off attribuibile al preambolo).

**Criterio di caduta che ne segue**: H-B1 cade A TAVOLINO se P < 10%; se
scritta, cade se il calo misurato < max(P/2, 3× lo spread relativo della coppia
≈ 1,5%). Caveat da scrivere: D è marginale in pipeline out-of-order, quindi
tetto SOFFICE verso il basso.

## Mandato inverso — l'OGGETTO

**Sappiamo oggi**: (1) il fattore conteggio è chiudibile (19→11 vs 7, shape in
albero); (2) il residuo è ~8× PER OPCODE, pinnato: 11 op a 9,87 ns vs 7 a 1,23;
(3) il dispatch nudo costa ~1,4 ns → il grosso è nei corpi; (4) la v3 vince la
sua categoria (−30,7%) pur avendo perso l'aggregato; (5) il fold commutativo è
un buco osservabile (ordine operandi in "Unsupported operand types").
**Non misurato e dovuto**: la decomposizione per-opcode degli 11 residui (quale
dei corpi porta i ~8,5 ns?); il census dei collaterali; la banda di str; il
preambolo sull'ASM corrente; prop resta 13,1 flag-on (seconda peggiore) senza
un solo numero nuovo sul suo meccanismo.

## Emendamenti

- **A-GR-99-1**: str/re riscritti come BANDE in ha1-registers.out; vietata
  l'etichetta «rumore» senza `delta/spread/banda`.
- **A-GR-99-2**: eseguire M1 (noop micro + census + ASM) e scrivere P prima di
  ogni riga di codice H-B1; ordine H-B1/H-B2 deciso da P, non dal piano.
- **A-GR-99-3**: census flag-on/off su prop e calls per attribuire i collaterali.
- **A-GR-99-4**: ogni confronto cross-binario porta la riga
  `deriva_inter_build=` esplicita.

## Kill-switch

- **KS-GR-99-1**: H-B1 non scrive codice VM finché P non è nel .out; P < 10% ⇒
  H-B1 cade a tavolino e si passa a H-B2.
- **KS-GR-99-2**: nullo qualsiasi claim di miglioramento < 3× lo spread relativo
  della propria coppia stesso-binario.
- **KS-GR-99-3**: vietato «rumore»/«coerente» su numeri di binari diversi senza
  la deriva inter-build dichiarata.

════════════════════════════════════════════
## FILE: verbali/team-forma-hb1.md

# Nota di team «forma-hb1» — Concilio WP-99

**Relatore**: team forma-hb1. **Fonti**: SOLO verbale-1-hoare.md e verbale-2-matsakis.md.

## 1. Convergenze — la riformulazione di H-B1 che entrambe le sedie accettano

Le due sedie refutano in indipendenza la stessa premessa da due lati (Hoare: l'insieme dei confini «call/ret/throw» è sottostimato di un ordine; Matsakis: la forma letterale è E0499 da manuale) e convergono sulla stessa forma sostitutiva:

> **H-B1 riformulata**: loop interno del run_loop su **split-borrow strutturale di campo** — `let fr = self.frames.last_mut()` (Matsakis) / handler su `(&mut Frame, &mut VmRest)` (Hoare) — con `fr.ip` che resta il **campo vero del frame** mutato attraverso il prestito vivo; **confine = ogni opcode che richiede un metodo `&mut self`** (funnel, gc_note, typed_ref_assign, flush_diags, chiamate): lì si fa break al loop esterno e il borrow checker FORZA la resa del frame. Guardia di profondità dove `frames` può crescere.

Vincoli KS congiunti, tutti compatibili tra loro (nessuna tensione):
- **No `unsafe`, no raw pointer, no cache di `as_mut_ptr()`/`NonNull<Frame>`** (KS-HO-99-1 ≡ KS-MA-99-3).
- **No `mem::take` del frame** attraverso archi di ri-entrata: Hoare per GC root-scan/backtrace ciechi (KS-HO-99-2), Matsakis per unsoundness semantica verso gli osservatori cross-frame — backtrace, `current_frame_args`, `var_dyn_read` (KS-MA-99-3). Stessa proibizione, due giustificazioni complementari.
- **No `ip` locale con write-back «ai confini»** (KS-MA-99-2): ogni sito che emette diag legge `frames[top].ip` via `cur_line`; un ip staccato sposta le righe dei messaggi in silenzio. Coerente con l'osservazione di Hoare che `flush_diags` è esso stesso un arco di ri-entrata (`set_error_handler`).
- **Criterio numerico scritto PRIMA del codice** (A-HO-99-4 ≡ A-MA-99-2): entrambe bollano «sotto il rumore R=3» come predicato quasi vacuo; la predizione di canale (ns attesi dalle 4 indicizzazioni + 2 `len()`) precede l'implementazione.
- Convergenza implicita anche sul **conto dei guadagni**: il beneficio copre solo il fast-path, mentre S-97.1 mostra che il costo sta negli opcode che entrano nel funnel `&mut self`.

## 2. Conflitti reali

Nessun conflitto di sostanza. Due differenze di posizione, da NON appianare:
- **Meccanismo della sonda**: Matsakis prescrive una **sonda per ADDIZIONE** (+2 indicizzazioni/opcode dietro flag; se `arith` non sale oltre il rumore, H-B1 cade senza essere scritta — refutazione capitale n.2). Hoare chiede solo la predizione numerica a priori, senza sonda. La sonda è più forte e la ingloba; ma la paternità e l'obbligo sono di Matsakis.
- **Granularità del confine**: Hoare enumera gli **archi di ri-entrata** come insieme da censire (gc_note→`__destruct`, flush_diags→`set_error_handler`, binary_value_ab→`__toString`/GMP/BcMath/lazy-init, typed_ref_assign, to_bool su oggetti); Matsakis dà il criterio **sintattico** («ogni metodo `&mut self`»). Il criterio di Matsakis è il sovra-insieme verificato dal compilatore; la lista di Hoare è il censimento semantico che dice QUANTI confini sono, cioè se il risparmio sopravvive.

## 3. Priorità proposte per l'ordine S-98.0 (vista di questo team)

1. **Enumerazione degli archi di ri-entrata PRIMA di ogni design** (A-HO-99-5): senza il conteggio della loro frequenza sul workload, il risparmio non è stimabile.
2. **Sonda per addizione + predizione di canale scritta in apertura** (A-MA-99-2 + A-HO-99-4): se la sonda non morde, H-B1 muore a costo zero.
3. Solo dopo, eventuale implementazione nella forma vincolata §1, con **KS-MA-99-1** attivo (`binary_fast` resta `fn(BinOp, &Zval, &Zval) -> Option<Zval>`, pura, senza canale diags).
4. **Fixture dovute** (flag-only, non bloccanti ma prima di ogni promozione): typed-ref by-ref su `*Dst` (A-HO-99-2), drop-order su TypeError in catch (A-MA-99-3), riga su espressione multilinea (A-HO-99-3); nessuna promozione flag-on senza corpus 1418 per NOME flag-ON (KS-HO-99-3).

════════════════════════════════════════════
## FILE: verbali/team-giudice.md

# Team «giudice» — Concilio WP-99 (relatore)
Fonti: verbale-3-klabnik.md, verbale-4-hejlsberg.md. Nessun altro verbale letto.

## 1. Convergenze — lista MINIMA che sblocca «gate PASS» e il collaudo dello strumento

Le due sedie convergono sullo stesso principio, già lezione di S-97.1: **nessuna evidenza senza controllo positivo che il dente ha morso**.

Riparazioni del giudice (manifest), minimo per il claim «gate PASS»:
- **M1 (A-KL-99-1)**: delibera che ripari il manifest — WP_SESSION_97 a `judge=yes` (o carta A-SK-71 emendata), WP_SESSION_94/95 declassate, e per i tre `.out` di S-97.0 (`micro-baseline`, `arith-decomposition`, `ha2-sweep`) o la riga o l'esclusione DICHIARATA di classe. Vincolo KS-KL-99-2: senza questo, la riga «gate cifre PASS» NON si scrive nel report di chiusura S-98.
- **M2 (A-KL-99-4)**: gli `.out` flag-ON devono autodescriversi (stato del flag nell'header, non solo nel nome file).

Collaudo dello strumento census (pin 9,87 ns = grado VERDICT su strumento mai ri-collaudato dopo WP-44):
- **M3 (A-KL-99-2)**: smoke flag-ON in CI di sessione: `PHPR_REG_LOWER=1` su arith_small + controllo positivo `PHPR_DUMP_OPS` che DEVE contenere `BinarySSDst`.
- **M4 (KS-KL-99-1)**: nessun futuro `.out` VERDICT flag-ON senza parità corpus per NOME flag-ON sullo STESSO albero.
- **M5 (A-KL-99-5)**: il test flag-off non si auto-skippa in silenzio se l'ambiente esporta la variabile: fallire rumorosamente.

## 2. Batteria fedele al punto di pipeline di produzione

RC-1 (Hejlsberg): `lowered()` del harness applica il pass POST-cessione WP-65 (`{main}.slot_names` vuoti ⇒ `fold_slot` non matcha), mentre la produzione lo applica in `compile_body` PRE-cessione. I 13 snippet top-level sono quindi potenzialmente vacui sulle gambe `{main}`.

**Nota di contesto vincolante**: il census di PRODUZIONE (CLI, flag-on) HA foldato il top-level di arith_small (19→11 op/iter, dump con BinarySC/CmpJmpSC). La vacuità eventuale riguarda la COPERTURA DEL TEST, non il comportamento di produzione.

Piano (A-HE-99-1 + A-KL-99-3, convergenti):
- **B1 — chiusura del buco**: spostare la batteria sul funnel VERO (pass invocato al punto di produzione, env-flag in-process) E asserire ≥1 forma fusa in `{main}` — il controllo positivo, non l'uno o l'altro. Un semplice assert sul modulo post-cessione non basta: pinnerebbe il punto sbagliato. KS-HE-99-3: finché B1 non è verde, le gambe `{main}` non contano come evidenza.
- **B2 — estensione Klabnik**: target a metà finestra, finestra multi-linea + parità riga del warning, Spaceship const-first non-fold, TypeError operand-order nei due ordini, guardia u16 esercitata, test lowered() ≡ funnel sull'insieme dei corpi (property hook inclusi).

## 3. Conflitti

Nessun conflitto: perimetri disgiunti (Klabnik = giudice/manifest/strumento; Hejlsberg = pipeline/batteria) e principio comune. Unica tensione minore: Klabnik chiede più casi nella batteria attuale, Hejlsberg la vuole spostata di punto di pipeline — l'ordine giusto è B1 PRIMA di B2 (estendere una batteria vacua è lavoro sprecato).

## 4. Priorità S-98.0 (regola di ammissione: entra SOLO ciò che blocca il prossimo passo sull'oggetto; timebox mezza sessione)

1. **M3+M5** (smoke flag-ON + no-skip) — **BLOCCA**: il prossimo passo (H-B1 e ogni misura flag-ON) poggia sul census; senza collaudo il pin 9,87 ns e i futuri numeri non sono ammissibili (M4).
2. **B1** (batteria al punto di produzione + assert `{main}`) — **BLOCCA**: ogni evoluzione del pass (H-B*) sarebbe verificata da un test vacuo sui top-level.
3. **M1+M2** (manifest + header `.out`) — **BLOCCA il CLAIM, non l'oggetto**: senza, il report S-98 non può scrivere «gate PASS» (KS-KL-99-2). Costo minutario, si fa nella stessa mezza sessione.
4. **B2** (estensione batteria) — **NON blocca**: entra solo se avanza tempo nel timebox; altrimenti in coda per NOME.
5. **M4** (KS parità flag-ON stesso albero) — regola permanente da verbalizzare, costo zero.

Timebox complessivo apparato: mezza sessione; il resto di S-98.0 va all'oggetto (programma H-B1 con criterio in ns/op derivato dal 3×, A-KL-99-6).

════════════════════════════════════════════
## FILE: verbali/team-semantica-confine.md

# Team «semantica-confine» — Concilio WP-99 (Stogov · Pedersen · Leijen)

## 1. Convergenze — GUARDIE dovute PRIMA di promozione/estensione del flag

**Mirror dei confronti (Stogov RC-1/RC-2)** — «i compare non lanciano» è FALSO (GMP `__cmp`→ValueError, BcMath `opCoerce` branda Left/Right). Il mirror vive su TRE contingenze da pinnare in fixture, non in commenti:
1. gate `operand_cmp_ok`/`bc_str_wellformed` devia le malformate al braccio *uncomparable* simmetrico;
2. il messaggio GMP è position-free PER CASO;
3. dispatch a NOME ESATTO (`overload_receiver`): le sottoclassi non ridefiniscono `__cmp`.
Invariante da scrivere: **antisimmetria di `__cmp` + errori position-free sul percorso compare**. Guardie: property-test `eval(op,a,b)==eval(mirror(op),b,a)` su fast E funnel (matrice tag × NaN/±0.0/i64::MAX±1/stringhe), snippet GMP/Number flag-on≡flag-off INCLUSI i messaggi (A-ST-99-2); contratto nome-warning WP-65 esercitato da fixture flag-on via unit-cache/{main}, o `debug_assert!` in `reg_load_slot` (A-ST-99-1); le SETTE fixture-trappola AssignOp (RHS-first, typed-ref, `.=`, overflow, ordine warning, `gc_note`, mai commutare) scritte PRIMA di qualunque fold (A-ST-99-3). KS-ST-99-1: allargare `overload_receiver` a instanceof con mirror attivo = unsound silenzioso.

**Flag (Pedersen R1)** — `enabled()` è un OnceLock lazy: `putenv("PHPR_REG_LOWER=1")` da una richiesta può decidere il modo dell'INTERO worker prima della prima compile. Guardie: lettura EAGER a confine nominato (bootstrap Vm) + test cargo che putenv non flippa il modo (A-PE-99-1); `reg=` nel vocabolario uc_log — un A/B on/off è valido SOLO come due processi a env fissato allo spawn (A-PE-99-2, KS-PE-99-2); `.unwrap_or(b"")` in `reg_load_slot` → fail-loud, mai warning col nome vuoto (A-PE-99-3).

**Confini di layout (Leijen)** — dente `N_OPS < 256` accanto al pin 48B (A-LE-99-1, KS-LE-99-2: scatta ⇒ ridisegno prima di ogni misura); «−42% opcode = guadagno footprint» è REFUTATO (dispatch dinamici ≠ byte statici): la promozione flag-on si regge sulla CPU soltanto.

## 2. Debiti di parità pendenti (per urgenza)

1. **Server restapi 3508 + option 413 per NOME sotto `env -i`** (PHPR_REG_LOWER assente dalla lista) + sentinella output-capture. **BLOCCA** qualunque uso/misura del server su 832568a72b925dd1 (KS-PE-99-1 = VOID): H-A2 è nel binario INCONDIZIONATAMENTE, l'emissione flag-off è cambiata, il collaudo è DOVUTO per la regola n.2. Non blocca però S-98.0 se l'oggetto resta CLI (H-B1) — rientra in ordine solo quando si tocca il server; timebox mezza sessione.
2. **Coppia peak `/usr/bin/time -l` agganciata al prossimo collaudo parità WP** (A-LE-99-2, zero run aggiuntivi; m90 fermo, deriva non misurata). NON blocca H-B1 (attesa footprint-neutrale) ma **BLOCCA la promozione a baseline** (KS-LE-99-1). Correzione immediata del pin server in NEXT_SESSION (A-PE-99-4, R3: due fonti di verità).

## 3. Conflitti fra le sedie

Nessun conflitto sostanziale. Unica tensione di sequenza: Pedersen rende il debito server esigibile «al primo uso», Leijen aggancia il peak «al prossimo collaudo WP» — si compongono: un solo collaudo server per NOME che acquisisce ANCHE la coppia peak nello stesso run soddisfa entrambi.

## 4. Priorità S-98.0

1. A-PE-99-4 (pin server in NEXT_SESSION) — un minuto, subito.
2. A-PE-99-1 + A-PE-99-3 (flag eager + dente putenv; fail-loud) — piccoli, chiudono R1.
3. A-ST-99-2 property-test mirror + fixture GMP/Number (con A-ST-99-1 nome-warning).
4. A-LE-99-1 dente N_OPS<256 accanto al pin 48B.
5. A-ST-99-3 (sette trappole) SOLO se S-98.0 apre il fold AssignOp; il collaudo server+peak SOLO se si tocca il server. Apparato in ordine solo se blocca l'oggetto; timebox mezza sessione.

════════════════════════════════════════════
## FILE: verbali/team-tetto-misura.md

# Nota team «tetto-misura» — Concilio WP-99 (relatore)
Fonti: SOLO verbale-5-bak.md + verbale-9-gregg.md. Entrambi: CON EMENDAMENTI.

## 1. Convergenze (indipendenti, stesso numero)

**Tetto numerico condiviso**: dal marginale dello Sweep noop di ha2-sweep
(0,07 s / 50M dispatch) entrambe le sedie derivano **~1,4 ns/op come TETTO del
preambolo di dispatch**. Su `arith` flag-off: Bak lo scrive come −17% sul costo
per opcode (8,24 ns); Gregg come 19×1,4 = 26,6 ns su 156,6 ns/iter ⇒ tetto ~17%,
banda ~8–27% (il Δ 0,07 s è appena 2× lo spread). Corollario comune: **il
fattore ~8 residuo vive nei CORPI degli handler** (catene self→frames→slots,
push su Vec<Zval>, discriminazione di tipo/lavoro Zval), non nel preambolo.

**Conseguenze sull'ordine S-98.0**:
- **H-B1 è DECLASSATA**: non è «l'asse che chiude il fattore 8» (refutazione
  capitale n.2 di Bak; non-sequitur di Gregg). Se eseguita, lo è solo come
  sotto-passo in scope «preambolo», con criterio DERIVATO dal tetto (A-BA-99-1)
  e SOLO dopo la misura M1 con predizione P scritta (KS-GR-99-1). Lo scope
  «frame-esteso dentro i corpi» va dichiarato a parte (costo `&mut self`, Bak).
- **H-B2 è PROMOSSA** all'asse principale: entrambi la indicano come sede degli
  ~8,5 ns residui (A-BA-99-4; Gregg: decomposizione per-opcode degli 11 residui
  = «non misurato e dovuto»).
- Concordano anche sul metodo: il criterio −40% di H-A1 era irraggiungibile con
  dati già posseduti (Bak: uniforme = −42,1%, non-uniformità già misurata) —
  mai più criteri-cifra-tonda; si deriva dal controfattuale (KS-BA-99-1).

## 2. Ricetta di misura unificata (M1, zero codice VM) — che entrambi firmerebbero

1. **Micro noop**: `for($i=0;$i<200000000;$i++){}` con ricetta run-micro.sh
   (coppia stessa-sera, R=3, stesso binario) → **D = ns/dispatch** dei soli op
   economici, su ~4 miliardi di dispatch (statistica stretta, non il Δ 0,07 s).
2. **Census**: ops/iter del noop (attesi ~4-5: IncDecSlot, CmpJmpSC, Jump,
   Sweep[, Pop]) + census flag-on/off su prop e calls per attribuire i
   collaterali (A-GR-99-3); conteggio statico dei re-borrow `self.frames[top]`
   eliminati per opcode del residuo arith (A-BA-99-2 = la predizione-misurata).
3. **ASM del binario CORRENTE**: bounds check/len realmente emessi nel run_loop
   (prof95-media è di un'altra build; LLVM può già eliderne).
4. **Predizione P scritta nel .out PRIMA di ogni riga di codice**:
   P = 19·D/156,6 ns (quota massima di arith attribuibile al preambolo).
   Caveat: D è marginale in pipeline out-of-order ⇒ tetto soffice verso il basso.

**Criterio di caduta numerico congiunto**: H-B1 cade A TAVOLINO se P < 10%
(KS-GR-99-1); se scritta, cade se risparmio < 0,7 ns/op — metà del tetto —
ossia arith flag-off netto > 7,2 s (8,24 → ≤ 7,55 ns/op) sulla coppia R=3
stessa-sera (A-BA-99-1), e comunque se il calo < max(P/2, 3× spread relativo
≈ 1,5%). **Nulli i claim < 3× spread della propria coppia stesso-binario**
(KS-GR-99-2). Ogni confronto cross-binario porta `deriva_inter_build=`
esplicita (A-GR-99-4; deriva nota 0,6%); str/re riscritti come BANDE, vietata
l'etichetta «rumore» (A-GR-99-1: str ~[−11%, +4%]).

## 3. Conflitti

Nessuno sostanziale. Sfumatura: Bak dà il criterio operativo assoluto
(0,7 ns/op / 7,2 s) prima ancora di M1; Gregg subordina TUTTO a P misurata.
Composizione: si esegue M1, e il criterio finale è il PIÙ severo tra i due
(P/2 vs 0,7 ns/op), con la soglia a tavolino P < 10% che precede entrambi.
Nota: Bak ritira la tariffa «corpi caldi» come obiezione ai micro (WP-44 vale
solo per il giudice aggregato) — nessuna sedia la oppone.

## 4. Priorità S-98.0

1. **M1 completa** (noop + census + ASM + P nel .out) — primo atto, zero codice.
2. **Decisione H-B1 dal numero**: P < 10% ⇒ a tavolino ⇒ dritti a H-B2;
   altrimenti sotto-passo preambolo col criterio composto di §2.
3. **H-B2 come asse**: decomposizione per-opcode degli 11 residui di arith
   (quale corpo porta gli ~8,5 ns), poi intervento sui corpi.
4. Igiene permanente: bande su str/re, deriva inter-build dichiarata, prop
   (13,1 flag-on, seconda peggiore) pretende almeno un numero nuovo.
