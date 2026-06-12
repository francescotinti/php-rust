# Fase 1 — Semantic model (scope Tier 1: PHP procedurale)

> Generato con assistenza AI (Claude Fable 5). Data: 2026-06-13.
> Sorgente di riferimento: PHP 8.5.7, `/Volumes/Extreme Pro/Claude/php-8.5.7`.
> Questo documento descrive il **comportamento osservabile** che la reimplementazione Rust
> deve riprodurre. Non descrive l'architettura interna di Zend se non dove necessario a
> chiarire la semantica.

## 1. Modello dati: zval

Il valore PHP è una union taggata a 16 byte: union `zend_value` a 8 byte
(`Zend/zend_types.h:335-353`) + type tag e campi ausiliari (`zend_types.h:355-380`).
Type tag rilevanti per Tier 1 (`zend_types.h:620-631`): `IS_UNDEF=0, IS_NULL, IS_FALSE,
IS_TRUE, IS_LONG, IS_DOUBLE, IS_STRING, IS_ARRAY` (più `IS_REFERENCE`, differito).

Semantica osservabile:
- **IS_UNDEF è distinto da IS_NULL**: leggere una variabile mai assegnata produce
  `Warning: Undefined variable $x` e il valore NULL; la distinzione esiste solo nel motore.
- `IS_FALSE`/`IS_TRUE` sono due tag separati nel C; per noi è `Bool(bool)` — non osservabile.
- I tipi heap (string, array) sono refcounted con **copy-on-write**: l'assegnazione
  `$b = $a` condivide il payload; la scrittura su un valore condiviso lo separa prima
  (`SEPARATE_ARRAY` / `zend_string_separate`). Osservabile: modificare `$b` non tocca `$a`,
  e modificare l'array iterato by-value durante un `foreach` non altera l'iterazione.

## 2. Stringhe (zend_string)

`Zend/zend_types.h:393-398`: `{ gc; hash h; size_t len; char val[] }`.
- **Le stringhe PHP sono sequenze di byte arbitrarie**, non UTF-8. `strlen` conta byte.
  Vincolo Rust: `[u8]`, mai `String`.
- Hash lazy: calcolato alla prima richiesta, 0 = non calcolato (`Zend/zend_string.h:130-133`).
  Non osservabile nell'output → qualunque hash interno è corretto.
- Interning dei literal: ottimizzazione, non semantica.

## 3. Array (zend_array / HashTable)

`Zend/zend_types.h:408-432`. L'array PHP è un **ordered hash** con chiavi `int|string`.
Comportamenti osservabili da riprodurre:

1. **Ordine di iterazione = ordine di inserimento**, anche dopo `unset` (foreach,
   var_dump, ecc.). La distinzione interna packed/mixed (`Zend/zend_hash.c:345`) è
   invisibile.
2. **Canonicalizzazione chiavi** (`Zend/zend_hash.c:3300`, `_zend_handle_numeric_str_ex`):
   una chiave stringa diventa chiave int se: segno `-` opzionale, poi solo cifre,
   **niente leading zero** (`"08"` resta stringa; `"0"` da solo è int 0), lunghezza entro
   `MAX_LENGTH_OF_LONG` (`Zend/zend_long.h:112` = 20 su 64-bit), niente overflow
   (controlli espliciti per positivo e negativo), **`"-0"` resta stringa**.
   Inoltre (dalla conversione zval→chiave): `1.5` → 1 con Deprecated, `true` → 1,
   `false` → 0, `null` → `""`.
3. **next free element**: l'append `$a[] = v` usa max(indici int mai inseriti)+1;
   **non decresce dopo unset dell'ultimo elemento**.
4. Internal pointer (`reset/next/current/key/end`): differito a quando i builtin
   relativi entreranno in scope.

## 4. Stringhe numeriche (il cuore del type juggling)

`_is_numeric_string_ex`, `Zend/zend_operators.c:3620-3750`:
- Grammatica: `[ \t\n\r\v\f]* [+-]? ( digits [. digits]? | . digits ) ( [eE] [+-]? digits )?`
  con trailing whitespace ammesso solo in modalità `allow_errors`.
