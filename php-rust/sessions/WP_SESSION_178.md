<!-- Verbale redatto da ChatGPT Astra 6 (worktree codex a98b, 2026-09-20), importato in S-180 senza modifiche di contenuto. Raw: ~/Claude/phpr-s178-investigation-results/ -->
# WP_SESSION_178 — indagine ORM per origine, private/readonly e ripetizioni
La suite attribuita a Doctrine include molte scritture del framework di test e del runtime: ora sono distinte, ma i tempi restano da misurare.
**SCOREBOARD storico, non rimisurato:** arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 = · WP t20 1,765× · ORM [6,936;7,014]×. **Leve spedite: 0.**
- Richiesta utente: partire dall'indagine ORM, nessuna successiva ottimizzazione di produzione. CPU >500%, quattro `yes` ~100% ciascuno: utente sceglie esplicitamente lasciare processi attivi e consegnare census+limite tempi. Nessuna misura temporale valida.
- HEAD a1eb40e8 archiviato; Rust stable verificato 1.98.1, release fat-LTO/1CGU, target APFS dedicata. Sonda in archivio, nessuna modifica ai crate; pin S-175 5de14d6856d760a8 verificato invariato. HEAD non verde: blocco loc_dente/corpus invariato; L-CM1 senza verdetto.
- Build finale rc=0, sonda SHA256/16 0cef791ff4c19d94. Fixture oracle=pin=sonda byte-identica, 32 chiamate e categorie/rewrite esatti, incluse private omonime padre/figlio e readonly su nuove istanze.
- ORM count/off: entrambi 3484 test, 11989 assert, 3E/13F/59S/2I, rc=2; fail-set baseline16 identico. Nessuna pretesa di full-output parity ORM.
- Totali: 8.998.101 handler, init 9.219, non-init 8.988.882, miss 4.979.840; hit plain 3.277.200/typed 731.842. Questi margini coincidono esattamente con S-177.
- Incroci miss: private∩RO 2.438.339; private∩non-RO 1.795.090 (36,05% miss); non-private∩RO 512.099; non-private∩non-RO 234.304; unknown 8.
- Private non-RO: 535.617 ripetizioni oggetto/slot, 1.777.221 ripetizioni condizionate sito. Private RO: zero ripetizioni oggetto/slot, 2.437.117 ripetizioni sito. Write-once non significa sito freddo.
- Riceventi PHPUnit: 36,21% miss, 53,93% readonly; non spiegano tutto. PDOStatement primo per miss (400.473); ORM Mapping\Column 328.594 readonly. Prelude esecutore 3.956.010 entrate: conteggi, non tempo.
- Versione corretta tarball: PHPUnit 11.5.56 (non 13), ORM 3.6.x-dev 2bae808d, DBAL 4.4.3. Sorgente PHPUnit 522 readonly class; ORM 0 classi ma 118 private readonly.
- Prossima IPOTESI DA MISURARE: cammino private non-RO, PDOStatement + controlli UnitOfWork/PersistentCollection. Perimetro ammissibile scope/slot ignoto; nessuna percentuale di beneficio difendibile. Readonly seconda candidata separata.
- Rapporto/comandi/criterio: `wp178-harness/REPORT.md`, `README.md`, `criterio.md`; revisione adversariale finale `revisione-s178.md`, senza rilievi bloccanti sul census.
- Raw persistenti: `/Users/francescotinti/Claude/phpr-s178-investigation-results/` (MANIFEST, identità, TSV, log, parità, archivio sorgente). Temporanei originali `/private/tmp/phpr-s178-investigation/`; target `/Users/francescotinti/Claude/phpr-s178-investigation-target`.
- Serena/Vexp assenti, fallback mirato dichiarato. Nessun pin, promozione, modifica dati canonici o arresto di processi utente. Lock s178-orm-a98b proprio, watchdog Data≥10GiB e timeout, run sequenziali detached.
