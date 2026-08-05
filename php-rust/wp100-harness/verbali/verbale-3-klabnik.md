# Verbale Sedia 3 — Klabnik (spec, testabilità, matrici, gate) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO
**S-98.0: RIFIUTATO NELLA FORMA** — le misure (M1, D_add=6,07) reggono, ma
due claim di evidenza sono SOVRADICHIARATI (refutazioni 1 e 2). **S-99.0:
EMENDATO** — il punto 4 è sottodimensionato rispetto al tag-space di Add.

## Refutazioni capitali

**R1 — Il «controllo positivo» del funnel controlla il DUMP, non l'OUTPUT.**
`reg_lower_funnel.rs` asserisce solo `off_out == on_out` + tre forme nel
chunk `{main}`: non pinna il valore atteso dello stdout né l'exit status.
Un binario che sbaglia UGUALE nei due modi (o che fatala a runtime DOPO il
dump di compile-time) passa il test. Il claim «controllo positivo sul
{main}» in NEXT_SESSION è quindi più forte del test scritto. Inoltre la
guardia M5 vive in UN solo test: un `cargo test behavioral_parity` filtrato
con `PHPR_REG_LOWER` esportato compila già lowered in `compile()` e applica
il pass DUE volte via `lowered()`, in silenzio.

**R2 — Il gate per NOME è UN BIT per test fallito.** Ho verificato io:
flagoff = flagon = canone wp82, byte-uguali dopo sort (1418) — la catena
NON è induzione, i tre anelli sono confrontati davvero. Ma l'identità del
SET non dice nulla sul CONTENUTO dei 1418 fallimenti: un test che fallisce
con un diff DIVERSO tra flag-off e flag-on è invisibile al gate. Per la
parità di regressione va bene; come **gate di PROMOZIONE è insufficiente**
(risposta a M4: serve il riga-per-riga degli output effettivi dei fail).

**R3 — La matrice smoke di BinaryAdd copre 5 celle di un tag-space ≥12.**
Coperti: overflow positivo, "2"+3, 2.5+1, [1]+[2], concat. Mancano per
NOME (tutti percorso MISS→fallback, dove vive il rischio di ordine
pop/diag): **Ref** (slot alias su `+=`, coda Swap→BinaryAdd), **Bool**,
**Null** (null+1 silente), **Str non-numerica** ("abc"+3: TypeError che
nomina gli operandi IN ORDINE — la stessa classe del fold commutativo
rimosso), `" 2"+3`, string numerica oltre int-range, **overflow negativo**
(PHP_INT_MIN), Long+Double nei DUE ordini, **GMP e BcMath\Number overload**
(A-ST-99-2 lo nomina ma S-99 lo mette in un timebox ½ sessione col resto),
resource+int, undefined-var nel miss path.

**R4 — Il budget cifre alzato DUE volte in sessione (24561→24643→24645).**
Il primo rialzo è forgia legittima (fonti nuove, atto deliberato A-SK61).
Il secondo (+2, «fix decimale») è il gate ri-adattato all'artefatto DOPO
che ha morso: escape hatch usato due volte = inizio di abitudine. Il file
contiene UN numero nudo: l'audit richiede archeologia git.

## Emendamenti
- **A-KL-100-1**: `reg_lower_funnel.rs` pinna lo stdout atteso (letterale)
  e l'exit status; la guardia anti-env M5 sale nell'helper condiviso.
- **A-KL-100-2**: gate di promozione M4 esteso — diff riga-per-riga degli
  output dei 1418 fail flag-off vs flag-on, salvato come evidenza.
- **A-KL-100-3**: matrice fixture Add per NOME (elenco R3) nella batteria
  PRIMA del rollout Sub/Mul; diventa il template della famiglia.
- **A-KL-100-4**: il budget porta il breakdown per fonte nel file stesso;
  un secondo rialzo in-sessione enumera i token nuovi, non solo la causa.
- **A-KL-100-5**: debito B2 (Hejlsberg) in ordine S-99 punto 4 — i 13
  snippet della parità comportamentale rigiocati al funnel VERO (binario
  spawannato); oggi al funnel è provata UNA forma di controllo, zero
  diagnostiche, zero eccezioni, zero funzioni.

## Kill-switch
- **KS-KL-100-1**: promozione flag-on→default VOID senza l'artefatto
  riga-per-riga di A-KL-100-2.
- **KS-KL-100-2**: ogni nuova specializzazione della famiglia (Sub/Mul,
  coda AssignOp) VOID finché la matrice A-KL-100-3 non è nella batteria.
- **KS-KL-100-3**: terzo rialzo del budget in una stessa sessione senza
  enumerazione dei token = gate dichiarato non-mordente, SELFTEST da
  rieseguire prima di ogni altro uso.