- `"0x1A"` NON è numerica (niente hex). `".5"` e `"5."` sono double.
- Interi troppo lunghi (≥ `MAX_LENGTH_OF_LONG` cifre) → double.
- Tre classi osservabili: **numerica** (intera o double), **leading-numeric**
  (`"5abc"`: prefisso numerico + trailing data), **non numerica**.

## 5. Conversioni

- **to bool** (`Zend/zend_operators.c:687-756`): falsy = `null`, `false`, `0`, `0.0`/`-0.0`,
  `""`, `"0"` (solo esattamente `"0"`: `"0.0"` e `"00"` sono truthy), `[]`.
  Tutto il resto truthy.
- **double → int** (`Zend/zend_operators.h:126-172`): troncamento verso zero se nel range;
  NaN/Inf/out-of-range → comportamento d'errore (deprecation/0) — verificare nei .phpt.
- **to string**: `null→""`, `false→""`, `true→"1"`, int → decimale, double → §8.
- **Cast esplicito `(int)"abc"` = 0 senza errore**; aritmetica implicita su stringhe è in §7.

## 6. Confronti

- **`===`** (`Zend/zend_operators.c:2474-2510`): tipi diversi → false; double con
  semantica IEEE (`0.0 === -0.0` true, `NAN === NAN` false); stringhe byte-a-byte;
  array: stessa sequenza ordinata di (chiave, valore) con valori ricorsivamente identici
  — **l'ordine conta**.
- **`==`, `<`, `<=`** (`zend_compare`, `Zend/zend_operators.c:2306-2470`): tabella per
  coppia di tipi. Punti salienti PHP 8:
  - `int|double vs string`: se la stringa è numerica → confronto numerico
    (`compare_long_to_string`, `Zend/zend_operators.c:3278`); se NON numerica → il numero
    viene convertito a stringa e si confronta come stringhe (cambio PHP 8: `0 == "abc"` è false).
  - `string vs string` entrambe numeriche → confronto numerico
    (`zendi_smart_streq`, `Zend/zend_operators.c:3373-3418`); altrimenti byte-compare.
  - `null == ""` true; `null == false` true; bool vs qualunque → entrambi a bool.
  - `>`/`>=` sono compilati come `<`/`<=` a operandi scambiati.
  - NAN in confronto ordinato → mai minore/uguale.

## 7. Aritmetica e concatenazione

- **add/sub/mul** (`add_function`, `Zend/zend_operators.c:1200`; fast path
  `Zend/zend_operators.h:704-806`): int+int con **overflow → double** (controllo sign-bit);
  promozione int→double se un operando è double.
- **array + array**: union, le chiavi del primo operando vincono.
- Operando stringa: numerica → numero; **leading-numeric → numero con
  `Warning: A non-numeric value encountered`**; non numerica →
  `TypeError: Unsupported operand types: string + int` (PHP 8).
- **div**: risultato int solo se divisione esatta tra int, altrimenti double;
  divisione per zero → `DivisionByZeroError` (uncaught → Fatal error, exit 255).
- **mod**: operandi a int; `% 0` → DivisionByZeroError.
- **concat `.`** (`concat_function`, `Zend/zend_operators.c:2017`): entrambi a stringa
  (null→"", false→"", true→"1", int/double→stringa); array → TypeError
  ("Array to string conversion" Warning per echo di array: `"Array"`).
- **++/--** (`increment_function`, `Zend/zend_operators.c:2712`): `++` su null → 1;
  `--` su null → resta null; `++/--` su bool → no-op; su stringa numerica → aritmetica;
  `++` su stringa alfanumerica → incremento Perl-style con carry
  (`"a"→"b"`, `"z"→"aa"`, `"a9"→"b0"`, `"Az"→"Ba"`); dettagli di deprecation 8.3+
  da verificare sui .phpt.

