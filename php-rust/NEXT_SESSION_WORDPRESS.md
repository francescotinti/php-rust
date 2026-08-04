# NEXT_SESSION_WORDPRESS.md — S-96.0: A-ZV2 SOSPESA (non chiusa) → WP-97(sessione)

**Ultima sessione**: S-96.0 (2026-08-04, sera) — **l'ordine del Concilio WP-97
eseguito nei suoi passi, e chiuso da un verdetto invece che dal tempo**. Passo 0:
apparato A-SK-93..97 (`env -i` + lista CHIUSA, denti T27-T30, **SELFTEST PASS
rc=0**). Passo 1: fix di soundness A-TH-97-1 + match esaustivi + varianti
mancanti + contatore `would_take_safe_ref`, poi RICONTEGGIO — P2 soddisfatta,
bande invariate, e **i delta F1 esattamente ZERO**: il difetto è reale (provato
a macchina dalla fixture `t4-first-op-def.php`) ma la forma che lo espone non
ricorre in questo corpus. Passo 2: il confronto col piano B — **la strada lunga
NON vince sul perimetro fedele**, quindi `TakeSlot` non e' stato scritto e F4 non
è applicabile. Dettaglio: `sessions/WP_SESSION_96.md`.

**⏱ FONDAMENTALI (regola utente 2026-08-03, aggiornare a OGNI rotazione)**:
ultima misura full/media = **WP-94 (2 sessioni fa)** · ultima campagna
sull'oggetto footprint = m90 in WP-90 (6 sessioni fa). S-96.0 non ha
cronometrato: ha contato (conteggi esatti) e ha DECISO di non costruire la leva.
**Il cronometro è fermo da due sessioni e la rotta CPU-VM ha appena perso il suo
prossimo passo: la prima voce del §WP-97 deve essere una leva sull'OGGETTO, non
apparato.**

## Stato gate

- **phpr (CLI, parità release)**: **d5ce86e3342f3926 INVARIATO** (tutto il
  lavoro A-ZV2 vive dietro la feature `zval-census`; nessun ri-stash). Corpus
  Zend per NOME 1418 + refl 290 (non rimisurato: nessun cambio al binario —
  valido per costruzione).
- **php-server**: **f8f4295a1dcdb627** (invariato, non toccato in S-96.0).
  ⚠️ Il pin storico d45b57843eeb1375 resta NON riproducibile — voce APERTA.
- **Gate cifre v3+A1+A-SK-93..97**: `--all` **PASS a HEAD** dopo ogni commit
  (budget alzato con delibera nello stesso commit del nuovo raw). **Il canale
  env di git è CHIUSO**: `SELFTEST PASS rc=0` con T27-T30, ciascuno col proprio
  morso sul giudice pre-cura. **KS-SK-97-1 è soddisfatta**: i PASS di parità
  futuri sono verdict-grade.
- Build di strumentazione: post-fix `3e0e861c5fdbcb9b`
  (`phpr-census-target/`), pre-fix `e318fbfc248a8e35` (`phpr-pre-target/`;
  ricetta per ricostruirlo nella testata di
  `wp96-harness/check-liveness-fixtures.sh`).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM 3E/13F ·
  hk 1665).

## Permanent Binding Rules (invariate, più una)

