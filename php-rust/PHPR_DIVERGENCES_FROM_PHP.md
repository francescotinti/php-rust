# phpr — Divergenze note rispetto a PHP standard (8.5.7)

> Catalogo vivo delle **anomalie** di `phpr` rispetto al PHP di riferimento
> (oracle: PHP 8.5.7). Ogni voce è un punto in cui phpr **non** riproduce
> byte-per-byte il comportamento dell'interprete C, oppure lo riproduce solo
> parzialmente. Serve come mappa per rientrarci in modo mirato.
>
> Principio guida del progetto: **correct-or-absent** — uno stub che mente è
> peggio di una funzione assente. Molte voci qui sotto sono "assenze
> consapevoli" o "divergenze circoscritte", non bug silenziosi.
>
> Ultimo aggiornamento: 2026-07-09 (Sessione VII stdlib Tier-A).

---

## 1. Gap trasversali dell'architettura builtin (impattano molti phpt "di tipo")

Questi sei gap nascono tutti dalla stessa radice architetturale: le funzioni
del crate **`php-builtins`** sono pure (`fn(args, ctx) -> Result<Zval, PhpError>`)
e **non hanno accesso allo stato della VM**. Non possono quindi rientrare nel
motore per invocare metodi utente, generare backtrace "veri", o consultare lo
stato ZPP dell'engine. Limitano principalmente i **phpt di edge-case/di tipo**,
non l'uso reale delle webapp target (WP/Composer/Doctrine/Laravel/Symfony).

### 1.1 Coercion di oggetti `Stringable` nelle builtin pure  ✅ IN GRAN PARTE CHIUSO (`6f7cb31`+`f1fee67`)
- **Era il gap più frequente**: una builtin che coerce un argomento a stringa
  non invocava `__toString()` (il crate `php-builtins` è VM-stateless), emettendo
  un warning spurio "Object of class X could not be converted to string".
- **Meccanismo implementato** (rispecchia il precompute `__debugInfo` di
  `var_dump`): `Ctx` ha una mappa `stringify: &HashMap<u32, ZStr>` (id oggetto →
  risultato `__toString` precomputato) + helper `Ctx::to_zstr(&Zval)`. La VM,
  **prima del dispatch** e SOLO per le builtin che coercono *incondizionatamente*
  (gate `ref_builtin_string_coerces` / `value_builtin_string_coerces`), invoca
  `__toString` via `resolve_method_runtime`+`call_method_sync` (`Vm::compute_stringify`,
  no ricorsione negli array annidati, cycle-guard per id). Le builtin chiamano
  `ctx.to_zstr` invece di `convert::to_zstr`.
- **Coperti**: `natsort`/`natcasesort` (by-ref) + ~28 value builtin via
  `string::str_at` (str_contains/starts_with/ends_with, substr_*, add(c)slashes,
  strtr, wordwrap, levenshtein, htmlspecialchars/htmlentities, strip_tags, …).
  Byte-identici all'oracle; introspection (is_string/gettype/get_class/var_dump)
  **esclusi** → nessuna chiamata `__toString` spuria (verificato). Zend 2322→2323.
- **Residuo** (non ancora coperto): funzioni con oggetti dentro ARRAY-arg
  (`implode`, `str_replace` con array, `array_map`→callback), e i format
  (`sprintf`/`printf` %s). Estendere allowlist + far ricorrere `compute_stringify`
  negli array per quei casi specifici. `to_zstr` VM-side (echo/concat) già OK da prima.

### 1.2 Deprecation ZPP `null → parametro non-nullable`
- **Sintomo**: passare `null` a un parametro interno non-nullable (es.
  `strlen(null)`) deve emettere `E_DEPRECATED` da PHP 8.1. phpr **non** emette la
  deprecation.
- **Radice**: la validazione ZPP di phpr non modella la distinzione
  "null implicito coercibile ma deprecato".

### 1.3 `#[SensitiveParameter]` non onorato
- **Sintomo**: i parametri marcati `#[SensitiveParameter]` devono comparire come
  `Object(SensitiveParameterValue)` nei backtrace/messaggi d'errore. phpr mostra
  il valore reale.
- **Impatto**: sicurezza/diagnostica; nessun impatto funzionale.

### 1.4 Validazione ZPP dei callable "upfront"
- **Sintomo**: le funzioni che accettano un `callable` devono validarlo **prima**
  di eseguire il corpo (ZPP). phpr in alcuni casi valida tardi, quindi l'ordine
  e il testo dei `TypeError` divergono in edge-case.

### 1.5 Location di `ArgumentCountError` da callback invocati internamente
- **Sintomo**: quando una builtin invoca un callback utente con troppi pochi
  argomenti, il file/linea riportati nell'`ArgumentCountError` non coincidono
  con quelli dell'oracle (che punta al sito inline del callback).

### 1.6 Preservazione degli elementi **by-ref** nei risultati array
- **Sintomo**: alcune funzioni array che in PHP restituiscono/mantengono
  riferimenti agli elementi (es. propagazione di reference dentro array
  risultato) in phpr producono copie. Diverge solo negli scenari con reference
  espliciti negli elementi.

---

## 2. Assenze consapevoli (funzioni non implementate per mancanza di infrastruttura)

Non sono bug: sono funzioni **volutamente assenti** perché un'implementazione
fedele richiede stato/infra non ancora presente, e uno stub violerebbe
correct-or-absent.

