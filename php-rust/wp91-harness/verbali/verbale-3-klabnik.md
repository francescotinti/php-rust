# VERBALE — Steve Klabnik, sedia 3, Concilio WP-91

VERDETTO: **CON EMENDAMENTI — con UNA REFUTAZIONE CAPITALE (A-SK56).**

Metodo: gate invocato SOLO su copie in scratchpad; albero verificato pulito
(`git status --porcelain` = solo `?? php-rust/wp91-harness/`). Copia intatta
di MEASURE89 → PASS (baseline).

## Q1a — A-SK55 TIENE
Forge non committati con nomi che matchano i glob del corpus
(`wp78-harness/measure-out/m89.zzklab-forge.log`, `wp89-harness/zzklab-forge.out`,
`forged=777444111`) + doc doctored che cita `777.444.111 B` → **FAIL** entrambi
("not in committed corpus"). Il corpus HEAD-only morde.

## Q1b — A-SK56 REFUTATA A MACCHINA (capitale)
Corpus estratto dal canale cache: **22.399 token**. Riga piantata su copia:
`b_base rivisto = 19.600.000 B = 18,69 MiB per worker [derivata: 23.000.000 − 3.400.000]`
→ **PASS**. Controllo senza tag → FAIL su ENTRAMBI i token. Gli operandi non
sono misure: sono cifre di **indirizzi esadecimali vmmap**
(`b22c00000-b23000000`, `9f3000000-9f3400000`); **5.262/22.399 token (23,5%)
del corpus esistono SOLO in file vmmap**. La verifica X−Y è esistenziale sulla
CHIUSURA DELLE DIFFERENZE: densità diretta in [19e6,20e6) = 4 valori; con
[derivata] i valori raggiungibili entro ±2% di b_base sono **89.881 (~11,5%)** —
amplificazione ~2×10⁴. Non serve N arbitrario (12.345.678 è irraggiungibile):
basta un N *plausibile*, e quelli abbondano.
Secondo morso, sulla forma REALE del doc: su riga [derivata] è vincolato solo
il token seguito da `B`. `2σ = [11.111.111, 20.745.049] B [derivata: companion /1048576]`
→ **PASS** (bound inferiore fabbricato); senza tag → FAIL. "Figure-scope" è in
realtà "token-seguito-da-B-scope". Nota: `--selftest` resta PASS (verificato,
>120 s: nessuna cache in quel modo) — il suo caso A-SK56 usa un tag SENZA
espressione, quindi morde solo il falsario ingenuo.

## Q2 — A-SK53-bis: tiene, ma due buchi
Fuori M85 `232±1` → FAIL (dente ok). (a) `3.605.572 B ±5%` (banda KL-85-2
**ritirata** da KB-90-2) è in allowlist GLOBALE: **PASS** come banda VIVA in un
doc MEASURE89 — la ritirata è prosa, non dente. (b) le grazie per-doc sono
keyed a una REGEX DI NOME: copia chiamata `MEASURE85_zzforge_RESULTS.md` (che
`--all` peschera dal glob) eredita `±5%` e `232±1` → **PASS**. Corollario:
`MEASURE100_RESULTS.md` con `232 MiB … ±7%` senza companion → **PASS**
(bytes-first spento da `/MEASURE8[4-9]|MEASURE9\d/`); stessa riga in MEASURE89
→ FAIL. Tutta la famiglia A-DL26/A-SK40/A-SK43/A-SK53-bis **scade a WP-100**.

## Q3 — cache AVVELENABILE
`GATE_CIFRE_CORPUS_CACHE` è input non autenticato: `cp corpus.cache poison2;
echo C777444111 >>` → lo stesso doc che senza cache FALLISCE ottiene **PASS**.
Chiave = solo HEAD rev; nessun digest della lista sorgenti, nessun nonce.
`--all` riassegna la var (mktemp) ⇒ battery schermata; ogni invocazione singola
no. Bonus: la cache è anche **esfiltratore** del corpus (l'ho usata per costruire
il forge Q1b).

## Q4 — gG: nessun dente, solo KS
`gate-measure-cifre` è l'UNICO gate che legge i doc (doc-purge scansiona .rs).
Copia di MEASURE89 con `g3`→`g1` (file citato e "generazione MASSIMA") → **PASS**.
Inoltre i .out di generazioni FAIL/superseded stanno nel corpus (`wp89-harness/*.out`):
a questo HEAD 0 token esclusivi di g1/g2 ⇒ buco **latente**, non vivo.

## Emendamenti
- **A-SK60**: abolire l'evaluator X−Y libero. Un [derivata] byte-figure è legale
  solo se ogni operando è risolto per PROVENIENZA (`file:riga` committato,
  riletto a HEAD) e il gate STAMPA la risoluzione; meglio ancora: la cifra
  derivata la emette l'emitter in un .out.
- **A-SK61**: igiene corpus — escludere i digit-run da indirizzi (righe vmmap
  `[0-9a-f]+-[0-9a-f]+`) e i flatten `nodot` dei decimali; stampare cardinalità
  del corpus e pinnarla (budget) per HEAD.
- **A-SK62**: su riga [derivata] giudicare OGNI token ≥3 cifre (bracket 2σ,
  se, conteggi), esentando solo i ricalcolabili a macchina.
- **A-SK63**: allowlist ± da MANIFEST committato (doc path + blob sha + bande
  concesse), non da regex di nome; `3.605.572B±5%` fuori dalla lista globale.
- **A-SK64**: perimetro e bytes-first dal manifest; `--all` FAIL bidirezionale
  (doc in manifest mancante / `MEASURE*_RESULTS.md` fuori manifest).
- **A-SK65**: cache via argv + nonce generato dal padre; env ignorato.
- **A-SK66**: dente generazioni — il doc deve citare `verdict<NN>.a<A>.g<G>.out`
  con G massimo per A e ultimo verdetto PASS; i .out superseded escono dal corpus.

## Kill-switch (serie WP-91)
- **KS-SK-91-1**: finché A-SK60/A-SK62 non atterrano, ogni PASS di
  gate-measure-cifre su doc con [derivata] è NON verdict-grade (forge dimostrato).
- **KS-SK-91-2**: cifra non giudicata perché il NOME del doc è fuori regex ⇒
  PASS del doc VOID; manifest obbligatorio prima del primo doc post-WP-99.
- **KS-SK-91-3**: PASS ottenuto in un process-tree con `GATE_CIFRE_CORPUS_CACHE`
  ereditata = VOID.
- **KS-SK-91-4**: doc che cita generazione non massima o FAIL senza dente ⇒
  la citazione di provenienza non è verdict-grade.

— Steve Klabnik, sedia 3
