# Verbale sedia 8 — Stogov (engine Zend / fedeltà oracle) — WP-98

## VERDETTO

S-96.0 è metodologicamente sana e la mia refutazione WP-97 è stata accolta nel
punto giusto (perimetro Str, non F2 intero). **Il verdetto del passo 2
sopravvive** — ma non per le ragioni scritte: `would_take_safe_str` non è un
perimetro fedele, è un **TETTO**. La lista `observes_scope` è indicizzata sul
nome **SCRITTO**, e in phpr il nome scritto non è il nome eseguito. Poiché il
perimetro vero è ≤ quello contato, la strada lunga esce ancora più debole: il
«non vince» regge, la cifra che l'ha deciso no. Nessuna banda derivata da questi
conteggi va citata come stima finché A-DS-98-1/2/3 non sono chiusi.

## Emendamenti

- **A-DS-98-1** — `Op::CallNsFallback { name, fallback, argc }` (bytecode.rs:516):
  `renounce()` interroga `observes_scope(name)` e **ignora `fallback`**.
  `namespace X; extract($a);` porta `name = X\extract` → nessuna rinuncia, e a
  runtime esegue `ho_extract` (vm/mod.rs:14879). Consultare ANCHE `fallback`,
  su entrambe le varianti Args. Fixture negativa obbligatoria.
- **A-DS-98-2** — nome risolto a runtime: `CallValue`, `CallValueArgs`,
  `CallNamed`, `CallSpread`, `MakeFcc` non sono name-checked, e `compact`/
  `extract`/`get_defined_vars` sono dispatchati per nome in un match runtime.
  O si rinuncia su questi op, o si **prova con una fixture** (non con un
  ragionamento) che `$f='extract'; $f($a);` non raggiunge lo scope del chiamante.
- **A-DS-98-3** — il canale ARGOMENTI (era A-DS-97-4, ancora fuori dai
  predicati) **non è chiudibile da una lista di nomi, per forma**: l'osservatore
  sta nella discendenza, non nel corpo. Canali: `getTrace()`/`getTraceAsString()`
  di QUALUNQUE Throwable costruito più in basso (default compilato
  `zend.exception_ignore_args=0` ⇒ gli args ci sono); `debug_backtrace()` da un
  callee, da un handler `set_error_handler`/`set_exception_handler`, da una tick
  function, da un callback `ob_start`, da un comparatore `usort`, da uno
  `spl_autoload_register`; **`ReflectionFiber::getTrace()`**, che osserva gli
  args di frame che non contengono `Fiber::suspend`. Rimedio: gli slot PARAMETRO
  escono dal take, oppure si prova che la trace materializza gli args
  indipendentemente dagli slot.
- **A-DS-98-4** — nomi. Mancano `func_num_args` (frame observer, innocuo qui:
  non legge valori) e va **verificato `$http_response_header`**: i wrapper http
  iniettano un locale per nome nello scope del chiamante da `file_get_contents`/
  `fopen`/`file`/`readfile` — un builtin che SCRIVE lo scope, in nessuna lista.
  Da NON aggiungere, con la ragione a verbale: `$errcontext` (rimosso in 8.0),
  `assert()` con stringa (non valuta più dall'8.0), `ReflectionFunction`,
  `Closure::bindTo` (cambia `$this`/scope, non i locali), `get_object_vars`.
- **A-DS-98-5** — `would_take_safe_str` è fedele **all'output**, non alla
  memoria: prendere lo slot invece di clonarlo abbassa il refcount del buffer e
  sposta il **momento della COW**; `memory_get_usage`, `memory_get_peak_usage`,
  `debug_zval_dump` sono osservatori dell'EFFETTO e nessuna rinuncia li copre né
  deve. Sede: il gate per NOME. Etichettare il campo, non allargare la lista.
- **A-DS-98-6** — §3.10: il perimetro si misura al **SITO**, non al builtin. Un
  contatore `(builtin, tipo in ingresso, strict_types del chiamante)` al punto
  unico di coercizione, dietro `zval-census`, R=1 sul media group. La domanda che
  decide non è «quanti builtin divergono» (5) ma «quante volte accade, e sotto
  `declare(strict_types=1)`» — dove PHP lancia sempre e phpr coercizza sempre.
- **A-DS-98-7** — A-ZV1 è la leva giusta **per forma** (è ciò che fa Zend:
  specializzazione per LOCAZIONE dell'operando, nessun corpo caldo nuovo). Quattro
  trappole: (1) fast path chiuso ai tipi che non rientrano nella VM — `__toString`/
  `__get` invalidano ogni riferimento preso da uno slot; (2) la conversione
  numerica **emette diagnostici**, che rientrano via `set_error_handler`: nessun
  borrow vivo attraverso il punto di emissione; (3) div-by-zero/overflow devono
  lasciare `ip`, `getLine` e la exc_table invariati — verificarlo; (4) prima della
  §Correzione, produrre il **conteggio esatto** della frazione di siti `Binary`
  con ENTRAMBI gli operandi letture di slot: senza, si confronta una predizione
  con una banda, l'errore di grado che design96 §5 ha appena denunciato.

## Kill-switch

- **KS-DS-98-1**: se la fixture di A-DS-98-1 o A-DS-98-2 morde, tutti i conteggi
  F1/F2 di S-95.0 e S-96.0 decadono a TETTO e nessuna banda derivata resta citabile.
- **KS-DS-98-2**: divergenza in `memory_get_usage` senza divergenza d'output ⇒ la
  classe di siti torna al clone. Mai correggere il contatore.
- **KS-DS-98-3**: A-ZV1 si chiude PRIMA di scrivere codice se la frazione
  slot-slot è sotto la predizione — è il criterio che è mancato ad A-ZV2.
- **KS-DS-98-4**: qualunque leva che aggiunga un braccio prima di O1 ⇒ stop
  (A-LB-97-1 insoddisfacibile per costruzione). O1 resta la sola voce che rimette
  in moto il cronometro senza discutere di perimetri: 241,7 KiB in una funzione
  contro ~192 KiB di L1i è il fatto più grosso ancora non attaccato.

## Refutazioni capitali

1. **«`observes_scope` è il perimetro»** — è indicizzata sul nome scritto;
   `CallNsFallback` porta il nome risolto in `fallback` e non lo consulta.
2. **«`would_take_safe_str` è il perimetro fedele»** — è fedele all'output; la
   COW spostata è osservabile.
3. **«§3.10 è una divergenza di testo»** — cambia il FLUSSO, e morde più forte
   sotto `strict_types`, cioè su vendor/, non su WordPress.
