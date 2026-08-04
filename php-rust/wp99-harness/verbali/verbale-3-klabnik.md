# Verbale sedia 3 — Klabnik (spec, testabilità, matrici, gate) — Concilio WP-99

## VERDETTO: CONCORDO CON EMENDAMENTI

Il verdetto H-A1 (caduta sul criterio scritto) è ben evidenziato e la parità
flag-off è solida. Rifiuto però lo STATO che il report attribuisce a due cose:
il gate cifre e il pin 9,87 ns.

## Refutazioni capitali: SÌ (due)

**R1 — «Gate cifre --all PASS» prova meno di quanto afferma.** Le 4 righe
manifest nuove violano la carta del manifest stesso (blocco A-SK-71, righe
37–44): `WP_SESSION_97.md` — l'ULTIMA sessione, doc di rotazione ATTIVO — è
registrata `judge=no`, mentre WP_SESSION_94 e 95, sessioni CHIUSE, restano
`judge=yes`. La regola è violata in entrambe le direzioni: le cifre della
sessione attiva sono NON giudicate. Inoltre i tre `.out` gemelli di S-97.0
(`micro-baseline.out`, `arith-decomposition.out`, `ha2-sweep.out` — stessa
classe, stessa directory, stessa cifra-corpus) non hanno riga ALCUNA: o la
classe esige la riga (e allora il dente bidirezionale non ha morso: FAIL
latente), o non la esige (e allora le tre righe nuove sono decorative). PASS
sì, ma su un perimetro incoerente con la propria carta.

**R2 — Il pin che «decide la prossima mossa» (9,87 ns, −30,7%) proviene da uno
strumento non collaudato.** È misurato flag-ON, e la parità flag-ON sul corpus
per NOME fu fatta in WP-44 su un albero di luglio; da allora: liveness
riclassificata, `reg_load_slot` seed-aware, fold const-lhs rimosso, elisione
Sweep H-A2. Su QUESTO albero il flag-on è attestato da 13 snippet. Un numero di
grado VERDICT che orienta il programma non può poggiare su uno strumento mai
ri-collaudato.

## Matrice di fold e buchi dei TEST

La matrice implementata è coerente con la spec del module-doc, con tre riserve:
(a) `[LoadVar,PushConst,CmpJmp]` non è gestita — regge sull'invariante NON
scritto «slot-vs-const compare ⇒ sempre CmpJmpConst all'emissione», mai
asserito; (b) il non-fold Spaceship const-first (`3 <=> $x`) non ha alcun test
(nel battery `5 <=> 3` è const-const); (c) la ragione stessa del fold rimosso —
l'ORDINE degli operandi in "Unsupported operand types" — non ha una fixture che
faccia scattare il TypeError nei due ordini. Mancano inoltre: jump-target a
METÀ finestra (guardia `blocked` esiste, zero test strutturali), finestra su
linee sorgente MISTE (guardia `lines` mai esercitata, parità del numero di
riga del warning non provata), indici >u16::MAX (guardie = rami morti), e il
check del battery sugli `Addr` accetta QUALUNQUE indirizzo in range — solo la
parità dinamica può smascherare un remap sbagliato-ma-in-range, con un solo
snippet try/catch. Infine: `lowered()` nei test enumera i corpi a mano
«rispecchiando il funnel», ma nulla pinna che i due insiemi coincidano (i
corpi dei property hook?).

## Il dormiente ha un gate? Solo a metà

`cargo test` esercita `lower_func` e i 7 handler via `run(&lm)` — ma bypassa
ESATTAMENTE le cuciture dove il dormiente marcirà: il funnel `enabled()`, la
chiave `reg_mode` della unit-cache, il corpus. E
`stage2v3_flag_off_emits_no_register_forms` si auto-disattiva in silenzio se
`PHPR_REG_LOWER` è esportata: un dente spegnibile dalla cosa che sorveglia.

## Programma H-B1

Il criterio «scende in modo netto … sotto il rumore della coppia» è un criterio
di CADUTA, non di successo: col rumore a ~0,5% quasi ogni effetto reale passa.
H-A1 cadde a −30,7% contro 40; H-B1 così scritto vincerebbe a −2%. Il numero
va derivato dall'obiettivo: ≤3× su arith ⇒ ~1,4 ns/op complessivi contro 8,24.

## Emendamenti

- **A-KL-99-1**: una delibera che ripari il manifest: WP_SESSION_97 `judge=yes`
  (o carta emendata), 94/95 declassate, riga (o esclusione dichiarata di
  classe) per i tre `.out` di S-97.0.
- **A-KL-99-2**: smoke flag-ON nel CI di sessione: `PHPR_REG_LOWER=1` su
  arith_small + controllo positivo `PHPR_DUMP_OPS` (il dump DEVE contenere
  `BinarySSDst`) — la lezione del `tail` codificata come dente, non come prosa.
- **A-KL-99-3**: completare la batteria: target a metà finestra (assert
  strutturale), finestra multi-linea + parità della riga del warning,
  Spaceship const-first non-fold, TypeError operand-order nei due ordini,
  guardia u16 esercitata; più un test che pinna lowered() ≡ funnel di
  produzione sull'insieme dei corpi.
- **A-KL-99-4**: gli `.out` devono autodescriversi: `micro-ha1-{on,off}.out`
  sono copie byte-identiche dell'header S-97.0 e NON registrano lo stato del
  flag — la provenienza vive solo nel nome del file.
- **A-KL-99-5**: il test flag-off non deve auto-skipparsi in silenzio: se
  l'ambiente inverte la premessa, FALLIRE rumorosamente.
- **A-KL-99-6**: il criterio di H-B1 si scrive in ns/op derivato
  dall'obiettivo 3×, non dal rumore.

## Kill-switch

- **KS-KL-99-1**: nessun futuro `.out` di grado VERDICT può pubblicare un
  numero flag-ON senza citare una parità corpus per NOME flag-ON eseguita
  sullo STESSO albero.
- **KS-KL-99-2**: se alla chiusura di S-98 il manifest ha ancora l'ultima
  WP_SESSION `judge=no` (o i tre `.out` di S-97.0 senza delibera), la riga
  «gate cifre PASS» non si scrive nel report di chiusura.
