# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-94 (4 sessioni
fa)** — ⚠️ e ora il collaudo di parità WordPress è **DOVUTO** (S-98.0 ha
cambiato l'emissione flag-off: regola n.2) · ultima campagna sull'OGGETTO =
**S-98.0 (questa: M1 tre-strumenti + coppia add/arith R=5 stessa-sera,
due corpus per NOME)**.

**Ultima sessione (S-98.0, 2026-08-05)**: l'ordine del Concilio WP-99
eseguito per intero. **M1 → H-B1 CADUTA A TAVOLINO senza una riga di
codice** (`wp98-harness/m1-preamble.out`: P = 0%, banda [0–6,7%] < 10% —
la sonda A/B mostra il reload del preambolo GRATIS sul core OoO, la forma
split-borrow perfino −0,53 ns/op sotto pressione). **H-B2 CONFERMATA E
SPEDITA** (`hb2-addspec.out`: `Op::BinaryAdd` all'emissione, solo
flag-off; giudice `add` −16,2%, **D = 6,07 ns/occorrenza = 8,7× la soglia
pre-registrata**, guardia contata; flag-on bit-identico). Corpus 1418 per
NOME identico flag-OFF e flag-ON sullo stesso albero (M4 ✓); B1+M3+M5
saldati (test al funnel vero con controllo positivo sul `{main}`).
⭐⭐ il fattore ~8 del costo per opcode vive nel PLUMBING dei corpi (call +
marshalling Zval + pop/push ≈ 6 ns), non nel preambolo → l'asse resta la
specializzazione, che COMPONE col −30,7%.
Dettaglio: `sessions/WP_SESSION_98.md` + i due `.out` di `wp98-harness/`.

**Cambio di rotta deciso dall'utente (2026-08-04, a valle di S-96.0)**: il
progetto stava andando «a tentoni», con l'agenda di ogni sessione fatta dal
residuo della precedente. Ora c'è un OBIETTIVO, un GIUDICE e una sequenza di
IPOTESI con i loro criteri di caduta. **Si va dritti al PHP: WordPress dopo.**

## Il fatto che ha cambiato tutto

Il progetto ha misurato per sei sessioni UN solo numero — il rapporto sulla
suite WordPress — e ha scelto le leve guardando il profilo di phpr **da solo**.
Un profilo a un lato solo dice dove phpr spende tempo, non dove ne spende PIÙ
dell'oracle: seleziona proprio le cose che entrambi i motori fanno. E
l'aggregato WordPress è **diluito**, perché la maggior parte di quel tempo è
I/O, database e builtin, dove phpr regge bene.

Misurando lo STESSO sorgente PHP su entrambi i motori, una categoria alla volta
(`wp97-harness/micro-baseline.out`, R=3, mediana e spread, al netto dei due
pavimenti di avvio — che sono DIVERSI):

| categoria | rapporto phpr/oracle |
|---|---|
| aritmetica pura | **campo `rapporto_arith`** |
| accesso a proprietà | **campo `rapporto_prop`** |
| chiamate di funzione | campo `rapporto_calls` |
| stringhe | campo `rapporto_str` |
| array associativi | campo `rapporto_arr` |
| regex (l'estensione) | campo `rapporto_re` |

**Il nucleo interprete è oltre un ordine di grandezza più lento**, non il ~1,9×
che dava la suite. Tre conseguenze, tutte operative:

1. **La coda di leve inseguita finora è fuori scala.** Vale nell'insieme pochi
   punti percentuali di un numero già diluito, contro un divario che è un
   fattore. Non si chiude un ordine di grandezza con un'analisi di liveness.
2. **Le estensioni riscritte a mano sono la parte SANA.** La regex — che è
   `fancy_regex` contro PCRE2 — è la categoria MIGLIORE. L'ipotesi «il problema
   sono le estensioni» è stata formulata e REFUTATA dalla misura in dieci
   minuti. Il problema è l'interprete generico.
3. **Misurare su WordPress era l'errore metodologico.** Su un aggregato dove il
   segnale è dell'1% servono gate da settanta minuti e concili a nove sedie per
   litigare se lo 0,8% è vero. Su `arith` il segnale è di ordine 10× e un
   miglioramento reale si vede in otto secondi.

⚠️ **Il pavimento di avvio non è opzionale**: l'oracle parte più lentamente di
phpr, e non sottrarre i due pavimenti sbaglia i rapporti di un fattore 3
(errore commesso e corretto alla prima misura). Lo strumento lo fa da sé.

## OBIETTIVO

**X = nucleo interprete ≤ 3× l'oracle sulle categorie pure.** Giudice: le sei
micro-categorie di `wp97-harness/micro/`, non WordPress. WordPress resta il
collaudo finale di PARITÀ e di non-regressione, non il cronometro.

## LE IPOTESI, IN SEQUENZA

Ogni ipotesi si misura sulla SUA micro-categoria PRIMA di toccare WordPress, e
si abbandona sul criterio scritto prima. Il concilio giudica le ipotesi e i
criteri, non il residuo della sessione precedente.

### ~~H1 — Il costo è la lettura degli operandi~~ → **REFUTATA senza scrivere codice**

Era A-ZV1, il «piano B» rimandato per tre sessioni. `Op::Binary` prende gli
operandi dalla **PILA** («pop rhs then lhs»): quando `binary_value_ab` gira il
clone è GIÀ avvenuto in `LoadSlot`, quindi un fast-path per riferimento **lì**
non risparmia nulla. E in un ciclo aritmetico i valori sono interi, quindi il
clone di `read_slot` non è nemmeno un refcount. Era mal indirizzata dall'inizio.
Dettaglio in `wp97-harness/arith-decomposition.out`.

### Il divario si scompone in DUE fattori, entrambi misurati

`wp97-harness/arith-decomposition.out` (VERDICT, conteggi esatti):

- **conteggio di opcode**: `opcodi_per_iterazione_oracle` contro
  `opcodi_per_iterazione_phpr` → fattore `rapporto_conteggio_opcodi`
- **costo per opcode**: `ns_per_opcode_oracle` contro `ns_per_opcode_phpr` →
  fattore `rapporto_costo_per_opcode`
- il prodotto (`prodotto_dei_due_fattori`) riproduce il `rapporto_arith`
  misurato in modo indipendente: **la decomposizione torna**.

Entrambi i fattori sono grandi: chiuderne uno solo lascia l'altro intatto.

### ~~H-A1 — gli operandi transitano dalla PILA~~ → **ESEGUITA E CADUTA sul suo criterio** (S-97.1, `wp97-harness/ha1-registers.out`)

Riarmata la forma v3 «raw registers» di WP-44 dalla storia git (f4c80cf):
7 shape monomorfe u16, pass a finestre con remap totale, dietro
`PHPR_REG_LOWER`. **Braccio 1 passato**: opcode/iterazione 19 → **11** (< 12).
**Braccio 2 no**: `arith` 7,83 → 5,43 s = **−30,7%**, sotto il −40% richiesto
→ il criterio scatta e la sessione l'ha abbandonata senza negoziare (regola
n.3): il fold ulteriore della coda AssignOp (11→9 possibile) è NOMINATO nel
`.out` ma non scritto.

È comunque il singolo calo di `arith` più grande mai misurato (rapporto
18,2→12,6; collaterali: prop −12,3%, calls −15,7%, arr −11,5%), e il flag-on
**vince** il suo micro — WP-44 (aggregato) e S-97.1 (micro) coesistono:
giudici diversi, claim diversi. Il codice resta in albero dietro il flag
(flag-off zero-delta; no-revert). ⚠️ Soundness, divergenza dalla v3: il fold
commutativo const-lhs è stato RIMOSSO (`3+$x`→`$x+3` inverte l'ordine dei
nomi in "Unsupported operand types"; il corpus WP-44 non lo prese).

**⭐⭐ Il dato che decide la prossima mossa**: tolto il 42% degli opcode, il
costo MEDIO per opcode è SALITO da 8,24 a 9,87 ns. Il conteggio è quasi
chiuso (11 contro 7 dell'oracle); il divario vive nel **COSTO per opcode**
(~8× sul residuo). L'asse è H-B1/H-B2.

### ⚖️ Concilio WP-99 (2026-08-05, su S-97.0+S-97.1 e programma S-98.0) — ESEGUITO in S-98.0 (punti 1-3 + debiti B1/M3/M5; resta la parità server)

Verbali integrali + note di team + sintesi in `wp99-harness/` (9/9 CON
EMENDAMENTI, sette refutazioni capitali). Le TRE che riscrivono l'ordine:

1. **La forma letterale di H-B1 è FALSA** (Hoare+Matsakis): gli archi di
   ri-entrata (gc_note, flush_diags, __toString, dtor) attraversano quasi
   ogni handler e ogni diag legge `frames[top].ip`. Forma safe possibile:
   loop interno su split-borrow, confine = ogni opcode con metodi
   `&mut self`. KS: no unsafe/raw ptr, no mem::take, no ip locale.
2. **Il tetto di H-B1 è ~1,4 ns/op (−17%, banda 8–27%)** (Bak+Gregg dal
   dispatch noop già pinnato in ha2-sweep): il fattore ~8 vive nei CORPI ⇒
   **H-B1 declassata a sotto-passo, H-B2 promossa ad asse**. Prima di ogni
   codice: misura M1 (noop da duecento milioni di iterazioni + census + ASM)
   e predizione P scritta nel
   .out — P < 10% ⇒ H-B1 cade a tavolino.
3. **Parità server DOVUTA** (Pedersen): il pin php-server 832568a72b925dd1
   contiene H-A2 incondizionata e NON è verificato — restapi+option per
   NOME sotto env -i prima di ogni uso del server.

**ORDINE S-98.0 (eseguito)**: M1 ✓ (P=0% ⇒ H-B1 a tavolino) → H-B2 ✓
(spedita, D=6,07 ns/occ) · debiti: smoke flag-ON ✓ + assert {main} ✓ +
M5 ✓ · parità server NON eseguita (oggetto rimasto CLI) → S-99.

### ⚖️ Concilio WP-100 (2026-08-05, su S-98.0 e programma S-99.0) — VINCOLANTE

Verbali integrali + note di team + sintesi in `wp100-harness/` (9/9 CON
EMENDAMENTI; cinque refutazioni capitali, `verbali/SYNTHESIS.md`). Le TRE
che riscrivono l'ordine: (1) **D=6,07 NON è ereditabile come criterio del
rollout** (SEI sedie convergenti: è plumbing del percorso PILA; le forme
registro inlineano già `binary_fast` su prestiti — il criterio nasce da
controfattuale+misura DEL percorso registro, ogni spedizione col criterio
ereditato è VOID); (2) **la bozza era VOID al punto 1** (il collaudo WP
passa da php-server NON collaudato ⇒ parità server = prima gamba, stessa
sessione; sigillo eager + anti-putenv promossi a GATE DI PROMOZIONE);
(3) **evidenza sovradichiarata** (il funnel non pinna stdout/exit; il gate
per NOME è un bit/fail — la promozione esige il diff riga-per-riga; peak
solo con `/usr/bin/time -l` + env mimalloc WP-94; N_OPS 186/256 senza
gate; terzo rialzo di budget senza delibera = gate non-mordente).

**ORDINE S-99.0 (fissato dal Concilio WP-100)**:

1. **Parità server** (prima gamba, precondizione): restapi+option per
   NOME sotto `env -i` sul pin **365f4d4069513de3** + sentinella
   output-capture.
2. **Collaudo WordPress full+media stessa-sera** (parità per NOME,
   launcher con backup/wipe/restore uploads) **+ coppia peak** con
   `/usr/bin/time -l` + env mimalloc pinnate da WP-94. Nessun claim CPU
   nuovo e nessun rollout prima della chiusura per NOME (KS-GR-100-1).
3. **Ri-baseline delle sei categorie** su ENTRAMBI i motori stessa
   finestra (sana la gamba oracle stantia; rianima i criteri H-C/H-D).
4. **Pre-misura del rollout, SOLO misura** (niente codice del rollout in
   S-99): controfattuale statico del percorso registro (A-ST-100-1) +
   build intermedia che decompone D in call/marshalling vs pop/push
   (A-BA-100-1) + baseline flag-on → il criterio del rollout nasce QUI,
   mai da 6,07. Se il timebox regge: sigillo eager + test anti-putenv.

Precondizioni per NOME e BACKLOG: in `wp100-harness/verbali/SYNTHESIS.md`
§Ordine (diff riga-per-riga, N_OPS≤255, matrice Add, tripwire
zero-BinaryAdd, visit_addrs esaustivo, trappole A-ST-99-3, registro pin
`collaudato:`, sanatoria dump/lowered, ecc.).

**⭐ DECISIONE UTENTE (2026-08-05, post-chiusura S-97.1): il −30,7% SI TIENE
e ci si costruisce sopra.** La caduta di H-A1 sul criterio decide dove
investire la prossima sessione, NON butta il guadagno misurato: la
**PROMOZIONE del flag-on a default è un OBIETTIVO NOMINATO** (non backlog
generico), coi gate che il concilio ha già elencato — corpus flag-ON per
NOME sullo stesso albero, parità server, contingenze del mirror pinnate in
fixture, smoke con controllo positivo, coppia peak stessa-sera. Le leve di
costo (H-B1/H-B2, fold coda AssignOp) si COMPONGONO col −30,7% — meno
opcode × opcode più economici — e si indagano con la promozione come
SBOCCO, non in alternativa ad essa.

### ~~H-B1 — ogni opcode costa troppo, il preambolo~~ → **CADUTA A TAVOLINO in S-98.0, zero codice** (`wp98-harness/m1-preamble.out`)

M1 eseguita come da ordine (noop da duecento milioni di iterazioni +
census + ASM + sonda A/B):
P = 0% su tutta la banda (tetto anti-hiding 6,7%) < 10% ⇒ KS-GR-99-1
scatta. Il dato che chiude: su questo core OoO le istruzioni del preambolo
(reload L1 a indirizzo costante, fuori dal cammino critico) sono GRATIS, e
la forma split-borrow sotto pressione di registri è risultata PIÙ LENTA di
0,53 ns/op. ⭐⭐ mai più criteri derivati dal CONTEGGIO di istruzioni senza
chiedersi se stanno sul cammino critico.

### ~~H-A2 — `Sweep`~~ → **CONFERMATA E SPEDITA** (`wp97-harness/ha2-sweep.out`)

Il doppione era il blocco fra graffe: il lowering lo rende un `StmtKind::Block`
annidato, quindi il `block_of` interno emette il Sweep dell'ultimo statement e
quello esterno ne emette subito un altro — con ZERO opcode in mezzo. Eliso
all'emissione, **solo per `Block`**: la regola generale sarebbe SCORRETTA
(in `if (c) {...}` il ramo falso atterra sulla posizione del Sweep dell'`if`
senza aver eseguito quello del corpo).

Resa: `opcodi_per_iterazione_prima` → `opcodi_per_iterazione_dopo`, tempo
`calo_tempo_arith_pct`. Parità provata per NOME (corpus identico salvo tre test
che usano `random_bytes`; tutte e sei le sentinelle dei distruttori verdi).

**⭐ La scoperta che conta è di lato**: tolto il 5% degli opcode il tempo scende
meno dell'1%, quindi un `Sweep` noop costa circa **un quinto** dell'opcode
medio. **Il costo per opcode NON è uniforme**: è concentrato negli opcode che
fanno traffico di pila e lavoro su `Zval`. Il fattore «costo per opcode» è una
MEDIA su opcode molto diversi — e i costosi sono esattamente il bersaglio di
H-A1, che quindi dovrebbe rendere più del suo peso nominale.

Quattro indicizzazioni con bounds check di `self.frames[top]` più due `len()`
a OGNI opcode, e `Frame` è 176 byte. Documentato in `prof95-media.out`
§PREAMBOLO, mai affrontato. (`Op` è 48 byte contro i 32 di `zend_op`: lo
stream di istruzioni è una volta e mezza più largo.) Dopo S-97.1 il bersaglio
è pinnato: ~9,9 ns per opcode sul residuo di `arith` contro 1,23 dell'oracle.

- **faccio**: frame corrente tenuto in un registro, ricaricato SOLO ai confini
  (call/ret/throw); guardia di profondità spostata dove `frames` cresce.
- **criterio di caduta (fissato dal Concilio WP-99, team tetto-misura)**:
  PRIMA di ogni codice la misura M1 produce la predizione P nel `.out`;
  P < 10% ⇒ cade a tavolino. Se si esegue: cade se il risparmio è sotto
  max(P/2, sette decimi di ns per opcode), cioè se `arith` flag-off resta
  sopra 7,2 s.
- **nota**: si misura flag-OFF (la strada di parità); il flag-on resta
  strumento di misura, non baseline. Forma vincolata: split-borrow del
  team forma-hb1 (KS: no unsafe/raw ptr, no mem::take, no ip locale).

### ~~H-B2 — manca la specializzazione per tipo~~ → **CONFERMATA E SPEDITA su UN opcode** (S-98.0, `wp98-harness/hb2-addspec.out`)

`Op::BinaryAdd` emesso al posto di `Binary(Add)` SOLO flag-off (il modo è
nella chiave della unit-cache; flag-on bit-identico, census 1.100.019
invariato). Handler: guardia tag (Long,Long) → checked_add in-place sulla
cima della pila; MISS → fallback integrale a `binary_value_ab`
(KS-ST-99-3 ✓, guardia contata). **D = 6,07 ns/occorrenza** (add −16,2%;
arith in banda [−3,−5]%). Corpus 1418 per NOME flag-off E flag-on.

**Il meccanismo è provato; la CODA di H-B2 è il rollout** (per NOME, ogni
occorrenza col suo criterio pre-registrato): Sub/Mul/compare int-int sul
percorso stack; le stesse specializzazioni DENTRO le forme registro
flag-on (BinarySS/SC/Dst hanno lo stesso plumbing generico da togliere —
è lì che compone col −30,7%); la coda AssignOp fusa (11→9 op/iter) SOLO
dopo le sette fixture-trappola di Stogov (A-ST-99-3).

### H-C — Il costo dell'accesso a proprietà è la risoluzione ripetuta

`rapporto_prop` è la seconda categoria peggiore, con `PropsLayout::slot_of` e
`resolve_method_runtime` nel profilo, nonostante le PropIc esistenti.

- **si attiva se**: dopo H-A1 e H-B1 `prop` resta sopra 5×.

### H-D — Il costo è l'ABI di chiamata

`enter_callee` + `recycle_frame` + `bind_params` + drop del `Frame`.

- **si attiva se**: dopo H-A1 e H-B1 `calls` resta sopra 5×.

## Regole di metodo (sostituiscono la prassi precedente)

1. **Il giudice è la micro-categoria.** Una leva si misura dove il suo
   meccanismo domina, non dove è diluito.
2. **WordPress è un collaudo di PARITÀ**, e si esegue quando cambia
   l'emissione — non per cronometrare.
3. **Ogni ipotesi porta il suo criterio di caduta scritto PRIMA**, e la
   sessione che la esegue la abbandona se il criterio scatta, senza negoziare.
4. **L'apparato non entra nell'ordine** se non blocca l'ipotesi in corso. Il
   timebox di mezza sessione resta, e vale anche per riparare l'apparato dopo
   un concilio.

## Stato gate

- **phpr (parità release)**: **4e268c3f61e6573d** — NUOVO da S-98.0
  (contiene H-B2 `BinaryAdd` incondizionato flag-off + le 7 varianti
  registro dormienti dietro `PHPR_REG_LOWER`; il bin viene rilinkato dal
  test d'integrazione del funnel: l'hash cambia a ogni `cargo test`, il
  SORGENTE è HEAD). Stash additivo `phpr-old-target/release/phpr-s98-hb2`;
  precedenti 0dd98ebbb7eb2d96 (`phpr-s97-ha1`), 2f6c1a696b560755
  (`phpr-s97-ha2`). Corpus Zend per NOME **1418 invariato** flag-OFF e
  flag-ON sullo stesso albero (evidenze in `wp98-harness/evidence/`).
- **php-server**: **365f4d4069513de3** — ricostruito con H-A2+H-B2,
  parità MAI verificata (KS-PE-99-1: VOID ogni uso/misura del server
  prima di restapi+option per NOME sotto env -i). Pin precedenti
  832568a72b925dd1 (`php-server-s97`, mai collaudato), f8f4295a1dcdb627
  (`php-server-wp94`); ⚠️ pin storico d45b578 NON riproducibile — voce
  aperta.
- **Gate cifre**: `--all` PASS a HEAD · **SELFTEST PASS rc=0** con i denti
  T0–T31 (il canale env di git è chiuso; il perimetro guarda i BYTE e non la
  forma che git sceglie di stampare).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM 3E/13F ·
  hk 1665).
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA da rust-analyzer e il volume
  locale sta al limite: rimuoverla è parte del pre-flight. La taglia si misura
  con `du -sh`, non si cita a memoria.

## Che cosa è SOSPESO, e perché (non abbandonato)

- **A-ZV2** (liveness + `TakeSlot`): sospesa. Non perché sbagliata, ma perché
  il suo guadagno è dell'ordine dell'1% dove serve un fattore. L'analisi resta
  su disco dietro la feature `zval-census`; se H2 la rende utile, riparte —
  ma prima vanno chiusi i buchi di soundness che il Concilio WP-98 ha nominato
  e che nessuna fixture prova ancora (canale cross-frame `current_frame_args`,
  fallback di namespace su `extract`, arco di ri-lancio `EndFinally`), più il
  debito dell'identità STRUTTURALE dell'analisi (oggi cache con chiave per
  PUNTATORE, accettabile in sola misura, VIETATA in emissione).
- **L'ordine del Concilio WP-98** (`wp98-harness/`): i suoi verbali restano
  validi come critica del METODO — le otto refutazioni capitali sono tutte in
  piedi — ma il suo ORDINE era mirato al bersaglio sbagliato, perché il
  concilio giudicava la sessione precedente e non il divario. Sopravvivono per
  NOME: **P-AMEND-ORFANO** (un artefatto registra `head=`, poi un `--amend`
  sostituisce l'oggetto e al primo `gc` la provenienza è irrisolvibile) e la
  CLASSE aperta dei ~20 siti `git status --porcelain` che quotano e collassano.
- **Roadmap footprint**: ferma. Il footprint non è misurato da m90 e nessuna
  leva è nominata; si riapre quando il nucleo CPU è sotto controllo.

## NON riproporre

Tutti i NON-riproporre WP-83..97 restano. Nuovi da S-98.0:

- **H-B1 in ogni forma** («frame in registro», split-borrow, riduzione del
  preambolo): refutata per MISURA su questo hardware — il reload
  L1-resident fuori dal cammino critico è gratis e la ristrutturazione può
  perdere. Si riapre SOLO con hardware diverso o con un dato nuovo di
  cammino critico.
- **criteri derivati dal conteggio di istruzioni** senza analisi del
  cammino critico (il tetto 1,4 ns «da 11/28 istruzioni» sbagliava
  direzione).
- **misurare con una build o altro carico concorrente** (serie bimodale
  4,8/6,5 scartata in sessione).
- **uccidere un processo su UNA sola evidenza** (`ps | grep` mentiva,
  `pgrep -fl` vedeva il runner: un corpus perso al 90%).

Ereditati e ribaditi:

- **«il rapporto sulla suite WordPress misura la velocità del motore»** — è un
  aggregato diluito: nasconde un nucleo interprete oltre un ordine di
  grandezza più lento.
- **«il problema sono le estensioni riscritte a mano»** — REFUTATO per misura:
  la regex è la categoria migliore, e `ext/gd` è già Rust nativo (nessun FFI a
  libgd nel workspace).
- **«scegliere una leva dal profilo di phpr»** — un profilo a un lato solo non
  può trovare un divario: seleziona ciò che entrambi i motori fanno.
- **confrontare due motori senza sottrarre i pavimenti di avvio** (sbaglia di
  un fattore 3).
- e i divieti di merito già in vigore: «la strada lunga non aggiunge opcode al
  percorso caldo» (falso); «il piano B è la superistruzione `LoadSlot+Binary`»
  (riferimento pendente: il piano B su disco è A-ZV1); il perimetro F2 intero
  come base di un F3 fedele; sanificare un ambiente togliendo le variabili che
  si conoscono; «una fixture che non morde prova che il difetto non c'è»; il
  tetto sui corpi caldi usato come TARIFFA; un predicato soddisfatto dal
  proprio testo.

---
**Riscritto**: 2026-08-04 (spina dorsale, decisione utente); rotazione
S-98.0 il 2026-08-05. Apertura/chiusura sessioni = skill `apri-sessione` /
`chiudi-sessione`. Harness di sessione: `wp98-harness/`.
