# Architettura delle Estensioni PHP in Rust

Una delle sfide più colossali nella riscrittura di un linguaggio è la gestione del suo ecosistema di estensioni. Storicamente, estensioni come `curl`, `mysqli` o `imagick` sono dei wrapper leggeri scritti in C attorno a librerie di sistema C (es. `libcurl`, `libmysqlclient`, `libmagickwand`).

Se in `php-rust` provassimo a usare FFI (Foreign Function Interface) per parlare con le librerie C originali, distruggeremmo i tre vantaggi principali di Rust: **memory safety**, **asincronia pura**, e **distribuzione single-binary** (perché torneremmo ad avere dipendenze `.so`/`.dll` di sistema).

Ecco la strategia e la roadmap per re-implementare l'ecosistema.

---

## 1. La Strategia: "Rust-Native First" e Architettura a Crate

L'approccio vincente si basa su due pilastri fondamentali:

### Pilastro A: Modelli a Crate (L'Ecosistema Estensioni)
Invece di appesantire il core (`php-builtins`), la soluzione migliore è abbracciare un'**architettura modulare a crate**, rispecchiando esattamente come PHP gestisce le sue estensioni. Creeremo nel workspace una serie di crate dedicati:
- `crates/php-ext-curl`
- `crates/php-ext-mysqli`
- `crates/php-ext-imagick`

Questo garantisce un disaccoppiamento totale: chi compila il runtime potrà decidere tramite i `features` di Cargo quali estensioni includere nel binario finale, mantenendolo leggero. Ogni crate esporrà una funzione di "registrazione" (es. `php_ext_curl::register(&mut Registry)`) che inietterà i propri built-in nel motore.

### Pilastro B: Abbandonare le librerie C legacy
Il secondo pilastro è fare da "layer intermedio" (wrapper) verso i crate Rust nativi più famosi e testati, abbandonando le vecchie librerie C.
- Il codice PHP rimane **identico** (stesse funzioni, stesse costanti).
- Il crate (es. `php-ext-curl`) intercetta la funzione PHP e la "traduce" in chiamate al crate Rust (`reqwest`).
- **I disallineamenti 1-a-1**: Inevitabilmente, l'API del crate Rust sarà diversa da quella C. Il crate dell'estensione servirà proprio per "fingere" il comportamento originale, ignorando le feature legacy non mappabili (generando un `E_NOTICE`) o colmando le differenze.

---

## 2. Specifiche per Estensione (I Big 3)

