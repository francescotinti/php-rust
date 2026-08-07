# Verbale sedia 3 — KLABNIK (spec, testabilità, matrici, gate) — Concilio WP-108

**VERDETTO: emendamenti LEGITTIMI NELLA SOSTANZA, INDEBOLITI IN DUE PUNTI NOMINATI; pin eb555106 REGGE; nessuna refutazione capitale.**

## 1. reg_lower_funnel — emendamento dichiarato, ma il sito è ceduto

Verificato sul sorgente (`crates/php-cli/tests/reg_lower_funnel.rs:97-146`): l'attesa BinaryDst del `{main}` è sostituita da BinarySTDst e BinaryDst è ri-collocato nella probe. Due difetti:

- **R-KL-108-1** — Il `{main}` non contiene PIÙ ALCUN sito BinaryDst (`$s += …` fonde oltre; il Sub stack-stack resta generico e non è tripwire). La lettera era nata ESATTAMENTE perché il `{main}` era la gamba cieca (header B1/A-HE-99-1): una regressione della finestra Stack+tail SPECIFICA del top-level (pipeline pre-cessione slot_names) ora passa verde, coperta solo dal corpo-funzione. Cura da un rigo: aggiungere in main `$s = $s + $i + $i;` e ri-asserire BinaryDst nel `{main}`.
- **R-KL-108-2** — Manca il NEGATIVO «niente LoadSlot/Swap residuo nel `{main}` ON»: se la finestra emette BinarySTDst senza consumare il prefisso (fold parziale, output corretto, leva silenziosamente morta), il funnel resta verde — precisamente il difetto che l'header promette di prendere. Il sorgente della fixture non ha altri produttori di Swap nel main: `!chunk.contains("Swap")` è implementabile subito.

Il tripwire `Binary(Add)`, i controlli OFF speculari (BinarySTDst assente OFF, riga 140) e gli stdout derivati restano denti sani.

## 2. census op_index — legittimo

`census.rs:639-657`: indici RELATIVI (N_OPS−2, −1), BinaryAdd chiude ancora la tabella, `op_names_are_unique` vige. Nessun indebolimento. Nit: N_OPS=187 citato a verbale ma non pinnato nel test — accettabile, il relativo è più robusto dell'assoluto.

## 3. Matrice ST — larga e NON testata per NOME

**A-KL-108-1**: la finestra «morde ogni `$local <op>= expr`» (claim trasversale a verbale) ma la batteria prova solo Add/Sub su RHS aritmetico locale. Fuori matrice: `$s -= $o->x` (il 2° beneficiario prop vive SOLO in un dump .out non ripetibile); `.=` vs ConcatAssignSlot (chi vince la finestra? nessun test in nessuna direzione); Mul/Div/Mod/shift/bitwise; **jump target dentro la finestra** (nessun negativo che il fold RIFIUTI quando un'etichetta cade tra LoadSlot e BinaryDst — il pericolo classico del peephole); path diagnostico (`diagnostic-safe` è dedotto, non fixturato: serve un caso che lancia dentro l'op fusa con parità messaggio+riga nei due modi). Corpus 1417 copre per fortuna, non per NOME (lezione WP-96).

## 4. fx21 gate — fail-closed DAVVERO

Verificato al byte: goldens "2,3 22" (phpr) vs "2,3 23" (oracle) — la cura §3.15 rende ROSSA la riga 73-75; pin bilaterale exit 66; golden mancante 66; deriva oracle 66; `sed '5d'` full-cmp copre anche drift del conteggio righe phpr. Bidirezionale (regressione ≠ cura, stesso rosso, messaggio distinto). Tooth intatto. Nit: «golden stesso commit» resta procedurale, non meccanico — D-21 lo arbitra.

## 5. Churn hash₁→hash₂ — mislabel

**R-KL-108-3**: «relink + reindex census INERTE» è ETICHETTA SBAGLIATA: il reindex è un edit di SORGENTE di produzione (tabelle OP_NAMES/op_index in php-runtime) fatto DOPO l'A/B — non churn. La sostanza però regge: batteria/fixture/corpus/micro girati TUTTI sul sorgente emendato (hash₂), micro su entrambi gli hash (Δ≤0,4). Regola: se tra hash₁ e hash₂ cambia UN file sorgente, il verdetto dichiara «build emendata post-A/B, ri-validata sul pin» ed enumera i file.

## 6. Igiene cargo check

Dichiarato exit 0 (A-KL-107-1) ma NESSUN .out lo archivia in wp106-harness — per KS-KL-107-4 un rc non archiviato è un marker. **A-KL-108-2**: rc di cargo check in un .out come ogni gate.

## KS-KL-108

1. **KS-KL-108-1** — Un controllo ri-collocato conserva la FORMA ma può cedere il SITO: la lettera che nasce per un sito deve conservare UN positivo su quel sito.
2. **KS-KL-108-2** — Una fusione si prova DUE volte: forma nuova PRESENTE e residuo ASSENTE.
3. **KS-KL-108-3** — Churn è solo relink: un edit di sorgente dentro il churn si dichiara build emendata.

## Ordine S-107 — APPROVATO CON EMENDAMENTO

Sequenza sana (denti→§3.15→leva). Due riserve: (a) R-KL-108-1/2 entrino nel punto 1 come denti (test-only, ~10 minuti); (b) punto 1 ha cinque denti + fedeltà PRIMA della leva — rischio speculare della S-106 (leva affamata invece dei denti): timebox esplicito o il ritmo-leva salta.
