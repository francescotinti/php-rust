# WP_SESSION_45 — Fase 0 roadmap footprint: attribuzione COMPLETATA

> ⚡ **WP-45 (2026-07-24, `36b7c6e`→`b9bcd84`)** — **FOOTPRINT_CPU_ROADMAP
> aperta (piano "concilio" approvato dall'utente: footprint-first,
> safe-only, TUTTE le fasi comunque, NIENTE revert su insuccesso) e
> Fase 0 CHIUSA con verdetto netto: dei ~4,3G di eccesso, ~3,08G
> (⅔ del gap 11,9×) sono GARBAGE CICLICO Rc IRRAGGIUNGIBILE mai
> raccolto — `gc_note` buffera solo Object, i cicli via Ref/Array/
> Closure senza oggetti non diventano MAI cycle-root** (Zend traccia
> anche gli array: per questo sta piatto a 0,4G).

## Strumenti costruiti (feature `mem-census`, misura-only)

- `php-types/src/memcensus.rs`: canali {str, arr, obj, unit} con
  live/peak/cumulative/conteggi; STR esatto (funnel `PhpStr::new` + Drop
  cfg), ARR/OBJ death-accounted (byte esatti al drop + vivi via choke
  clone/default/`next_id`/`free_object_id`), UNIT esatto ai due
  `Box::leak`; watermark dump ogni +128MB proxy + atexit su
  `$PHPR_MEM_CENSUS`; **root-walk** `deep_size` (dedup per Rc-ptr,
  depth-cap 2000) + `report_roots`.
- Root-walk nel Vm pre-shutdown (vm/mod.rs, prima di
  `run_shutdown_functions`): superglobals · globals(main-frame) ·
  constants · fn-statics · static-props · closure-statics ·
  generator-frames · created · gc_buf · **unit-consts** (walk dei const
  pool di tutti i moduli linkati).
- `PHPR_GC_THRESHOLD_MAX` (env, letto once, ramo freddo; default =
  costante 1e9): cap dell'escalation adattiva WP-21 per esperimenti.
- Build census SEPARATO: `CARGO_TARGET_DIR=phpr-mem-target cargo build
  --release -p php-cli --features mem-census` → binario
  `phpr-mem-target/release/phpr-memcensus`. Il binario di parità non è
  mai toccato. cargo normale 1637/0 ✔.

## I numeri (gruppo media, 762 test; peak esterno stabilissimo 4,67-4,78G vs oracle 0,39-0,40G = 11,9×)

1. **Day-zero**: `/usr/bin/time -l` riporta GIÀ "peak memory footprint"
   (fisico vero) → 18 run WP-44 riesumate: 11,9× STRUTTURALE, non MADV.
   `MIMALLOC_PURGE_DELAY=0`: peak invariato ⇒ non è retention allocatore.
2. **Include** (PHPR_LOG=debug, zero-codice): 12.393 eventi, 1.993
   distinti, **0 unit-cache hit** (il fingerprint di catena avanza a ogni
   load ⇒ il cache è solo cross-Vm); ma il modello torna con 15
   subprocess × ~693 file di bootstrap ⇒ ridondanza intra-master ~50
   eventi = il "template leak" è PICCOLO in conteggio (Fase 0.5: non
   necessaria come fix di leak; resta il canale CPU cross-process, pari
   condizione con l'oracle CLI senza opcache).
3. **Canali a fine run (master)**: str **1,29G** (23,8M vive, monotone
   0,15→1,29) · arr **~1,90G** (3,11M vive, monotone) · obj 0,045G
   (140k vivi; 2,1M morti regolarmente durante la run) · unit **0,30G**
   (2.046 moduli, mai liberati by design). Totale censito 3,53G = 74%
   del peak (residuo ~1,2G = rounding size-class non campionato +
   tabelle Vm/ob/session non censite).
4. **GC**: collects **1** (roots 50.000, **freed 1**), threshold_last
   100k col cap; cap 100k = numeri IDENTICI ALLA CIFRA (3.113.063 vs
   3.113.062 array) ⇒ il collector gira e non trova nulla: i root che
   vede sono vivi VERI.
5. **Root-walk (la prova regina)**: root PHP-visibili = **149,3MB
   TOTALI** (created 115,5 + static-props 31,2 + globals 2,0 +
   unit-consts 1,6 + resto <1). ⇒ **3,53G − 0,149G − 0,30G(unit) ≈
   3,08G di grafo IRRAGGIUNGIBILE** = cicli Rc mai registrati.

## Tabella decisionale (compilata)

| canale | GB | quota eccesso | azione |
|---|---|---|---|
| cicli fluttuanti Ref/Array/Closure | ~3,08 | ~72% | **WP-46: root-tracking esteso (modello Zend)** |
| non censito (rounding, tabelle Vm) | ~1,2 | ~28% lordo | campionare mi_usable_size dopo WP-46 |
| unit ritenute | 0,30 | ~7% | shrink_to_fit + Box<[T]> + drop HIR seeds (Fase 1.1) |
| stato PHP legittimo | 0,15 | ~3,5% | niente |

## ⭐⭐ Lezioni

- **⭐⭐ Il gap footprint NON è overhead per-struttura ma LIFETIME**: la
  crescita è monotona e test-driven; l'ipotesi "created pinna tutto" e
  l'ipotesi "soglia GC spenta" sono state entrambe FALSIFICATE dai dati
  prima di scrivere una riga di fix (metodo: strumento nuovo per ogni
  ipotesi, mai fix alla cieca).
- **⭐⭐ `gc_note` object-only = classe intera di cicli invisibile al
  collector**: Ref-cell cycles (`$a[] = &$a`), array condivisi in cicli,
  closure che si catturano — Zend li traccia (possible roots = array E
  oggetti). È il difetto di design n.1 del GC phpr.
- ⭐ `/usr/bin/time -l` su macOS espone "peak memory footprint": il
  protocollo vmmap serve solo per snapshot intermedi.
- ⭐ Il unit-cache non può MAI fare hit intra-Vm (fingerprint di catena
  avanza a ogni load): è un replay-cache cross-processo, non un opcache.
- ⭐ Root-walk con dedup per Rc-ptr e figli CLONATI per gli Object
  (guardie RefCell) = pattern riusabile; unit-consts walkabili dai
  moduli linkati.

## Prossimo (WP-46, la roadmap continua — direttiva: tutte le fasi, no revert)

**Estendere il cycle-collector ai root non-oggetto** (Ref-cell e Array
condivisi; valutare Closure): buffering/demote/collect per contenitori
alla Zend, con sentinelle dtor-order pinnate PRIMA e gate famiglia gc/
dtor del corpus. Bersaglio: ~3G. In coda: Fase 1.1 (shrink unit ~0,3G),
disciplina di confine per-test (`mi_collect` + drain pool), interning se
il censimento duplicati lo giustifica. CPU track (RET_DEREF+ret_shape,
Sweep elision) resta in Fase 2.
