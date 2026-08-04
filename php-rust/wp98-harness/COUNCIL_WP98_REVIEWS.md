# COUNCIL_WP98_REVIEWS.md — Concilio a 9 sedie sul report S-96.0 e sul programma §WP-97

Protocollo a DUE FASI (regola utente 2026-08-02): fase 1 = nove bozze INDIPENDENTI
(nessuna sedia ha visto le altre); fase 2 = tre team tematici composti sui punti di
questa sessione, ogni relatore legge SOLO i verbali del proprio team. I verbali
individuali sono la fonte VINCOLANTE; le note di team alimentano la sintesi.

Indice: sintesi · note di team (oggetto, analisi, catena) · nove verbali integrali.

---

# SINTESI DI CONVERGENZA — Concilio WP-98 (su report S-96.0 + programma §WP-97)

Protocollo a due fasi: 9 bozze INDIPENDENTI (`verbale-*.md`), poi 3 team
tematici composti sui punti di questa sessione (`team-oggetto`, `team-analisi`,
`team-catena`). I verbali individuali restano la fonte VINCOLANTE.

## §FONDAMENTALI (in testa per regola utente 2026-08-03)

**(a) Avanzamento dell'OGGETTO in S-96.0 — Gregg, mandato inverso: NO.**
Due soli fatti nuovi sul MOTORE — che il difetto di soundness non ricorre nel
corpus WordPress (delta F1 esattamente zero, con la fixture che prova che il
difetto è reale) e la divergenza §3.10 sui `TypeError` dei builtin, **trovata
per caso** — contro sei sull'APPARATO che lo misura, e **zero misure di tempo**.
Nessuna sedia difende il contrario. Il fatto più prezioso della sessione (§3.10)
è quello che nessuno stava cercando.

**(b) Contatore sessioni-senza-misura**: ultima full/media cronometrata = WP-94,
**due sessioni fa**; ultima campagna footprint = m90, sei sessioni fa. La rotta
CPU-VM ha consumato tre sessioni (WP-95, WP-96 e la mattina di WP-95) senza mai
far girare un orologio, e si è conclusa decidendo di non costruire la leva.

**(c) Rischio d'oggetto più trascurato**: **una decisione di rotta è stata presa
su un canale SCREEN**. Il moltiplicatore §P1 (il valore del canale) viene da un
profilo R=1 senza spread; tutte le bande di due sessioni ne dipendono. Gregg:
*«SCREEN × VERDICT = SCREEN, e SCREEN non chiude un passo dell'ordine»*. Bak
aggiunge il colpo che rende il conto insalvabile: il costo di un corpo caldo
**non è una costante** — in WP-44 passare da 2 a 9 corpi costò MENO che passare
da 2 a 4 — quindi il tetto è stato usato come una **tariffa**, che non è.

**Regola di ammissione**: l'ordine WP-97 proposto qui sotto è composto di leve
sull'OGGETTO. L'unica voce d'apparato ammessa (P-AMEND-ORFANO) entra perché
rende **irrisolvibile** la provenienza di ogni misura futura, non solo passata.

## Verdetti (9 sedie, NESSUNA benedizione)

Hoare CON EMENDAMENTI · Matsakis esito confermato / motivazione REFUTATA ·
Klabnik **FAIL** (forgia atterrata) · Hejlsberg §WP-97 punto 1 NON istruito ·
Bak PROCEDI CON EMENDAMENTI, chiusura declassata a **sospensione** · Pedersen
**RESPINTO IN PARTE** · Leijen non refutato nel merito, due refutazioni sulle
premesse · Stogov il passo 2 regge ma il perimetro è un TETTO, non un perimetro
fedele · Gregg **APPROVATO CON DECLASSAMENTO**, oggetto NON avanzato.

## Refutazioni capitali

1. **Klabnik — FORGIA ATTERRATA sull'apparato spedito poche ore prima.** Un
   doc-cifra con un accento nel NOME è uscito dal perimetro mentre il gemello
   ASCII veniva nominato: git QUOTA i path non-ASCII e la virgoletta iniziale
   fa fallire l'ancoraggio `^php-rust/`. Il perimetro si fidava della FORMA che
   git sceglie di STAMPARE. **[CHIUSA IN SESSIONE: `-z` + `core.quotePath=false`
   su `ls-files`, la stessa correzione su `check-ignore -v` (che quotava a sua
   volta e faceva sembrare non-ignorati i sidecar accentati), e su `ls-tree`
   — il lato COMMITTATO aveva lo stesso punto cieco, trovato da team-catena.
   Dente permanente T31 col suo morso.]** La CLASSE resta aperta: ~20 siti
   `git status --porcelain` (che quota E collassa le directory untracked:
   servono `-z` e `-uall` insieme) e `diff --name-only`.
2. **Hoare — il raw contiene un delta che il changeset non può produrre.**
   `slot_reads_rc` si conta al sito di lettura e non dipende dall'analisi:
   il −14 è rumore di suite, dello stesso ordine dei −42 già documentati fra
   `before` e F1. Ne segue che le attribuzioni −21/−18/−6 sono state scritte
   **senza controllare il pavimento di rumore**. **[ACCOLTA: annotata nel raw;
   servirebbe una coppia A/A che misuri il pavimento, e non è stata fatta.
   Resta in piedi ciò che non dipende da questo: `would_take`/`would_take_rc`/
   `sites_movable` sono identici al byte fra F1, F2 e riconteggio.]**
3. **Bak — il tetto A-LB-97-1 usato come tariffa.** Il costo per corpo caldo non
   è una costante: dipende da quanto quel corpo è caldo e da che cosa sposta
   fuori dalla i-cache. La chiusura del passo 2 va declassata a SOSPENSIONE.
4. **Matsakis — canale cross-frame.** `current_frame_args` legge gli slot vivi
   di OGNI frame: un canale che il test di TIPO a runtime non vede. Trovato
   indipendentemente anche da Stogov (team-analisi: B6/B7).
5. **Stogov — `namespace X; extract($a)` sfugge alla rinuncia**, provato a
   macchina: `observes_scope` è indicizzata sul nome SCRITTO, e il fallback di
   namespace lo aggira. `would_take_safe_str` è un TETTO, non un perimetro
   fedele.
6. **Leijen — `would_take_safe_str` conta `Rc::clone` elisi, ZERO allocazioni.**
   Non c'è nessuna lettura in chiave footprint da fare su quel numero: chi
   volesse convertirlo in byte risparmiati sbaglierebbe canale.
7. **Pedersen — P-AMEND-ORFANO.** L'identity registra `head=`, poi un `--amend`
   sostituisce l'oggetto: la citazione sopravvive, il referente no, e al primo
   `gc` la provenienza diventa **irrisolvibile**, non solo indimostrata. E
   l'amend che ha orfanizzato quel commit è proprio quello che portava nel
   giudice la cura di Klabnik: *la cura dell'uno è la causa dell'altro*.
8. **Hejlsberg — `LoadSlot{take}` è la peggiore delle due forme.** Il layout è
   neutro (`LoadSlot` usa 4 byte sullo slack di `Op`), quindi il flag non compra
   nulla, e tassa 60,6 M letture per servirne 9,99 M.

## Ordine WP-97 proposto (convergenza dei tre team)

1. **Ri-profilo R≥3, stesso workload, ZERO cambi di codice** — con
   mispredict-indiretti/op e L1I-miss/op normalizzati su `op-census`, peak
   registrato, e regola di lettura scritta PRIMA. È l'unica voce che serve i tre
   mandati insieme: rimette in moto il cronometro, ripara il denominatore da cui
   dipendono tutte le bande di due sessioni, e dice se O1 ha un canale prima di
   scriverla.
2. **O1 (outlining dei bracci freddi)** con controllo positivo DOPPIO (taglia
   predetta + outlineati ∩ `op-census`; L1I-miss/op giù, `op-census`
   invariante) e coppia stessa-sera.
3. **Braccio NULL cronometrato, nelle DUE forme** (dentro `run_loop` — Gregg; Δ
   di taglia equivalente FUORI — Bak): è ciò che rende decidibile il conto del
   passo 2.
4. **La forma dell'emissione** solo DOPO che l'entropia del bit è misurata: la
   sua motivazione scritta è oggi refutata (Bak: il bit è preso il 42,33% delle
   volte, quindi «per-sito quindi ben predetto» è falso).
