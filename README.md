# php-rust

> **PHP, reimplementato da zero in Rust.** Un runtime PHP 8.5 moderno, memory-safe e
> predisposto all'asincronia — guidato dal comportamento osservabile, non dall'architettura
> interna dello Zend Engine.

```bash
phpr script.php        # un drop-in di `php`, ma è Rust fino in fondo
```

---

## 💡 Idea

Lo Zend Engine — il cuore di PHP — è ~280.000 righe di C accumulate dal 1999. Porta con sé
gestione manuale della memoria, un garbage collector custom, un layer di thread-safety (TSRM),
una VM generata da macro e un JIT contorto. È solido ma fragile: intere classi di vulnerabilità
(*use-after-free*, *buffer overflow*) vivono lì per costruzione.

L'intuizione del progetto è ribaltare il problema:

> **Il contratto da preservare non è il *design* di Zend, ma l'*output osservabile* di PHP.**

E quell'output ha già un oracolo perfetto: i **~21.500 test ufficiali `.phpt`** del sorgente PHP.
Qualunque runtime che produce lo stesso identico output *è* PHP. Questo trasforma il lavoro da
*«traduzione del C»* a *«reimplementazione guidata dalla specifica»*, dove il C si legge solo
per inchiodare la semantica nei casi ambigui.

Il risultato è un engine in cui Rust fa il lavoro pesante a costo zero: l'**ownership** sostituisce
`zend_alloc`, `Rc`+copy-on-write sostituiscono il refcounting manuale, `Send`/`Sync` rendono
il multi-threading una proprietà del tipo invece di un sottosistema (TSRM), e un processo
residente rende l'engine async-ready per costruzione.

---

## 🎯 Obiettivo

Un runtime PHP che sia, nell'ordine:

1. **Fedele** — bug-for-bug compatibile con PHP 8.5 sul corpus ufficiale `.phpt` (incluse le
   idiosincrasie del type juggling, i warning legacy, lo stack trace byte-identico).
2. **Sicuro** — niente segfault a livello di core; le classi di bug della memoria del C eliminate
   dal type system di Rust.
