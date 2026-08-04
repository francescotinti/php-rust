# Verbale 6 — Anders Pedersen (confine per-richiesta/per-run, lifecycle, ricevute in banda)
## Concilio WP-96 su S-94.0 · mandato: REFUTARE

## VERDETTO: **CONTRARIO CON EMENDAMENTI** — l'apparato è migliorato, le
**ricevute NON sostengono i claim**. Tre difetti di confine, uno capitale.

### 1. «ADVISORY» non è un confine, è un aggettivo
Il pin `php-server` non riproducibile è stato NOMINATO (bene) e poi
declassato con una parola. Non esiste un predicato che neghi
`grade=VERDICT` a un numero prodotto da quel binario: la degradazione vive
in prosa, in due file di rotazione, e muore alla prima citazione. Serve
etichetta IN BANDA nell'artefatto e un dente che morda. Inoltre la sessione
ha rinviato l'esperimento che DECIDE fra (a) e (b): due rebuild puliti allo
stesso HEAD sono due comandi. Rinviare un falsificatore da due comandi
mentre si spendono tre morsi sull'apparato è un errore di priorità.

### 2. CAPITALE — la malattia di `php-server` è UNFALSIFIED su `phpr`
`d5ce86e3342f3926` è **registrato in banda** (`pair-out/pair94.identity`,
`battery61-accettazione.out`, header di `pair94.out`: concordi) — quindi
l'identità del FILE è verificata, non solo asserita. Ma è uno sha di
CONTENUTO **senza ricevuta di provenienza**: nulla lo lega a un albero.
È esattamente la condizione in cui `d45b578` è marcito. Se un pin ha già
divorziato dal suo albero in questo repo, «INVARIATO» per `phpr` prova
solo che *lo stesso ignoto* è stato misurato due volte. Su questo ignoto
poggia l'intera baseline della leva.
Aggravanti misurate: lo sha è calcolato **una volta**, prima di quattro run
(~45 min di orologio: epoch pair94 1785799229 → battery61 1785801803), mai
riverificato dopo; **non è mai CONFRONTATO** con il pin dichiarato (se
avesse differito, lo script avrebbe prodotto numeri lo stesso: registra,
non giudica); l'oracle è identificato da `php -v`, **stringa, non hash**.

### 3. `pair94.identity` è **untracked** — l'autorità è una TRASCRIZIONE
`git ls-files` di `wp94-harness/pair-out/` restituisce **solo i quattro
`.time`**. Non sono in repo: `pair94.identity`, `full-*.txt`, `media-*.txt`,
`progress.txt`, `.done`. Quindi (i) l'unica identità che sopravvive a HEAD è
il blocco copiato **a mano** in `pair94.out`; (ii) `pair94-ratios.out` —
il file che lo script stesso elegge ad autorità («il documento CITA questo
file») — porta `grade=VERDICT` e **zero campi d'identità**: numeri senza
misurato; (iii) **i conteggi 762/1912/52 e 30472/4558029 e i due nomi di
failure non sono ricevutati**: i `.txt` da cui provengono non esistono a
HEAD e `pair94.sh` **non contiene alcun giudice di parità** — calcola
meccanicamente i rapporti e lascia la PARITÀ alla prosa. È la violazione
letterale di `gate-diff-fail-set-not-count`, commessa dal file che si
vanta di non fare aritmetica in prosa.

### 4. battery61 — il confine per-run NON esiste
`serve_and_capture` pulisce la **directory di output**, non lo **stato**.
Nessun reset DB, nessuna guardia uploads (che `pair94.sh` invece usa).
La gamba oracle esegue un **login POST + dashboard**: scrive session token
in usermeta, arma wp-cron, muove transient/option. La gamba phpr parte da
quel DB mutato, **sempre seconda**: asimmetria sistematica, non rumore. E
lo stato **esce** dal run: l'installazione viva resta modificata.
Inoltre: (a) il confronto è **body-only** — degli header si legge la sola
prima riga, `Set-Cookie`/`Location`/`Content-Type` non sono mai diffati, e
«BYTE-ID» si legge come identità di risposta; (b) probe 5 è `bytes=0`:
due corpi vuoti coincidono per costruzione — il dente è vacuo, la prova del
login la porta solo il 200 del probe 6; (c) `norm()` è `[0-9a-f]{10}\b`,
**generico di FORMA** mentre il commento sopra dichiara «per NOME, nessun
normalizzatore generico»: maschera qualunque token 10-hex in 142 KB.

## Emendamenti
- **A-PP-79** — *Identità giudicata, non registrata*: ogni harness di misura
  confronta lo sha calcolato col pin ATTESO passato in argomento e
  **fail-CLOSED** su divergenza; ricalcolo **dopo l'ultima gamba**, entrambi
  nel receipt; oracle per **sha**, non per `-v`.
- **A-PP-80** — *Nessuna autorità senza identità*: il file citato dal gate
  cifre incorpora il blocco identità; `grade=VERDICT` negato se assente.
- **A-PP-81** — *Ricevuta di provenienza del pin*: sha + HEAD + rustc +
  ricetta di build emessi **dallo script di build**, e probe di determinismo
  (2 rebuild puliti) per `php-server` **e per `phpr`** prima di usare
  S-94.0 come baseline della leva.
- **A-PP-82** — *Giudice di parità in banda*: `pair94.sh` estrae conteggi e
  **SET dei nomi** dai due `.txt` e scrive `PARITY=OK|DIFF` + set nel
  ratios; i `.txt` (o un estratto integrale macchina-prodotto) committati.
- **A-PP-83** — *Confine per-run di battery61*: reset DB + guardia uploads
  **prima di ogni gamba**, ordine **alternato** in un secondo giro,
  header diffati per NOME, `norm()` ancorato a `_wpnonce` con conteggio
  sostituzioni uguale sui due lati, probe 5 giudicato su `Location`.

## Kill-switch
- **KS-PP-96-1**: qualunque numero la cui catena d'identità risalga a un
  binario **senza ricevuta di provenienza riproducibile** è `grade=ADVISORY`
  **meccanicamente**; il gate cifre rifiuta `VERDICT`. Vale anche per `phpr`
  finché A-PP-81 non è eseguito.
- **KS-PP-96-2**: un `.done` che scrive `rc=0` **incondizionatamente**
  (`pair94.sh:67`) è una ricevuta che non sa dire NO: vietato: il `.done`
  porta l'rc reale di ogni gamba, o il run è NON CONCLUSO.
- **KS-PP-96-3**: nessun claim di parità/accettazione senza il RAW da cui è
  derivato presente a HEAD. Prosa + artefatto untracked = claim ritirato.

## Refutazioni
- **REFUTO** «la coppia gira sul pin invariato» come garanzia: è verificata
  l'identità del file, **non** la sua provenienza — il difetto che ha
  affondato `d45b578` non è stato escluso per `phpr`.
- **REFUTO** «ADVISORY è ciò che Pedersen chiedeva»: chiedevo una
  **ricevuta**, non un'etichetta in prosa senza dente.
- **REFUTO** l'autosufficienza di `pair94.out`: è trascrizione manuale di un
  artefatto untracked, e i suoi conteggi non hanno giudice.
- **REFUTO** «i volatili per NOME» di `battery61.sh`: `norm()` è per FORMA.
- **CONCEDO**: rapporti meccanici, `PHP_CLI_SERVER_WORKERS` nominato,
  failure elencati per NOME, regresso media non attribuito — corretti.
