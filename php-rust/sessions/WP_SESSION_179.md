<!-- Verbale redatto da ChatGPT Astra 6 (worktree codex cb7f, 2026-09-21), importato in S-180 senza modifiche di contenuto. Raw: ~/Claude/phpr-s179-cost-resume-results/ -->
# WP_SESSION_179 — diagnosi private non-readonly conclusa
Serie densa completa: circa 0,42–0,43s inclusivi per suite ~37s; priorità fill bassa, nessuno speedup dimostrato.
**Scoreboard storico non rimisurato:** arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 =; WP t20 1,765×; ORM [6,936;7,014]×. **Leve spedite: 0.**
- Richiesta: costo lento private non-RO, PDOStatement + UnitOfWork/PersistentCollection; nessuna produzione/promozione.
- Primo pre-flight fallito preservato in wp179-harness/REPORT.md e phpr-s179-cost-results.
- Ripresa autorizzata: quattro yes verificati e terminati (SIGTERM); nessun altro processo utente/macOS terminato.
- Rust stable 1.98.1; due build release sequenziali in target privata nuova, SOURCE_DATE_EPOCH=0/CARGO_INCREMENTAL=0.
- Archivio sorgente a1eb40e8, controllo puro e sonda senza Weak; crate del checkout e pin preservati.
- Fixture output/stderr byte-identici oracle/pin/control/off/count/time; 17 miss; manca negativo lazy/enum dichiarato.
- AppleDouble eliminati solo dalla nuova suite estratta; cache PHPUnit ripristinata da seed S-178 prima di ogni corsa.
- Census: 4.979.840 miss, private non-RO 1.795.090, private RO 2.438.339, esattamente S-178.
- Parità ORM riepilogo/fail-set: 3484 test, 11989 assertion, 3E/13F/59S/2I, rc2 e16nomi baseline; non full-output.
- Compatibili sette guardie + successo: 1.222.283 =68,090% private non-RO =24,545% miss; non hit futuri.
- Compatibili untyped568.775 / typed653.508; 571.661 esclusi scope/slot/presenza, altri1.146 typed-ref.
- PDOStatement400.472 compatibili; UnitOfWork77.659 e PersistentCollection70.513 typed; Pdo\Sqlite144.258 esclusi.
- Serie rada respinta dopo11run validi: anche terza replica ideale lascerebbe scarto≥5,9813% >5%; raw conservati.
- Criterio denso preventivo separato: R3 off/time16/off/time64, semi20011/30011/40009, stessi binari e workload.
- Dodici run validi, otto tentativi esclusi (2pre-flight,6background); E2×4 a30s, Data≥10GiB, watchdog/lock propri.
- CPU mediana off35,67s; time16/off+0,841%, time64/off−0,056%; off/puro−0,419%, deriva0,810%: entro5%.
- Mediane private non-RO0,417629504s /0,431558400s, scarto3,228%: protocollo aggregato superato.
- Stime inclusive da campioni reali, floor41ns/p9542ns; possibili rientri sovrapposti, non CPU esclusiva/costo candidati.
- PDO ~55–61ms e UOW ~24–30ms con scarti densità6,3%/9,7%; PC ~17–18ms, scarto3,9%.
- Limiti: bias locale metadati, puro precedente non reintercalato, nessuna quota evitabile o speedup misurati.
- Raccomandazione: bassa priorità al fill per questo giudice; eventuale leva richiede A/B concreto e collaudo dedicato.
- Driver concluso, lock rilasciato; hash finali pin/control verificati; final-identity.json include sonda e oracle8.5.7.
- Rapporto wp179-harness/REPORT-resume.md; revisione indipendente MISURA in revisione-s179-resume.md, nessun bloccante.
- Raw persistenti /Users/francescotinti/Claude/phpr-s179-cost-resume-results; criteri e riproduzione in wp179-harness.
- HEAD a1eb40e8 non verde (loc_dente, corpus non raggiunto); gc-idle/L-CM1 non promossi; nessun commit/push.
- Serena disponibile, usata per simboli; Vexp disponibile ma indice incompleto/oltre limite, non prova di copertura.
