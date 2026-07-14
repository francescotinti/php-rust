# Prossima sessione: WORDPRESS-TRACK (kickoff operativo: NEXT_SESSION_WORDPRESS.md)

Riprendiamo phpr (PHP 8.5.7 in Rust, questo repo). La sessione 8 (2026-07-14)
ha **CHIUSO symfony/http-kernel: 1663 test, 0 error / 0 failure** (da 0E/25F;
l'oracle ha 0 fail — parità completa). Dettaglio e lezioni in memoria:
`php-rust-symfony-http-kernel` (sezione SESSIONE-8) e nel changelog di
`PHPR_DIVERGENCES_FROM_PHP.md` (2026-07-14).

Cosa è entrato in sessione 8 (commit gated): visibilità del costruttore a
`new`; is_callable con semantica ZPP completa (static-style, $syntax_only,
&$callable_name); FILTER_VALIDATE_REGEXP; range-check nella weak coercion a
int; enum from/tryFrom = port di zend_enum_from_base; comparazione
DateTime per istante (date_object_compare) + loose_eq array con valori loose;
flock(2) reale sui file stream; INI error_log onorata; attributi sulle
interfacce; ctor Exception/Error condizionale; ⭐ **sweep distruttori eager
dopo ogni statement in ogni body** (prima solo top-level).

## Dove siamo
- symfony/http-kernel **0E/0F (1663)** · http-foundation 0E/12F (soli test
  `php -S`, fuori scope) · ORM 3E/13F · Zend corpus e suite ext/*: baseline
  aggiornate nello scratchpad di sessione 85e6296a (gate-f).
- cargo test **1550/0**.

## Obiettivo: WP-track — kickoff completo in `NEXT_SESSION_WORDPRESS.md`
Ordine roadmap (memoria `php-rust-roadmap-wp-first`): timezone ✅ →
harness wp-cli → SAPI server → WP su SQLite (plugin ufficiale) → mysqli →
gd/media. Policy: byte-parity per ciò che resta in stringhe PHP,
functional-parity via crate per ciò che esce dal processo.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle 8.5.7 · corpus per NOME
  (`--list-fails`) · ext/session+date+reflection per nome · ORM se si toccano
  ref/arg/reflection/date · cargo test.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena per
  nav/edit Rust; Vexp per il C di php-8.5.7; Read tool per i .php/.tpl (RTK
  collassa i body); log con `LC_ALL=C tr -d '\0'`; df PRIMA dei run pesanti.
