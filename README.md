# php-rust — reimplementazione moderna di PHP in Rust

Esperimento di traduzione **PHP 8.5.7 (C) → Rust** guidato dal comportamento
osservabile, non dall'architettura interna dello Zend Engine.

> **Principio guida**: il contratto da preservare non è il design di Zend (1999–2004)
> ma l'**output osservabile** di PHP. L'oracle esiste già: i ~21.500 test `.phpt` del
> sorgente ufficiale. Qualunque runtime che produce lo stesso output *è* PHP.
> Questo trasforma il lavoro da *traduzione del C* a *reimplementazione guidata dalla
> spec*, dove il C si legge solo per inchiodare la semantica.

Metodologia: skill `legacy-port`, adattata (Strategia A adapter per il front-end +
full port semantico del solo `zend_operators.c`).

## Stato attuale

| Step | Contenuto | Stato |
|---|---|---|
| 0 | Scaffolding workspace + diary + Phase 0 reconnaissance | ✅ |
| 1 | `php-types`: `PhpStr`, `Zval`, `PhpArray` | ✅ |
| 2 | Operatori + conversioni (`zend_operators.c`) + **differential 37.835 casi, 0 mismatch** | ✅ |
| 3 | Bridge mago → HIR | ⏳ |
| 4 | Evaluator (echo, variabili, controllo di flusso) | ⏳ |
| 5 | Builtins nucleo + `var_dump` | ⏳ |
| 6 | `phpt-runner` + prima baseline su `Zend/tests` | ⏳ |
| 7–11 | array end-to-end, foreach, funzioni utente, diagnostica, chiusura Tier 1 | ⏳ |

**Risultato chiave (step 2)**: il porting di `zend_operators.c` — type juggling,
confronti PHP 8, formattazione float, increment Perl-style, bitwise su stringhe — è
verificato **byte-per-byte** contro un binario PHP 8.5.7 compilato dal sorgente, su
37.835 casi (47 valori × 47 × 17 operatori binari + 6 unari), diagnostica inclusa.

## Perché Rust semplifica Zend

| Sottosistema Zend | LOC C | Sostituto Rust | LOC Rust |
|---|---|---|---|
| VM generata (`zend_vm_execute.h`) + `zend_execute.c` | ~146.000 | evaluator tree-walk su HIR | 3–5K |
| `zend_compile.c` (AST→opcodes) | 12.400 | lowering AST→HIR | 1–2K |
| lexer re2c + parser Bison + AST | ~25.000 | dipendenza `mago` + bridge | ~500 |
| `zend_alloc` / `zend_gc` / TSRM / Optimizer / opcache / win32 | ~88.000 | ownership, `Rc`+COW, `Send`/`Sync`, processo residente | ~0 |
| `zend_operators.c` (type juggling) | 3.900 | **full port fedele** (l'anima di PHP) | ~1.500 |

~280K LOC del core → ~8–10K LOC Rust stimati.

## Struttura

```
php-rust/crates/
  php-types      Zval / PhpStr / PhpArray + operatori (zero dep interne)
  php-runtime    HIR, lowering da mago, evaluator        (in arrivo)
  php-builtins   trait Builtin + funzioni                (in arrivo)
  php-cli        binario `phpr`                           (in arrivo)
  phpt-runner    harness .phpt + differential             (in arrivo)
diary/           00-reconnaissance … 99-conclusions + metrics
```

## Build & test

```bash
cd php-rust
cargo test                       # unit + integration
# differential vs oracle (richiede un binario php):
#   build dal sorgente:  ./configure --disable-all --enable-cli && make
PHP_ORACLE=/path/to/php cargo test -p php-types --test differential
```

Il differential si auto-salta con un messaggio se l'oracle non è disponibile.

## Diario

Il deliverable principale dell'esperimento è il **diario metodologico** in `diary/`,
non solo il codice: decisioni (`02-mapping-table.md`), log per step
(`03-translation-log.md`), divergenze trovate, conclusioni.

---

*Generato con assistenza AI (Claude Fable 5). Codice e commenti tecnici in inglese,
diario in italiano.*
