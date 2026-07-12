# Prossima sessione: recon symfony/http-kernel (+ residui ext/session)

Riprendiamo phpr (PHP 8.5.7 in Rust, questo repo). La sessione 2026-07-12 ha
CHIUSO il filone **ext/session** (5 commit `e2b2675`→`f652cce`): INI table
mutabile, 23 funzioni + SessionHandler/interfacce, sezione `--INI--` nel
phpt-runner, diagnostiche dentro lo stack OB (fix engine), PHPUnit process
isolation (CLI `-d`/`-n`/`-f`/`-r`/stdin). Dettaglio e lezioni in memoria:
`php-rust-ext-session-2026-07-12`.

## Dove siamo
- Zend corpus **2429** · cargo 1528 · ext/session phpt **150/229** ·
  ext/date 157.
- Symfony http-foundation FULL suite (Tests/Session riammessa): **1790 test,
  10E/27F**; config no-session invariata **0E/12F** (i 12 = `php -S`).
- Workspace: scratchpad sessione 56c2e188 (se è evaporato, ricetta nel topic
  file http-foundation in memoria; composer SEMPRE via oracolo).

## Obiettivo primario: recon symfony/http-kernel
- Clone + composer install (oracolo), baseline della suite sotto phpr,
  categorizzazione per cluster (il playbook è quello di http-foundation:
  RECON-FIRST, probe oracle per ogni contratto, batch gated).
- event-dispatcher è già verde; la session c'è: i prerequisiti noti sono chiusi.

## Coda opzionale ext/session (se un cluster http-kernel ci sbatte contro)
- Tail Symfony Session: 10E/15F (NativeSessionStorageTest regenerate/setup,
  MockFileSessionStorage, AbstractSessionHandlerTest#testSession data-set).
- phpt residui: trans-sid URL rewriting (~15-19 test), costante SID (+
  deprecation-on-read), unserialize offset reale + `r:`/`R:`, `&` nel var_dump
  di $_SESSION, open_basedir, ReflectionFunction su builtin interni.

## Invarianti (identici alle sessioni precedenti)
- Gate per OGNI commit: byte-id vs oracle 8.5.7 · Zend corpus diff per NOME
  (`--list-fails`; col runner che amplia la coverage: nuovi fail ammessi SOLO
  se ex-skip) · ext/session phpt per nome · cargo test · ORM gate se si toccano
  ref/arg/reflection (baseline 3e/15f).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; log con
  `LC_ALL=C tr -d '\0'`; Serena per nav/edit Rust; Read tool per i .php/.tpl
  (RTK collassa i body anche in `head`).
