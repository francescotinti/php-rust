# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

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

### ▶️ H-A1 — LA PROSSIMA, ed è il pezzo grosso: gli operandi transitano dalla PILA

Dodici opcode su venti, per iterazione, sono idraulica di pila e GC:
caricamenti di operando, `PushConst`, `Pop`, `Dup`, `Swap`. L'oracle non ne ha
NESSUNO, perché i suoi opcode sono indirizzati a registro
(`T3 = MUL CV1($i) int(3)`).
**L'infrastruttura in phpr ESISTE GIÀ ed è codice morto**: `enum Operand
{ Stack, Slot(u16), Temp(u16), Const(u16) }` in `bytecode.rs`, commentato
«stage 2 wires it into Binary/CmpJmp», con UN SOLO riferimento nel repo — la
propria definizione.

- **faccio**: cablo `Operand` su `Binary` (e `CmpJmp`) — lo «stage 2» mai fatto
  — così gli operandi si leggono per riferimento da slot/temp/const senza
  passare dalla pila. Misuro `arith`.
- **falsificata se**: il conteggio di opcode per iterazione non scende sotto
  12, **oppure** scende ma `arith` non migliora di almeno il 40%.
- **attenzione**: è la strada che l'arco WP-39..44 chiuse come fallita. Ma quel
  verdetto fu dato sull'aggregato WordPress, dove un fattore quasi 3 sul
  conteggio di opcode è invisibile. Qui il giudice vede.
- **tetto**: attacca ENTRAMBI i fattori insieme (meno opcode E meno traffico di
  pila per opcode), quindi è la candidata a resa più alta.

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

### H-B1 — Ogni opcode costa troppo: il preambolo

Quattro indicizzazioni con bounds check di `self.frames[top]` più due `len()`
a OGNI opcode, e `Frame` è 176 byte. Documentato in `prof95-media.out`
§PREAMBOLO, mai affrontato. (`Op` è 48 byte contro i 32 di `zend_op`: lo
stream di istruzioni è una volta e mezza più largo.)

- **faccio**: frame corrente tenuto in un registro, ricaricato SOLO ai confini
  (call/ret/throw); guardia di profondità spostata dove `frames` cresce.
- **si attiva**: in parallelo a H-A1 — è indipendente e attacca l'altro fattore.

### H-B2 — Ogni opcode costa troppo: manca la specializzazione per tipo

PHP 8 genera migliaia di handler specializzati per combinazione
opcode × tipo × classe di operando; phpr ha UN braccio generico che discrimina
il tipo a runtime a ogni esecuzione.

- **faccio**: specializzo **un solo** opcode (`Binary` Add intero-intero)
  deciso a COMPILAZIONE, e misuro `arith`.
- **si attiva se**: dopo H-A1 e H-B1 il costo per opcode resta sopra 3 ns.

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

## Stato gate (invariato da S-96.0)

- **phpr (parità release)**: **2f6c1a696b560755** — NUOVO, ricostruito dopo
  H-A2 (il precedente d5ce86e3342f3926 è stato la baseline fino a S-96.0).
  Stash additivo in `phpr-old-target/release/phpr-s97-ha2`. Corpus Zend per
  NOME **1418 invariato**, riverificato sul binario ricostruito.
- **php-server**: f8f4295a1dcdb627 (⚠️ pin storico d45b578 NON riproducibile —
  voce aperta).
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

Tutti i NON-riproporre WP-83..96 restano. In più:

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
**Riscritto**: 2026-08-04, su decisione dell'utente. Apertura/chiusura sessioni
= skill `apri-sessione` / `chiudi-sessione`.