| Funzione | Perché è assente / diverge | Cosa servirebbe |
|---|---|---|
| `get_defined_constants` | `resolve_constant` è un `match` non enumerabile: non esiste un registro iterabile delle costanti host | Registro costanti iterabile (host + estensioni) |
| `parse_ini_file` / `parse_ini_string` | Il parser INI di PHP è un lexer flex con semantica di coercizione (NORMAL/TYPED) ed edge-case; difficile essere byte-identici | Port fedele del lexer INI + tabella coercizioni |
| `get_include_path` / `set_include_path` | `include_path` come stato scope-aware non è modellato | Stato `include_path` nella VM + interazione con lo stream wrapper |
| `preg_last_error` / `preg_last_error_msg` / `preg_filter` | Nessuno stato d'errore PCRE globale esposto dal motore regex | Ponte allo stato d'errore dell'engine PCRE |
| `get_cfg_var` / `ini_get_all` | Nessun registro INI reale (in CLI la maggior parte darebbe `false`) | Tabella INI runtime |
| `timezone_identifiers_list` | ~348 nomi statici legati alla versione tzdata; rischio di divergere dalla tzdata dell'oracle | Dataset tz allineato alla build oracle |
| `getimagesize` (formati rari + out-param) | Implementati GIF/JPEG/PNG/BMP/WebP; mancano formati rari e il parametro `&$image_info` | Parser per formati residui + supporto out-param |
| `opcache_*` | Nessun opcache | (fuori scope) |

---

## 3. Divergenze di engine circoscritte (documentate nei topic-file di memoria)

| Area | Divergenza | Nota |
|---|---|---|
| Chiamate dinamiche | 5 test Zend "Cannot call X dynamically" non rifiutati | manca il reject per alcune funzioni non chiamabili dinamicamente |
| `extract` | `EXTR_REFS` non supportato | il resto dei flag EXTR_* è fedele |
| PDO/sqlite UDF | Le User-Defined Function SQLite sono deferite | richiedono re-entrancy della VM dentro il callback rusqlite |
| `FETCH_CLASS` protected / `PDORow` / `FETCH_LAZY` | modalità PDO fetch residue | deferite |

### 3.1 Divergenze delle tabelle di encoding (codec mbstring)
Il codec mbstring di phpr usa `encoding_rs` per gli encoding non gestiti a mano
(UTF-8/ASCII/Latin-1/UTF-16 sono diretti). Alcune **tabelle di conversione**
differiscono da quelle di libmbfl, e alcuni encoding non sono mappati. Questo
impatta ogni `mb_*` che decodifica/ricodifica (`mb_convert_encoding`,
`mb_encode_numericentity`, …), **non** la logica delle singole funzioni.

| Encoding | Divergenza | Esempio |
|---|---|---|
| `ISO-2022-JP` | `encoding_rs` decodifica il segno di sterlina (`!r`) in `U+FFE1` (fullwidth) invece di `U+00A3` (regola libmbfl) | `mb_encode_numericentity` test #11 |
| `UCS-4` / `UCS-4LE` / `UCS-2` … | non presenti in `resolve_encoding` → `ValueError "must be a valid encoding"` | `mb_decode_numericentity` test (linea 54) |
| `SJIS`/`EUC-JP` (casi rari) | possibili scostamenti di mapping su codepoint di confine | (potenziale) |

Nota: la **logica** di `mb_encode_numericentity`/`mb_decode_numericentity` è
byte-identica all'oracle (convmap, offset/mask, overflow, `;` opzionale,
pass-through) — verificata su tutte le asserzioni edge-case dei phpt, che
riportano `(Good)`. Gli unici fail residui sono queste tabelle di encoding.

---

## 4. Punti di forza da NON toccare (invarianti verificati byte-identici)

Per evitare regressioni, questi comportamenti sono **già** byte-identici con
l'oracle e vanno preservati:

- **`mt_rand` / `mt_srand`**: bit-esatti con il Mt19937 di PHP. Di conseguenza
  ogni builtin RNG-based (`array_rand`, `str_shuffle`, `shuffle`, …) è
  byte-identica **dopo il seed** via `crate::math::mt_range`. Non reimplementare
  l'RNG.
- **`strnatcmp` / `strnatcmp_ex`**: comparazione naturale fedele (riusata da
  `natsort`/`natcasesort`).
- **hashing** (`md5`, `sha1`, `md5_file`, `sha1_file`, `password_hash` bcrypt):
  digest byte-identici.

---

## 5. Come si verifica una divergenza (procedura)

1. **Probe oracle**: eseguire lo stesso snippet con
   `~/Claude/php-oracle/php-src/sapi/cli/php` (o brew `/opt/homebrew/opt/php/bin/php`).
2. **Leggere il C**: fonte esatta in `php-8.5.7/ext/**` (via Read/Vexp, non grep).
3. **Diff byte-per-byte** stdout+stderr phpr vs oracle.
4. **Gate corpus Zend** (`phpt-runner --isolate`) confronto per NOME dei blocchi
   `^---`: zero regressioni obbligatorio.
5. Se non si può essere fedeli → **lasciare assente** e annotare qui.

---

### Changelog di questo documento
- 2026-07-09: creazione. Catalogati i 6 gap trasversali builtin, le assenze
  consapevoli Tier-A, le divergenze di engine circoscritte, gli invarianti
  byte-identici.