### A. cURL (`ext-curl`)
- **Crate Consigliato**: `reqwest` (lo standard de facto per l'HTTP in Rust).
- **Implementazione**: 
  - `curl_init()` alloca una nuova `Zval::Resource` contenente un builder di `reqwest::Client`.
  - `curl_setopt()` è la vera sfida: cURL in PHP ha un centinaio di costanti `CURLOPT_*`. Il layer intermedio dovrà usare un enorme `match` per tradurre queste costanti in logica `reqwest` (es. `CURLOPT_HTTPHEADER` si traduce in `reqwest::RequestBuilder::header()`).
  - **Disallineamenti**: `reqwest` non supporta alcuni protocolli oscuri di `libcurl` (es. gopher, telnet) o flag storici. Se lo sviluppatore passa un flag non supportato, il wrapper Rust restituirà `true` ma lo ignorerà (o emetterà un warning), preservando il 99% degli script moderni.
  - `curl_exec()` chiamerà l'invio asincrono tramite `tokio`, sfruttando i Fiber parcheggiati della nostra VM.

### B. Database (MySQLi / PDO)
- **Crate Consigliato**: `sqlx` (asincrono, compile-time checked, puro Rust) o `mysql_async`.
- **Implementazione**:
  - PHP usa il concetto di connessione e statement preparati. Con `sqlx`, il layer creerà un pool di connessioni nativo in Rust gestito dalla VM.
  - Quando lo script chiama `mysqli_query()`, la stringa SQL viene passata a `sqlx`. Dato che `sqlx` restituisce i dati in modo fortemente tipizzato, il layer di Rust ispezionerà il tipo delle colonne nel DB e costruirà un `PhpArray` (l'array associativo in PHP) castando interi, float e stringhe nei corrispettivi `Zval`.
  - **Vantaggio folle**: Usando un crate nativo asincrono, PHP potrà fare 100 query parallele al DB senza bloccare mai il thread del server!

### C. Elaborazione Immagini (GD / ImageMagick)
- **Crate Consigliato**: `image` (puro Rust, eccellente per manipolazioni classiche come ridimensionamento/crop) oppure binding FFI come `magick-rust` se si vuole il subset completo di Imagick.
- **Implementazione GD**: GD (`imagecreatefromjpeg`, `imagecopyresampled`) è perfetto per essere re-implementato usando il crate `image`. Il layer intermedio nasconderà la conversione dei pixel e le astrazioni matematiche. Poiché il crate `image` non richiede librerie C esterne, il runtime PHP rimarrà un singolo eseguibile!

---

## 3. L'Architettura del "Layer Intermedio" (Opaque Resources)

Come fa lo script PHP a mantenere il "puntatore" a una struttura dati complessa di Rust (come un `reqwest::Client` o una connessione MySQL)? 

In PHP esiste il tipo `Resource`. All'interno della VM `php-types`, dovrete estendere `Zval` per supportare **Risorse Opache**:
```rust
pub enum Zval {
    Int(i64),
    String(Rc<String>),
    Array(Rc<RefCell<PhpArray>>),
    Object(Rc<RefCell<Object>>),
    // La novità: un wrapper generico per le estensioni
    Resource(Rc<RefCell<dyn std::any::Any>>), 
}
```
In questo modo, quando `curl_init()` viene invocato, restituirete al codice PHP uno `Zval::Resource` contenente l'oggetto Rust. Quando il programmatore fa `curl_exec($ch)`, il built-in in Rust farà il downcast di `$ch` per recuperare il proprio oggetto nativo ed eseguire l'operazione.

---

## 4. Roadmap di Sviluppo delle Estensioni

Per governare questa mole di lavoro, l'implementazione andrebbe scaglionata in questo modo:

### Tier 1: Le Estensioni "Facili" (Manipolazione Dati)
Si possono fare subito, la logica pura in Rust esiste già e non richiedono I/O asincrono.
- **`ext-json`**: `json_encode` / `json_decode` mappati direttamente usando il potentissimo crate `serde_json`.
- **`ext-mbstring`**: Usando crate per l'encoding e la manipolazione UTF-8.
- **`ext-hash`**: Usando il crate `rust-crypto` (sha1, md5, bcrypt).

### Tier 2: Rete e HTTP (`ext-curl`)
L'introduzione della rete richiede che la VM riesca a sospendere l'esecuzione durante l'I/O (usando Tokio).
- Re-implementare prima `file_get_contents` su URL esterni tramite `reqwest`.
- Poi procedere con le funzioni basi di cURL. Il match sulle costanti `CURLOPT_*` sarà lungo e richiederà testing estensivo sul corpus `.phpt`.

### Tier 3: Database (`PDO` / `MySQLi`)
Il database è il cuore delle app PHP.
- Implementare l'interfaccia `PDO` (che è a oggetti, quindi si sfrutta l'OOP implementato nella VM) usando `sqlx`. 
- Creazione della classe nativa `PDO` in Rust, esponendo metodi come `prepare()`, `execute()` e `fetch()`.

### Tier 4: Manipolazione File e Immagini (`ext-gd`, `ext-zip`)
- Agganciare il crate `image` per `ext-gd`.
- Agganciare il crate `zip` per manipolare gli archivi.

## Conclusione
Non cercare mai il matching al 100% delle estensioni legacy: il 20% delle feature (le più vecchie e mai utilizzate) paralizzerebbe lo sviluppo. Focalizzatevi su un mapping solido e *idiomatico* verso i crate Rust per i casi d'uso principali, restituendo Warning controllati quando lo sviluppatore usa feature non implementate nel layer intermedio.

---

# Appendice A — Analisi dello stato reale del codice e prerequisiti per le estensioni I/O (2026-06-25)

> Audit del codice `php-rust` alla luce di un caso d'uso concreto: **collegare lo script PHP a un database MySQL**. Generalizzabile a qualsiasi estensione che fa I/O verso una risorsa esterna (cURL, Redis, socket). Da riprendere quando si attaccherà il Tier 2/3 della roadmap sopra.

I gap si dividono in **prerequisiti infrastrutturali del motore** (la parte importante e meno ovvia) e **superficie dell'estensione**.

## Layer 0 — Primitive del motore ancora mancanti (i veri prerequisiti)

1. **Handle nativi dentro gli oggetti / risorse opache generiche.** Una connessione MySQL/PDO deve "vivere" da qualche parte legata a un valore PHP. Stato attuale:
   - `Zval::Resource(Rc<RefCell<crate::Resource>>)` *esiste* (`crates/php-types/src/zval.rs`) ma è **cablato sul tipo stream a byte** (`crate::Resource` in `stream.rs` = file/stream locale), **non** è il contenitore opaco generico `Resource(Rc<RefCell<dyn Any>>)` proposto nel doc. Serve quel passaggio: payload `dyn Any` oppure un enum esteso che ogni estensione fa downcast.
   - `Object` (`crates/php-types/src/object.rs`) ha **solo `props` PHP + `id` + `info`, nessuno slot interno/nativo**. PDO è OOP: un'istanza `PDO` dovrebbe portare un handle (`sqlx::Pool`) nascosto, e oggi non c'è dove metterlo. Due opzioni:
     - estendere `Object` con `internal: Option<Rc<RefCell<dyn Any>>>`, oppure
     - una **side-table nella VM** `object_id → native_handle`.
   - **Questa è la primitiva chiave mancante**: senza, nessuna estensione stateful (DB, cURL handle, Redis) può esistere.
   - Ciclo di vita: l'handle va liberato a fine script / alla GC del valore (integrazione con `Drop`/destructor). Lo stream `Resource` ha già handle-semantics + `fclose`: **è il modello da generalizzare**.

2. **Meccanismo di registrazione delle estensioni (Pilastro A non ancora realizzato).** Non esiste l'hook `php_ext_*::register(&mut Registry)`: i builtin sono tutti nel monolite `php-builtins` (registrati con `add(b"...", fn)` in `lib.rs`). Per un `crates/php-ext-pdo` pulito servono:
   - il punto di iniezione nel `Registry`, **e**
   - uno spazio di **stato per-VM dell'estensione** (pool di connessioni, `last_insert_id`, `errno`/`error`). Oggi i builtin sono per lo più stateless o passano da `Ctx`; gli host-builtin raggiungono la VM, ma manca un "extension state" persistente dove vive il pool.

## Layer 1 — Sottosistema di rete (oggi: zero)

3. **Nessun I/O di rete.** Mancano `fsockopen`, `stream_socket_client`, `stream_socket_server`, `socket_create`, ecc. Lo stream subsystem fa **solo file locali**. Il protocollo MySQL gira su TCP/Unix-socket: anche un driver Rust puro richiede che il runtime *possieda* il socket e ne leghi il lifecycle a un `Zval`. È un blocco **trasversale** (serve anche a cURL, Redis, ecc.) e va costruito una volta sola.

## Layer 2 — Async (opzionale per la v1, obbligatorio per la "vision async")

4. **La VM è sincrona.** `tokio` è presente **solo in `php-server`** e usato via `spawn_blocking` (un thread bloccante per request, `crates/php-server/src/main.rs`). Fiber e Generator "parcheggiano" i frame sull'heap (`crates/php-runtime/src/vm/coroutines.rs`) ma la sospensione è **cooperativa solo ai punti `yield`/`Fiber::suspend`**: non c'è event loop e l'I/O nativo non può sospendere un fiber.
   - Per una **prima implementazione bloccante** (driver sync, o `block_on` dentro lo `spawn_blocking`) questo **non è necessario** → si parte subito.
   - Le "100 query parallele senza bloccare" del doc richiedono il pezzo grosso: **reattore Tokio integrato** + chiamate native che *cedono* il fiber durante l'attesa di rete. È il salto architetturale vero, da fare come step separato e successivo.

## Layer 3 — L'estensione vera e propria (tutta da scrivere)

5. **PDO** (`PDO`, `PDOStatement`, `PDOException`) e/o **mysqli** (procedurale + OOP: `mysqli`, `mysqli_stmt`, `mysqli_result`). Logica: parsing placeholder dei prepared statement, binding parametri, mapping **tipo colonna SQL → `Zval`**, `fetch_assoc`/`fetch_row`/`fetchAll`, transazioni, gestione errori (`errno`/`sqlstate`).

## Layer 4 — Linguaggio/stdlib di contorno: già pronto vs gap

Già disponibile (sblocca gran parte del codice DB-driven):
- ✅ OOP completo (classi/interfacce/ereditarietà) — PDO è pesantemente OOP.
- ✅ Gerarchia eccezioni (`PDOException extends RuntimeException`).
- ✅ `Iterator`/`Traversable` (`PDOStatement` iterabile in `foreach`).
- ✅ Type juggling/conversioni, array associativi, `DateTime`.
- ✅ `is_resource`/`get_resource_type`/`gettype`.

Gap residui che il codice DB reale tocca:
- ⚠️ **BIGINT/UNSIGNED a 64 bit**: `Zval::Long` è `i64`; gli `UNSIGNED BIGINT` oltre `i64::MAX` in PHP diventano stringhe — va gestito nel mapping colonne.
- ⚠️ `settype` mancante; alcune funzioni resource-introspection minori.
- ⚠️ DSN parsing + `PDO::ATTR_*` (superficie dell'estensione).

## Percorso minimo consigliato (MVP bloccante)

Per un `$pdo = new PDO('mysql:...'); $pdo->query(...)->fetchAll()` **bloccante**:
1. **Slot nativo negli oggetti** (o side-table `id→handle`) + `Resource` opaca generica. ← sblocca tutto.
2. **Hook di registrazione estensioni** + spazio di "extension state" nella VM (il pool).
3. **`crates/php-ext-pdo`** che wrappa `mysql_async`/`sqlx` con `block_on` dentro lo `spawn_blocking` esistente (niente async nella VM per ora).
4. `PDO`/`PDOStatement` come **classi native** (metodi Rust che fanno downcast dell'handle), non classi prelude PHP.

**Ordine consigliato:** PDO prima di mysqli (è OOP → riusa tutto l'OOP già fatto, è il path moderno). Rete/async = item separato e successivo: la v1 bloccante è realizzabile *senza* toccare il modello Fiber.
