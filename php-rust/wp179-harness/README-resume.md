# Riproduzione S-179 ripresa
Copioni diagnostici per questa sessione; nessuna ottimizzazione di produzione. Prima leggere criterio-resume.md, criterio-dense.md e REPORT-resume.md. I percorsi sono intenzionalmente specifici: per una nuova sessione usare copie dichiarate, nuovo output/target/token e criterio preventivo, senza sovrascrivere risultati o lock.

- Repository di lavoro: /Users/francescotinti/.codex/worktrees/cb7f/php-rust-experiment.
- Output: /Users/francescotinti/Claude/phpr-s179-cost-resume-results.
- Target: /Users/francescotinti/Claude/phpr-s179-cost-target.
- Baseline sorgente: source.tar S-178 (git archive a1eb40e8), estratto sotto output/src; unico adattamento controllo rust-toolchain.toml 1.96.0→1.98.1, scelta utente già autorizzata.
- Build controllo e sonda: `cargo +stable build --release -p php-cli`, env CARGO_TARGET_DIR nuova, SOURCE_DATE_EPOCH=0, CARGO_INCREMENTAL=0. Job JSON e guarded.py registrano comandi, cwd, rc reale, sentinelle. Controllo copiato prima di inject.py/build-probe; fixture e suite solo dopo build completata.
- inject.py opera esclusivamente nell'archivio nominato e verifica cardinalità degli anchor. Sonda nuova probe.rs: nessuna Weak, nessun nuovo fill. Gli hash dei sorgenti iniettati sono in identity.json.
- Fixture: `python3 validate.py <output>/phpr-probe`, contro oracle/pin/controllo e modalità off/count/time.
- Suite: tarball canonico SHA 206cf384f4102b9522c73e378310fad1685586dd1dc124e55fdfd2597220d1bb; escludere AppleDouble. Cache test-results congelata dalla suite S-178; cache-provenance.json ne registra sorgente/hash. Si ripristina solo la copia di lavoro prima di ogni corsa.
- `experiment.py` è la serie rada storica: interrotta dopo undici corse valide perché il criterio di stabilità non poteva più passare. Non riavviarla sopra i risultati esistenti. `experiment.done.json` documenta la decisione.
- `dense.py` gestisce tre sequenze off/time16/off/time64, con nuovi semi e gli stessi binari; produce dense-timings.json e dense.done.json. `python3 summarize-dense.py` genera dense-analysis.json; `python3 class-summary.py` produce class-summary.json con dispersione e numerosità per classe; il controllo puro è quello delle tre corse della fase precedente, limite dichiarato. Confronto fail-set per nome e riepilogo esatto; watchdog 1800s; ogni tentativo conserva raw propri.
- `python3 analyze.py <output>/count.tsv` aggrega conteggi; `python3 summarize.py` produce timing-analysis.json, valid_cost=false se guardie o numerosità non rispettate. La CPU del workload è workload_time da /usr/bin/time -lp; user/system del supervisore includono i processi sentinella e non sono il giudice.
- Guardie: E2 quattro campioni a 30s prima di ogni timing, background CPU <150% durante, Data≥10GiB, lock token proprio. Le corse interrotte non entrano nelle mediane. Dopo il primo stop sono stati aggiunti i top processi alle sentinelle e la causa esplicita di guardia, senza cambiare soglie.
- Il driver rilascia solo il proprio lock alla fine o errore. La continuazione richiede prima acquisizione atomica esclusiva del token. Non lanciare seconda istanza mentre il driver gira.
- La sorgente del corpus precedente, la target S-178 e i pin canonici restano preservati. I nuovi documenti non attestano batteria/corpus verdi e non autorizzano promozioni.
