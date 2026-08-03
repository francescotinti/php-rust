# Verbale sedia 3 — Steve Klabnik (Concilio WP-92)

**VERDETTO: CON EMENDAMENTI — refutazioni capitali sul gate cifre.**
Quattro forge REALI costruiti e fatti passare dal gate vivo (repo ripristinato,
`git status` pulito; sandbox `zz-attack-tmp` rimossa).

## Q1 — grandfathering `legacy_frozen` (A-SK60): **FORGE LANDED**
Il codice concede la semantica pre-WP-91 (evaluator X−Y libero) a *qualunque*
doc il cui blob coincida con lo sha di una riga di manifest. "Doc STORICO
CONGELATO" è un **commento**, non un predicato: nessun test di committed-ness,
di data, di appartenenza al set M84-M88. Inoltre il manifest è letto dal
**working tree** (`open "$here/gate-cifre-manifest.tsv"`), mentre il corpus è
letto da HEAD (A-SK55): input non autenticato.
Morso: doc creato 10 secondi prima + **una riga di manifest mai committata** →
`PASS` con `b_base = 19.600.000 B [derivata: 20594560 − 994560]`. Ripetuto sul
manifest REALE (poi ripristinato): stesso PASS. Misura fresca: l'evaluator
resuscitato copre oggi **25,21 %** della finestra [18,6M–20,6M] (era ~11,5 % al
WP-91: il buco è **cresciuto**).

## Q2 — provenance `prov N@path:riga` (A-SK60): **CAPITALE**
L'evaluator libero non è stato abolito: è stato **annotato**. Operandi
ammessi: qualsiasi cifra su qualsiasi riga di qualsiasi blob a HEAD —
**36.573** operandi indirizzabili, contro i 24.042 del corpus. Chiusura delle
differenze nella stessa finestra: **46,25 %** — *più larga* di quella che
A-SK60 doveva chiudere. Tre difetti di specifica, tutti morsi dal vivo:
1. **nessuna commensurabilità**: i due operandi possono venire da campagne
   diverse. Live PASS: `19.600.000 = 23000000@…axum.83c…vmmap.V1:1963 −
   3400000@…axum.82c…vmmap.V1:1964` (il forge esatto che T5 dichiara ucciso);
2. **A-SK61 non si applica al resolver**: le righe citate sono *address range*
   `b22c00000-b23000000` — i frammenti che A-SK61 toglie dal corpus rientrano
   dalla porta della provenance;
3. **l'operatore non è verificato**: `[derivata: prov X@p:1 diviso Y@p:2]` passa
   e stampa "provenance-verified", perché il test `/−|-/` è soddisfatto dal
   trattino dentro `wp89-harness`. Il gate verifica un'operazione diversa da
   quella scritta.

## Q3 — cache, budget, identità del giudice
`--cache/--nonce` (A-SK65) sposta il veleno da env ad **argv**, ugualmente non
autenticato per ogni invocazione diretta: il nonce lo sceglie il chiamante.
Morso a cardinalità invariante (tolgo un token, aggiungo il mio):
`PASS` su doc con `777444111 B` fabbricato, `corpus cardinality=24042
budget=24042`. Budget e manifest letti dal working tree; il gate stesso non
stampa il proprio sha: un giudice patchato produce un PASS indistinguibile.

## Q4 — perimetro e citazioni gG
`--all` cerca a ritroso solo `wp*-harness/MEASURE*_RESULTS.md`. Le cifre che
arrivano davvero all'umano e alla sessione dopo sono fuori: NEXT_SESSION (20
cifre) + `sessions/WP_SESSION_90.md` (23) = **43 non giudicate contro 28
giudicate** in MEASURE90. A-SK66 evaso due volte: riga «verdict89.a1.g1.out
NON e superseded» passa (parola-chiave, non prova) e la citazione **senza
`.out`** non è nemmeno vista.

## Emendamenti
- **A-SK-67** manifest, budget e giudice letti da HEAD; PASS stampa
  judge_sha+manifest_sha+budget_sha; working≠HEAD ⇒ FAIL.
- **A-SK-68** `legacy_frozen` abolito: i 5 doc storici o si rigiudicano, o
  `judge=no` con verdetto d'epoca citato. Nessuna riga nuova può concederlo.
- **A-SK-69** prov: operandi dallo **stesso file**, operatore parsato e
  verificato con i path mascherati, strip A-SK61 applicato alla riga citata,
  righe address-range rifiutate.
- **A-SK-70** cache abolita (un solo processo perl per `--all`) o rifiutata se
  non creata dal parent.
- **A-SK-71** perimetro = ogni .md committato che pubblica cifre di sessione
  (NEXT_SESSION, sessions/, design, verbali), riga di manifest obbligatoria.
- **A-SK-72** supersessione provata da riga di ledger committata; regex di
  citazione con `.out` opzionale.
- **A-SK-73** il budget A-SK61 governi anche il pool prov (36.573), altrimenti
  misura la superficie sbagliata.

## Kill-switch
- **KS-SK-92-1** PASS di gate/manifest/budget con blob ≠ HEAD ⇒ VOID.
- **KS-SK-92-2** `prov` con operandi di file/campagne diverse ⇒ non verdict-grade.
- **KS-SK-92-3** cifra pubblicata fuori perimetro manifest ⇒ non verdict-grade,
  anche se il MEASURE passa.
- **KS-SK-92-4** PASS con `--cache/--nonce` forniti dall'invocante ⇒ VOID.