## 8. Formattazione double → stringa (rischio n.1 del progetto)

Due modalità distinte:
- **echo / conversione a stringa**: precision=14 cifre significative, trailing zero
  rimossi, notazione esponenziale per magnitudini estreme (`1.0E+15`: il punto decimale
  è sempre presente nell'esponenziale). `0.1+0.2` → `"0.3"`.
- **var_dump / serialize / var_export**: serialize_precision=-1 → **shortest roundtrip**
  (`ext/standard/var.c:329`, `%.*H` con `PG(serialize_precision)`). `0.1+0.2` →
  `float(0.30000000000000004)`.
- Speciali: `INF` → `INF`, `-INF`, `NAN`; `-0.0` → `-0`.

## 9. Formato var_dump (oracle primario dei .phpt)

`ext/standard/var.c`:
- scalari (`:317-332`): `bool(false)`, `bool(true)`, `NULL`, `int(42)`,
  `float(1.5)`, `string(3) "abc"` (lunghezza in **byte**).
- array (`:156, :45`): `array(2) {\n  [0]=>\n  int(1)\n  ["k"]=>\n  ...\n}` —
  indentazione 2 spazi per livello, chiavi int `[0]=>`, chiavi stringa `["k"]=>`.
- Nesting oltre profondità → `*RECURSION*` (non raggiungibile senza riferimenti in Tier 1).

## 10. Diagnostica (formato esatto, metà degli EXPECTF la contiene)

- Errori non fatali a display_errors on (`main/main.c:1493`):
  `\n` + `Warning: <msg> in <file> on line <line>` + `\n`
  (stesso formato per `Deprecated:`, `Notice:`, `Fatal error:`).
- Uncaught exception (`Zend/zend_exceptions.c:756`):
  `Fatal error: Uncaught <Class>: <msg> in <file>:<line>` + `\nStack trace:\n#0 {main}\n  thrown in <file> on line <line>`.
- Messaggi chiave Tier 1: `Undefined variable $x` (Warning), `Undefined array key "k"` /
  `Undefined array key 0` (Warning), `Too few arguments to function f(), 1 passed in %s
  on line %d and exactly 2 expected` (ArgumentCountError).
- **Exit code**: fatal error → 255 (`Zend/zend.c:1625`); `exit(n)` → n; default 0.

## 11. Edge case noti (tabella di guardia per i test)

| Caso | Comportamento atteso |
|---|---|
| `PHP_INT_MAX + 1` | double `9.2233720368547758E+18` |
| `"10" == "1e1"` | true (entrambe numeriche) |
| `"abc" == 0` | false (PHP 8) |
| `[] == false`, `[0] == true` | true, true |
| `$a[1.5] = x` | chiave 1, Deprecated: Implicit conversion |
| `$a["08"]` vs `$a["8"]` | chiavi DIVERSE (stringa vs int) |
| `unset($a[ultimo]); $a[]=v` | riusa indice successivo al max storico |
| `intdiv` overflow, `0/0`, `1%0` | DivisionByZeroError |
| `"z"++` | `"aa"`; `"9"++` → int 10 |
| echo di `true`/`null` | `1` / stringa vuota |

## 12. Pipeline di esecuzione (osservabile)

1. Solo il codice tra `<?php ... ?>` è eseguito; il testo fuori dai tag è emesso verbatim
   (incluso l'HTML before/after — Tier 1: gestire testo prima/dopo i tag).
2. Le funzioni utente sono **hoisted**: chiamabili prima della dichiarazione testuale
   (se la dichiarazione è top-level e incondizionata).
3. `echo` accetta più argomenti separati da virgola; `print` ritorna 1.
4. Output unbuffered su stdout per il CLI (l'ordine output/warning è interleaved).

## Criterio di completezza

Un lettore (umano o LLM) che parta da questo documento + i .phpt di `Zend/tests` deve
poter implementare il Tier 1 senza leggere altro C, ricorrendo al sorgente solo per i
casi ambigui che i test evidenzieranno.
