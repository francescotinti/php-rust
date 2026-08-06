# Verbale sedia 7 — LEIJEN (allocatore, footprint, census memoria) — Concilio WP-107 su S-105

Fase 1 rilanciata post-limite API; verbali delle altre sedie NON letti.

## VERDETTO: CON EMENDAMENTI (nessuna refutazione capitale; la promozione della forma 2 regge sui co-primari)

## R-LE-107-n (refutazioni)

**R-LE-107-1 — «Operazionalizzata» è un overclaim: la lettera-costo è stata
SOSTITUITA, non operazionalizzata.** La lettera diceva «se il costo NON sale
⇒ STOP»; il gate G1 misura la TAGLIA. Un gate a taglia può fallire SOLO per
mis-attribuzione: non può mai fermare la leva per «alloc gratis» — e la
forma 1 ha poi dimostrato che l'alloc era quasi gratis. Però la lettera era
MAL POSTA, non solo impraticabile: il costo marginale di un cap-bump 32→64 B
(due size-class entrambe fast-path TL) non stima il beneficio di RIMUOVERE
la coppia — ΔC≈0 anche a coppia cara — e il timing di una build census è
illeggibile per costruzione. La sostituzione era la mossa giusta, dichiarata
male: la parentesi nel criterio rivendica una fedeltà che non ha. La domanda
soppressa («l'alloc vale qualcosa?») è migrata sui co-primari dell'A/B solo
DE FACTO — per fortuna del disegno, non per dichiarazione. E un cost-STOP
fedele avrebbe UCCISO una leva vincente: il valore non era nell'alloc ma nel
volume (reverse+Vec+transito). Gate di attribuzione e gate di valore sono
oggetti DISTINTI; il valore lo decide solo l'A/B.

**R-LE-107-2 — La banda [6,14] (mia) è refutata come banda di LEVA, ma NON
è né refutata né confermata come prezzo dell'allocatore.** −14/+23 refuta
l'equazione «Δ leva = prezzo della coppia TL»: l'oggetto di un A/B è il
delta di FORMA del codice, dominato da volume mosso e struttura di inline,
non dal microcosto malloc (errore a segno ignoto, eco di R-GR-103-1). Ma la
coppia TL in isolamento non è MAI stata misurata: il «~9 ns alloc+free» in
circolo è aritmetica tra A/B distinti (KS-HE-107-1: indizio, mai cifra).
Non registrare «mimalloc TL = 9 ns» come fatto. Per H-C3: nessuna banda
derivata prezzando componenti.

**R-LE-107-3 — La disciplina binario-per-misura è stata violata e sanata
post-hoc.** Il criterio (11cc23a) ordinava: probe «NON si committa, si
reverta». Il campione G2(b) è girato su un binario che lo montava ANCORA.
Il numero 73,1% regge nel merito — `arity_note(args.len())` conta len, e
len è indipendente da capacity per costruzione — ma il perimetro «solo la
riga argarity è citabile» è stato scritto ALLA LETTURA, non prima del run.
In più il «campione reale wptests» è UNA fetta (functions.php, 214 test):
la generalizzazione al carico WP resta da provare.

## A-LE-107-n (emendamenti)

- **A-LE-107-1**: sostituire la lettera di un gate = EMENDAMENTO dichiarato:
  si nomina il test soppresso e si dice DOVE verrà risposto (qui: A/B
  co-primari). Mai «operazionalizzato» per una sostituzione d'osservabile.
- **A-LE-107-2 (H-C3)**: pre-registrare segno + soglie di promozione
  (pavimento 4, max(rumore, layout)); l'attesa di magnitudine è orientativa
  e NON vincolante; bande STOP/bisect solo da distribuzione MISURATA dello
  stesso estimatore (mai da componenti prezzate).
- **A-LE-107-3**: rieseguire il census arità su binario PULITO prima della
  prossima decisione di sito calls — si compone gratis col contatore
  hit/miss A-BA-107-1 (stesso run, stessa build).
- **A-LE-107-4**: le attese census si scrivono negli SPIGOLI del histogramma
  reale: il pre-registrato «(16,32]→(32,64]» nomina un bucket che NON
  esiste (spigoli le16/le32/le48/le64); la lettura è atterrata su le64 =
  (48,64]. Qui innocuo (64 B ∈ entrambe le dizioni), ma un'attesa su bucket
  inesistente lascia al lettore la libertà di dichiarare il match.
- **A-LE-107-5 (registro)**: «mimalloc TL quasi gratis sul sentiero caldo»
  ha ora DUE conferme indirette (H-C2, H-D forma1-vs-forma2) e ZERO misure
  in isolamento: resta ipotesi di lavoro — licenzia lo scarto delle leve
  micro-costo alloc, non cifre.

## KS-LE-107-n

- **KS-LE-107-1**: ogni lettura census cita hash del binario + MANIFEST
  dell'apparato montato; perturbazione non registrata ⇒ citabile solo il
  campo con argomento di indipendenza scritto PRIMA della lettura; se il
  campo diventa portante oltre la sessione ⇒ rerun su binario pulito.
- **KS-LE-107-2**: nessuna banda ottenuta prezzando componenti può fare da
  trigger STOP o bisect; tali bande sono SOLO orientamento (generalizza
  R-LE-107-2; la banda R=7 34,64 resta il modello: distribuzione misurata).

## Verifica (d) — convenzione S-102: RISPETTATA

Verificato a sorgente: `crates/php-types/src/memcensus.rs:1429-1438`
(GA_ARITY + arity_note; chiamante feature-gated mem-census, build di parità
pulita) e `crates/php-runtime/src/vm/zvalcensus.rs:361-368`: la riga
`argarity` è APPESA dopo `freehist` (r.358); `allochist`/`freehist`
conservano formato byte-identico — i denominatori S-104 (183.929, 144.845)
riletti in S-105 con lo stesso parsing lo confermano a valle. Additività ok.

## Priorità S-106 (questa sedia)

1. Lettura coppia WP + grado server (ordine provvisorio, condiviso).
2. Census arità+hit/miss su binario pulito (A-LE-107-3 ∘ A-BA-107-1).
3. H-C3 con criterio A-LE-107-2. 4. Backlog sponsorizzato: memory_get_usage
cablato a galloc/gfree (A-LE-106-6, ancora aperto); design per-fase peak.
