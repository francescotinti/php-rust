# Piano di Refactoring Strategico: Segmentazione di `prelude.php`

Questo documento contiene le direttive per Claude AI per lo smembramento del file monolitico `crates/php-runtime/src/lower/prelude.php` (circa 5.050 righe). L'obiettivo è suddividere il codice userland PHP in moduli logici separati, mantenendo inalterata la compilazione e l'esecuzione del runtime.

**Regola d'oro:** Nessun cambiamento alla logica o alle firme delle funzioni PHP. Solo operazioni di spostamento.

---

## 1. Preparazione dell'ambiente
1. Crea una nuova cartella: `crates/php-runtime/src/lower/prelude/`
2. All'interno, prepara i file vuoti corrispondenti ai domini logici principali (vedi sezione successiva).

## 2. Lo Smembramento Logico
Apri l'enorme `prelude.php` e taglia/incolla le classi e interfacce nei seguenti file all'interno della cartella `prelude/`:

- **`core.php`**: `stdClass`, `Exception`, `Error` e tutte le loro derivate (`TypeError`, `ValueError`, ecc.), `PhpToken`, `Fiber`, `WeakReference`, `WeakMap`, `Closure` e le interfacce base (`Stringable`, `Throwable`, `Traversable`, `Iterator`, `ArrayAccess`, `Countable`, `JsonSerializable`, `Serializable`).
- **`spl.php`**: Tutto ciò che riguarda la Standard PHP Library. Iteratori (es. `EmptyIterator`, `RegexIterator`, `RecursiveIteratorIterator`, `FilterIterator`, ecc.), Strutture dati (`SplDoublyLinkedList`, `SplFixedArray`, `SplObjectStorage`), e File (`SplFileInfo`, `SplFileObject`, `DirectoryIterator`).
- **`reflection.php`**: Tutte le classi e interfacce che iniziano con `Reflection` (es. `Reflector`, `ReflectionClass`, `ReflectionMethod`, `ReflectionType`, ecc.).
- **`date.php`**: `DateTimeInterface`, `DateTime`, `DateTimeImmutable`, `DateTimeZone`, `DateInterval` e le relative eccezioni.
- **`pdo.php`**: `PDO`, `PDOException`, `PDOStatement`, `PDORow`.
- **`sqlite3.php`**: `SQLite3`, `SQLite3Exception`, `SQLite3Stmt`, `SQLite3Result`.
- **`dom.php`**: Tutto ciò che riguarda XML/DOM. `DOMException`, `DOMDocument`, `DOMNode`, `SimpleXMLElement`, `XMLReader`, ecc.
- **`session.php`**: `SessionHandler`, `SessionHandlerInterface`, ecc. (se presenti in abbondanza, altrimenti uniscile a `core.php`).

*(Nota: Adatta liberamente il nome dei file se trovi altri grossi cluster logici non listati qui, l'importante è svuotare completamente il file originale).*

## 3. L'Integrazione in Rust (`lower/mod.rs`)
Questa è la parte chiave per evitare problemi di I/O o overhead a runtime. 
Attualmente, `crates/php-runtime/src/lower/mod.rs` include il file così:
```rust
const PRELUDE_SRC: &[u8] = include_bytes!("prelude.php");
```

Sostituisci quella riga sfruttando la potenza della macro `concat!` (che agisce a compile-time stringendo i file). 
Inserisci le inclusioni **nello stesso ordine** in cui si trovavano originariamente in `prelude.php` (specialmente se alcune classi estendono altre classi definite precedentemente, sebbene in PHP userland di solito non importi se dichiarate nello stesso ciclo, è più sicuro).

```rust
const PRELUDE_SRC_STR: &str = concat!(
    include_str!("prelude/core.php"),
    include_str!("prelude/spl.php"),
    include_str!("prelude/reflection.php"),
    include_str!("prelude/date.php"),
    include_str!("prelude/pdo.php"),
    include_str!("prelude/sqlite3.php"),
    include_str!("prelude/dom.php")
    // ...aggiungi altri file se ne hai creati di più
);

const PRELUDE_SRC: &[u8] = PRELUDE_SRC_STR.as_bytes();
```

*(Il file `prelude_ns.php` viene caricato da una riga separata `PRELUDE_NS_SRC` e, essendo piccolo, puoi lasciarlo dov'è, o spostarlo in `prelude/ns.php` aggiornando coerentemente il percorso).*

## 4. Verifica e Testing
1. Assicurati di eliminare il vecchio `prelude.php` una volta svuotato e trasferito.
2. Lancia `cargo check -p php-runtime` per verificare che la macro `concat!` trovi tutti i file.
3. Lancia il corpus Zend completo (`phpt-runner --isolate Zend/tests`). Il risultato **deve essere byte-identico** alla baseline (zero scarti `pass -> fail`). Se ci sono fallimenti, è probabile che l'ordine di caricamento nel `concat!` abbia rotto l'ereditarietà di qualche classe: correggi l'ordine.
4. Commit incrementale per confermare la segmentazione.
