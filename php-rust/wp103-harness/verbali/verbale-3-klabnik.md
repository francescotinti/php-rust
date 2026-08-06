# Verbale Sedia 3 — Klabnik (spec, testabilità, matrici e gate) — Concilio WP-103

**VERDETTO: S-101 REGGE — nessuna refutazione capitale. Quattro emendamenti
vincolanti: la matrice fixture ha buchi NOMINATI, il gate fixture non pinna
il proprio insieme, la carve-out §3.13 è sana ma non portabile, e f9e9f22 è
un precedente di staging da sigillare con una regola meccanica.**

## 1. Matrice fixture vs unione hc-canale WP-102 §4

Verificata voce per voce: specie (01-03), aliasing (04-11), ciclo di vita
(12-13) — l'unione dichiarata è coperta; 04/09/12 lette integralmente, le
attese sono in testa e giuste (oracle-verificabili). MANCANO per NOME:
(a) **giudice di MISURA per specie** (micro `prop-string.php`/`prop-array.php`)
— 02/03 sono semantiche pass/fail, non misurano nulla: H-C1c resta
correttamente chiusa finché non esistono (A-BA-102-3, già a backlog);
(b) **`clone $o` con proprietà refcounted** (+`__clone`): il percorso clone
duplica la tabella proprietà con addref per slot — un under-addref lì è
invisibile a tutte le 13; asse ciclo-di-vita, va aggiunta (fixture 14);
(c) la **finestra del cycle-collector AUTOMATICO**: 13 collauda solo
`gc_collect_cycles()` esplicito; l'«under-noting delays a destructor» sulla
soglia automatica (gc_status/threshold) non è esercitato — copertura parziale
dichiarabile, non silenzio.

## 2. Carve-out §3.13

SANA nella struttura: diff ESATTO via `cmp -s` contro expected, applicato a
ENTRAMBI i modi, mai «non vuoto» — qualunque divergenza nuova cambia il diff
e il gate va ROSSO (fail-closed: non maschera). Due fragilità: (i) l'expected
contiene il **PATH ASSOLUTO** `/Volumes/Extreme Pro/...` — spostare il repo o
cambiare oracle-prefix rompe il gate per ragione spuria, e la risposta
d'istinto («rigenera il diff») è ESSA la mascheratura; (ii) se S-102 punto 6
FIXA §3.13, il gate va rosso senza che l'ordine nomini la conseguenza.

## 3. Ordine e rc dei gate

VERIFICATI sui file, non sui riassunti: sequenza commit 725d5a1→…→f808017 ==
ordine dichiarato; `hc1-fixtures.sh` giudica sui FILE di output e l'rc finale
è `[ $fails = 0 ]` (mai rc di pipe); `corpus-gate/progress.txt` mostra il
gate eseguito su ENTRAMBI gli stadi (0ef9498d e 48a5d438), `.fails` da 1418
righe per NOME, «insieme IDENTICO wp82»; corpus-diff 2 volte con carve-out
nondet nominata (3 settype). Trappole note: non ricorrono. UN difetto di
spec: il gate fixture stampa `fixtures_fail=0` ma **non asserisce N=13** —
se una fixture sparisce dal glob, il gate resta verde con 12. Insieme per
NOME vale anche per le fixture.

## 4. Commit f9e9f22

Confermato da `git show --stat`: il commit «gate TUTTI VERDI» contiene
`reg_lower.rs +63` e `run.rs +36` (codice H-C1b NON ancora gated). I gate
girarono sul tree 5baa369: la DICHIARAZIONE nel commit successivo salva
l'onestà, non la bisecabilità — a f9e9f22 HEAD asserisce gate verdi su un
tree diverso da quello gated. Serve la regola meccanica, non il proposito.

## Emendamenti

- **A-KL-103-1**: un commit di gate-evidence non può toccare `*.rs`: prima
  del commit, `git diff --cached --name-only` senza sorgenti; violazione ⇒
  il commit si spezza.
- **A-KL-103-2**: `hc1-fixtures.sh` asserisce l'INSIEME atteso per NOME
  (13 basename pinnati), non solo `fails=0`.
- **A-KL-103-3**: expected-divergence a path NORMALIZZATO (strip del prefisso
  `$F` via sed su entrambi i lati prima di `cmp`), conservando esattezza su
  righe e testo.
- **A-KL-103-4**: S-102 punto 6, conseguenza NOMINATA: fix §3.13 ⇒ rimozione
  di `09-*.expected-divergence.diff` + fixture 09 byte-identica + chiusura
  della voce a catalogo. Punti 2/3: criterio-file scritto-prima stile
  hc1a/b-criterio.out (banda, pavimento, rinuncia) — punto 3 oggi ha il
  controfattuale (~6 ns) ma NON la clausola di rinuncia — e coppia WP
  nominata per punto (sono cambi di EMISSIONE, regola 2).

## Kill-switch

- **KS-KL-103-1**: nessuna riga H-C1c senza i giudici per specie di
  A-KL-103-... (a) IN-TREE e misurati sui due motori.
- **KS-KL-103-2**: gate fixture che passa con N≠13 (o insieme ≠ pinnato) =
  gate VOID, si rifà.

**Refutazioni capitali: nessuna.**
