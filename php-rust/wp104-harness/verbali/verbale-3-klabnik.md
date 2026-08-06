# Verbale sedia 3 — Klabnik (spec, testabilità, matrici, gate) — Concilio WP-104

## VERDETTO

S-102 è una sessione di guardie ben condotta, ma il suo gate più citato ha
il verdetto archiviato ROSSO. **APPROVATA CON RISERVA**: la citazione
«corpus 1417×2 per NOME» è prosa non coperta da un artifact verde.

## REFUTAZIONE CAPITALE — sì, una

**RC-KL-104-1 — il gate corpus citato verde ha rc=2 archiviato.**
`wp102-harness/corpus-gate/progress.txt` registra «DIVERSO» su ENTRAMBE le
gambe e `corpus-gate.done` dice `rc=2 11:08:27`: il run è avvenuto contro
il riferimento VECCHIO (1418). Il riferimento è stato aggiornato alle
11:27 (commit ee842d0, miglioria 015.phpt ben documentata), ma **nessun
ri-giudizio contro il riferimento nuovo è mai stato registrato**. Il
«1417×2 per NOME + diff ZERO» di NEXT_SESSION §Stato gate poggia sulla
lettura umana di un diff di 86 byte, non su un rc=0. Il fix è sano; lo
STATO EVIDENZIALE del gate no: due fonti di verità che divergono (il .done
dice rosso, la rotazione dice verde). Cura a costo zero: le liste
`corpus-s102-{off,on}.fails` sono su file — si ri-giudicano con `cmp`
contro il riferimento nuovo e si registra `rc=0` in un artifact nominato.

## Perimetro (a) — aggiornamento riferimento

Disciplina buona a metà: miglioria nominata nel commit, stesso commit del
fix, byte-id ×2 modi asserito. Manca la REGOLA scritta e manca il passo
finale (ri-verdetto). Senza regola, il prossimo «set che scende» si
assorbe a discrezione.

- **A-KL-104-1 — regola «quando il set fail SCENDE» (da scrivere in
  NEXT_SESSION/Regole)**: (1) diff = SOLO righe rimosse, pena regressione;
  (2) ogni test rimosso assente da ENTRAMBE le gambe; (3) riferimento
  aggiornato nello STESSO commit della causa con nota MIGLIORIA per NOME;
  (4) ri-giudizio MECCANICO delle liste archiviate contro il riferimento
  nuovo, rc=0 registrato in un .done — solo allora il gate è citabile
  verde. Primo atto S-103: applicare il punto 4 retroattivamente.
- **KS-KL-104-1**: un diff del riferimento con righe AGGIUNTE non è MAI
  assorbibile come miglioria — gate rosso, punto.
- **KS-KL-104-2**: un gate il cui .done archiviato è rosso non si cita
  verde nella rotazione; la citazione deve puntare a un artifact rc=0.

## Perimetro (b) — gate pinnati per NOME (13 e 5)

Il confronto set (`echo $seen | tr ' ' '\n' | sort`) è robusto
all'ordinamento e a duplicati/sparizioni; si rompe con spazi o glob nei
nomi (`echo $seen` non quotato espande `*`). Rischio basso ma il pin è
testuale. Buchi veri:

- **A-KL-104-2**: i due gate fixture non verificano QUALE phpr giudicano —
  nessun hash, nessun FAIL-CLOSED (il collaudo-server ce l'ha, loro no).
  Un binario stantio in `php-rust-output` passa in silenzio. Aggiungere
  echo+check del hash come gamba 0.
- **A-KL-104-3**: nessun mode-probe — le gambe on/off si fidano che
  `PHPR_REG_LOWER` sia onorato; se il flag muta nome, due gambe
  stesso-modo = falso verde (l'R1 che il corpus-gate cura e i fixture-gate
  no). Un probe dal dump su UNA fixture per braccio basta.
- **KS-KL-104-3**: nomi fixture SENZA spazi/glob — assert nel gate.

## Perimetro (c) — matrice collaudo server

Celle coperte: off/on × req 1/2/3 stesso worker (workers=1) + cross-mode
+ oracle sanity. Celle VUOTE: **cb su worker DIVERSO** (workers=2 esiste
solo nella sentinella, che non serve cb1 — il capture-boundary su worker
fresco dopo che l'altro ha servito è non testato); **richiesta dopo
richiesta in ERRORE** (reset boundary post-errore); keep-alive vs
connessione nuova. → **A-KL-104-4**: al collaudo del pin NUOVO (primo
atto S-103) aggiungere cb con workers=2 e marker per-worker + una cella
errore-poi-successo.

## Perimetro (d) — criteri S-103 scritti PRIMA

Punto 1 ✓ (regola A/B pre-registrata, con clausola segni opposti).
Punto 2: la banda [8,22] c'è, il CRITERIO no — **A-KL-104-5**: fissare
ORA il nome del file (`wp103-harness/hc2-criterio.out`) così non può
nascere dopo la misura; per H-D pre-registrare il controllo positivo (il
census taggato deve sommare a ~2 alloc/chiamata già contate, pena
strumento rotto). Punto 5 igiene: aggiungere il ri-verdetto A-KL-104-1(4).
