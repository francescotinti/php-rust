# Verbale sedia 2 — MATSAKIS (ownership/aliasing; arbitri e mutation-testing) — Concilio WP-106 su S-104

## VERDETTO: APPROVO CON EMENDAMENTI (nessuna refutazione capitale)

Il metodo KS-MA-105-1 ha funzionato esattamente come progettato: il mutante
Str→forget su fx20 non ha confermato l'arbitro — l'ha **smontato**, scoprendo
che `memory_get_usage` di phpr è uno stub costante e che il verdetto in-script
era vacuo. Il ridisegno col braccio RSS e il rosso archiviato (301 nei DUE
modi) è esemplare. Sulla riapertura di RC-MA-104 per 19a/19b: la demozione
fail-closed è CORRETTA, ma la sua motivazione è sovra-estesa (R-MA-106-1).

## R-MA-106-n (refutazioni)

- **R-MA-106-1 — Due perturbazioni bastano a DEMOTERE, non a CONDANNARE.**
  Le due mutazioni (nota saltata nel dispose; soglia 50000→50001) sono
  specchi del rischio della LEVA H-C2, non dell'osservabile di 19a/19b —
  che è il conteggio holder-esterno mid-arm (OBS-8, `count − 2 > in_edges`,
  il −1 del MOVE). S-104 stesso lo ammette («la soglia±1 non sposta il loro
  osservabile»): perturbazioni mal accoppiate all'arbitro. Conclusione
  legittima: 19a/19b NON guardano H-C2 e non contano come arbitri (nessun
  rosso ⇒ fail-closed, RC-MA-104 aperto). Conclusione ILLEGITTIMA: «non
  arbitrano il meccanismo» tout court — per condannarle o riabilitarle serve
  la TERZA mutazione, mirata al LORO osservabile (es. −2→−1 nel confronto
  OBS-8, o non contare l'handle del braccio come holder): se le fa rosse,
  restano arbitri del MOVE (loro scopo originario), mai di H-C2.
- **R-MA-106-2 — I mutanti sopravvissuti sono privi di disposizione.** «La
  sweep di fine statement compensa la nota saltata» è un'ipotesi, non una
  classificazione verificata (mutante-equivalente vs coverage-gap). E
  inquieta: 19a/19b girano il collector MID-statement, PRIMA della sweep —
  se la nota saltata non morde nemmeno lì, o la nota è difesa morta (canale
  ridondante da nominare) o le fixture non raggiungono il suo sentiero.
  Un mutante sopravvissuto si disposiziona, non si archivia come tally.
- **R-MA-106-3 — Il cap 150 arbitra solo il difetto capitale.** Derivato da
  N=1 per braccio (clean «~<60», mutante 301), tarato sul mutante che perde
  TUTTE le ~1M stringhe (+250 MiB). Un leak parziale — un solo sentiero dei
  due (Pop vs overwrite), ~+25-30 MiB ⇒ RSS ~80 — PASSA sotto il cap. La
  portabilità regge su un VOID accidentale non documentato (`/usr/bin/time
  -l` non-macOS ⇒ rss vuoto ⇒ VOID; awk assume ru_maxrss in BYTE, vero solo
  su macOS). Fail-closed per fortuna, non per progetto.

## A-MA-106-n (emendamenti)

- **A-MA-106-1**: terza mutazione su 19a/19b mirata al conteggio
  holder-esterno OBS-8, rosso archiviato; ALTRIMENTI riclassificarle per
  NOME nell'header di `s103-recv-fixtures.sh` come «regressione byte-parity,
  NON arbitro di meccanismo» — mai più citate come guardia di una leva.
- **A-MA-106-2**: tabella di disposizione dei mutanti sopravvissuti
  (equivalente / coverage-gap / ridondanza-da-nominare) accanto ai rossi in
  `denti-rossi/`; la voce «sweep compensa» va verificata o degradata a
  ipotesi.
- **A-MA-106-3**: il cap è una BANDA — derivazione pre-registrata (mediana
  clean R≥3 + rumore; floor del mutante) + guardia d'erosione (clean ≥
  cap/2 ⇒ VOID, mai PASS silenzioso) + mutante di sensibilità parziale
  (leak del SOLO sentiero Pop deve scattare) + nota bytes/macOS nel gate.

## KS-MA-106-n (vincolanti)

- **KS-MA-106-1**: nessuna fixture nei set pinnati può fondare il verdetto
  su `memory_get_usage` finché resta stub: il gate lo verifica (grep sulla
  fixture ⇒ VOID) o lo stub diventa contatore vero (backlog per NOME).
- **KS-MA-106-2**: un rosso di mutation-check è valido solo se il mutante
  perturba l'OSSERVABILE dichiarato dell'arbitro; mutanti-specchio della
  leva demotano, non condannano né promuovono.

## Priorità S-105

1. Terza mutazione 19a/19b (~30′): decide ritiro-per-NOME o riabilitazione.
2. Disposizione dei due sopravvissuti (A-MA-106-2).
3. Braccio RSS: derivazione cap + guardia erosione + mutante parziale.
4. Backlog per NOME: `memory_get_usage` contatore vero (KS-MA-106-1).