**Un privilegio che vale per il processo non vale per la sua discendenza.**
**Un dente che smette di mordere non lo annuncia** (pretendere l'rc ESATTO).
**Un predicato non deve dipendere da ciò che esso stesso introduce.**
**Un confronto identico non è valido se entrambi i lati stanno fallendo.**
**Il rc del runner non è il giudice di una coppia.**
**NUOVA (S-96.0) — una cura ENUMERABILE contro un attacco NON enumerabile è
vacua per costruzione**: l'ambiente di un giudice si COSTRUISCE (lista chiusa),
non si sottrae (lista di negazione).

## ⚖️ Concilio WP-98 ESEGUITO (2026-08-04, verbali VINCOLANTI): `wp98-harness/COUNCIL_WP98_REVIEWS.md`

9 sedie, protocollo due fasi (team: oggetto, analisi, catena), NESSUNA
benedizione. **§FONDAMENTALI (Gregg, mandato inverso): l'OGGETTO NON è
avanzato** — due soli fatti sul motore (di cui uno, §3.10, trovato per caso)
contro sei sull'apparato, e zero misure di tempo. **Otto refutazioni capitali**,
tre già APPLICATE in sessione: (1) Klabnik FORGIA ATTERRATA — il perimetro era
cieco ai nomi non-ASCII, perché git li QUOTA e la virgoletta rompe l'ancoraggio
[APPLICATA: `-z`+`core.quotePath=false` su `ls-files`, `check-ignore` e
`ls-tree`, dente T31 col morso]; (2) Hoare — il raw contiene un delta che il
changeset non può produrre, quindi le attribuzioni sono state scritte senza
controllare il pavimento di rumore [APPLICATA: annotata nel raw]; (3) Bak — il
tetto sui corpi caldi usato come TARIFFA, ma in WP-44 passare da 2 a 9 corpi
costò MENO che passare da 2 a 4 ⇒ la chiusura del passo 2 è declassata a
SOSPENSIONE; (4) Matsakis — `current_frame_args` legge gli slot vivi di OGNI
frame, canale che il test di tipo non vede; (5) Stogov — `namespace X;
extract($a)` sfugge alla rinuncia (provato a macchina): `observes_scope` è
indicizzata sul nome SCRITTO; (6) Leijen — `would_take_safe_str` conta
`Rc::clone` elisi, ZERO allocazioni: nessuna lettura in chiave footprint;
(7) Pedersen — **P-AMEND-ORFANO**: un artefatto registra `head=`, poi un
`--amend` sostituisce l'oggetto e al primo `gc` la provenienza diventa
IRRISOLVIBILE; (8) Hejlsberg — `LoadSlot{take}` è la peggiore delle due forme
(layout neutro, e tasserebbe TUTTE le letture di slot per servirne la sola quota `would_take_safe_str`).
Sintesi + ordine in `wp98-harness/verbali/SYNTHESIS.md`.

## §WP-97(sessione) — RIMETTERE IN MOTO IL CRONOMETRO (ordine del Concilio WP-98)

L'ordine qui sotto SOSTITUISCE le tre candidate che questa sessione aveva
proposto: il concilio le ha riordinate e ne ha refutata la motivazione (il
punto «forma dell'emissione» era primo, ora è quarto e la sua motivazione
scritta è falsa).

**P0**: pre-flight standard + `--all` PASS a HEAD + pin phpr invariato.
⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA da rust-analyzer (rimossa DUE
volte in S-96.0, la seconda dopo poche ore) e il volume locale sta al limite dei
15G: rimuoverla è parte del pre-flight, non un'eccezione. La taglia si misura
sul momento con `du -sh`, non si cita a memoria.

### Il fatto da cui partire

A-ZV2 non è archiviata: è **SOSPESA** (declassamento imposto da Bak — la
derivazione che la chiudeva usa il tetto sui corpi caldi come una tariffa, e
non lo è). Il problema che la sessione ha davvero portato alla luce non è
quale leva scegliere: è che **la rotta CPU-VM decide su un denominatore che
nessuno ha mai rimisurato**, e che il cronometro è fermo da due sessioni.

1. **Ri-profilo R≥3, stesso workload, ZERO cambi di codice.** Mispredict
   indiretti/op e L1I-miss/op normalizzati su `op-census`, peak registrato
   insieme (costo zero, Leijen), regola di lettura scritta PRIMA, mediana **e
   spread** pubblicati — un intervallo, non un punto. È l'unica voce che serve
   i tre mandati insieme: rimette in moto il cronometro, ripara il
   denominatore da cui dipendono TUTTE le bande di due sessioni, e dice se O1
   ha un canale prima di scriverla. *Finché manca l'intervallo vale Gregg: una
   decisione senza intervallo non è una decisione, è una preferenza.*
2. **O1 di Bak — outlining dei bracci freddi** (i ~140 opcode rari
   `#[inline(never)]`, restano inline i ~40 caldi), con **controllo positivo
   DOPPIO**: taglia predetta + outlineati ∩ `op-census` da un lato; L1I-miss/op
   in calo con `op-census` INVARIANTE dall'altro. Coppia stessa-sera. È
   l'unica leva che ABBASSA il numero di corpi caldi, quindi il prerequisito
   di qualunque leva che ne aggiunga uno.
3. **Braccio NULL cronometrato, nelle DUE forme**: dentro `run_loop` (Gregg,
   per leggere il pedaggio reale) e come Δ di taglia equivalente FUORI da
   `run_loop` (Bak, per separare i-cache da dispatch). Sono due esperimenti
   diversi ed entrambi necessari: è ciò che rende decidibile il conto del
   passo 2 di S-96.0.
4. **La forma dell'emissione — SOLO DOPO che l'entropia del bit è misurata.**
   La motivazione scritta in S-96.0 («per-sito, quindi ben predetto») è
   REFUTATA da Bak: il bit sarebbe preso il 42,33% delle volte. E Hejlsberg
   aggiunge che il layout è neutro, quindi il flag non compra nulla e tasserebbe
   tutte le letture di slot per servirne la sola quota `would_take_safe_str`. Klabnik dissente sul GRADO (è un
   argomento, non una misura): il conflitto è registrato.
5. **P-AMEND-ORFANO** (unica voce d'apparato ammessa, perché rende
   IRRISOLVIBILE la provenienza di ogni misura FUTURA): `refs/measure/<run>`
   piantata prima del run, oppure `head=` scritto DOPO l'ultimo amend.
6. **Footprint**: nessuna leva nominata. Il falsificatore T_max di Leijen in
   timebox; se non è misurato entro la sessione, **la leva arene si dichiara
   CHIUSA** invece di slittare per la quarta volta.

**Vincolo di grado**: il moltiplicatore del canale (§P1) viene da un profilo
R=1 ed è SCREEN. Qualunque banda derivata da lì eredita quel grado, comprese
quelle di `design96-confronto-piano-b.md`. La voce 1 esiste per togliere
questo vincolo, non per aggirarlo.

**Debito ISCRITTO qui perché non evapori** (Hejlsberg A-AH-98-3 + team-catena):
l'analisi di liveness usa oggi una cache con chiave per PUNTATORE
(`zvalcensus.rs`), accettabile in sola misura e VIETATA in emissione; serve
identità STRUTTURALE. E i buchi di soundness che il concilio ha nominato e che
nessuna fixture prova ancora — canale cross-frame `current_frame_args`
(Matsakis, Stogov), fallback di namespace su `extract` (Stogov, provato a
macchina), arco di ri-lancio `EndFinally` (Hoare) — vanno chiusi PRIMA di
qualunque emissione, non insieme.

### Dopo, per NOME (non «più avanti»)

Denominatore omogeneo in GAP_TREND (KS-BG-96-3) · leva arene per-file del
preludio con α RI-DERIVATO (Leijen) · probe slope v2 fuso · attribuzione dello
slope · il pin php-server che non torna · **divergenza §3.10 (argomenti
`string` dei builtin: coercizione con warning invece di `TypeError`) — il
perimetro NON è misurato, e misurarlo è il primo passo**.

### BACKLOG PER NOME (invariato, più le voci nuove)

A-AH-78/79 · A-MS-65/66 · A-DS-96-1/2/3 (registry wrapper) · A-PP-83
(battery61 senza reset fra le gambe) · A-SK-92-PROBE · A-AH-70/74/75 · A-AH-73
· audit A-BG-72 · debito WP-94 non-A (ancoraggi campo, perimetro root, sigilli
E1→E3, checker LSP D1→D4) · residui A-DS51 fasi 2-3 · `stream_get_wrappers`
incompleto · gc_note_frame bitmask per-funzione (Stogov §3) · hash cachato in
PhpStr (Stogov §4).

### Criteri di CHIUSURA del fronte Axum/php-server (invariati)

1. Slope attribuito per NOME — PARZIALE. 2. Leva per-file eseguibile.
3. Parità + ricevuta pin — APERTA (pin php-server). 4. Apparato CONGELATO
fuori quota. 5. Batteria riproducibile — SODDISFATTO con riserva.

**NON riproporre**: tutti i NON-riproporre WP-83..95 restano; in più —
**«la strada lunga non aggiunge opcode al percorso caldo»** (falso: `TakeSlot`
è un braccio nuovo, Bak A-LB-97-1); **«il piano B è la superistruzione
`LoadSlot+Binary`»** (il riferimento a `design95-leva-zval.md` §Correzione è
PENDENTE: quella sezione non esiste, il piano B su disco è A-ZV1 e non è una
superistruzione); **«il perimetro F2 intero»** come base di un F3 fedele
(refutato da Stogov); **«sanificare l'ambiente togliendo le variabili che
conosciamo»** (lista di negazione = vacua per costruzione); **«una fixture che
non morde prova che il difetto non c'è»** (il controesempio di Hoare è vero e
non morde nella sua forma letterale).

---
**Chiusura**: 2026-08-04. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
