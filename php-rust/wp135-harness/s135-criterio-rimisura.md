# Criterio S-135 p.2 — RIMISURA dbal + ORM sul pin s134 (REGISTRAZIONE, non leva) — commit PRIMA del run

1. Oggetto: rimisura BILATERALE **doctrine/dbal** (sqlite) e **doctrine/orm**
   sul pin **s134 61896da1** vs oracle 8.5.7 — DOVUTA (S-133/S-134: due leve
   object-dense spedite dopo le baseline dbal 8,57–8,60 · ORM 8,51–8,56 @ pin
   s125). Rapporti PER workload, MAI aggregato.
2. Metodo = ricetta mappa (s125-criterio-mappa + emenda S-127): N=2 per lato,
   dentro ogni gamba oracle PRIMA (con `-d memory_limit=-1`, §3.14) poi phpr,
   workspace ri-untarrato per gamba dai tarball CONGELATI s126
   (`wp9-harness/gates/{orm,dbal}-work.tgz`), watchdog, `/usr/bin/time -l`;
   pavimento per-binario med3 `phpunit --version` PER WORKSPACE (orm e dbal,
   divergenza dichiarata vs s125 che usava un solo pavimento); **cifra
   canonica = ratio_NET**, raw companion.
3. Contesa: gate in **ictx/s** (emenda S-127) = ictx assoluti / wall (`real`)
   della gamba; soglia >1,5× la mediana del workload; assoluti a verbale.
   Finestra protetta: lock `/private/tmp/phpr-measure.lock` (CI in attesa),
   rust-analyzer/LSP spento prima del run, nessuna altra run.
4. Parità: ORM fail-set phpr per NOME == baseline 16
   (`wp125-harness/orm-baseline-failnames.txt`, pena cifra NULLA); dbal
   fail-set phpr IDENTICO tra le 2 gambe (pena NULLA) + diff vs oracle ≤1%
   dei test ⇒ canonica (baseline 10 nomi Portability/parser-unicode).
5. Arbitro `s135-rimisura.sh` = COPIA DICHIARATA di
   `wp125-harness/s125-mappa-run.sh` collaudata con `scripts/copia-gate.sh`
   (manifest `s135-rimisura-copia.diff` committato); cifre citabili SOLO da
   `s135-rimisura-verdetto.out`; PERF_MAP si aggiorna SOLO da quel verdetto.
6. Lettura pre-registrata (falsificabile): le leve s133+s134 mordono il
   cammino typed-set/ctor, visibile nel profilo ORM (churn multi-%) ⇒ attesa
   DIREZIONE ↓ su entrambi i rapporti; magnitudine NON predetta (nessun A/B
   proprio su suite: qualunque Δ resta «direzione», REGOLE §4). Rapporto che
   sale o non scende = REPERTO a verbale (le leve non parlano alle suite),
   non si aggiusta a valle.
