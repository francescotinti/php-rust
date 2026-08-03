# Verbale sedia 8 — Stogov (Zend/opcache, semantica engine) — Concilio WP-95

**VERDETTO: CON EMENDAMENTI.** La leva per-file è legittima e ha un omologo
Zend; ma il rischio nominato dal mandato (l'ordine globale di hoist) NON è il
rischio vero, e la cifra proiettata in huge-sites.out è sbagliata di due
ordini di grandezza.

## Q1 — Osservabili dipendenti dall'ordine di hoist

**Refuto il pericolo come inquadrato.** Con per-file l'ordine RELATIVO dentro
ciascuna tabella non cambia: classi f1,f2,… e funzioni f1,f2,… restano in
ordine di file (oggi il loop funzioni itera il concat nello stesso ordine).
`get_declared_classes()`/`get_defined_functions()` e i class-id sono stabili
PER COSTRUZIONE; cambia solo l'interleaving fra tabelle e gli slot statics
(non osservabili PHP). Verificato con audit vivo (oracle, token_get_all sui 9
file): **zero riferimenti extends/implements cross-file in avanti** (core 52
decl, spl 28, reflection 25, …, tidy 0); zero trait; enum solo in core.php.
Il rischio VERO è la sentinella `b"prelude"` (vedi Q3). Oracle verificato:
classi builtin → `getFileName`/`getStartLine`===false, `isInternal`===true;
ordine `get_declared_classes` = ordine di registrazione engine.

Fixture oracle-morse PRIMA del codice (giudice = baseline phpr d5ce86e3 per
F1-F3/F8; oracle vivo per F4/F5/F7):
- **F1-F3**: liste INTERE `get_declared_classes/interfaces/functions` + count, byte-id pre/post.
- **F4**: una classe campione PER file (Exception, ArrayObject, ReflectionClass, DateTime, PDO, SQLite3, DOMDocument, SessionHandler + funzione tidy): getFileName===false, getStartLine===false, isInternal===true.
- **F5**: eccezione lanciata DA codice preludio: `getFile()/getLine()/getTraceAsString()` — nessun `prelude:<riga>` affiora (oracle: call-site utente).
- **F6**: autoloader-logger: zero invocazioni per nomi del preludio.
- **F7**: "Cannot redeclare" su classe preludio: messaggio senza file/riga (vm/mod.rs:6598).
- **F8** (unit Rust): snapshot name→id di class_index/fn_index invariato; falsifier A-AH22 esteso: mutare UNO QUALSIASI dei 9 file muove main_chain_fp.

## Q2 — Precedente Zend

Zend NON ri-parsa mai gli stub per thread: classi interne registrate a MINIT
una volta per processo; opcache preload = classi `ZEND_ACC_IMMUTABLE` in SHM,
mutabile (statics, runtime cache) per-request via MAP_PTR. **Rank**: per-file
= palliativo del solo picco transiente, giusto PRIMO passo (safe-only);
l'omologo Zend è (b) tabelle immutabili condivise per-processo con statics
per-thread — attacca anche lo slope ~18,8 MB/worker (il clone per-thread di
LoweredPrelude); oltre, (c) preludio precompilato a build-time (≈ file cache)
elimina anche il parse CPU. Il fronte NON si chiude su per-file.

## Q3 — Numeri di riga / File ephemeral

La sentinella è UGUAGLIANZA AL BYTE `f == b"prelude"` in ~20 siti portanti:
host_reflect.rs:1511 (getFileName→false), vm/mod.rs:853 (**take_while sul
PREFISSO** delle funzioni condivise), :13221 (soppressione frame backtrace),
:16793 (relocation), :6598/12145 (redeclare), calls.rs:1324 (TypeError),
host.rs:3014 (internal in get_defined_functions), hir.rs:203, bytecode.rs:1769.
Le 6 unità extra usano GIÀ `b"prelude"` con righe che ripartono da 1: il
per-file non introduce una classe nuova di osservabile, PURCHÉ ogni unità
mantenga il nome esatto. Le righe non affiorano da Reflection (sentinella →
false); affiorano solo via F5. Gate: refl 290 + corpus 1418 per NOME + F4/F5.

## Q4 — Priorità S-94.0 (FONDAMENTALI-first)

1. Fixture F1-F8 + checker cross-file committati PRIMA del codice; 2. leva
per-file col contatore per-file (predizione-misurata WP-48); 3. gate parità
completi + battery-91pre (MAI girata — la ricompila la fa scattare) stessa
sessione; 4. misura hello CLI pre/post + slope W{1,4}. battery61 resta debito
separato (mezza sessione), non caricarlo sulla leva.

## Emendamenti

- **A-DS-66**: nome unità `b"prelude"` IDENTICO AL BYTE per ognuno dei 9 file + unit test che lo asserisce; in alternativa predicato unico migrato in TUTTI i ~20 siti — mai a metà.
- **A-DS-67**: fixture F1-F8 oracle-morse committate prima del codice (stile A-DS51).
- **A-DS-68**: checker cross-file forward-ref come gate pre-nascita; scope dichiarato (extends/implements; NON copre const-expr cross-file né attributi — estendere se l'hoist li risolve eager).
- **A-DS-69**: ri-quantificare il picco per-file col contatore PRIMA del claim: "peak arena = max file ~74 KB" confonde sorgente con arena (rapporto misurato ~97×, 39.534.144 B / ~406 KB): attesa ~5-8 MB, non 74 KB. Variante da misurare: una arena + `reset()` fra i file (bumpalo trattiene il chunk maggiore, niente ri-raddoppio).
- **A-DS-70**: rank leve a verbale: per-file → condiviso per-processo → precompilato build-time; nessuna chiusura di fronte su per-file (legge no-front-closure).

## KS

- **KS-DS-95-1**: unità rinominata (≠ `b"prelude"`) senza migrazione dei siti sentinella → STOP.
- **KS-DS-95-2**: codice della leva scritto prima delle fixture F1-F8 → STOP.
- **KS-DS-95-3**: claim di risparmio pubblicato senza contatore per-file predizione-vs-misurato → solo ADVISORY.

## Refutazioni capitali: SÌ

1. La cifra proiettata in `wp93-harness/huge-sites.out` ("peak arena = max
   file ~74 KB") scambia byte di SORGENTE per byte di ARENA: fattore ~97×.
2. Il rischio "cambia l'ordine globale" è REFUTATO come pericolo primario:
   l'ordine per-tabella è invariante per costruzione e l'audit cross-file è
   PASS; il pericolo reale è la sentinella `b"prelude"`.
