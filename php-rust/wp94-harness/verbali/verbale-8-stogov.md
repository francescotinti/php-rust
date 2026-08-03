# Verbale SEDIA 8 — Stogov (Zend/opcache, contratto LSP) — Concilio WP-94

Oracle: PHP 8.5.7 `/opt/homebrew/opt/php/bin/php`. Probe: `/tmp/stogov94` (33 file, tutti eseguiti). Oggetto: `crates/php-runtime/src/lsp_check.rs` (fase 1, 56 unit).

## VERDETTO

Il checker è SOLIDO sul perimetro delle 51 fixture: ordine bitmask, DNF, null, static, parent/self, canale proprietà — tutti CONFERMATI al byte. Ma il perimetro NON è il contratto: **2 refutazioni capitali** fuori-fixture (iterable-desugar; ctor-prototype da abstract) + 3 under-fatal e 4 famiglie messaggi mancanti. Fase 2 NON consumabile senza A-DS61..65.

## Q1 — raffinamento bitmask (20 probe, canale `&m()` + canale prop)

CONFERMATI: reversal totale `bool|float|int|string|array|object|callable` → `callable|object|array|string|int|float|bool` (= BUILTIN_ORDER esatto); named e gruppi DNF in ordine DICHIARATO (`(A3&B3)|string|(C3&D3)` → `(A3&B3)|(C3&D3)|string`); null ultimo (`int|null|string` → `string|int|null`); collapse `?T` solo binario (`?int`, `?false`, `?static`; `(A4&B4)|null` NON collassa); `static` in posizione FISSA fra named e builtin anche se dichiarato primo (`static|Foo|int` → `Foo|static|int`); `parent` stampato RISOLTO (`A7`/`P`); `Type of C::$x must be string|int` = stesso formatter.

**REFUTATO (capitale): `iterable` non è un atomo di stampa.** Zend lo desugara: `Traversable` entra nella lista NAMED alla posizione dichiarata, `array` nel bitmask: `iterable|int` → `Traversable|array|int`; `Zoo|iterable` → `Zoo|Traversable|array`; `iterable|Zoo` → `Traversable|Zoo|array`; `?iterable` → `Traversable|array|null` (NIENTE collapse). Il checker stamperebbe `iterable`/`?iterable`. `iterable` solo → `Traversable|array` anche standalone. `array|Traversable` esplicito ≡ identico.

## Q2 — famiglie mancanti provate (per NOME)

1. `Class C contains 2 abstract methods and must therefore be declared abstract or implement the remaining methods (I::m, I::n)` — grammatica singolare/plurale; membro-hook stampato `I::$x::get` (proprietà d'interfaccia non implementata).
2. `Abstract function A::f() cannot be declared private` (in classe; nel trait lecito e l'override private è esente — oracle CLEAN ✓ checker).
3. **final const via INTERFACCIA**: `C::X cannot override final constant I::X` (anche enum `E::X`) — il checker raccoglie i const SOLO dal parent: under-fatal.
4. **set-hook in EREDITARIETÀ è famiglia "Declaration of"**: `Declaration of C::$x::set(string|int $v): void must be compatible with P::$x::set(string|int|float $v): void` — con `: void` stampato; contravariante (widen CLEAN). Il checker controlla solo GET: under-fatal. DS60-5 resta vera SOLO intra-classe.
5. set param esplicito NON tipato su prop tipata = fatal famiglia x2 (`Type of parameter $v of hook C::$x::set…`) — il checker passa silenzioso.
6. `Hooked properties cannot be readonly` (anche promoted).
7. RTWC su hook NON sopprime (fatal normale) ✓ checker già conforme; `static` nei param = `Cannot use the static modifier on a parameter` (parser, fase 2); enum case vs const iface non-final: CLEAN.

## Q3 — A-DS59, probe a 3-4 livelli

x6-replica ✓ (cita I); Mid SILENTE ✓ (cita I); Mid che ALLARGA (`int|string`) → il fatal cita SEMPRE I originale ✓; abstract-Mid-implements-I con ctor concreto → cita I ✓.

**REFUTAZIONE CAPITALE (q3b)**: `abstract class A { abstract __construct(int) }` → `B` ctor concreto → `C(string)`: oracle **FATAL** `…must be compatible with A::__construct(int $x)`; il checker dice CLEAN (il ramo `parent.is_abstract` guarda solo il parent DIRETTO e non registra `ctor_proto`). Propaga a profondità 4 (r_q3e, cita sempre A). Il prototipo-ctor nasce da iface **O da ctor ABSTRACT**, e propaga uguale. Il ctor CONCRETO in classe astratta resta esente (q4a/unit ✓). Viceversa (checker fatala dove Zend no): NON trovato.

## Q4 — rischi fase 2 ORM/hk (per NOME)

- **Stub interni tentative con firme ESATTE**: `ArrayAccess::offset{Exists,Get,Set,Unset}`, `Countable::count`, `IteratorAggregate::getIterator`, `JsonSerializable::jsonSerialize` — q4a: Deprecated con `ArrayAccess::offsetGet(mixed $offset): mixed`; senza registry tentative → falsi FATAL di massa su doctrine/collections legacy e proxy ORM.
- **Grafo builtin**: q4b `getIterator(): ArrayIterator` CLEAN solo se il registry conosce `ArrayIterator`/`Generator`/`Closure`/`UnitEnum`/`BackedEnum`; altrimenti falso `…because class ArrayIterator is not available` su `ArrayCollection::getIterator`, Symfony `ParameterBag/HeaderBag::getIterator`.
- Doctrine: proxy generati (extends entity), guardie `class_exists` condizionali (timing t3), trait `insteadof/as` non modellato (`ClassMetadata`), `iterable` nei tipi (desugar Traversable in ogni messaggio).

## Emendamenti

- **A-DS61**: desugar `Iterable` → `Named("Traversable")` in posizione dichiarata + flag array; vietato il collapse `?iterable`.
- **A-DS62**: `ctor_proto` registrato ANCHE dal ctor ABSTRACT (chiave=prototipo; messaggio cita la classe del prototipo).
- **A-DS63**: check set-hook in ereditarietà (contravariante, `: void` in firma) + fatal x2 su set param esplicito non tipato.
- **A-DS64**: const finali raccolti dalle INTERFACCE + famiglia abstract-methods-count (grammatica sing/plur, membri hook).
- **A-DS65**: registry fase-2 con seed builtin (grafo + tentative) PRIMA del gate ORM/hk.
- **KS-DS-94-1**: ogni emendamento nasce con fixture oracle-morsa e pin length-prefixed PRIMA del codice (legge KS-DS-92-2).
- **KS-DS-94-2**: gate ORM 3E/13F / hk 1665 NON consumabile senza A-DS65.

## Refutazioni capitali: **SÌ — 2** (q3b ctor-proto abstract: il checker benedice dove Zend fatala; iterable-formatter: byte-divergenza su ogni messaggio contenente iterable).
