# NEXT_SESSION_WORDPRESS.md — S-94.0: IL FALSIFICATORE CHE HA MORSO CHI LO SCRIVEVA → WP-95(sessione)

**Ultima sessione**: S-94.0 (2026-08-04) — ordine del Concilio WP-95
eseguito. **Apparato A1-A4**: i tre canali di Klabnik RIPRODOTTI a macchina
(ognuno firmava un `PASS --all` rc=0 col judge_sha pristino), chiusi con un
re-exec sanificante sul self FISICO il cui marker anti-loop è **lo stato
sanificato stesso**; falsificatore A-SK-91 = denti T24/T25/T26, **selftest
PASS rc=0**. **OGGETTO**: la **coppia full stessa-sera** è tornata a
esistere e la **battery61** è riproducibile sul modo nativo (criterio 5
SODDISFATTO). Dettaglio: `sessions/WP_SESSION_94.md`,
`wp94-harness/MEASURE94_RESULTS.md`, `gaps/REPORT_GAP_94.md`.

**⏱ FONDAMENTALI (regola utente 2026-08-03, aggiornare a OGNI rotazione)**:
ultima misura full/media = **WP-94 (QUESTA sessione — il contatore è
AZZERATO dopo otto sessioni)** · ultima campagna sull'oggetto = m90 in
WP-90 (4 sessioni fa). ⚠️ **Ma il Concilio WP-96 (Bak, Hoare, Gregg in
convergenza indipendente) ha stabilito che la coppia NON mostra movimento
di phpr**: la gamba phpr è PIATTA su ogni asse, dentro lo spread; il
«record» e il «regresso» della prima lettura erano la gamba ORACLE.
**Nove sessioni di roadmap footprint senza movimento misurabile su phpr**:
questo è il numero da guardare in faccia. Il guadagno vero di S-94.0 è il
METRO rimesso in funzione + la batteria WordPress nativa. **Restano non
misurati**: probe slope v2 e attribuzione dello slope (criterio 1
PARZIALE).

## Stato gate

- **phpr (CLI, parità release)**: **d5ce86e3342f3926 INVARIATO** — la
  coppia full E battery61 girano su questo pin. Corpus Zend per NOME 1418
  + refl 290 (non rimisurato in S-94.0: nessuna ricompilazione).
- **php-server**: **f8f4295a1dcdb627** (stash additivo `php-server-wp94`).
  ⚠️ **Il pin dichiarato d45b57843eeb1375 NON è riproducibile a HEAD** e
  **A4 è esclusa per differenza misurata** (stesso sha con e senza la
  patch). Voce APERTA: o il pin nacque da un albero diverso da quello
  dichiarato, o la build non è riproducibile byte-a-byte. Finché non è
  deciso, ogni claim che vi poggia è ADVISORY.
- **Gate cifre v3+A1**: `--all` **PASS a HEAD**; il budget in vigore vive
  in `wp81-harness/gate-cifre-corpus.budget`, MAI citato nelle prose
  (A-SK-77 ha morso per la terza volta). I tre canali WP-95 sono chiusi e
  provati (T24/T25/T26, rc esatti, ognuno col morso sul giudice pre-cura).
  ⚠️⚠️ **MA il gate è di NUOVO AGGIRATO da un canale NUOVO** (Concilio
  WP-96/Klabnik, riprodotto a macchina): le variabili d'ambiente di **git**
  — `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.excludesFile …` produce un
  `PASS --all` rc=0 firmato col judge_sha pristino mentre un doc con cifre
  inventate sta nel perimetro; e un clean filter iniettato per env sconfigge
  anche il self-tether A-SK-78. **Classe del difetto**: la sanificazione
  SOTTRAE variabili note (`env -u`), ma l'insieme delle variabili pericolose
  non è enumerabile. La cura è COSTRUIRE l'ambiente (`env -i` + allowlist
  chiusa), non sottrarre — **prima voce di apparato di S-95.0**
  (A-SK-93..97, denti T27-T30).
