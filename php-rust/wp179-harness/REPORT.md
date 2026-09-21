# S-179 — verifica della finestra di misura ORM
Il carico artificiale occupa ancora quattro core: la finestra non è ammissibile per misurare le scritture private secondo il protocollo.

**Scoreboard storico, non rimisurato:** arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 =. WP t20 1,765×; ORM [6,936;7,014]×. **Leve spedite: 0.**

## Esito e limite
Pre-flight nuovo sul worktree cb7f, non dedotto dai PID storici. E2 richiede CPU totale <150% per quattro campioni distanti 30 secondi. **E2 FALLITO:** 454,0%, 511,6%, 477,7%, 477,9%. Data libera 19,58 GiB circa, swap usato 1760,88–1768,88 MiB. `preflight.done.json` contiene stato logico rc=8 (non exit code del processo supervisore), E2_pass=false. Lock proprio rilasciato e assenza verificata. Quattro processi `yes` (62666, 62668, 62670, 62672) risultano tuttora attivi, ciascuno vicino al 100%. Nessun processo estraneo è stato terminato.

Non sono stati eseguiti build, suite, census o timer. Costo effettivo, quota ammissibile dinamica e beneficio quantitativo restano **non misurati**. Il blocco riguarda la finestra temporale; non dimostra che l'ottimizzazione sia utile o inutile. I tempi della sonda S-178 non sono riciclabili: il campo ns è zero e Weak/mappe alterano il lavoro osservato.

## Perimetro semantico verificato staticamente
Lettura di `crates/php-runtime/src/vm/run.rs`, righe 960–1278, HEAD a1eb40e8: hit con classe coerente, assenza lazy/enum, slot presente e vincoli sui riferimenti; il lento controlla hook, visibilità, magic, readonly e coercizione, quindi scrive usando la chiave effettiva. Il fill NP corrente richiede chiave semplice, slot risolto e assenza strutturale di __set. Rimuovere il solo confronto key==name non è una proposta validata: il fallback dell'hit usa ancora il nome semplice e il nuovo perimetro deve conservare scope/layout e chiave mangled.

I conteggi S-178 (1.795.090 private non-readonly; 400.472 PDOStatement) restano soltanto frequenze. Non provano che tutti questi tentativi superino le guardie di un fill privato. Pdo\Sqlite eredita da PDO: non assimilarlo a obj_class==scope. UnitOfWork e PersistentCollection restano controlli ORM; nessuna nuova graduatoria per costo è disponibile.

## Decisione
Mantenere PDOStatement come ipotesi da misurare, senza implementare il fill. Beneficio plausibile qualitativo: evitare risoluzioni ripetute per il sottoinsieme compatibile; nessuna percentuale o tetto temporale difendibile. Il prossimo passo resta una finestra E2 valida, census delle guardie, sonda temporale validata e calibrata sul carico completo e solo successivamente A/B diagnostico con stessa toolchain/controllo nullo. Non promuovere pin o L-CM1.

## Provenienza e riproduzione
Criterio preventivo: `criterio.md`. Supervisore originale di questa verifica: `/Users/francescotinti/Claude/phpr-s179-cost-results/preflight.py`; comando `python3 /Users/francescotinti/Claude/phpr-s179-cost-results/preflight.py`, avviato detached e verificato con pgrep. Lock acquisito con O_EXCL, token `s179-cost-cb7f`, rilasciato soltanto se ancora proprio. Questo è un pre-flight senza processo di workload: non richiede watchdog PHPUnit; eventuali corse future lo richiedono.

Raw persistenti: `/Users/francescotinti/Claude/phpr-s179-cost-results/`: ps-0..3.txt, sentinels.json, preflight.done.json, log, rustc.txt, sources.json con SHA-256 delle fonti lette e pin CLI. Nessun harness storico eseguito o copiato con path canonici. Fonti nuove S-178 lette direttamente dal worktree a98b, preservate; manifest/identità/validazioni persistenti letti dal deposito S-178. Non sono nuovi risultati di questa sessione.

Rust stable verificato 1.98.1 (48a229cea, LLVM 22.1.8); nessuna build. HEAD a1eb40e8 invariato, gc-idle/L-CM1 non promossi; CI loc_dente nota rossa, corpus non raggiunto. Pin CLI verificato per hash, server non usato; licenze e checkout canonico intatti. Serena/Vexp non esposti: fallback di lettura mirata dichiarato. Nessun commit/push con gate HEAD noto rosso.

Revisione adversariale: `revisione-s179.md`, nessun rilievo invalidante sul blocco. Prima di una futura misura precisare regione cronometrata, riferimento perturbazione 5%, frequenze, normalizzazione e denominatore; il criterio attuale autorizza soltanto questa verifica preliminare.
