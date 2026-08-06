# Verbale Sedia 6 — Pedersen (confine per-richiesta, lifecycle, server) — Concilio WP-104

## VERDETTO

**S-102: AMMESSA.** Il debito NON condizionato (A-PE-103-1) è stato saldato come
primo atto (09:03, prima di ogni commit): 2c4242b6 GRADATO al minimo con la
lettera intera di A-PE-103-2 — sentinella estesa bimodale con mode-probe, dente
cb1 su workers=1 servito 3×, byte-id inter-richiesta, corpo identico all'oracle,
mode-probe dedicato sul dump dell'unità (KS-PE-103-3 rispettato in entrambi i
bracci), cross-mode byte-id, fail-closed sull'hash. Riga aggiunta al registro
col grado esplicito. KS-PE-103-1: NON violato (la grazia precede ogni build; il
49a91e4d è nato dopo, senza feature, e non è stato usato né citato con cifre).
KS-PE-103-2: non innescato (fails=0×2). Nessuna cifra server attribuita.

**Programma S-103, punto 1: AMMESSO CON EMENDAMENTI.** La dottrina è propagata
correttamente (runtime cambiato in S-102 ⇒ pin nuovo con ricetta + collaudo =
primo atto, non condizionato). Ma lo STRUMENTO prescritto è quello di S-102
invariato, e quello strumento è cieco proprio al pezzo che motiva il pin nuovo.

## Refutazione capitale

**RC-PE-104-1 — «PIN_SRV_ATTESO aggiornato + s102-collaudo-server.sh com'è» NON
è il collaudo del pin nuovo.** Il pin nuovo imbarca il fix §3.13:
`diag_line_marks` cambia QUALE riga i warning stampano, e `flush_diags` è
esattamente il genere di flush che al confine per-richiesta può cadere dopo la
chiusura della finestra di capture. cb1.php è deterministico e «nessun warning
atteso» PER DESIGN: nessun braccio del collaudo minimo osserva un warning
servito. Gradare il pin §3.13 con un dente warning-free ripete ciò che il
corpus CLI già prova — la stessa vacuità che A-PE-103-2 nasceva per evitare.

## Punti deboli del dente cb1 (registrati, non refutanti per S-102)

1. **Cross-fixture assente**: workers=1 garantisce lo stesso worker, ma cb1 è
   l'UNICA unità che quel worker serve. Il caso «worker che ha già servito
   ALTRO poi serve la fixture coi distruttori» (residuo RetainSet/unit-cache
   che sposta il tick di un DTOR) non è esercitato: la sentinella interleaved
   usa fixture che non emettono al teardown. La combinazione è scoperta.
2. Il braccio D (static reset) e B2 mordono davvero; B1 (marker) è ridondante
   rispetto a B3 (byte-id oracle) ma innocuo.

## Emendamenti

- **A-PE-104-1**: il collaudo del pin NUOVO (primo atto S-103) = launcher S-102
  + **braccio warning-line cb2**: fixture che emette il warning §3.13
  (lettura di proprietà undef) sia nel corpo sia in `__destruct`/shutdown,
  byte-id vs oracle `php -S` nei 2 modi — righe dei warning comprese.
- **A-PE-104-2**: braccio **interleaving cross-fixture**: su workers=1,
  alternare cb1 con un endpoint della sentinella (E-cb1-E-cb1-cb1); le
  risposte cb byte-id in OGNI posizione.
- **A-PE-104-3**: riga **49a91e4d nel PIN_REGISTRY come NON-pin** (precedente
  365f4d40/832568a7: i rifiutati si registrano). La dichiarazione in rotazione
  non basta: il registro è l'unica fonte dei binari, e d45b578 insegna cosa
  succede agli hash orfani.
- **A-PE-104-4**: il grado del pin nuovo resta **MINIMO** e BASTA per l'ordine
  S-103 (A/B peak, H-C2, H-D sono tutte gambe CLI/phpr): il grado pieno
  (option 413 + restapi 3508 env -i) NON è dovuto — non anticiparlo.

## Kill-switch

- **KS-PE-104-1**: un collaudo del pin §3.13 senza il braccio warning-line
  NON grada — il cambiamento imbarcato si osserva al confine, o il grado è nullo.
- **KS-PE-104-2**: se al pre-flight S-103 il binario al path PHPSRV non è il
  pin atteso, si RICOSTRUISCE con la ricetta (o ripristina dallo stash) e si
  ricollauda — mai declassare il fail-closed a «hash aggiornato a mano».
- **KS-PE-104-3**: qualunque cifra server su pin a grado MINIMO = cifra nulla
  (riafferma KS-PE-100-3 sul pin nuovo; vale anche per il peak se mai passasse
  dal server).
