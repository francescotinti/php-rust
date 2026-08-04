# Verbale sedia 6 — Pedersen (WP-97)
Perimetro: confine per-richiesta/per-test, lifecycle, identità dei misurati.
Oggetto: S-95.0 (A-ZV2 F1+F2 sola misura) + §WP-96 (F3 TakeSlot).

## VERDETTO
**RESPINTO IN PARTE.** I conteggi F1/F2 reggono come misure (somme macchina
verificate: f1-out/f2-out `.sum` = trascrizioni `.out`, F1↔F2 identici al
contatore). REFUTATA la provenance di F2: il run parte alle 10:00:08 con
`f2.identity head=fb0599ba53ab`, ma `zvalcensus-f2.out` dichiara «HEAD
ee3f551» — commit creato alle **10:00:22, 14 secondi DOPO l'avvio del run**.
Il binario 2a321e3b è stato costruito da un albero NON committato; nulla lo
lega a ee3f551. L'identità in banda esiste ma **contraddice l'header
pubblicato** e non basta: manca il porcelain (stato dirty), manca il re-hash
post-run, manca ogni identità della suite misurata oltre ai conteggi.

La somma su 16 processi NON è un'identità ben definita: è una popolazione
EMERGENTE («chi ha ereditato la env, ha contato, ed è uscito via `exit()`»).
`atexit` non scatta su morte per segnale/`_exit`; un figlio morto sparisce
in silenzio dalla somma; la riga raw non porta pid; il summer perl SCARTA
in silenzio le righe malformate (append concorrente ⇒ rischio riga
spezzata). Nessun dente pretende N=16: il 16 è osservato, mai asserito.

«Determinismo pieno fra i due run»: la coppia identica è reale e
machine-verified, ma è **N=1** e convive col rumore di 42 eventi vs il
before della stessa mattina, la cui sorgente non è nominata. Osservazione
forte, generalizzazione indebita.

L'esito-suite «IDENTICO» è parità di CONTEGGI (762/1912/52), in violazione
della regola di progetto «gate per NOME, mai solo conteggio».

**Binding rule output-capture-before-request_end**: F3 non la viola
strutturalmente — `TakeSlot` ANTICIPA i drop (il valore muore all'operazione
consumante), mai li posticipa oltre `request_end()`; l'output di un
`__destruct` anticipato resta dentro la finestra di cattura. Il rischio
reale è fratello, non figlio: l'ordine dei distruttori è semantica per-test
osservabile, e il perimetro F2 intero (oggetti/array) lo espone; il nucleo
`_str` è a rischio zero per costruzione. Le 5 trappole elencate sono un
elenco finito contro una classe semantica: servono il caso per-richiesta e
un kill-switch, non solo test.

## Emendamenti
- **A-PP-97-1**: `.identity` DEVE includere `git status --porcelain`
  (vuoto o hash del diff) + re-hash del binario census a FINE run; gli
  header dei `.out` si trascrivono DALL'identity, mai a memoria. Correggere
  `zvalcensus-f2.out`: head reale fb0599b, contenuto ee3f551 non dimostrato.
- **A-PP-97-2**: riga raw con `pid=`/`ppid=`; il runner ASSERISCE il numero
  atteso di processi (16) — mismatch = FAIL; il summer conta e FALLISCE su
  righe scartate/malformate.
- **A-PP-97-3**: identità della suite per NOME (diff dei nomi/esiti tra
  run), non per conteggio — la regola vale anche per la suite di misura.
- **A-PP-97-4**: le trappole F3 includano un caso PER-RICHIESTA
  (php-server): `__destruct` che emette output a ridosso di
  `request_end()`, verificando la cattura invariata; più generatore sospeso
  attraverso il confine di richiesta.
- **A-PP-97-5**: declassare «determinismo pieno» a «coppia identica, N=1»;
  nominare la sorgente dei 42 eventi PRIMA di usare il determinismo come
  premessa di F4.

## Kill-switch
- **KS-PP-97-1**: in F3, QUALUNQUE divergenza di ordine dei distruttori su
  qualunque gate (corpus/refl/ORM/hk/battery61) ⇒ restrizione automatica al
  nucleo `_str` nella stessa sessione — nessun dibattito.
- **KS-PP-97-2**: in F4, `slot_reads_avoided` fuori dalla quantità predetta
  da `would_take_safe` (tolleranza nominata ex-ante) ⇒ il Δ tempo NON è
  attribuibile: nessuna rivendicazione.
- **KS-PP-97-3**: conteggio processi ≠ atteso in un run census/F4 ⇒ run
  INVALIDO, si ripete; mai sommare popolazioni diverse.

## Refutazioni capitali
**SÌ, una**: la provenance F2 — l'header «HEAD ee3f551» è falsificato
dall'identity in banda (fb0599b) e dalla cronologia dei commit; build da
albero non committato, irriproducibile come dichiarata.