- **Misure**: coppia full S-94.0 — le CIFRE sono verdict-grade (raw
  `pair94.out`, rapporti macchina `pair94-ratios.out`), ma **ogni confronto
  con le bande storiche è RITIRATO** (sanatoria Concilio WP-96: la ricetta
  storica divide per un oracle CONGELATO a 5:39, e con quel denominatore lo
  stesso numeratore da un rapporto tutt'altro (valore in `pair94-ratios.out`); sul media il rapporto peggiora perché l'oracle è sceso). Valgono
  solo i rapporti **same-evening**. Slope per-worker: nessuna cifra nuova.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (nuove da S-94.0)

**Un privilegio che vale per il processo non vale per la sua discendenza**:
se il giudice delega a sottoprocessi, si sanifica l'AMBIENTE CONSEGNATO,
non la shell (`bash -p` non rimuove `BASH_FUNC_*`). **Un dente che smette
di mordere non lo annuncia**: un range che si apre su un pattern presente
anche nella riga che lo usa tronca in silenzio, e il dente diventa vacuo,
non fallito — solo l'rc ESATTO lo rivela. **Un predicato non deve dipendere
da ciò che esso stesso introduce**. **Un confronto identico non è valido se
entrambi i lati stanno fallendo.** **Il rc del runner non è il giudice di
una coppia** (la suite WordPress fallisce anche sull'oracle).

## ⚖️ Concilio WP-96 ESEGUITO (2026-08-04, verbali VINCOLANTI): `wp96-harness/COUNCIL_WP96_REVIEWS.md`

**9 sedie, NESSUNA benedizione. TRE refutazioni capitali riprodotte a
macchina**: (1) le letture comparative erano artefatti del denominatore
(Bak+Hoare+Gregg, convergenza indipendente) → **sanatoria APPLICATA in
chiusura**; (2) **il gate è di NUOVO aggirato** dalle env di git
(Klabnik); (3) A-AH-71 era forma e non origine (Hejlsberg, misurato) →
A-AH-76/77 **applicati in chiusura**. Leijen è CONTRARIO al grado VERDICT
sul footprint (picco R=1 = SCREEN; α da ri-derivare: l'albero è mimalloc
v3.0.2 e sotto PURGE_DELAY=0 decommitta). Sintesi §FONDAMENTALI + ordine
in `wp96-harness/SYNTHESIS_WP96.md`.

## §WP-95(sessione) — la CPU della VM, con la mappa in mano

**Riscritto il 2026-08-04 dopo la ricognizione di profiling** (decisione
utente: «misurare la lentezza l'abbiamo compreso, ora serve invertire la
rotta e migliorare le prestazioni»). L'ordine del Concilio WP-96 su
denominatore/leva del preludio resta valido ma **scende di priorita**: la
misura ha mostrato dove sta davvero il costo.

**P0**: pre-flight standard + `--all` PASS a HEAD + pin phpr
d5ce86e3342f3926 invariato.

### L'OGGETTO: A-ZV2 fase F1 (analisi di ultimo uso, SOLA MISURA)

Contratto completo in `wp95-harness/design95-liveness.md` (predizione
firmata). In breve:

1. **F1 — calcolare l'analisi e CONTARLA, senza usarla.** Ultimo uso per
   slot su ogni funzione compilata; contatore `would_take` dietro la feature
   `zval-census`. **Rischio zero**: nessun bit del binario di parita cambia.
   **Criterio di prosecuzione scritto PRIMA: se le letture spostabili sono
   < 20% di `slot_reads_rc` (=53561241, misurato in
   `wp95-harness/zvalcensus-before.out`), la leva NON vale la sua
   complessita e si passa al PIANO B** (superistruzione LoadSlot+Binary,
   `design95-leva-zval.md` §Correzione).
2. **F2 — il perimetro conservativo** (compact/extract/get_defined_vars/
   variabili variabili/eval/closure by-ref/generatori/Ref/try-finally e
   soprattutto i DISTRUTTORI: spostare un valore ANTICIPA un `__destruct`,
   che e osservabile e non fallisce in modo rumoroso). Ri-contare: se la
   prudenza taglia piu del 40%, fermarsi.
3. **F3 — l'opcode `TakeSlot`** con gate di parita COMPLETI nello stesso
   commit + i test delle trappole (`$a .= $a`, distruttore che osserva
   l'ordine, generatore sospeso, `compact()` dopo l'ultimo uso apparente).
4. **F4 — la misura**: coppia oracle-vs-phpr stessa sera con l'oracle
   RIMISURATO (mai il denominatore congelato) + coppia A/A per lo spread.

### Il contesto che rende questo l'ordine giusto (ricognizione 2026-08-04)

- Profilo del workload reale: `wp95-harness/prof95-media.out`. Il 49,4% del
  wall e ATTESA (il master dorme su `ChildStderr::read`); la CPU vera e
  nella VM.
- **Il tetto del dispatch e 4,33% della CPU** (~2,2% di wall): azzerarlo del
  tutto varrebbe meno di quanto valgono `Zval` clone+drop (10,05%) e il
  ciclo di vita dei Frame. Consulenze in `consulenza-bak-dispatch.md` e
  `consulenza-stogov-engine.md`: **due scuole diverse, stessa conclusione —
  non il dispatch.**
- L'ipotesi «run_loop troppo grande per la i-cache» e **REFUTATA per
  misura**: 241,7 KiB totali ma working set caldo 27,6 KiB (7,8 KiB per il
  90% del tempo), L1i di questo M4 = 128 KiB.

### Dopo A-ZV2, per NOME (non «piu avanti»)

Denominatore omogeneo in GAP_TREND (KS-BG-96-3, era P0 del Concilio WP-96) ·
leva arene per-file del preludio con α RI-DERIVATO (Leijen: l'albero e
mimalloc v3.0.2 e sotto PURGE_DELAY=0 decommitta) · probe slope v2 fuso ·
attribuzione dello slope · il pin php-server che non torna.

### BACKLOG PER NOME (non «più avanti»)

Il «regresso» del media footprint è **RITIRATO** (era la gamba oracle) ·
A-AH-78/79 (writer ancorato al commit, BREV fail-closed) · A-MS-65/66
(eprintln! panicante nel GlobalAlloc, drop-guard sul flag di rientranza) ·
A-DS-96-1/2/3 (registry unica dei wrapper: `is_builtin_scheme` ne rivendica
12 mentre la lista ne dichiara 5 — incoerenza fra tabelle) · A-PP-83
(battery61 non resetta lo stato fra le due gambe) · A-SK-92-PROBE (grado rc=65) ·
A-AH-70/74/75 (ancore ledger) · A-AH-73 (HIR plain-data, precond. leva
#2) · audit A-BG-72 (derivate m90 che consumarono malloc_huge come
retained) · debito WP-94 non-A (ancoraggi campo, perimetro root, sigilli
E1→E3, checker LSP D1→D4) · residui A-DS51 fasi 2-3 ·
`stream_get_wrappers` incompleto (divergenza confermata dalla full).

### Criteri di CHIUSURA del fronte Axum/php-server

1. Slope attribuito per NOME — **PARZIALE** (invariato). 2. Leva applicata
e misurata o refutazione motivata — **la leva per-file è ora eseguibile**.
3. Parità intatta + ricevuta pin — **APERTA** (il pin php-server non
torna; e Pedersen osserva che anche il pin phpr è uno sha di CONTENUTO
senza provenienza: la stessa malattia non è esclusa). 4. Apparato
CONGELATO fuori dalla quota. 5. Batteria WordPress riproducibile —
**SODDISFATTO con riserva**: Hoare e Stogov chiedono un predicato POSITIVO
(la batteria passò anche con login fallito su entrambi i lati) e il
reset di stato fra le gambe; senza bite-test il criterio 5 torna PARZIALE
(KS-DS-96-3).

**NON riproporre**: tutti i NON-riproporre WP-83..95 restano; in più —
«sanificare la shell e credere sanificati i figli»; «un dente che non
fallisce sta mordendo» (può essere vacuo: pretendere l'rc esatto);
«un predicato che dipende da ciò che introduce»; «due lati identici =
confronto valido» (possono fallire entrambi); «il rc del runner come
giudice di una coppia»; «citare il budget corpus in una prosa»;
**«leggere un miglioramento in un rapporto»** — un rapporto peggiora anche
quando il denominatore migliora: il claim si fa sulla GAMBA, mai sulla
frazione (Bak/Hoare/Gregg, WP-96); **«provare l'esistenza dal vuoto di una
pipe»** — sha256 del vuoto è e3b0c44298fc1c14 e la guardia non scatta mai
(Hejlsberg, misurato).

---
**Chiusura**: 2026-08-04. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
