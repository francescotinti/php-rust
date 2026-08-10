# s126-criterio-mappa2.md — MAPPA perf v2 (estensione PERF_MAP) — PRE-registrato

1. Oggetto: REGISTRAZIONE (non leva) del rapporto phpr(pin s125 002e6cc1)/oracle su: **doctrine/dbal** (sqlite) · **symfony/http-foundation** (no-session, i Functional `php -S` esclusi dal fail-gate come da storico 0E/12F) · **doctrine/collections** · **composer install OFFLINE** (cache calda, `COMPOSER_DISABLE_NETWORK=1`, vendor rimosso prima di ogni gamba). Rapporti PER workload, MAI aggregato.
2. RINVIATE E DICHIARATE (nessun tetto silenzioso): lexer/inflector/event-manager · wp-cli · PHPUnit-self → coda S-127.
3. Metodo = s125-criterio-mappa: N=2 per lato, dentro ogni gamba oracle PRIMA poi phpr, workspace ri-untarrato per gamba (tarball in `wp9-harness/gates/`, costruiti oggi con l'oracle e congelati PRIMA del run), watchdog anti-hang, `/usr/bin/time -l`; cifra canonica = user CPU; pavimento per-binario = med3 `phpunit --version` nel workspace dbal (per composer: med3 `composer.phar --version`). Oracle SEMPRE con `-d memory_limit=-1` (asimmetria §3.14 dichiarata, emenda S-125 p.7).
4. Contesa (az. rev. S-125 #1): ictx per gamba a verbale; gamba >1,5× la mediana del workload = SEGNALATA; se le gambe dello stesso lato divergono >3% E una è segnalata ⇒ gamba segnalata NULLA.
5. Parità per workload: fail-set phpr per NOME IDENTICO tra le 2 gambe (pena cifra NULLA); differenza phpr↔oracle dichiarata per NOME; cifra CANONICA se |failset_phpr diff oracle| ≤1% dei test, altrimenti INDICATIVA. Composer: gamba valida se rc=0 e `vendor/autoload.php` esiste.
6. Arbitro `s126-mappa2-run.sh` committato QUI; cifre citabili solo da `s126-mappa2-verdetto.out`; PERF_MAP.md si aggiorna SOLO da quel verdetto. Nessuna predizione di magnitudine (prime misure).
7. EMENDA (run1, PRIMA della rimisura): la gamba compoff-phpr è crollata a t=0 con `Parse error: unsupported construct (stmt:HaltCompiler)` — phpr NON esegue il phar (stub `__halt_compiler`; capability phar onestamente assente, cfr. stream_get_wrappers). Run1 compoff: gambe phpr NULLE; gambe oracle valide ma NON pubblicabili come rapporto (manca il numeratore). STRUMENTO EMENDATO: composer ESTRATTO (`Phar::extractTo` con l'oracle → `composer-x/bin/composer`), STESSO strumento sui due lati; pavimento = med3 `composer-x/bin/composer --version` per binario; smoke bilaterale rc=0 esatto prima del congelamento del tarball. Rimisura SOLO compoff via `s126-compoff-rerun.sh` (committato QUI), in CODA all'aboff (attesa di `aboff.done`: run pesanti sequenziali); verdetto separato `s126-compoff-verdetto.out`.

## EMENDA S-127 (az. rev. S-126 #3-#5; vale per OGNI esecuzione futura della mappa)
1. **Cifra canonica = ratio_NET** (netto-pavimento per-binario, coerente con REGOLE §3);
   il raw resta companion tra parentesi. PERF_MAP riallineata di conseguenza dallo
   STESSO verdetto (dbal 8,57/8,60 · hf 2,547/2,559 · coll net 8,22 — resta INDICATIVA:
   denominatore netto 0,09 s sotto-scala).
2. **Gate contesa in ictx/s** (ictx diviso durata wall della gamba), soglia >1,5× la
   mediana delle gambe in ictx/s: due gambe di durata 8× diversa non sono più
   SEGNALATE per costruzione. I conteggi assoluti restano nel verdetto come raw.
3. **Correzione a verbale hf** (reperto #3): la frase «diff 17 nomi = famiglia php -S»
   era FALSA per ≥3 nomi (testIntlLocale, testSetTrustedHostsKeepsPatternsIndependent,
   testTrustedHostsAreNotAccumulated: unit puri); la canonicità di hf regge sul
   CONTEGGIO (17/1854 = 0,92% ≤1%), non su quella motivazione. Le righe summary
   phpr/hf/coll nel verdetto s126 sono VUOTE (cattura rotta): il denominatore per-suite
   va d'ora in poi CATTURATO NEL verdetto (`Tests: N` di entrambi i motori); finché la
   cattura non è rifatta, il denominatore hf si cita dal conteggio oracle (1854, run
   S-126 p.2) con questa nota.
