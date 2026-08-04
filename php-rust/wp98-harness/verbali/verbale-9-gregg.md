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
