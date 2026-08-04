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
  storica divide per un oracle CONGELATO a 5:39 → 838,59/339 = 2,474, non
  1,873; sul media il rapporto peggiora perché l'oracle è sceso). Valgono
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

## §WP-95(sessione) — la leva, finalmente con un giudice

**P0**: verificare `--all` PASS a HEAD e che il pin phpr sia ancora
d5ce86e3342f3926 (la coppia full di S-94.0 è la baseline della leva: se il
binario cambia prima della misura, la predizione perde il suo «prima»).

### L'OGGETTO (il corpo della sessione)

0. **PRIMA DELLA LEVA (KS-BG-96-3 + A-TH-76/A-BB-67)**: rendere OMOGENEO
   il denominatore del trend — pubblicare in GAP_TREND le quattro cifre
   ASSOLUTE per gamba e il Δ sulla **gamba phpr**, mai sulla frazione. Un
   claim sul numeratore si fa sulla gamba. Senza questo, la leva verrebbe
   giudicata di nuovo da una frazione.
1. **LEVA arene PER-FILE del preludio** — il «prima» fresco esiste ed è di
   S-94.0, sullo stesso pin. I 16 obblighi del
   team-leva (`wp95-harness/verbali/team-leva.md` §5): contatore per-unità
   parse-only col controllo positivo `Σ T_i ≈ 25795552 B` (touched reale
   ±10%), dente sulla semantica bumpalo, fixture F1-F8 oracle-morse,
   sentinella `b"prelude"` migrata in TUTTI i ~20 siti (mai a metà),
   **predizione ex-ante firmata** nella banda fissata dal team-leva (la
   banda e la sua condizione di falsificazione vivono in `team-leva.md`,
   non qui), gate parità COMPLETI + ricertificazione baseline phpr NELLO
   STESSO commit.
2. **Probe slope v2 FUSO** (slittato da S-94.0, invariato nel disegno):
   MI_STAT=1 dichiarato nel banner (mai TRACE nello stesso run) + coppia
   alloc/free in-band nel GlobalAlloc (soglia ≥524288) + eco d'arm
   `fired==W` (raw senza ⇒ VOID) + R≥5 interleaved W∈{1,2,4} mediana±2se +
   doppia metrica peak+residency + `huge_note` simmetrico su realloc.
3. **Attribuzione slope ~18,8 MB/worker per NOME** — SOLO dal probe fisico
   on-thread (A-DL-55), MAI da committed (A-DL-66). Criterio 1 del fronte.
4. **Il pin php-server che non torna**: decidere fra le due ipotesi con una
   misura (rebuild ripetuto a parità di albero → riproducibilità; oppure
   ricostruzione dell'albero storico → provenienza).

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
