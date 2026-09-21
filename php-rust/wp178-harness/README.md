# Riproduzione S-178

Indagine diagnostica, **nessuna ottimizzazione di produzione**. `criterio.md`
precede i run; l'emendamento richiesto dall'utente esclude tempi con i quattro
processi di carico attivi. `probe.rs` registra conteggi, la colonna finale è
sempre zero e non rappresenta un costo misurato.

Gli script richiedono macOS, toolchain Rust `stable` verificata **1.98.1**,
Python 3, oracle e tarball indicati nel rapporto. Il file Rust della sonda viene
iniettato esclusivamente nell'archivio esterno: nessun crate del checkout cambia.

1. Verificare assenza di altre misure/build, Data ≥10 GiB e acquisire con
   creazione esclusiva `/private/tmp/phpr-measure.lock`, testo `s178-orm-a98b`.
   Non sovrascrivere lock o directory esistenti. Gli output di questa corsa
   sono in `/private/tmp/phpr-s178-investigation`; archiviarli prima di un replay.
2. `git archive a1eb40e8 php-rust` → `source.tar`, estrarre in
   `/private/tmp/phpr-s178-investigation/src/`; estrarre `orm-work.tgz` in
   `/private/tmp/phpr-s178-investigation/work/`. Conservare SHA-256 del tarball.
3. Eseguire `python3 inject.py /private/tmp/phpr-s178-investigation/src/php-rust`.
   Lo script rifiuta una seconda iniezione e verifica i punti di inserimento.
4. Avviare detached `run_guarded.py --cwd <src/php-rust> build-final -- cargo +stable build --release -p php-cli`
   con `CARGO_TARGET_DIR=/Users/francescotinti/Claude/phpr-s178-investigation-target`,
   `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0`. Leggere il JSON `.done` prima di proseguire.
5. `python3 validate.py <target>/release/phpr`: richiede output byte-identico
   a oracle/pin e conteggi esatti. Poi, senza build concorrenti, avviare detached
   `run_guarded.py --cwd <work/orm-work> census -- <target>/release/phpr vendor/bin/phpunit --no-coverage`
   con `PHPR_S178_MODE=count PHPR_S178_OUT=/private/tmp/phpr-s178-investigation/census.tsv`.
6. Attendere `.done`, confrontare i nomi falliti e il riepilogo con
   `wp125-harness/orm-baseline-failnames.txt`; rc PHPUnit=2 è atteso, non è rc del gate.
   `python3 analyze.py <census.tsv>` produce JSON aggregato. Eseguire anche
   la suite sullo stesso binario con sonda spenta, sequenzialmente, e confrontare.
7. Re-hash del pin e sonda; rimuovere soltanto il lock ancora di propria proprietà.

`site_repeat` è condizionato alla riga (sito IC × classe × file × percorso ×
categoria), non globale; `object_repeat` usa identità Weak e chiave storage.
Sono tentativi nell'handler osservato, non tutte le scritture possibili del
runtime: Reflection, altre forme opcode e fallimenti prima del lazy forwarding
non rientrano. I valori -1 sono non classificati. Hook/magic non risolti non
alimentano lo storico dello slot. Weak non conserva il payload vivo, ma
trattiene temporaneamente il blocco Rc: il census non è un benchmark.
