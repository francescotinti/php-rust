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
