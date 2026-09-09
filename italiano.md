# php-rust

*Leggi in [English](README.md).*

*🌐 Sito ufficiale: **[phprust.com](https://phprust.com/)***

*📊 [Copertura misurata](php-rust/COVERAGE.md) — funzioni per estensione, corpus `.phpt` di Zend e gli stack reali che girano byte-identici.*

> **PHP, reimplementato da zero in Rust.** Un runtime PHP 8.5 moderno, memory-safe e
> predisposto all'asincronia — guidato dal comportamento osservabile, non dall'architettura
> interna dello Zend Engine.

```bash
phpr script.php        # un drop-in di `php`, ma è Rust fino in fondo
```

> **Stato (2026-09-10, pin della sessione S-172).** L'**intera suite di test del core
> di WordPress** (30.472 test single-site, 31.278 multisite, wordpress-develop trunk)
> gira a **parità effettiva con l'oracle** — una sola divergenza deliberata e a catalogo —
> su **MySQL reale** tramite un protocollo `mysqli` nativo e la SAPI server integrata.
> Anche Composer, PHPUnit 9/11/13, Doctrine ORM/DBAL, symfony/http-kernel (0/0),
> http-foundation, wp-cli e Monolog girano a parità. **Il fronte corrente è la
> performance**, misurata su un micro-benchmark a sei categorie contro la CPU
> dell'interprete di riferimento: **arith 2,7× · regex 2,5× · array 3,0× · property
> 3,8× · string 4,1× · calls 4,7×** (da 9,3 / 3,5 / 3,9 / 7,9 / 5,3 / 5,1 ad agosto);
> la suite WordPress completa è a **~1,77×** la CPU dell'oracle. Obiettivo: **parità
> (1×)**; la tappa ≤3× è raggiunta su arith e regex, con array sulla soglia.

---

## 💡 Idea

Lo Zend Engine — il cuore di PHP — è ~270.000 righe di C accumulate dal 1999. Porta con sé
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

Il banco di prova non è un microbenchmark: è **far girare Composer**, poi una suite di test
completa di un framework, poi un'applicazione reale end-to-end. Quei traguardi stressano OOP,
autoloading e Reflection più di qualsiasi test sintetico — e WordPress su MySQL reale è stata
la prima applicazione a cadere.

---

## 🗺️ Roadmap

| Fase | Traguardo | Stato |
|---|---|---|
| **1. Nucleo semantico** | Type juggling fedele all'oracle (`zend_operators.c`), `==`/`===`, coercizioni | ✅ Fatto |
| **2. Linguaggio completo** | Espressioni, control-flow, funzioni, array, reference, closure | ✅ Fatto |
| **3. OOP** | Classi, ereditarietà, visibility, `static`/LSB, magic methods, enum, trait, interfacce, property hooks PHP 8.4, visibilità asimmetrica, lazy objects | ✅ Fatto |
| **4. Eccezioni & errori** | `try/catch/finally`, engine error catchabili, stack trace, line tracking | ✅ Fatto |
| **5. VM a bytecode** | Generatori, `yield from`, Fiber su frame espliciti — **niente `unsafe`, niente coroutine stackful** | ✅ Fatto |
| **6. Memoria** | Cycle collector a modello Zend su oggetti e contenitori, soglie adattive, timing dei distruttori fedele a Zend | ✅ Fatto |
| **7. Libreria standard** | 1017 funzioni interne registrate (misurate; stdlib del linguaggio 539/654 = 82%): array/string/math/json/preg/mbstring/hash/file/stream/date/session… | ✅ Sostanziale (coda lunga rinviata) |
| **8. Composer reale** | `composer require monolog/monolog` **end-to-end**: risoluzione, download HTTPS (rustls), unzip, autoload — e il pacchetto **gira** | ✅ Fatto |
| **8b. Ecosistema reale** | **PHPUnit 9/11/13 byte-identico** (incluso l'isolamento di processo); Doctrine **DBAL 3769 / 0 / 0**, **ORM 3484 test a 3 err / 13 fail, stabili per nome**; **symfony/http-kernel CHIUSO 1665 test 0/0**, http-foundation 0 errori; Monolog, wp-cli, collections, inflector, instantiator… | ✅ Fatto |
| **9. Applicazione reale** | **WordPress 7.0.1 su MySQL reale** (`mysqli` nativo), servito dalla SAPI integrata byte-identico; **suite PHPUnit del core completa a parità effettiva, single-site E multisite**; pipeline media a parità di byte su libgd/libxslt/libtidy di sistema via FFI | ✅ Fatto |
| **10. Performance** | Parità con la CPU dell'oracle. Tappa ≤3× per micro-categoria: raggiunta su arith e regex, array sulla soglia; property, string e calls in corso. Suite WordPress completa ~1,77× | 🔄 **Fronte corrente** |
| **11. Secondo framework** | Laravel come prossimo bersaglio di validazione (stessa ricetta di gate di ORM/http-kernel) | ⏳ In coda |
| **12. Async & single-binary** | Event loop Tokio + web server Axum residente, distribuzione standalone | ⏳ Futuro |
| **13. JIT (Tier 3)** | Bytecode pulito → Cranelift/LLVM per il codice macchina al volo | 🔭 Visione |

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
                 compile/ (HIR→bytecode, fusione di superistruzioni) + vm/ (ciclo di dispatch,
                 inline cache, eccezioni, coroutine, array, OOP, chiamate, GC)
  php-builtins   registry dei builtin puri (var_dump, array_*, sprintf, json_*, preg_*,
                 mb_*, hash/encoding, file/stream, …); insieme agli host builtin lato VM
                 (reflection, callable, PDO/sqlite, dom/xml, curl, proc_open, session,
                 mysqli, …) phpr registra 1017 funzioni interne (misurate con la sonda)
  php-cli        binario `phpr` — drop-in di `php`, stream CLI-faithful + exit code fedele
  php-server     web server nativo sulla stessa SAPI cli-server (semantica di `phpr -S`)
  phpt-runner    esegue i `.phpt` ufficiali con capability-scan e diff unificato vs oracle
diary/           diario metodologico: 00-reconnaissance … 99-conclusions + metriche
```

**Perché Rust collassa Zend** — il payoff strutturale, in cifre (LOC misurate con
`wc -l` su PHP 8.5.7 e su questo repo, 2026-09-10):

| Sottosistema Zend | LOC C | Sostituto Rust | LOC Rust |
|---|---:|---|---:|
| VM generata + `zend_execute.c` | ~136.000 | VM a bytecode (motore unico): `php-runtime/src/vm/` | ~62.300 |
| `zend_compile.c` (AST→opcodes) | ~12.400 | HIR + lowering + compilatore (`hir.rs`, `lower/`, `compile/`) | ~19.200 |
| lexer re2c + parser Bison + AST | ~23.400 | dipendenza `mago`; il bridge AST→HIR è contato nella riga del lowering | — |
| `zend_alloc` / `zend_gc` / TSRM / opcache (JIT incluso) / win32 | ~108.000 | ownership, `Rc`+COW, `Send`/`Sync` + cycle collector | ~1.000 |
| `zend_operators.c` + stringhe numeriche (type juggling) | ~3.900 | full-port fedele (`ops.rs`, `convert.rs`, `numstr.rs`) | ~1.900 |

**~270K LOC di C core di Zend (estensioni escluse) → ~112K LOC di Rust di motore**
(`php-runtime` + `php-types`). L'intero progetto oggi è **~153K LOC di Rust** più ~9,3K
di preludio PHP — e quel totale include anche la libreria standard (~34K), mysqli,
PDO/sqlite, dom/xml, TLS e il tooling phpt, funzionalità che lato C vivono in
`ext/`/`sapi/` e *non* fanno parte dei 270K. Il motore è cresciuto di ~45K righe da
luglio: è il prezzo dell'interprete specializzante (fast path tipizzati, superistruzioni
fuse, inline cache) che ha portato la suite WordPress da 4,1× a ~1,77× la CPU dell'oracle.

---

## 📍 Dove siamo

Il linguaggio **core è completo e fedele**: tutto il control-flow, le funzioni, gli array, il
sistema di reference, le closure, l'**OOP completo** (classi, ereditarietà, visibility, `static`
+ late-static-binding, magic methods, enum, trait, Reflection framework-grade), le **eccezioni**
(incluso stack trace byte-identico e gli engine error catchabili), i **generatori** e i **Fiber**
— questi ultimi implementati parcheggiando i frame su uno stack esplicito della VM, **senza
`unsafe` e senza coroutine stackful**. Del PHP moderno ci sono anche i pezzi difficili:
**property hooks** di PHP 8.4 (by-ref compresi), **lazy objects** (ghost/proxy), visibilità
asimmetrica, first-class callable, `strict_types` risolto per-unit dal call-site, fusi orari
IANA reali con la semantica gap/fold di timelib e una cache per-richiesta delle unità in stile
opcache.

Il salto vero è che **l'ecosistema reale gira**:

- **WordPress 7.0.1 gira su MySQL reale** (protocollo `mysqli` nativo) tramite la SAPI server
  integrata — wp-admin, front page, login, REST e pretty permalink **byte-identici su HTTP**.
  La **suite PHPUnit del core completa** (30.472 test single-site, 31.278 multisite) è a
  **parità effettiva con l'oracle**: una sola divergenza deliberata e a catalogo, stabile per
  nome a ogni run. La pipeline media raggiunge la parità di byte via **libgd / libxslt /
  libtidy di sistema attraverso FFI** (+ exif e fileinfo nativi). **wp-cli** gira da sorgente
  a parità.
- **Composer** installa pacchetti end-to-end: risoluzione, download **HTTPS nativo**
  (ureq + rustls), unzip nativa, dump dell'autoloader — e il pacchetto installato **esegue**.
- **PHPUnit 9.6 / 11.5 / 13** partono e producono output **byte-identico** all'oracle,
  isolamento di processo compreso (i figli lanciano `phpr`).
- **Doctrine DBAL: 3769 test, 0 errori, 0 failure** — su un'implementazione di
  **PDO / pdo_sqlite / ext-sqlite3 nativa in Rust** (rusqlite bundled, semantiche
  SQLSTATE/errmode/metadata verificate una a una contro l'oracle). **Doctrine ORM: 3484 test,
  3 errori / 13 failure**, dichiarati e stabili per nome (il resto è triagiato: XSD
  `schemaValidate`, casi limite dei proxy lazy). Collections, inflector, lexer,
  event-manager, instantiator: **verdi**.
- **Symfony**: **http-kernel CHIUSO — l'intera suite di 1665 test a 0 errori / 0 failure**
  (il container DI compila, dumpa e ricarica; timing dei distruttori fedele a Zend);
  http-foundation a 0 errori; String / Console / Process verdi.
- Estensioni modellate senza C: `pdo`, `pdo_sqlite`, `sqlite3`, `mysqli`, `dom`, `libxml`,
  `simplexml`, `xml` (SAX), `curl` (easy-API su ureq), `openssl`/TLS (rustls), `zip`,
  `mbstring`, `pcre`, `hash`, `json`, `session`, `pcntl`, `posix`, `ctype`, `bcmath`, `gmp`,
  `tokenizer`, `fileinfo`, un sottoinsieme di `intl`; sulle **librerie di sistema via FFI**
  (parità di byte con le stesse dylib dell'oracle): `zlib`, `gd` (+exif), `xsl`, `tidy`.

Tutti e tre i «draghi» storici di un porting PHP sono stati affrontati:

- 🐉 **Riferimenti circolari** → un **cycle collector** a modello Zend su oggetti *e*
  contenitori (buffer dei possible-roots, conteggi di `gc_collect_cycles()` esatti come Zend,
  soglie adattive), con sweep O(candidati): un test patologico da 87.380 oggetti ciclici è
  passato da ~11s a ~0,25s.
- 🐉 **Bug-for-bug compatibility** → l'intera strategia è ancorata al corpus `.phpt` e alle
  suite dei framework reali; ogni build promossa passa un gate a fail-set congelato **per
  nome**, in due modalità di esecuzione, più fixture bilaterali eseguite su entrambi i motori.
- 🐉 **L'ecosistema di estensioni C (PECL)** → riscritture native mirate dove la semantica vive
  in stringhe PHP (PDO/sqlite, mysqli, dom/simplexml, curl), e **FFI verso le stesse dylib di
  sistema che usa l'oracle** dove la parità di byte è una proprietà della libreria (gd, xslt,
  tidy, zlib).

**Fedeltà** (al 2026-09-10, pin della sessione S-172): differential type-juggling vs PHP reale a
**0 mismatch** (37.835 casi — è il differential degli *operatori*, metrica distinta dal corpus
`.phpt`); **1.748** test Rust unit/integration verdi; sul corpus `Zend/tests` ufficiale
**2655 phpt passano** (65,3% dei runnable, con gate congelato «zero
pass→fail per nome» su ogni build promossa); suite WordPress completa a parità con **CPU
full-suite a ~1,77×** l'oracle (mediana dell'ultima coppia misurata, banda [1,74; 1,80]).
Copertura misurata: **[php-rust/COVERAGE.md](php-rust/COVERAGE.md)**; mappa perf
multi-workload: **[php-rust/PERF_MAP.md](php-rust/PERF_MAP.md)**; rotta corrente:
**[php-rust/NEXT_SESSION_WORDPRESS.md](php-rust/NEXT_SESSION_WORDPRESS.md)**.

> Lo storico dettagliato dei ~70 step di costruzione è in **[HISTORY.md](HISTORY.md)**; il diario
> metodologico replicabile è in **[diary/](diary/)**; dall'arco WordPress in poi ogni sessione
> di lavoro ha il suo file in **[php-rust/sessions/](php-rust/sessions/)** e la sua istantanea
> del gap perf in **[php-rust/gaps/](php-rust/gaps/)**.

---

## ⚡ Performance: dov'è il divario e come lo si chiude

Da agosto il progetto lavora sotto un protocollo di misura scritto
([php-rust/REGOLE.md](php-rust/REGOLE.md)): ogni leva è pre-registrata col suo criterio,
misurata A/B interallacciato contro il binario pinnato al netto dei pavimenti di avvio per
binario, presidiata da guardie sulle categorie non bersaglio, e promossa solo attraverso un
gate a script (build → hash → batteria di test → fail-set del corpus congelato per nome ×2
modi → fixture bilaterali → micro R=5). Ogni sessione si chiude con una revisione
adversariale; la traccia intera è in `php-rust/sessions/` e `php-rust/gaps/GAP_TREND.md`.

Micro-benchmark, stesso sorgente PHP sui due motori, rapporto di CPU utente (phpr / PHP 8.5.7):

| categoria | ago 2026 (S-110) | **set 2026 (S-172)** | tappa ≤3× |
|---|---:|---:|:---:|
| arith | 9,3× | **2,7×** | ✅ |
| regex | 3,5× | **2,5×** | ✅ |
| array | 3,9× | **3,0×** | ✅ |
| property | 7,9× | **3,8×** | 🔄 |
| string | 5,3× | **4,1×** | 🔄 |
| calls | 5,1× | **4,7×** | 🔄 |

Applicazioni reali: **suite WordPress completa ~1,77×** la CPU dell'oracle (da 4,1× all'inizio
dell'arco; il divario di picco di memoria è passato da 11,9× a ~2,3×, ultimo rapporto misurato
ad agosto), **suite Doctrine ORM ~7,1×** (da 8,4×; carico dominato dagli oggetti, il più duro
della mappa).

Cosa hanno stabilito le misure, nell'ordine: l'ipotesi del dispatch threaded è stata refutata
(il dispatch puro costa 1,75 ns/op — quanto l'intera istruzione dell'oracle); il divario vive
nel **corpo dei handler**, cioè nel ciclo di vita degli `Zval` temporanei attorno a ogni
operazione. La leva che ne è seguita — una *forma sigillata Long* che tiene l'aritmetica
intera calda e il read-modify-write sulle proprietà su `i64` nudi, senza temporanei — ha
dimezzato il giudice aritmetico (46,8 → 23,4 ns/iter) e portato l'accesso alle proprietà da
5,2× a 3,8×, con zero `unsafe`. La stessa forma si sta ora applicando a chiamate e stringhe.
Bocciati dalla misura, e non tornano: NaN-boxing, dispatch a tabella di funzioni, arena di
oggetti, BOLT/PGO.

---

## 🚀 Prossimi passi

1. **Performance fino alla parità** — la rotta è fissata dalla misura: forme sigillate sui
   corpi caldi restanti (peephole fetch di proprietà + aritmetica, poi il frame di chiamata,
   poi concatenazione di stringhe e `substr`), ciascuna sotto il proprio criterio
   pre-registrato e giudice per categoria. Rotta e quesiti aperti:
   `php-rust/NEXT_SESSION_WORDPRESS.md`.
2. **Laravel** — il secondo bersaglio di validazione framework, in coda dietro il fronte perf
   (decisione utente); metodo = la ricetta di gate collaudata su ORM/http-kernel.
3. **Doctrine ORM a zero** — 3 errori / 13 failure, stabili per nome e triagiati.
4. **Superfici di estensione restanti su richiesta** — xmlwriter, calendar, sockets; le
   estensioni database / crypto / rete non ancora iniziate (pgsql, sodium, ldap, odbc) sono
   il grosso del divario 47%→100% sulle funzioni, non feature di linguaggio mancanti.
5. **Robustezza** — convertire gli `unwrap`/`expect` raggiungibili da input utente in errori VM
   tipizzati + fuzzing della pipeline `lower/compile`, per una garanzia *no-panic*.
6. **Salto async** — integrare un event loop **Tokio** e consolidare `php-server` in un
   runtime residente, verso un PHP nativamente concorrente e un **singolo binario** distribuibile.

---

## 🛠️ Quickstart

```bash
cd php-rust
cargo build --release                    # i binari finiscono nella target-dir configurata
phpr script.php                          # esegui uno script
cargo test --release                     # unit + integration test (1.748)

# Differential vs oracle (richiede un binario php; si auto-salta se assente):
PHP_ORACLE=/path/to/php cargo test -p php-types --test differential

# Esegui il corpus ufficiale .phpt attraverso la VM:
phpt-runner --isolate /path/to/php-src/Zend/tests
phpt-runner --isolate --list-fails <path>   # un test = un sotto-processo, con diff

# Servi un'applicazione PHP (WordPress compreso) sulla SAPI integrata:
php-server --port 8080 --docroot /path/to/wordpress
```

Diagnostica: `PHP_RUST_TRACE=hir|body|exec|all phpr script.php` mostra su **stderr** l'HIR
abbassato e/o la traccia d'esecuzione, senza inquinare lo stdout confrontato con l'oracolo.

---

## 🤝 Contribuire

L'idea *«riscrivere PHP in Rust per renderlo asincrono e safe»* è un magnete per la community Rust.
Il modo migliore di contribuire una volta presa confidenza: prendere un builtin mancante o un
gruppo di `.phpt` che falliscono (`phpt-runner --list-fails`), riprodurli contro l'oracolo, e
chiudere il gap restando byte-identici. Le deviazioni deliberate sono a catalogo per nome in
[php-rust/PHPR_DIVERGENCES_FROM_PHP.md](php-rust/PHPR_DIVERGENCES_FROM_PHP.md); un builtin che
restituisce risultati *plausibili ma sbagliati* non viene mai registrato (**corretto o
assente**). La regola d'oro del progetto: **l'oracolo ha sempre ragione.**

## 📄 Licenza

MIT.