3. **Moderno** — distribuibile come **singolo binario** (l'effetto Go/Deno), con web server
   nativo integrato e una base **nativamente asincrona e multi-thread** — superando il limite
   storico *shared-nothing / single-threaded* di PHP.

Il banco di prova non è un microbenchmark: è **far girare Composer** e poi far rispondere una
rotta *Hello World* di **Laravel/Symfony**. Quei traguardi stressano OOP, autoloading e
Reflection più di qualsiasi test sintetico.

---

## 🗺️ Roadmap

| Fase | Traguardo | Stato |
|---|---|---|
| **1. Nucleo semantico** | Type juggling fedele all'oracle (`zend_operators.c`), `==`/`===`, coercizioni | ✅ Fatto |
| **2. Linguaggio completo** | Espressioni, control-flow, funzioni, array, reference, closure | ✅ Fatto |
| **3. OOP** | Classi, ereditarietà, visibility, `static`/LSB, magic methods, enum, trait, interfacce | ✅ Fatto |
| **4. Eccezioni & errori** | `try/catch/finally`, engine error catchabili, stack trace, line tracking | ✅ Fatto |
| **5. VM a bytecode** | Generatori, `yield from`, Fiber su frame espliciti — **niente `unsafe`, niente coroutine stackful** | ✅ Fatto |
| **6. Memoria** | Cycle collector per i riferimenti circolari (l'altro grande «drago») | ✅ Fatto |
| **7. Libreria standard** | ~500 builtin: array/string/math/json/preg/mbstring/hash/file/stream/date… | ✅ Sostanziale (coda lunga in corso) |
| **8. Composer reale** | `composer require monolog/monolog` **end-to-end**: risoluzione, download HTTPS (rustls), unzip, autoload — e il pacchetto **gira** | ✅ Fatto |
| **8b. Ecosistema reale** | **PHPUnit 13.2 verde byte-identico**; Doctrine **DBAL 3769 test / 0 err / 0 fail** su PDO+sqlite nativi; **ORM 3484 test / 12 err**; Monolog, collections, inflector, instantiator… | ✅/🔄 In corso |
| **9. Framework bootstrap** | *Hello World* su Laravel / Symfony | ⏳ Prossimo |
| **10. Async & single-binary** | Event loop Tokio + web server Axum residente, distribuzione standalone | ⏳ Futuro |
| **11. JIT (Tier 3)** | Bytecode pulito → Cranelift/LLVM per il codice macchina al volo | 🔭 Visione |

---

## 🏗️ Architettura

Un solo motore di produzione: una **VM a bytecode**. Il sorgente passa per
`parser (mago) → AST → HIR → bytecode → VM dispatch loop`. (Il progetto è nato con un
tree-walker, poi rimosso una volta che la VM ha raggiunto la piena parità: vedi
[HISTORY.md](HISTORY.md).)

```
php-rust/crates/
  php-types      Zval / PhpStr / PhpArray / Object + operatori (l'anima di PHP:
                 type juggling full-port da zend_operators.c). Zero dipendenze interne.
  php-runtime    HIR + lowering da `mago`, e la VM a bytecode:
                 compile.rs (HIR→bytecode) + vm/{mod,exceptions,coroutines,arrays,oop,calls}.rs
  php-builtins   registry di ~380 builtin puri (var_dump, array_*, sprintf, json_*, preg_*,
                 mb_*, hash/encoding, file/stream, …) + ~120 host builtin VM-side
                 (reflection, callable, PDO/sqlite, dom/xml, curl, proc_open, …)
  php-cli        binario `phpr` — drop-in di `php`, stream CLI-faithful + exit code fedele
  php-server     web server nativo (Axum + Tokio) — la testa di ponte verso l'async
  phpt-runner    esegue i `.phpt` ufficiali con capability-scan e diff unificato vs oracle
diary/           diario metodologico: 00-reconnaissance … 99-conclusions + metriche
```

**Perché Rust collassa Zend** — il payoff strutturale, in cifre:

| Sottosistema Zend | LOC C | Sostituto Rust | LOC Rust |
|---|---:|---|---:|
| VM generata + `zend_execute.c` | ~146.000 | VM a bytecode (motore unico) | dentro `php-runtime` |
| `zend_compile.c` (AST→opcodes) | ~12.400 | lowering AST→HIR + compile.rs | dentro `php-runtime` |
| lexer re2c + parser Bison + AST | ~25.000 | dipendenza `mago` + bridge | ~500 |
| `zend_alloc` / `zend_gc` / TSRM / opcache / win32 | ~88.000 | ownership, `Rc`+COW, `Send`/`Sync` + cycle collector | ~1.000 |
| `zend_operators.c` (type juggling) | ~3.900 | full-port fedele | ~1.500 |

**~280K LOC di C core (senza contare le estensioni) → ~68K LOC di Rust totali oggi** — engine,
stdlib, PDO/sqlite, dom/xml, TLS e tooling inclusi. Il rapporto ~4:1 regge anche a
funzionalità cresciute di un ordine di grandezza rispetto alle prime stime.

---

## 📍 Dove siamo

Il linguaggio **core è completo e fedele**: tutto il control-flow, le funzioni, gli array, il
sistema di reference, le closure, l'**OOP completo** (classi, ereditarietà, visibility, `static`
+ late-static-binding, magic methods, enum, trait, Reflection framework-grade), le **eccezioni**
(incluso stack trace byte-identico e gli engine error catchabili), i **generatori** e i **Fiber**
— questi ultimi implementati parcheggiando i frame su uno stack esplicito della VM, **senza
`unsafe` e senza coroutine stackful**. Del PHP moderno ci sono anche i pezzi difficili:
**property hooks** e **lazy objects** (ghost/proxy) di PHP 8.4, le first-class callable,
`strict_types` risolto per-unit dal call-site.

Ma il salto vero è che **l'ecosistema reale gira**:

- **Composer** installa pacchetti end-to-end: risoluzione, download **HTTPS nativo**
  (ureq + rustls), unzip nativa, dump dell'autoloader — e il pacchetto installato **esegue**.
- **PHPUnit 13.2** boota e produce output **byte-identico** all'oracle.
- **Doctrine DBAL: 3769 test, 0 errori, 0 failure** — su un'implementazione di
  **PDO / pdo_sqlite / ext-sqlite3 nativa in Rust** (rusqlite bundled, semantiche
  SQLSTATE/errmode/metadata verificate una a una contro l'oracle).
- **Doctrine ORM: 3484 test, 12 errori / 22 failure** (e in discesa) — hydration,
  UnitOfWork, mapping XML compresi. Collections, inflector, lexer, event-manager,
  instantiator: **verdi**.
- Estensioni modellate senza C: `pdo`, `pdo_sqlite`, `sqlite3`, `dom`, `libxml`,
  **`simplexml`** (sul DOM Rust), `curl` (easy-API su ureq), `openssl`/TLS (rustls),
  `zip`, `mbstring`, `pcre` (3 engine), `hash`, `json`, `pcntl`, `posix`, `ctype`.

Due dei tre «draghi» storici di un porting PHP sono già stati domati:

- 🐉 **Riferimenti circolari** → un **cycle collector** stile Zend (algoritmo *possible-roots*),
  con sweep O(candidati): un test patologico da 87.380 oggetti ciclici è passato da ~11s a ~0,25s.
- 🐉 **Bug-for-bug compatibility** → l'intera strategia è ancorata al corpus `.phpt` e alle
  suite dei framework reali, l'unico vero scudo contro le regressioni.

Il terzo drago — l'**ecosistema di estensioni C (PECL)** — è aggredito per riscrittura nativa
mirata (PDO/sqlite, dom/simplexml e curl sono già caduti così); il layer FFI di compatibilità
resta l'opzione di lungo termine per la coda.

**Fedeltà**, oggi: differential type-juggling vs PHP reale a **0 mismatch**; ~1.500 unit/integration
test Rust verdi; sul corpus `Zend/tests` ufficiale **2.071 phpt passano** (56% dei runnable, in
crescita a ogni sessione, con gate «zero pass→fail» su ogni commit); ~640 commit di storia
tracciata sessione per sessione.

> Lo storico dettagliato dei ~70 step di costruzione è in **[HISTORY.md](HISTORY.md)**; il diario
> metodologico replicabile è in **[diary/](diary/)**.

---

## 🚀 Prossimi passi

1. **Allargare il corpus eseguibile** — `--run-skipif` nel runner (sblocca ~200+ phpt di
   ext/pdo, ext/sqlite3 e altri) e chiusura data-driven dei costrutti che oggi fermano la
   compilazione di un'intera unit (named arguments su call dinamiche, ecc.): ogni costrutto
   chiuso sblocca in cascata corpus e suite dei framework.
2. **Doctrine ORM a zero** — gli ultimi 12 errori sono triagiati (XSD `schemaValidate`,
   typed-prop sui proxy lazy, singleton); il grosso del lavoro è fatto.
3. **Framework bootstrap** — *Hello World* su Laravel/Symfony: lo stress-test definitivo per
   autoloading e Reflection, ora a portata visto che PHPUnit e Doctrine già girano.
4. **Robustezza** — convertire gli `unwrap`/`expect` raggiungibili da input utente in errori VM
   tipizzati + fuzzing della pipeline `lower/compile`, per una garanzia *no-panic*.
5. **Salto async** — integrare un event loop **Tokio** e consolidare `php-server` (Axum) in un
   runtime residente, verso un PHP nativamente concorrente e un **singolo binario** distribuibile.

---

## 🛠️ Quickstart

```bash
cd php-rust
cargo run -p php-cli -- script.php       # esegui uno script con `phpr`
cargo test                               # unit + integration test

# Differential vs oracle (richiede un binario php; si auto-salta se assente):
PHP_ORACLE=/path/to/php cargo test -p php-types --test differential

# Esegui il corpus ufficiale .phpt attraverso la VM:
cargo run -p phpt-runner -- /path/to/php-src/tests /path/to/php-src/Zend/tests
cargo run -p phpt-runner -- --isolate --list-fails <path>   # un test = un sotto-processo, con diff
```

Diagnostica: `PHP_RUST_TRACE=hir|body|exec|all phpr script.php` mostra su **stderr** l'HIR
abbassato e/o la traccia d'esecuzione, senza inquinare lo stdout confrontato con l'oracolo.

---

## 🤝 Contribuire

L'idea *«riscrivere PHP in Rust per renderlo asincrono e safe»* è un magnete per la community Rust.
Il modo migliore di contribuire una volta presa confidenza: prendere un builtin mancante o un
gruppo di `.phpt` che falliscono (`phpt-runner --list-fails`), riprodurli contro l'oracolo, e
chiudere il gap restando byte-identici. La regola d'oro del progetto: **l'oracolo ha sempre ragione.**

## 📄 Licenza

MIT.