5. **P-AMEND-ORFANO** (voce d'apparato ammessa): `refs/measure/<run>` piantata
   prima del run, oppure `head=` scritto DOPO l'ultimo amend.
6. **Footprint**: nessuna leva nominata; il falsificatore T_max di Leijen in
   timebox, e se non è misurato entro la sessione la leva arene si dichiara
   CHIUSA.

## Conflitti REGISTRATI (non appianati)

- **Conoscenza o rinuncia?** Gregg: rinuncia · Bak: né l'uno né l'altro,
  derivazione refutata ⇒ sospensione · Leijen: disciplina.
- **Segno del passo 2**: Klabnik NON OMOGENEO · Pedersen NULLO (F3 sospesa, non
  archiviata) · Hejlsberg REGGE nel suo perimetro · Stogov regge ma su un tetto.
- **`LoadSlot{take}`**: Klabnik «mai valutata, e §5.1 ammette che cambierebbe il
  verdetto» · Hejlsberg «valutata e refutata». Il team catena registra:
  *Klabnik ha ragione sul GRADO (argomento, non misura), Hejlsberg sulla
  DIREZIONE*.
- **Prima voce**: Gregg O1 · Bak il denominatore prima di O1 · Leijen T_max.
- **`would_take_safe_ref`**: Matsakis lo dichiara refutato da una sola firma
  (una sola sede di conteggio non copre il canale cross-frame).

---

# Nota di team — OGGETTO (Gregg ⟂ Bak ⟂ Leijen) — Concilio WP-98

Relatore: sedia 9 (mandato inverso). I tre verbali individuali restano la fonte
VINCOLANTE; qui si riconcilia dove si può e si REGISTRA il dissenso dove no.

## 1. Convergenze (reali, non levigate)

- **Il verdetto del passo 2 non è verdict-grade.** Le tre sedie lo dicono con
  tre parole diverse ma nessuna lo tratta come una chiusura solida: Gregg
  «SCREEN × VERDICT = SCREEN» (RC-BG-98-1), Bak «declassare a SOSPENSIONE, non
  archiviazione», Leijen «non refutato NEL MERITO» — cioè non ne difende il
  grado, ne difende l'esito.
- **Il grado va dichiarato dove si legge il verdetto**, non in una nota a §5
  (Gregg A-BG-98-4; Bak «abuso di grado, il più grave»; Leijen KS-DL-98-1/2 sui
  canali che non si convertono l'uno nell'altro).
- **Un numero preso da un'altra leva/binario non entra in questo conto**: Gregg
  RC-BG-98-2 («non è un confronto, è un'analogia»), Bak A-LB-98-1 («la tariffa
  è vietata»), Leijen A-DL-98-4 (`nm -S` e `phys_footprint` non si sommano).
- **Le tre candidate del §WP-97 sono sull'oggetto**: riconosciuto da Gregg e
  affermato da Leijen. Il difetto non è la lista: è che nessuna ha come esito
  un tempo.
- **La coppia, quando si farà, registra ANCHE il picco** (Leijen A-DL-98-2):
  costo zero, e nessuno obietta.

## 2. Conflitti — posizione di ciascuna sedia

**(a) Conoscenza o rinuncia?** — Gregg: **rinuncia**, con un frammento vero (il
piano B fantasma); il verdetto non nasce da dati sull'oggetto. Bak: **né l'uno
né l'altro** — la derivazione è refutata (2→9 corpi costò MENO di 2→4, WP-44),
la conclusione *può* restare vera ma non è provata; quindi sospensione. Leijen:
**disciplina, non rinuncia**. Dissenso non componibile: registrato.

**(b) Prima voce.** Gregg: O1 per prima (A-BG-98-3) + braccio NULL cronometrato
(A-BG-98-1). Bak: **non O1** — prima il DENOMINATORE (ri-profilo R≥3, zero
cambi di codice, A-LB-98-2); O1 è il ramo su cui lui non ha mai scommesso.
Leijen: il falsificatore T_max da 20 minuti (A-DL-98-1) — ma lo stesso Leijen
refuta la leva arene come **non sull'oggetto** (RC-2: 1,8% media, 1,06% full).

**(c) Il braccio NULL.** Gregg lo vuole DENTRO, per leggere il pedaggio reale.
Bak corregge sé stesso (A-LB-98-4): il null corretto è un Δ-taglia equivalente
**FUORI** da `run_loop`. Sono due esperimenti diversi, entrambi necessari.

**(d) Punto 1 del §WP-97.** Bak lo dichiara **falso** («per-sito quindi ben
predetto»: il bit è preso il 42,33% delle volte). Gregg lo declassa perché non
produce un tempo. Leijen non lo tocca.

## 3. Ordine proposto — criterio: massimo sblocco dell'oggetto al minimo costo

1. **Ri-profilo R≥3, stesso workload, ZERO codice** (Bak A-LB-98-2), con
   mispredict-indiretti/op e L1I-miss/op normalizzati su `op-census`, e peak
   registrato (Leijen). Mezza giornata; **rimette in moto il cronometro**,
   ripara il denominatore da cui dipendono TUTTE le bande di due sessioni, e
   dice se O1 ha un canale prima di scriverla. È la sola voce che serve i tre
   mandati insieme.
2. **O1**, con controllo positivo DOPPIO (A-LB-98-3: taglia predetta +
   outlineati ∩ `op-census`; L1I-miss/op giù, `op-census` invariante) e coppia
   stessa-sera. Parity-preserving, ed è il prerequisito dichiarato del tetto.
3. **Braccio NULL cronometrato**, nelle DUE forme (Gregg dentro, Bak fuori): è
   ciò che rende §4 decidibile e fa scattare o cadere KS-BG-98-2.
4. **Candidata 1 (forma dell'emissione)** solo DOPO che l'entropia del bit è
   misurata: oggi la sua motivazione scritta è refutata (Bak A-LB-98-5).
5. **Footprint**: nessuna leva nominata (Leijen); T_max in timebox, e se non è
   misurato entro la sessione la leva arene si dichiara CHIUSA (KS-DL-98-3).

## 4. Il grado — misura MINIMA per rendere usabile il canale

Il moltiplicatore §P1 (4,5–6,5%) è R=1, senza spread, sul pin invariato
`d5ce86e3`. **Minimo sufficiente**: ripetere lo STESSO profilo, stesso
workload, nessun cambio di codice, **R≥3**, con regola di lettura scritta
PRIMA, e pubblicare mediana **e spread** — un intervallo, non un punto. Ciò
porta il canale da SCREEN a banda con intervallo: sufficiente a *derivare*
bande dichiarandone il grado, **non** a chiudere un passo dell'ordine. Per il
grado VERDICT serve, in aggiunta, una coppia adiacente A/B interleaved
stessa-sera sul binario reale (il braccio NULL della voce 3). Finché manca
l'intervallo, vale Gregg: *una decisione senza intervallo non è una decisione,
è una preferenza*.

---

# Team ANALISI — nota di riconciliazione (fase 2, Concilio WP-98)

Sedie: Hoare (v1), Matsakis (v2), Stogov (v8). I verbali individuali restano la
fonte VINCOLANTE; qui si riconcilia o si registra il dissenso.

## 1. Convergenze — buchi di soundness ANCORA aperti dopo il fix di S-96.0

Elenco unificato, senza duplicati. `M` = provato a macchina, `A` = argomentato.

| # | Buco | Sedia | Prova |
|---|---|---|---|
| B1 | Invariante non presidiata: un op con `defs` **e** `edges` insieme farebbe ricadere il kill su tutti gli archi normali (classe A-TH-97-1 riaperta). Serve `debug_assert!` | Hoare (A-TH-98-1) | A (verificato che oggi nessun op lo ha ⇒ latente) |
| B2 | Il loop che allarga `nbits` incatena `uses`/`defs`/`fall_defs` ma **non** gli edefs per-arco (`CatchMatch::var`): `Bits::clear` panica invece di degradare | Hoare (A-TH-98-1) | A |
| B3 | Esaustività sulle VARIANTI, non sui CAMPI (`Op::X { .. }`): un campo `Slot` nuovo compila muto | Hoare (A-TH-98-2) | M parziale — 3 varianti campionate, nessuna porta `Slot` ⇒ l'elenco regge, la cura no |
| B4 | `renounce()` ha `nbits` più stretta di `analyze` (si allarga solo su `LoadSlot`/`LoadVar`): `mark` scarta, `Bits::get` risponde «non rinunciato». Direzione: F2 **meno** conservativa | Hoare (A-TH-98-3) | A |
| B5 | Arco di ri-lancio di `EndFinally` verso handler **esterno** non modellato (`edges` = `after` + soli `ParkJump`) | Hoare (A-TH-98-4) | A, nessuna fixture |
| B6 | **Canale cross-frame**: `current_frame_args` (mod.rs:10493) legge gli slot vivi di OGNI frame; `renounce()` scorre solo `func.ops`, quindi un osservatore nella *callee* non fa scattare `observes_scope` | Matsakis (RC-MS-98-1) e Stogov (A-DS-98-3), **indipendentemente** | A (puntatori esatti: host.rs:4343, mod.rs:13288) |
| B7 | Lo stesso canale non è chiudibile **per forma** da una lista di nomi: `getTrace()` di qualunque Throwable costruito più in basso, handler d'errore/eccezione, tick, `ob_start`, `usort`, `spl_autoload`, `ReflectionFiber::getTrace()` | Stogov (A-DS-98-3) | A |
| B8 | `CallNsFallback`: `observes_scope` interroga `name` e ignora `fallback` — `namespace X; extract($a);` esegue `ho_extract` | Stogov (A-DS-98-1) | A, fixture obbligatoria |
| B9 | Nome risolto a runtime non name-checked (`CallValue`, `CallValueArgs`, `CallNamed`, `CallSpread`, `MakeFcc`): `$f='extract'; $f($a);` | Stogov (A-DS-98-2) | A |
| B10 | `$http_response_header`: builtin che **scrive** lo scope del chiamante, in nessuna lista | Stogov (A-DS-98-4) | da verificare |
| B11 | `would_take_safe_str` è fedele all'**output**, non alla memoria: il take sposta il momento della COW, osservabile da `memory_get_usage`/`_peak`/`debug_zval_dump` | Stogov (A-DS-98-5) | A |

Nessun buco è oggi provato da una fixture che morde. B6/B7 sono lo stesso buco
trovato da due sedie per vie diverse: è la convergenza più forte del team.

## 2. Il delta sospetto su `slot_reads_rc` — DECISIVO SUL GRADO

Fatto ricontrollato dal relatore sul raw: F1 dà `slot_reads=60598107`,
`slot_reads_rc=53561199`; F2 **dichiara** «riprodotti IDENTICI al run F1»; il
riconteggio dà 60598093 / 53561185 ⇒ **−14 su entrambi**. Il determinismo era
stato stabilito due volte e ora è rotto: la premessa di fatto di Hoare (R1) è
confermata, e `grade=VERDICT` non è difendibile su un run non riprodotto.
La premessa di *meccanismo* («nessuna modifica del changeset può toccare
`note_slot_read`») resta però ARGOMENTATA: la verifica a macchina non è stata
possibile (Serena indisponibile in fase 2).

Corroborazione: le altre due sedie **non affrontano il punto** — non lo
corroborano né lo contraddicono. Convergono però sulla stessa conseguenza
pratica per altre vie: Matsakis (RC-MS-98-2, la banda Str è un maggiorante,
numeratore di conteggio × tasso di costo medio) e Stogov (il perimetro è un
TETTO; KS-DS-98-1). **Le bande cadono per tre strade indipendenti**; solo la
prima colpisce il *grado* del raw.

Contro-nota a verbale (relatore, non appianabile in favore di nessuno):
`would_take`, `would_take_rc` e `sites_movable` hanno delta **esattamente 0**.
Un rumore d'esecuzione generico li avrebbe mossi. Il −14 è quindi confinato a
letture in siti non-movibili: o una sorgente di non-determinismo che tocca solo
quella classe, o un canale del changeset non ancora nominato. In nessuno dei due
casi −21/−18/−6 sono attribuibili ad A-SK-97-1. Sopravvive solo
`delta_would_take = 0`.

## 3. Conflitti — posizioni non appianate

**(a) Verdetto del passo 2 (Str-first vs piano B).** Esito concorde («non
vince»), motivazione in disaccordo.
- *Hoare* (R2): la formula scritta è sbagliata di segno. 0,84–1,21% lordo contro
  +2,9% di pedaggio non è «non distinguibile da zero», è **nettamente negativo**
  nella forma a braccio nuovo; **ignoto** nella forma per-braccio, mai valutata
  (§5.1). La rinuncia a `nm -S` è circolare.
- *Stogov*: il verdetto **sopravvive** ma non per le ragioni scritte — essendo
  `would_take_safe_str` un TETTO, il perimetro vero è ≤ e la strada lunga esce
  *ancora più debole*. Nessuna banda citabile finché A-DS-98-1/2/3 sono aperti.
- *Matsakis*: **esito confermato, motivazione refutata** — la banda è un
  maggiorante perché `Str` è la variante col drop più economico, sotto la sua
  quota di conteggio.

**(b) Forma della leva.** *Matsakis* (A-MS-98-3) dichiara il flag dentro
`LoadSlot` ownership-**equivalente** a `TakeSlot` (cambia solo il lato costo) e
chiede che l'analisi stia nel COMPILATORE, come funzione pura degli ops, per via
della unit cache TL (WP-81). *Hoare* (KS-TH-98-3) e *Stogov* (KS-DS-98-4) dicono
che **qualunque leva che aggiunga un braccio prima di O1 è stop**. Tensione
reale: la forma-flag che Matsakis lascia aperta è precisamente quella che
aggiunge un braccio nel corpo caldo. Registrata, non risolta.

**(c) `would_take_safe_ref`.** Il raw lo legge come «buco minuscolo ⇒ guard poco
costoso». *Matsakis* lo refuta: misura solo il canale visibile nel **tag della
cella**, è un limite inferiore di un canale enumerato, e KS-MS-98-2 impone di
ritirarlo, non difenderlo. *Hoare* e *Stogov* non lo toccano: la refutazione ha
una sola firma.

**(d) Priorità.** Stogov mette O1 (241,7 KiB in una funzione contro ~192 KiB di
L1i) sopra ogni perimetro; Hoare concorda (voce 1 §WP-97 solo INSIEME a O1);
Matsakis non si pronuncia.

## 4. Che cosa va rifatto prima di qualunque emissione (lista minima, in ordine)

1. **Ritirare `grade=VERDICT`** dal riconteggio → SCREEN, e ri-eseguire R≥2 a
   binario invariato finché il −14 non ha una causa nominata (o è registrato
   come sorgente di rumore). Nessuna cifra del raw si cita prima.
2. **Fixture negative che mordono sul binario pre-leva** per B8 e B9
   (`CallNsFallback.fallback`; `$f='extract'; $f($a);`). Se mordono, F1/F2 di
   S-95.0 e S-96.0 decadono a TETTO (KS-DS-98-1) e ogni banda si ritira.
3. **Chiudere il canale ARGOMENTI/cross-frame** (B6+B7): escludere gli slot
   `< n_params` da `movable_safe` e **ri-contare** (A-MS-98-1) — il delta È la
   taglia del canale invisibile. Trappola nel gate: callee che chiama
   `debug_backtrace()` e legge un parametro del chiamante, più un'eccezione con
   trace, più `ReflectionFiber::getTrace()` (KS-MS-98-1).
4. **Riparare i tre difetti interni**: `nbits` di `renounce()` = `nbits` di
   `analyze` + assert (B4); allargamento di `nbits` anche sugli edefs per-arco
   (B2); `debug_assert` dell'invariante `defs` ⊥ `edges` (B1).
5. **Fixture per l'arco di ri-lancio di `EndFinally`** verso handler esterno
   (B5) — stessa classe che è già costata una sessione.
6. Solo allora: ri-contare, riscrivere il netto come `gain(str) − guard(safe)`
   (A-MS-98-2), etichettare `would_take_safe_str` come fedele-all'output e
   portare la COW/`memory_get_usage` nel gate per NOME (B11, A-DS-98-5).
7. **Nessuna emissione — opcode o flag — prima di O1** (KS-TH-98-3, KS-DS-98-4).

Voci rimaste in piedi e non evase: A-TH-98-5 (ri-registrare A-TH-97-4 `gc_note`
e A-TH-97-5, evaporati dal backlog) e A-TH-98-6 (design96 arbitra con un +2,9%
che non trascrive: un documento di decisione non auditabile non decide).

---

# Team CATENA — nota di relatore (fase 2, Concilio WP-98)
Sedie: Klabnik (v-3), Pedersen (v-6), Hejlsberg (v-4). **I verbali individuali
restano la fonte vincolante**; questa nota riconcilia dove si può e REGISTRA i
dissensi dove non si può.

## 1. Un difetto o due? — due meccanismi, una forma
Klabnik: il perimetro si fida della FORMA che git sceglie di stampare (quoting
nello SPAZIO). Pedersen: l'identity si fida di un NOME (`head=`) il cui
referente può ancora cambiare (mutabilità nel TEMPO). Meccanismi distinti,
stessa classe: **il giudice tratta come dato ciò che è una rappresentazione
prodotta altrove**. E i due si toccano materialmente: l'amend che orfanizza
`7847cc0` è quello che porta in `83661e4` la cura del giudice (+13/−2 su
`gate-measure-cifre.sh`). *La cura dell'uno è la causa dell'altro.*

**Il pattern va nominato: P-AMEND-ORFANO** — un artefatto registra `HEAD`, poi
un `--amend` sostituisce l'oggetto: la citazione sopravvive, il referente no,
e al primo `gc` la provenienza diventa **irrisolvibile** (non solo
indimostrata). Il team adotta **A-PP-98-3** come dente, non come lezione:
o `refs/measure/<run>` prima del run, o `head=` scritto dopo l'ultimo amend.

## 2. La cura di Klabnik: forgia CHIUSA, classe APERTA
Verificato: riga 1265 usa `-c core.quotePath=false … ls-files --others -z`,
split su NUL; stessa correzione su `check-ignore -v`; **T31 con il suo morso**
(braccio pre-98 che deve vedere 1/2 gemelli, altrimenti il dente è vacuo).
Ma la classe ha almeno **due altri siti**, verificati a macchina:

- **`gate-measure-cifre.sh:897`** — `split /\n/, qx(git ls-tree -r --name-only
  HEAD)`. `ls-tree --name-only` **quota** (provato: `"perimetr\303\262.md"`).
  È l'autorità *committed-only* (A-SK55) e la fonte di `@headtree`/`%headset`:
  un file di corpus o un `m9*.campaign.ledger` con nome non-ASCII sarebbe
  **invisibile all'ancoraggio** e la campagna non verrebbe mai validata.
  Cura simmetrica: `-c core.quotePath=false … -z` + `split /\0/`.
- **~20 siti `git status --porcelain`** (measure8x/9x-campaign, battery-8xpre,
  e la cattura di `tree_dirty`). Qui i due difetti si sommano: porcelain
  **quota** *e* collassa una directory untracked in una riga (`?? d/`) —
  esattamente il «6 che sottostima senza limite» di Pedersen. Solo `-z` **e**
  `-uall` insieme chiudono entrambi (provato). Minore: `diff --name-only`
  (`battery-equivalence.sh:217/223/498`) quota allo stesso modo.

## 3. Dove iscrivere il debito di Hejlsberg
Convergenza netta: **A-AH-98-3 (iscrivere) + A-PP-98-7 (i riferimenti come
dente)**. Iscrizione in tre luoghi — TODO master PER NOME, `NEXT_SESSION`,
marker `TODO(port)`-grade sopra `zvalcensus.rs:81-85` — e **A-PP-98-7 che fa
risolvere quei riferimenti a HEAD**, altrimenti FAIL. Senza il dente,
l'iscrizione è un altro verbale: KS-AH-98-2 è la sola forma che morde.

## 4. Dissensi REGISTRATI (non appianati)
- **Verdetto passo 2.** Klabnik: **non omogeneo** — banda SCREEN R=1 contro
  costo storico di altro workload/era; F3 preclusa su un confronto mai fatto
  alla pari (A-SK-98-3). Pedersen: **NULLO, non contrario** — uno spareggio con
  entrambe le premesse refutate *decade*; F3 **SOSPESA**, non archiviata.
  Hejlsberg: **regge** — «argomentato invece che subìto»; è il §WP-97 punto 1 a
  non stare in piedi, e **non va messo per primo**.
- **La forma `LoadSlot{take}`.** Klabnik: **mai valutata**, e §5.1 ammette che
  cambierebbe il verdetto. Hejlsberg: **valutata e refutata** (RC-1: layout
  neutro, ma tassa 60,6 M letture per servirne 9,99 M — «la peggiore delle
  due»). Il team nota che Hejlsberg stesso dichiara il netto «indistinguibile
  da zero… il numero che nessuno ha intenzione di produrre»: **Klabnik ha
  ragione sul GRADO (argomento, non misura), Hejlsberg sulla DIREZIONE.**
- **Verdetti complessivi divergenti**: FAIL (K) · RESPINTO IN PARTE (P) ·
  REGGE-nel-perimetro (H).
- **Convergenza operativa**: KS-SK-98-2 (pre-fix `e318fbfc` non committato ⇒
  morso `SALTATO` e muto) e A-PP-98-2 (nessun `.out` è autorità se non
  riproducibile) sono **la stessa regola**: ricostruibile dal commit, o
  `grado=interno` scritto in testa al file.

---

# Verbale sedia 1 — Hoare (WP-98, su S-96.0 e programma §WP-97)

Perimetro: design linguaggio/runtime Rust, safe-only. Mandato: refutare.

## VERDETTO: CON EMENDAMENTI (con due refutazioni capitali)

**Stato dei miei emendamenti WP-97.** A-TH-97-1 applicato, e nella variante
MIGLIORE delle due che avevo offerto (`inb` clonato da `out` PRIMA del merge
exc, kill delle def, poi exc rifuso in entrambi): `out` — giudice della
movibilità — porta il contributo dell'handler senza il kill, `inb` lo riceve
dopo. **È sound.** A-TH-97-2 applicato ma solo a metà (vedi A-TH-98-2).
A-TH-97-3 eseguito (design96). **A-TH-97-4 e A-TH-97-5 sono EVAPORATI**: non
sono nel backlog per NOME né nei NON-riproporre. KS-TH-97-1/2/3 nessuna scatta.

## Emendamenti

**A-TH-98-1 — L'invariante che rende sound il fix non è presidiata da nulla.**
Oggi nessun op ha insieme `defs` ed `edges`: se ne avesse uno, il kill di
`e.defs` colpirebbe TUTTI gli archi normali, ricreando la stessa classe di
errore su un arco dove la def non è avvenuta. Serve `debug_assert!(e.defs
.is_empty() || (e.edges.is_empty() && e.fall_defs.is_empty()))`. In più: il
loop che allarga `nbits` incatena `uses`/`defs`/`fall_defs` ma **non gli edefs
per-arco** (`CatchMatch::var`) — e `Bits::clear` indicizza senza difesa, quindi
lì si panica invece di degradare. Un canale su quattro dimenticato.

**A-TH-98-2 — L'esaustività copre le VARIANTI, non i CAMPI.** Ogni arm
no-effect è scritto `Op::X { .. }`: aggiungere un campo `Slot` a `Sweep`,
`FillDefault` o `HookCall` compila in silenzio. Campionate tre sospette —
`CallBuiltinRefCell { name, argc }`, `StaticStore { id }`, `MakeFcc { name }`:
**nessuna porta uno Slot** (verificato sull'enum: l'insieme delle varianti che
portano `Slot`/`DimBase`/`FieldBase` è interamente classificato FUORI
dall'elenco no-effect). L'elenco di oggi regge; la cura no.

**A-TH-98-3 — `renounce()` scarta le rinunce in silenzio.** La sua `nbits` si
allarga solo su `LoadSlot`/`LoadVar`, mentre quella di `analyze` si allarga su
tutti gli usi; `mark` scarta l'indice fuori larghezza e `Bits::get` risponde
«non rinunciato». Doppio fallimento silenzioso, e nella direzione che rende F2
MENO conservativa. Stessa larghezza di `analyze` + assert.

**A-TH-98-4 — L'arco di ri-lancio di `EndFinally` non è modellato.** `edges` =
`after` + i soli bersagli di `ParkJump`. Un'eccezione parcheggiata e ri-lanciata
all'`EndFinally` raggiunge un handler ESTERNO: quell'arco esiste solo se l'op
cade dentro la regione esterna di `exc_table`, cosa che nessuna fixture prova.
Stessa classe di A-TH-97-1. Fixture obbligatoria prima di ogni emissione.

**A-TH-98-5 — Ri-registrare A-TH-97-4 (contabilità `gc_note` del valore
spostato) e A-TH-97-5.** Un emendamento il cui oggetto è rinviato non decade da
solo.

**A-TH-98-6 — design96 arbitra con un numero che non trascrive** (+2,9%,
`WP_SESSION_41`). Un documento di decisione non auditabile non decide.

## Kill-switch

**KS-TH-98-1**: un campo `Slot` nuovo su una variante dell'elenco no-effect →
tutti i conteggi F1/F2 invalidi.
**KS-TH-98-2**: un riconteggio a parità di binario che muove `slot_reads_rc` →
il raw scende da VERDICT a SCREEN e nessun delta sotto il rumore è attribuibile.
**KS-TH-98-3**: se il tetto A-LB-97-1 resta insoddisfacibile prima di O1, la
voce 1 del §WP-97 non può concludere e va eseguita SOLO insieme a O1.

## Refutazioni capitali — SÌ, due

**R1 — Il pavimento di rumore è dentro il raw stesso.**
`delta_slot_reads_rc_vs_f2 = -14` su un contatore che NESSUNA modifica del
changeset può toccare (`note_slot_read` conta al sito di lettura, indipendente
da `analyze` e da `observes_scope`). Quindi le due esecuzioni non sono
identiche. Ne segue: i delta −21/−18/−6 **non sono attribuibili** ad A-SK-97-1
né a nulla; `grade=VERDICT` è sbagliato (contare esattamente un'esecuzione non
deterministica dà un verdetto sul run, non sul programma); sopravvive solo
`delta_would_take = 0`, che è esatto.

**R2 — Il verdetto del passo 2 è insieme troppo debole e troppo largo.** Coi
numeri del documento: lordo 0,84–1,21% contro un pedaggio in casa di +2,9%. Non
è «non distinguibile da zero»: **in quella forma è nettamente negativo**. Nella
forma per-braccio, che §5.1 dichiara mai valutata, è **ignoto**. «Non
distinguibile da zero» non descrive nessuna delle due. E la rinuncia a `nm -S`
è circolare: non si misura perché si è deciso di non costruire, e si è deciso
per un pedaggio non misurato. Riscrivere: «la forma a braccio nuovo perde con
margine; la forma per-sito è indecisa e costa una misura, non una leva».

---

# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — WP-98

## VERDETTO: ESITO CONFERMATO, MOTIVAZIONE REFUTATA

Non costruire `TakeSlot` in S-96.0 è giusto, e tutto ciò che trovo spinge nella
stessa direzione (meno guadagno, più rischio). Ma la riga `nota-guard-di-tipo`
del riconteggio è un errore di lettura, e il perimetro Str-first è stato
calcolato con un numeratore senza il suo denominatore.

## Refutazioni capitali

**RC-MS-98-1 — «il buco è minuscolo» è REFUTATO.** `would_take_safe_ref`
(zvalcensus.rs:109) è `matches!(cell, Zval::Ref(_))`: misura l'unico canale di
aliasing **visibile nel tag della cella**. Il buco vero è «lo slot è osservabile
altrove», e ne esiste almeno uno che quel test non può vedere per costruzione:
`current_frame_args` (mod.rs:10493) legge `frame.slots[i]` **VIVI di OGNI frame
sullo stack** al momento della chiamata, e alimenta gli `args` di
`debug_backtrace` (host.rs:4343) e dell'altro costruttore di trace
(mod.rs:13288). `renounce()` scorre **solo `func.ops`**: se l'osservatore sta
nel corpo di una *callee* (o nella macchina delle eccezioni), `observes_scope`
non scatta mai, la cella resta una `Str` semplice per tutto il tempo, e uno slot
di parametro svuotato da un take si legge `Undef`→NULL nel trace. In Zend i CV
non si consumano mai (Stogov, WP-97): è divergenza garantita, non probabile.
3307 è dunque un **limite inferiore di un canale enumerato**, non la misura del
buco. La lezione del passo 0 di S-96.0 — «una cura enumerabile contro un attacco
non enumerabile è vacua per costruzione» — si applica alla lettera alla
conclusione del passo 1.

**RC-MS-98-2 — la banda Str è un numeratore orfano.** `guadagno_..._str` =
(9.989.963 / 53.561.185) × moltiplicatore. Il moltiplicatore §P1 è la quota CPU
del canale su **tutte** le letture rc: moltiplicare una frazione di CONTEGGIO
per un tasso di COSTO **medio** assume che clone/drop costino uguale per
variante. `Str` è precisamente la variante col drop più economico (dec + branch;
nessun distruttore, nessuna liberazione ricorsiva, nessun teardown dual-repr):
la sua quota di costo sta **sotto** la sua quota di conteggio. 0,84–1,21% è
quindi un maggiorante, oltre che SCREEN.

## Emendamenti

**A-MS-98-1 (misurare il canale invisibile, a costo di un run).** Gli slot
`< n_params` sono osservabili cross-frame: escluderli da `movable_safe` e
RI-CONTARE. Il delta è la taglia del canale che `would_take_safe_ref` non vede.
Finché quel numero non esiste, nessuna banda safe/str è citabile.

**A-MS-98-2 (base di costo ≠ base di guadagno).** La guardia si paga dove il
take è **emesso** (safe: 25.826.594 esecuzioni, 51.691 siti), il guadagno si
incassa dove il tipo combacia (9.989.963 = 38,7%). Ogni netto futuro sia scritto
come `gain(str) − guard(safe)`; oggi è stato scritto solo il primo termine.

**A-MS-98-3 (flag vs opcode: equivalenza dichiarata).** §WP-97 punto 1 è
ownership-**equivalente** a `TakeSlot`: identico trasferimento, identico
perimetro, identici A-MS-97-2/3/4. Cambia SOLO il lato costo. Va scritto, perché
nessuno ri-apra la questione di correttezza sotto la forma nuova. Due corollari:
(a) «deciso a compilazione» **non** dà conoscenza di tipo — la guardia runtime
resta (e costa poco: `read_slot` discrimina già il tag); (b) un bit dentro
`LoadSlot` viaggia invisibile nella unit cache TL (WP-81) in ogni richiesta
successiva: l'analisi stia nel COMPILATORE, il bit sia funzione pura degli ops,
mai mutato.

## Kill-switch

**KS-MS-98-1**: nessun take si emette prima che il gate contenga la trappola
cross-frame (callee che chiama `debug_backtrace()` e legge un parametro del
chiamante, più una eccezione con trace). Se la trappola non morde sul binario
pre-leva, non è un dente.
**KS-MS-98-2**: se `would_take_safe_ref` viene ancora citato come «il buco», il
numero si ritira invece di difenderlo.

---

# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — WP-98

## VERDETTO

**FAIL — una forgia è ATTERRATA.** Il canale env di git è chiuso (T27-T30
tengono), ma il perimetro A-SK-96 è CIECO ai nomi di file non-ASCII. Ho
provato tutte le vie del mandato; T27..T30 hanno retto ai loro attacchi
letterali (env, GIT_CONFIG_*, PERL5OPT, depth-marker). Ma A-SK-96 non passa
per l'ambiente: passa per come `git ls-files --others` STAMPA i path, e lì il
giudice si fida di un ancoraggio che il quoting di git rompe. Un doc non
committato con una cifra fabbricata e una lettera accentata nel nome ESCE dal
perimetro, non nominato, mai giudicato. `--all` a albero pulito darebbe
`PASS` rc=0 con la cifra in albero.

## Emendamenti A-SK-98-n

- **A-SK-98-1 (il buco)**: la lista untracked va letta con
  `git -c core.quotePath=false ls-files --others -z -- php-rust` e spezzata su
  NUL, MAI su `\n`. Oggi `qx(git … ls-files --others -- php-rust)` (riga 1223)
  con `core.quotePath=true` di default avvolge i path con byte >127 in
  `"php-rust/…\303\262….md"`: l'ancoraggio perl `qr{^php-rust/.*\.md$}` (1204)
  NON matcha la stringa quotata (inizia con `"`, finisce con `"`), e il doc
  sfugge. Il `-z` chiude anche il vettore newline-nel-nome (uno `split /\n/`
  spezza un nome con `\n` in due path fantasma).
- **A-SK-98-2 (dente permanente T31)**: pianta un doc con cifra e nome
  non-ASCII sotto un perimetro-classe, esegui `--all`, e PRETENDI che il path
  sia NOMINATO in una riga FAIL. Braccio-morso: lo stesso su un giudice
  pre-A-SK-98-1 deve NON nominarlo (la cecità è reale). Senza morso è un
  ricordo, non un dente.
- **A-SK-98-3 (grado del passo 2)**: il verdetto di `design96` va marcato
  SCREEN per campo. Confronta una banda derivata da canale R=1 (SCREEN) col
  costo storico di un corpo caldo (WP-41/44, altro workload/era): non
  omogenei. §5.1 nomina una forma d'emissione (`LoadSlot` con flag `take` a
  compilazione) «che potrebbe cambiare il verdetto» e MAI valutata. Il passo
  2 preclude F3 su un confronto che ammette di non aver fatto alla pari.

## Kill-switch KS-SK-98-n

- **KS-SK-98-1**: finché A-SK-98-1 non atterra e T31 non è verde, ogni
  `PASS --all` è provisional PER NOME: il perimetro non copre i nomi non-ASCII
  né i newline. Nessun PASS di cifre è verdict-grade sotto questa apertura.
- **KS-SK-98-2**: il «morso del fix» di `check-liveness-fixtures.sh` dipende
  da un binario pre-fix (`phpr-pre-target`, sha e318fbfc248a8e35) costruito a
  mano e NON committato. Se sparisce, l'arco `morso del fix` stampa `SALTATO`
  e tace: la differenza `would_take 6→4` non è più provata dal repo. Se il
  pre-fix manca all'apertura S-97, l'arco è dichiarato provisional finché non
  si ricostruisce dal commit 83661e4 (oggi raggiungibile).

## Refutazioni capitali (col comando riprodotto)

**SÌ — A-SK-96 non vede i nomi non-ASCII.** Piantati due doc-cifra gemelli,
uno ASCII e uno accentato; solo l'ASCII è stato nominato dal giudice:

```
$ printf 'ASCII: picco 123480 B\n'  > wp98-harness/zzforge-k13-ascii.md
$ printf 'accentato: picco 123472 B\n' > wp98-harness/zzforge-k13-perimetrò.md
$ bash wp81-harness/gate-measure-cifre.sh --all 2>&1 | grep zzforge-k13
FAIL … UNCOMMITTED php-rust/wp98-harness/zzforge-k13-ascii.md … NO manifest entry
# (zzforge-k13-perimetrò.md: NESSUNA riga — sfuggito)

$ git ls-files --others -- php-rust | grep zzforge-k13
"php-rust/wp98-harness/zzforge-k13-perimetr\303\262.md"   # quotato → ^php-rust/ non matcha
$ perl -e 'print( q{"php-rust/…\303\262….md"} =~ m{^php-rust/.*\.md$} ? "IN":"FUORI")'
FUORI
```

Il canale env resta chiuso (nessuna forgia env/config/exclude/argv è
atterrata: T27-T30 nominano correttamente ogni attacco letterale; `.git/config`
core.excludesFile è neutralizzato dal `-c …=/dev/null`, `.git/info/exclude`
cade su `head_blob_sha` vuoto). Il difetto è UNO: il perimetro fidava della
FORMA testuale che git sceglie di stampare.

**Residui**: tutti i miei forge (`zzforge-k13-*.md` + AppleDouble `._`)
rimossi; `.git/config`, `.git/info/exclude`, working tree = baseline
(verificato). Nessun commit.

---

# Verbale — Sedia 4 (Hejlsberg) — WP-98
Perimetro: compilatori incrementali, interning/dedup, emissione.

## VERDETTO

S-96.0 regge nel mio perimetro: il fix di soundness, i match esaustivi e il
riconteggio sono lavoro di compilatore fatto bene, e il verdetto del passo 2 è
argomentato invece che subìto. Il **§WP-97 punto 1 no**: è scritto su una
premessa di layout che è irrilevante e su una premessa di costo che è un errore
di categoria. Come formulato **non riapre A-ZV2 e non va messo per primo**.

## Il conto sul flag `take` (la domanda che mi è stata posta)

**La taglia di `Op` non c'entra, in nessuna delle due forme.** `Op` è
`#[derive(Clone, PartialEq)]`, allineamento 8, e la sua stride è fissata dalle
varianti **fredde** con `Rc<[u8]>` fat-pointer (`CallHostBuiltinOut` ≈ 44 B di
payload; `StaticCall` con `ClassTarget`+`MethodIc` è l'altro candidato).
`LoadSlot(Slot)` occupa **4 byte su ~40 disponibili**: quello slack è già pagato
oggi, su ogni op di ogni `Vec<Op>`. Quindi `LoadSlot { slot, take: bool }` →
`size_of::<Op>()` **INVARIATO**, stride invariata, streaming del bytecode
invariato, footprint per worker invariato. E le varianti sono ~180 < 256: il tag
resta 1 byte, **anche un `TakeSlot` nuovo è layout-free**. La premessa «un flag
cambia la taglia di `Op`» è **falsa**; ma è falsa anche per il piano opposto,
quindi il layout **non arbitra nulla**. Ritratto qui la parte di A-AH-97-4 che
lasciava credere il contrario: l'assert è una guardia di regressione, non un
argomento.

**Ciò che il flag sposta è dove cresce il codice caldo, non quanto.** Il braccio
`LoadSlot` (`run.rs:585`) oggi è tre righe. Col flag diventa due percorsi + il
guard di tipo su `Zval::Ref` dentro **il braccio più eseguito del `run_loop`**:
60.598.093 letture di slot sul media group, per servirne 25.826.594 safe (42%) e
9.989.963 stringhe (16,5%). Si tassano **tutte** le letture per pagarne un sesto.
Un opcode separato lascia il braccio caldo **intatto** e tassa solo i siti take.
Perciò «un `LoadSlot` con flag non è un corpo caldo in più» è **vero alla
lettera e falso nella moneta**: WP-39..44 non contava i simboli, contava il
working-set I-cache/BTB del `run_loop`, e il flag lo fa crescere nel punto di
massima frequenza. Stogov ha ragione.

Ordine di grandezza: guadagno lordo nucleo stringhe 0,84–1,21% (SCREEN, R=1);
un branch ben predetto + il guard su 60,6 M esecuzioni non è quotabile a mente,
ma è **dello stesso ordine**. Il netto resta indistinguibile da zero, ed è
esattamente il numero che nessuno ha intenzione di produrre.

## Emendamenti

**A-AH-98-1.** Il punto 1 si istruisce **solo** con l'A/A di A-AH-97-5 (build
gemella col percorso compilato e mai emesso) + `nm -S` di `run_loop`
prima/dopo, coppia adiacente. `nm -S` da solo misura la taglia, non il pedaggio
per esecuzione: non basta. Senza A/A, il punto 1 **non è una voce di sessione**.
**A-AH-98-2 (precedenza).** O1 (outlining) prima: è l'unica leva che abbassa i
corpi caldi, è misurabile con lo stesso strumento e **produce il giudice** che a
tutte e tre le candidate manca. Il punto 1 senza O1 rifà WP-39..44.
**A-AH-98-3 (debito non evaporato).** A-AH-97-1/3 e KS-AH-97-1/3 vivono SOLO
dentro `wp97-harness/`: non sono in NEXT_SESSION, non in TODO.md, non come
marker nel codice. Vanno iscritti al backlog PER NOME **e** come commento
`TODO(port)`-grade sopra la chiave di `zvalcensus.rs:81-85`, che oggi si
autogiustifica con «accettabile in una build di sola misura» — una condizione
che nulla presidia.
**A-AH-98-4 (archiviazione falsificabile).** Un perimetro è archiviato, e non
abbandonato, **solo se** il documento nomina l'artefatto datato il cui valore
ribalterebbe il verdetto. design96 §5 nomina tre voci ma nessun artefatto: va
aggiunta la riga «A/A + O1 ⇒ riapertura», altrimenti la distinzione è una
formula.

## Kill-switch

**KS-AH-98-1**: si scrive `take`/`TakeSlot` senza A/A a monte → reject.
**KS-AH-98-2**: cache d'analisi keyed-by-pointer in un percorso di emissione
(non di sola misura) → reject senza discussione (riaffermo KS-AH-97-3).
**KS-AH-98-3**: `size_of::<Op>()` non fissato da un test nello stesso commit
del primo cambio a `Op` → reject.

## Refutazioni capitali

**RC-1.** «Un `LoadSlot` col flag `take` non è un corpo caldo in più» —
**refutata**. Il layout è neutro (lo slack c'è già), quindi il flag non compra
nulla sul fronte che invoca; e sul fronte che conta sposta la crescita dal
numero dei corpi alla **taglia del corpo più caldo**, tassando 60,6 M letture
per servirne 9,99 M. Non è la terza forma: è la peggiore delle due.

**RC-2.** «Il verdetto ha sospeso il debito» — **falso**. Il codice
pointer-keyed è ancora nell'albero e la sua unica difesa è una condizione
(«sola misura») che il punto 1 stesso violerebbe riusando `analyze`
dall'emissione. Un debito che vive solo nel verbale che l'ha nominato non è
registrato: è **evaporato**. Prima voce di S-97.0 nel mio perimetro, prima di
qualunque leva.

---

# Verbale sedia 5 — Bak (VM: alloc-rate, code-cache, percorsi caldi, dispatch) — WP-98

## VERDETTO

**Il passo 2 ha ABUSATO del mio tetto. Verdetto di chiusura NON verdict-grade:
va declassato a SOSPENSIONE, non archiviazione.** Il §WP-97 è nell'ordine
giusto per due terzi, ma la sua prima voce è sbagliata e la sua motivazione al
punto 1 è tecnicamente falsa.

**L'abuso.** A-LB-97-1 dice «Δ netto bracci caldi ≤ 0», cioè impone un
CONTROLLO. Non ha mai detto «un corpo caldo costa ~1%». Il passo 2 ha letto il
tetto come una TARIFFA e l'ha sottratta a una banda. I nostri stessi numeri
falsificano la tariffa: WP-44 v1 (2→4 corpi) = **+1,17%**, v3 (2→9 corpi) =
**+1,01%**. Nove corpi costano MENO di quattro. Se ci fosse un prezzo per
corpo, quell'ordine sarebbe impossibile: la varianza di layout domina il
termine per-corpo. Accanto: WP-41 = +0,62% per ~60 siti inline, WP-33 = +2,9%
per UN branch mai preso. Il costo va da 0,6% a 2,9% e non è monotòno nel
numero di corpi. Un intervallo che copre 5× non sottrae niente a una banda di
0,84–1,21%.

**L'abuso di grado, che è il più grave.** design96 §4 dichiara con cura che il
guadagno è SCREEN — e poi gli sottrae un costo MISURATO (A/B interleaved,
oracle di giornata) preso da una leva di FORMA DIVERSA. Il documento applica la
disciplina del grado al lato che vuole salvare e non al lato che uccide. Un
costo misurato su un'altra leva non è una misura di questa.

**Il cerchio.** Il costo vero era predicibile a costo quasi nullo: `nm -S`
sulla taglia predetta. Non è stato calcolato «perché non si predice la taglia
di un braccio che non si scriverà» — ma la taglia era l'INPUT della decisione
di scriverlo. La decisione ha consumato il proprio output.

## Emendamenti (A-LB-98-n)

- **A-LB-98-1 (la tariffa è vietata)**: nessun conto netto può sottrarre un
  costo di corpo caldo da una banda finché quel costo non è misurato SU QUESTA
  FORMA. Il tetto A-LB-97-1 è un controllo a posteriori, non un addendo a
  priori. Il verdetto del passo 2 va riscritto come «non decidibile senza
  misura», che è un'altra cosa da «netto zero».
- **A-LB-98-2 (prima voce: il denominatore, non O1)**: il moltiplicatore
  4,5–6,5% è R=1 senza spread e ha diviso o moltiplicato ogni decisione di due
  sessioni. Ri-profilo **R≥3, stesso workload, ZERO cambi di codice**, più i
  contatori discriminanti della mia consulenza §1 (mispredict indiretti/op,
  L1I-miss/op, normalizzati su `op-census`). È misura sull'OGGETTO, non
  apparato; rimette in moto il cronometro in mezza giornata; e dice se O1 ha un
  canale PRIMA di scriverla. Regola scritta prima, come in consulenza §1.
- **A-LB-98-3 (controllo positivo di O1, che è DOPPIO)**: (a) meccanismo —
  `nm -S run_loop` con taglia predetta prima, PIÙ l'elenco dei simboli
  outlineati intersecato con `op-census`: zero dei top-40 outlineati, ogni
  outlineato sotto soglia di frequenza. La taglia da sola prova che LLVM ha
  outlineato, non che il working set caldo sia calato. (b) canale — L1I-miss/op
  DEVE calare; `op-census` totale INVARIANTE. Solo dopo, l'orologio.
- **A-LB-98-4 (il mio bite test era mal posto — lo correggo)**: «outlinea
  bracci mai eseguiti, predici CPU invariata» è incoerente, perché outlinare
  codice freddo interlacciato È il meccanismo i-cache. Il null control corretto
  è outlinare un Δ-taglia equivalente in una funzione FUORI da `run_loop`.
- **A-LB-98-5 (punto 1 del §WP-97, la frase falsa)**: «branch per-sito, quindi
  ben predetto dal BTB» è un errore di livello. Il flag `take` è statico per
  SITO DI BYTECODE, ma l'istruzione di branch sta a UN PC dentro `LoadSlot` ed
  è eseguita da tutti i siti: la predizione dipende dalla sequenza dinamica del
  bit, non dalla sua staticità. Col nostro raw quel bit è preso il 42,33% delle
  volte (18,65% sul solo nucleo stringhe): entropia quasi massima, il caso
  peggiore per un condizionale a due vie sul percorso di lettura più caldo che
  abbiamo. Vero che non aggiunge un braccio; il prezzo è che sposta la
  mispredizione da un indiretto (dove il predittore ha storia) a un diretto mal
  bilanciato. Le due forme vanno MISURATE, non argomentate.

## Kill-switch (KS-LB-98-n)

- **KS-LB-98-1**: se A-LB-98-2 dà mispredict/op < 0,10 **e** L1I-miss/op < 0,02,
  O1 non ha canale: non si scrive, si passa a O2 (dieta della testa).
- **KS-LB-98-2**: O1 con taglia calata ma L1I-miss/op invariato → la leva non è
  provata; nessun Δ tempo rivendicabile, anche se favorevole.
- **KS-LB-98-3**: `op-census` totale non invariante su O1 → è cambiato il
  lavoro, non il layout: revert, la coppia non è valida.
- **KS-LB-98-4**: qualunque documento futuro che sottragga una cifra di costo
  per-corpo-caldo senza misurarla su quella forma → respinto in radice
  (A-LB-98-1).
- **KS-LB-98-5**: terza sessione consecutiva senza cronometro sulla rotta
  CPU-VM → la rotta si dichiara SOSPESA per nome nell'handoff.

## Refutazioni capitali

**Tre.**

1. **La tariffa non esiste**: 2→9 corpi è costato meno di 2→4 (WP-44), quindi
   il costo di «un corpo caldo in più» non è una costante e non può essere
   sottratto da una banda. Il verdetto «netto non distinguibile da zero» del
   passo 2 è **refutato nella sua derivazione** — la conclusione può restare
   vera, ma non è provata da quel conto.
2. **O1 non è la prima voce giusta**: la mia stessa consulenza scommetteva sul
   PROLOGO (O2), non sull'i-cache. Mettere O1 per prima significa scrivere la
   leva del ramo che non ho mai dato per favorito, senza il contatore che
   discrimina i tre rami. Prima il discriminatore (A-LB-98-2), che è anche il
   modo più economico di far ripartire il cronometro.
3. **«Per-sito, quindi ben predetto» è falso** (A-LB-98-5): confonde la
   staticità del flag con la biettività del branch. Il bit è quasi 50/50.

**Sul cronometro fermo**: sì, è un problema, e non per igiene. Il pin phpr è
invariato, quindi il profilo di WP-95 descrive ancora questo binario — ma
descrive R=1. Due sessioni hanno prodotto decisioni derivate da un numero senza
spread. Non serve una leva per rimediare: serve ripetere una misura che già
sappiamo fare.

---

# Verbale sedia 6 — Pedersen (WP-98)
Perimetro: confine per-richiesta/per-test, lifecycle, provenienza degli artefatti.
Oggetto: S-96.0 (apparato, riconteggio, verdetto passo 2) e §WP-97.

## VERDETTO
**RESPINTO IN PARTE.** Ho ri-sommato il raw in modo indipendente (16 righe):
`.sum` e `.out` combaciano, le sei derivate tornano alla seconda cifra. I
conteggi sono macchina-fedeli. Ma **fedeltà non è provenienza**: il
riconteggio vale come DIFFERENZA (i delta vs F2 condividono lo stesso difetto
e si cancellano), è **nullo come ASSOLUTO** — nessuna sua cifra è citabile in
futuro come autorità indipendente. Scriverlo nel raw invece di nasconderlo è
un progresso vero: fissa il GRADO, non lo eleva.

**I miei emendamenti WP-97 sono applicati a metà.** A-PP-97-1 chiedeva tre
cose: porcelain con **hash del diff** (dato invece un CONTEGGIO di righe — una
directory untracked vale 1 riga per N file: il «6» sottostima senza limite);
**re-hash del binario census a fine run** (fatto invece sul **pin di parità**,
che in questo run non è stato eseguito: controllo giusto, oggetto sbagliato);
trascrizione dall'identity (unica applicata). A-PP-97-2 (pid= + asserzione di
N): NON applicata — nessun `pid=`, il summer stampa `processes=16` e non
fallisce mai. A-PP-97-3 (suite per NOME): NON applicata — `762/1912/52
IDENTICO` è di nuovo parità di conteggi.

**Cronologia a macchina**: build 11:12:43 · identity 11:20:35 (**otto minuti
dopo la build**: `tree_dirty` descrive un albero che non è quello da cui il
binario è nato) · run →11:21:50 · amend 11:25:13 · `dc560d2` (liveness.rs)
11:32:42. Venti minuti di editing scoperti. La verifica che chiuderebbe tutto
— ricostruire da `dc560d2` e ritrovare `3e0e861c5fdbcb9b` — manca, ed è la
stessa che in WP-94 FALLÌ sul pin php-server: precedente che la rende
necessaria, non superflua.

**L'orfano**: `git branch --contains 7847cc0` è vuoto — vive nel solo reflog e
sparisce al primo `gc`. Il successore `83661e4` differisce per
`gate-measure-cifre.sh` (+13/−2), cioè **per il giudice**. La provenienza non
è solo indimostrata: è destinata a diventare **irrisolvibile**.

**Corpus senza identità**: l'oggetto misurato è binario × workload, e
`recount.identity` non porta impronta alcuna di `wpdev` (src, vendor, versione
WP, schema DB). La comparabilità F1/F2/recount poggia su un'ipotesi tacita.

**Concorrenza col selftest**: legittima in principio — la traccia di opcode di
un workload deterministico è invariante allo scheduling, i contatori non sono
tempi. NON legittima come eseguita: (a) i test media dipendono da filesystem,
DB e tempo, e l'unico controllo che escluderebbe uno scivolamento è l'identità
di suite, che è per conteggio (uno skip che scambia con un pass lascia 52
intatto); (b) i due processi condividono risorse MUTABILI non recintate — il
worktree che il run campiona come identità, il DB `wptests` che DROPPA, la
uploads che azzera. Non è rischio di tempi: è isolamento per-test.

## Emendamenti
- **A-PP-98-1** — identity completa: porcelain integrale + **sha del diff**,
  campionata dallo script di BUILD e ri-campionata a fine run; re-hash del
  binario **che ha girato**, non del pin.
- **A-PP-98-2** — nessun `.out` è autorità se non riproducibile: o la
  ricostruzione dal commit ridà la sha, o il file porta in testa
  `provenienza=NON-RIPRODUCIBILE, grado=interno`. Retroattivo a recount/f1/f2.
- **A-PP-98-3** — vietato registrare un HEAD non raggiungibile: `head=` si
  scrive dopo l'ultimo amend, o il commit è ancorato (`refs/measure/<run>`)
  prima del run. Amend che orfanizza un commit citato in un raw = FAIL.
- **A-PP-98-4** — identità del CORPUS nel raw (hash albero wpdev, versione WP,
  schema DB): senza, due run non sono confrontabili nemmeno come differenza.
- **A-PP-98-5** — lock esclusivo su worktree, `wptests`, uploads per l'intera
  finestra di misura. Il selftest del giudice non è carico neutro.
- **A-PP-98-6** — A-PP-97-2 e A-PP-97-3 **RI-EMESSE INVARIATE**: un
  emendamento non applicato non decade, si ripresenta finché non morde.
- **A-PP-98-7** — controllo dei riferimenti come DENTE, non come lezione: ogni
  `file §sezione` a manifest deve risolversi a HEAD, altrimenti FAIL; una
  regola di spareggio dichiara gli artefatti che arbitra con le loro sha ed è
  **DORMIENTE** se una cambia.
- **A-PP-98-8** — `sites_*` sommati su 16 processi non sono una popolazione di
  siti (sono siti × processi): rinominare `_per_proc_sum`.

## Kill-switch
- **KS-PP-98-1** — ricostruzione che non riproduce la sha ⇒ decadono tutte le
  conclusioni ASSOLUTE del run; restano le sole differenze intra-run.
- **KS-PP-98-2** — altro processo dentro il recinto ⇒ run INVALIDO, si ripete.
- **KS-PP-98-3** — «suite IDENTICA» per soli conteggi ⇒ la dichiarazione si
  cancella dal raw.
- **KS-PP-98-4** — leva chiusa su una regola DORMIENTE si RIAPRE d'ufficio.

## Refutazioni capitali
**SÌ, due.**
1. **La provenienza è indimostrata e ora irrisolvibile**: `head` campionato
   otto minuti dopo la build, `tree_dirty` che è un conteggio, commit
   orfanizzato da un amend sul giudice, nessuna ricostruzione.
2. **Il verdetto del passo 2 è NULLO, non contrario.** Uno spareggio con
   entrambe le premesse refutate non si inverte: **decade**. Dedurne «la
   strada lunga non vince» e chiudere `TakeSlot` usa come autorità la regola
   appena dichiarata falsa — e il §WP-97 lo ammette («se così fosse, il
   verdetto del passo 2 cambierebbe»). F3 va riclassificata **SOSPESA**, non
   archiviata: chiudere un fronte sul singolo passo di una regola morta viola
   la regola utente di non-chiusura.

---

# Verbale sedia 7 — Leijen (allocatore mimalloc, footprint fisico) — WP-98

Oggetto: S-96.0 (apparato + fix di soundness + confronto piano B) e §WP-97.

## VERDETTO

**NON REFUTATO nel merito; DUE refutazioni capitali sulle premesse con cui il
mio perimetro è stato interrogato.** La chiusura di A-ZV2 per verdetto invece
che per tempo è disciplina, non rinuncia. Ma l'atto d'accusa che mi viene
rivolto («il contatore `would_take_safe_str` è un numero di ALLOCAZIONI
evitate») è **falso a macchina**: `Zval::Str(Rc<PhpStr>)`
(`crates/php-types/src/zval.rs:22`) — un clone di stringa è un incremento di
refcount, **zero allocazioni**. I 9.989.963 sono elisioni di `Rc::clone`, cioè
CPU e linee di cache toccate, non byte chiesti all'allocatore. Chi legge quel
contatore «in chiave footprint» sbaglia di un canale intero, e sbaglierebbe in
buona fede: nessuno lo ha ancora fatto, ed è bene che nessuno cominci.

Il canale footprint di A-ZV2 esiste, ma è **un altro** e non è contato: un
valore che arriva a rc=1 fa sì che il write successivo passi per `Rc::make_mut`
**in place** invece di separare (CoW evitata) — quello sì alloca, ma solo se un
write segue, e il censimento conta SITI di lettura, non coppie lettura-scrittura.
`would_take_safe_str` è quindi un **maggiorante lasco di un canale il cui
contenuto in byte può benissimo essere zero**. Errore di perimetro: sì, ma non
quello contestato.

## Emendamenti

- **A-DL-98-1 (il falsificatore da 20 minuti che nessuno ha lanciato).** La leva
  arene per-file è ferma da tre rotazioni perché in coda c'è una COSTRUZIONE
  quando in coda dovrebbe esserci una MISURA che la può uccidere: `N = 25.795.552
  − T_max`, e **T_max non è mai stato misurato** (39.534.144 è capacità, non
  touched — sanatoria A12/M2). Un run strumentato per-unità dà T_max; se T_max è
  grande, N è piccolo e la leva muore **senza scriverla**. Questo, non la leva,
  va in testa alla coda.
- **A-DL-98-2 (il picco viaggia con la coppia, sempre).** Qualunque leva CPU del
  §WP-97 si giudichi con una coppia, la coppia registra ANCHE il peak fisico:
  costa zero (`/usr/bin/time -l` già gira) e senza di esso il footprint resta
  non misurato da m90 per *scelta implicita*. Predizione ex-ante firmata Δ≈0.
- **A-DL-98-3 (α resta da RI-DERIVARE, A-DL-72 invariato).** Sotto
  `MIMALLOC_PURGE_DELAY=0` mimalloc decommitta subito: l'argomento «pagine
  committed riusabili» è falso. 15 arene per-file = decommit→recommit→re-fault;
  con T_max si firma anche una predizione di `page reclaims`, o la leva paga in
  CPU ciò che incassa in footprint.
- **A-DL-98-4 (O1: predizione sul metro GIUSTO).** O1 va predetta, ma non dove
  mi si chiede: le pagine di testo sono file-backed e pulite, e **non sono
  addebitate a `phys_footprint`**. La predizione ex-ante è Δpeak ≈ 0; e il
  `.text` TOTALE da `nm -S` può **crescere** (prologhi/epiloghi non più fusi,
  sequenze di call) mentre il testo caldo residente cala — crescita che NON è un
  fallimento della leva.

## Kill-switch

- **KS-DL-98-1**: qualunque ricevuta che converta `would_take_safe_str` (o
  `_safe`, o `would_take`) in byte, allocazioni o footprint ⇒ **NULLA**.
- **KS-DL-98-2**: qualunque claim footprint di O1 letto su `phys_footprint` ⇒
  **VOID**; e `nm -S` e `max_rss` non si sommano né si confrontano fra loro.
- **KS-DL-98-3 (decadenza)**: se T_max non è misurato entro la prossima
  sessione, la leva arene per-file si dichiara **CHIUSA**, non rinviata. Una
  voce che nessuno esegue per quattro rotazioni non è una voce: è un alibi.

## Refutazioni capitali

**SÌ, due.**

1. **`would_take_safe_str` non è un numero di allocazioni** (Rc). L'accusa di
   «errore di perimetro» poggia su una premessa falsa; l'errore vero è
   simmetrico e opposto — il canale CoW, che il contatore non vede.
2. **La leva arene per-file non è una leva sull'OGGETTO del roadmap.** A-DL-71:
   vale 47% sul picco CLI hello, **1,8% sul media e 1,06% sul full**. Nessuna
   cifra del trend può riceverla né falsificarla. Ecco perché tutte le sedie la
   approvano nel merito e nessuno la esegue: paga su un oggetto che non è
   giudicato. O il picco CLI diventa un oggetto DICHIARATO con una sua colonna,
   o la leva va chiusa. Continuare a chiamarla «la leva del footprint» è un
   errore di categoria, e dura da tre rotazioni.

Sul §WP-97: **tre candidate CPU è corretto**, e va scritto perché. Dopo la
sanatoria WP-96 il regresso media footprint 3,381× è RITIRATO (era la gamba
oracle): **il footprint oggi non ha un difetto nominato**, quindi non ha una
candidata migliore. Non è una lacuna da coprire — è uno stato da verbalizzare.

---

# Verbale sedia 8 — Stogov (engine Zend / fedeltà oracle) — WP-98

## VERDETTO

S-96.0 è metodologicamente sana e la mia refutazione WP-97 è stata accolta nel
punto giusto (perimetro Str, non F2 intero). **Il verdetto del passo 2
sopravvive** — ma non per le ragioni scritte: `would_take_safe_str` non è un
perimetro fedele, è un **TETTO**. La lista `observes_scope` è indicizzata sul
nome **SCRITTO**, e in phpr il nome scritto non è il nome eseguito. Poiché il
perimetro vero è ≤ quello contato, la strada lunga esce ancora più debole: il
«non vince» regge, la cifra che l'ha deciso no. Nessuna banda derivata da questi
conteggi va citata come stima finché A-DS-98-1/2/3 non sono chiusi.

## Emendamenti

- **A-DS-98-1** — `Op::CallNsFallback { name, fallback, argc }` (bytecode.rs:516):
  `renounce()` interroga `observes_scope(name)` e **ignora `fallback`**.
  `namespace X; extract($a);` porta `name = X\extract` → nessuna rinuncia, e a
  runtime esegue `ho_extract` (vm/mod.rs:14879). Consultare ANCHE `fallback`,
  su entrambe le varianti Args. Fixture negativa obbligatoria.
- **A-DS-98-2** — nome risolto a runtime: `CallValue`, `CallValueArgs`,
  `CallNamed`, `CallSpread`, `MakeFcc` non sono name-checked, e `compact`/
  `extract`/`get_defined_vars` sono dispatchati per nome in un match runtime.
  O si rinuncia su questi op, o si **prova con una fixture** (non con un
  ragionamento) che `$f='extract'; $f($a);` non raggiunge lo scope del chiamante.
- **A-DS-98-3** — il canale ARGOMENTI (era A-DS-97-4, ancora fuori dai
  predicati) **non è chiudibile da una lista di nomi, per forma**: l'osservatore
  sta nella discendenza, non nel corpo. Canali: `getTrace()`/`getTraceAsString()`
  di QUALUNQUE Throwable costruito più in basso (default compilato
  `zend.exception_ignore_args=0` ⇒ gli args ci sono); `debug_backtrace()` da un
  callee, da un handler `set_error_handler`/`set_exception_handler`, da una tick
  function, da un callback `ob_start`, da un comparatore `usort`, da uno
  `spl_autoload_register`; **`ReflectionFiber::getTrace()`**, che osserva gli
  args di frame che non contengono `Fiber::suspend`. Rimedio: gli slot PARAMETRO
  escono dal take, oppure si prova che la trace materializza gli args
  indipendentemente dagli slot.
- **A-DS-98-4** — nomi. Mancano `func_num_args` (frame observer, innocuo qui:
  non legge valori) e va **verificato `$http_response_header`**: i wrapper http
  iniettano un locale per nome nello scope del chiamante da `file_get_contents`/
  `fopen`/`file`/`readfile` — un builtin che SCRIVE lo scope, in nessuna lista.
  Da NON aggiungere, con la ragione a verbale: `$errcontext` (rimosso in 8.0),
  `assert()` con stringa (non valuta più dall'8.0), `ReflectionFunction`,
  `Closure::bindTo` (cambia `$this`/scope, non i locali), `get_object_vars`.
- **A-DS-98-5** — `would_take_safe_str` è fedele **all'output**, non alla
  memoria: prendere lo slot invece di clonarlo abbassa il refcount del buffer e
  sposta il **momento della COW**; `memory_get_usage`, `memory_get_peak_usage`,
  `debug_zval_dump` sono osservatori dell'EFFETTO e nessuna rinuncia li copre né
  deve. Sede: il gate per NOME. Etichettare il campo, non allargare la lista.
- **A-DS-98-6** — §3.10: il perimetro si misura al **SITO**, non al builtin. Un
  contatore `(builtin, tipo in ingresso, strict_types del chiamante)` al punto
  unico di coercizione, dietro `zval-census`, R=1 sul media group. La domanda che
  decide non è «quanti builtin divergono» (5) ma «quante volte accade, e sotto
  `declare(strict_types=1)`» — dove PHP lancia sempre e phpr coercizza sempre.
- **A-DS-98-7** — A-ZV1 è la leva giusta **per forma** (è ciò che fa Zend:
  specializzazione per LOCAZIONE dell'operando, nessun corpo caldo nuovo). Quattro
  trappole: (1) fast path chiuso ai tipi che non rientrano nella VM — `__toString`/
  `__get` invalidano ogni riferimento preso da uno slot; (2) la conversione
  numerica **emette diagnostici**, che rientrano via `set_error_handler`: nessun
  borrow vivo attraverso il punto di emissione; (3) div-by-zero/overflow devono
  lasciare `ip`, `getLine` e la exc_table invariati — verificarlo; (4) prima della
  §Correzione, produrre il **conteggio esatto** della frazione di siti `Binary`
  con ENTRAMBI gli operandi letture di slot: senza, si confronta una predizione
  con una banda, l'errore di grado che design96 §5 ha appena denunciato.

## Kill-switch

- **KS-DS-98-1**: se la fixture di A-DS-98-1 o A-DS-98-2 morde, tutti i conteggi
  F1/F2 di S-95.0 e S-96.0 decadono a TETTO e nessuna banda derivata resta citabile.
- **KS-DS-98-2**: divergenza in `memory_get_usage` senza divergenza d'output ⇒ la
  classe di siti torna al clone. Mai correggere il contatore.
- **KS-DS-98-3**: A-ZV1 si chiude PRIMA di scrivere codice se la frazione
  slot-slot è sotto la predizione — è il criterio che è mancato ad A-ZV2.
- **KS-DS-98-4**: qualunque leva che aggiunga un braccio prima di O1 ⇒ stop
  (A-LB-97-1 insoddisfacibile per costruzione). O1 resta la sola voce che rimette
  in moto il cronometro senza discutere di perimetri: 241,7 KiB in una funzione
  contro ~192 KiB di L1i è il fatto più grosso ancora non attaccato.

## Refutazioni capitali

1. **«`observes_scope` è il perimetro»** — è indicizzata sul nome scritto;
   `CallNsFallback` porta il nome risolto in `fallback` e non lo consulta.
2. **«`would_take_safe_str` è il perimetro fedele»** — è fedele all'output; la
   COW spostata è osservabile.
3. **«§3.10 è una divergenza di testo»** — cambia il FLUSSO, e morde più forte
   sotto `strict_types`, cioè su vendor/, non su WordPress.

---

# Verbale 9 — GREGG (mandato INVERSO: giudico dall'OGGETTO, non dal rigore)

## VERDETTO

**L'oggetto NON è avanzato.** S-96.0 è una sessione ben condotta su un oggetto
che non ha toccato. Il cronometro è fermo da **due** sessioni; le misure di
TEMPO prodotte da questa sessione sono **zero**; le righe nuove nella colonna
CPU di GAP_TREND sono **zero**. Il rischio che presidio si è materializzato:
il passo 2 è l'apparato che giudica una leva mai costruita e la archivia con un
numero che non ha intervallo.

## §OGGETTO — i fatti nuovi, contati

Falsificabili, prodotti da S-96.0:

1. `would_take_safe_ref = 3307 / 25.826.594 safe = 0,0128%` — gli slot che a
   runtime reggono un `Ref` pur sopravvivendo alla rinuncia statica sono una
   frazione minuscola. **MOTORE** (comportamento dinamico misurato).
2. Divergenza §3.10: su `preg_match`, `preg_split`, `explode`, `substr`,
   `strtoupper` phpr coercizza con Warning dove PHP 8.5 solleva `TypeError`;
   cambia il FLUSSO. **MOTORE** — ed è **accidentale**: trovata costruendo una
   fixture, non cercata. È il fatto più durevole della sessione, ed è finito in
   coda a «Dopo, per NOME».
3. Delta F1 esattamente zero dopo il fix di soundness; la forma che espone il
   difetto non ricorre nel media group. **CORPUS/ANALISI**, non motore.
4. Il piano B assunto non esiste (riferimento pendente, «superistruzione»
   fantasma). **APPARATO documentale.**
5. `env -i` + lista chiusa, T27-T30, forgia T28 rotta dallo spazio nel path,
   `--no-filters`, untracked senza `--exclude-standard`. **APPARATO** (cinque
   fatti, tutti sul giudice).

**Bilancio: 2 sul motore (di cui 1 accidentale), 1 sul corpus, ≥6
sull'apparato. Misure di tempo: ZERO.**

## Conoscenza o rinuncia?

*Pro conoscenza*: la refutazione del piano B fantasma è verificabile e
permanente; «corretto per fortuna del corpus» è una distinzione vera; una
decisione negativa argomentata è conoscenza.

*Pro rinuncia*: il §4 di design96 scrive testualmente che «il netto non è
distinguibile da zero con quello che sappiamo oggi, e l'unico modo di saperlo è
misurarlo» — e poi chiude. Da «non lo sappiamo» segue «misuriamolo».

**Decido: rinuncia**, con un frammento di conoscenza vero (il piano B
fantasma). Il verdetto non è stato prodotto da dati sull'oggetto: è stato
prodotto dal confronto fra una banda SCREEN di oggi e un costo storico di
WP-33/WP-39..44 mai rimisurato.

## Il contatore

Dall'ultima coppia cronometrata (WP-94): **2 sessioni**. Ma WP-94 era la prima
dopo **otto**. Su undici sessioni, **una** coppia. Dall'ultima campagna
footprint (m90, WP-90): **6 sessioni**. **Non è accettabile.**

## Il §WP-97 rispetta la regola?

**Le tre candidate sono tutte e tre sull'oggetto** — questo va riconosciuto. Ma
**nessuna delle tre ha come esito un tempo**: la 1 esige una taglia `nm -S`
predetta, la 3 esige una sezione di documento come primo atto, e solo la 2 (O1)
è cronometrabile subito. La coda «per NOME» e il BACKLOG sono apparato quasi
puro. L'oggetto è nell'ordine; il cronometro no.

## Emendamenti

- **A-BG-98-1** — *braccio NULL*: costruire un braccio nuovo mai emesso e
  cronometrare la coppia adiacente stessa-sera. Restituisce il pedaggio reale
  sul binario `d5ce86e3`. Senza, §4 non è decidibile.
- **A-BG-98-2** — la riga ⏱ diventa VINCOLO: a 3 sessioni senza tempo, la
  successiva apre con una coppia.
- **A-BG-98-3** — **O1 per prima**, e il suo esito è un tempo.
- **A-BG-98-4** — grado del canale più debole **nel titolo** del verdetto.
- **A-BG-98-5** — misurare il perimetro §3.10 (5 nomi a mano non sono un
  perimetro).

## Kill-switch

- **KS-BG-98-1** — S-97.0 senza misura ⇒ S-98.0 non apre apparato, neanche in
  timebox.
- **KS-BG-98-2** — pedaggio del braccio null sotto risoluzione ⇒ §4 RITIRATO e
  passo 2 riaperto d'ufficio.
- **KS-BG-98-3** — terza sessione che deriva bande da §P1 R=1 ⇒ moltiplicatore
  DECLASSATO, bande dipendenti ritirate.

## Refutazioni capitali

- **RC-BG-98-1 (capitale)** — SCREEN × VERDICT = SCREEN. Il verdetto del passo
  2 non è verdict-grade e **non può chiudere un passo dell'ordine**: una
  decisione di rotta senza intervallo non è una decisione, è una preferenza.
- **RC-BG-98-2 (capitale)** — «dello stesso ordine di grandezza» fra un
  guadagno di oggi e un costo di WP-33 su un altro binario, altro compilatore,
  altro layout, **non è un confronto: è un'analogia**.
- **RC-BG-98-3** — «non lo sappiamo, quindi chiudiamo» è un non sequitur: il
  costo della misura non è mai stato stimato prima di dichiararlo proibitivo.
