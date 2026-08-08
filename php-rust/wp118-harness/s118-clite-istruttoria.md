# s118-clite-istruttoria.md — disegno harness contatori C-lite (concilio S-116/S-117 R5; timebox ½ sessione)

**Obiettivo**: contare per ITERAZIONE e per CATEGORIA (sei giudici micro) gli
eventi del ciclo di vita Zval su ENTRAMBI i motori: (a) rc-op = incref+decref;
(b) alloc/free heap. Ordina i vagoni C1/C2 con l'aritmetica: prop 5,9× ⇒
~−45 ns/iter residui, contenibili solo dal lifecycle (concilio, refutazione
Matsakis).

## Gamba phpr (build strumentata, MAI il pin)
- Feature esistenti: `zval-census` (A-ZV2, WP-95) per rc-op; `mem-census`
  (CountingMi, GA_*_N) per alloc — build in target separato
  (`phpr-census-target`), stessa ricetta di `s109-census-run.sh`.
- Output: eventi totali / N_iter (letto dal sorgente del giudice, mai a
  memoria) = rc-op/iter e alloc/iter per categoria.

## Gamba Zend (oracle 8.5.7, MAI ricompilato)
- DTrace sul binario brew: probe `pid$target` su `zend_gc_addref`/`delref` se
  esportati, altrimenti su `_emalloc`/`_efree` (alloc) — da verificare con
  `dtrace -l` sul processo; fallback: build php-src locale con
  `--enable-dtrace` SOLO come strumento di conteggio (dichiarata: motore di
  conteggio, mai oracle di parità/tempo).
- Vexp (php-8.5.7) per la mappa statica dei siti incref/decref nei handler
  degli opcode caldi delle sei categorie (ZEND_ASSIGN_OBJ, FETCH_OBJ_R, …).

## Regole di misura
- Solo CONTEGGI: nessuna cifra di tempo esce dalle build census (REGOLE §3).
- Stesso .php per i due motori; N_iter dal sorgente; rapporto rc-op
  phpr/Zend per categoria = la CLASSIFICA che ordina i vagoni.
- Verdetto: tabella 6×4 (rc-op/iter, alloc/iter × 2 motori) + delta assoluto
  phpr−Zend per iter; i vagoni C1 si pescano dalle categorie col delta
  maggiore.

## Stato S-118
Disegno committato PRIMA dell'esecuzione; esecuzione nel timebox se la sera
lo consente, altrimenti S-119 (dichiarato nel report).
