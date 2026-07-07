# Execution Plan for Claude: Implementazione Nativa di `mysqli` in `phpr`

Ciao Claude! Questo è un piano di esecuzione strategico generato per te. Il tuo compito è implementare l'estensione nativa `mysqli` all'interno dell'engine `phpr` (scritto in Rust). Questo è un passo critico (la "Market Maker milestone") per poter far girare WordPress in modo nativo e performante.

## 🎯 Obiettivo
Creare un'implementazione Rust nativa dell'estensione `mysqli` (sia l'interfaccia Object-Oriented che quella procedural) per evitare l'overhead della FFI con la libreria C originale. L'implementazione si affiancherà a quella già esistente di `PDO` (`php-runtime/src/vm/pdo.rs`).

## 📚 Contesto Architetturale
- Il progetto usa `Rc` + `RefCell` (cow) per la gestione della memoria, non puntatori raw.
- Le istanze degli oggetti (es. un'istanza della classe `mysqli`) possono contenere un "payload" nativo Rust.
- Esiste già un precedente: guarda come è stato implementato PDO in `php-runtime/src/vm/pdo.rs` (che wrappa `rusqlite`). Dovrai usare un pattern identico per `mysqli`.

## 📦 Dipendenze
Aggiungi il crate Rust ufficiale per MySQL nel `Cargo.toml` di `php-runtime`.
```toml
# In php-rust/crates/php-runtime/Cargo.toml
mysql = "24.0" # Usa la versione stabile più recente (sincrona per ora, in linea con l'attuale architettura della VM)
```

---

## 🛠️ Piano di Implementazione (Step-by-Step)

### Step 1: Bootstrap del Modulo
1. Crea il file `php-rust/crates/php-runtime/src/vm/mysqli.rs`.
2. Registralo in `php-rust/crates/php-runtime/src/vm/mod.rs` (`mod mysqli;`).
3. Crea la funzione di registrazione iniziale `pub(super) fn register(vm: &mut Vm)` che inietterà classi e funzioni globali nell'engine al boot.

### Step 2: Implementazione dello Stato Interno (Native Payload)
1. Definisci una struct Rust per mantenere la connessione MySQL aperta:
   ```rust
   pub struct MysqliConnection {
       pub conn: std::cell::RefCell<mysql::Conn>,
   }
   // Implementa il trait necessario (es. `NativeObject` o simile usato in pdo.rs) per far sì che possa essere wrappato in uno Zval object.
   ```

### Step 3: Registrazione della Classe `mysqli` (OOP Interface)
1. Usa le API interne della VM (guarda `pdo.rs`) per registrare la classe `mysqli`.
2. Implementa il costruttore `__construct(string $hostname, string $username, string $password, string $database, int $port = 3306)`:
   - Fai il parsing degli argomenti Zval.
   - Costruisci la stringa di connessione (DSN/Opts) per il crate `mysql`.
   - Inizializza `mysql::Conn::new(opts)` e salvalo nel payload dell'oggetto corrente.
3. Implementa i metodi principali:
   - `query(string $query, int $resultmode = MYSQLI_STORE_RESULT): mysqli_result|bool`
   - `real_escape_string(string $string): string`
   - `close(): bool`

### Step 4: Implementazione della Classe `mysqli_result`
Quando `query()` esegue una `SELECT`, deve ritornare un'istanza di una nuova classe `mysqli_result`.
1. Definisci il payload Rust per `mysqli_result` (che terrà traccia del cursore `mysql::QueryResult` o del vettore di righe restituite in memoria).
2. Registra la classe `mysqli_result` e implementa i metodi:
   - `fetch_assoc(): array|null` (mappa il risultato Rust in uno `Zval::Array` / `PhpArray`).
   - `fetch_object(): object|null` (mappa il risultato in uno `Zval::Object` generico, classe `stdClass`).
   - `fetch_row(): array|null` (array indicizzato).

### Step 5: Implementazione dell'Interfaccia Procedurale (Procedural API)
WordPress usa prevalentemente la versione procedurale. In PHP, `mysqli_connect()` è di fatto un alias di `new mysqli()`.
1. Registra le funzioni globali nell'ambiente:
   - `mysqli_connect(...)` -> Ritorna un'istanza della classe `mysqli`.
   - `mysqli_query($link, $query)` -> Chiama il metodo `query` sull'oggetto `$link`.
   - `mysqli_fetch_assoc($result)` -> Chiama il metodo `fetch_assoc` sull'oggetto `$result`.
   - `mysqli_close($link)` -> Chiama il metodo `close`.
   - `mysqli_error($link)` / `mysqli_insert_id($link)`.
2. *Suggerimento per Claude:* Cerca di mappare le funzioni procedurali direttamente ai metodi OOP, estraendo l'oggetto `mysqli` dal primo argomento passato alla funzione.

### Step 6: Testing
1. Crea uno script PHP di test (es. `test_mysqli.php` nella root) per validare la connessione, una INSERT e una SELECT.
2. Controlla i test ufficiali Zend in `Zend/tests/ext/mysqli` (se disponibili nel repo) e verifica quanti riescono a passare con questa prima implementazione.

---

## ⚠️ Regole d'oro per Claude durante l'esecuzione:
1. **Guarda PDO come Stele di Rosetta:** Usa `php-runtime/src/vm/pdo.rs` come riferimento principale. Il modo in cui converte i tipi Rust in `Zval` e come registra i metodi è la *best practice* del progetto.
2. **Niente Panico:** Converti sempre gli errori del crate `mysql` (es. connessione fallita) in `Zval::False` o in eccezioni `mysqli_sql_exception` di PHP. Non usare mai `unwrap()` o `expect()` nei path raggiungibili dall'utente, altrimenti farai crashare l'intera VM `phpr`.
3. **Type Juggling:** Quando estrai valori dal database, fai attenzione ai tipi. I campi INT di MySQL devono diventare `Zval::Int`, le stringhe `Zval::String`. Usa la libreria `php-types` del progetto.

Buon lavoro Claude! Comincia analizzando `php-rust/crates/php-runtime/Cargo.toml` e `pdo.rs`.
