# Criterio S-147 p.1a — RIMISURA coppia dbal+ORM sul pin s145 (az.rev. S-146 #5: denominatori KS-146-1) — commit PRIMA del run

1. Oggetto: rimisura BILATERALE **doctrine/orm** + **dbal** (sqlite) sul pin
   **s145 a89faf32** vs oracle 8.5.7 — az.rev. S-146 #5: i denominatori di
   KS-146-1 (±0,7% coppia ORM ≈ 0,26–0,30 s) e il tetto 1,52/37,6 s poggiano
   su S-139 @ s138 e vanno RIFONDATI al pin corrente PRIMA del census.
   Baseline: ORM 8,59–8,71 · dbal 8,15–8,23 net @ s138 (S-139).
2. Metodo = ricetta S-139 INVARIATA (s139-criterio-rimisura.md p.2): N=2/lato,
   oracle PRIMA (`-d memory_limit=-1`, §3.14) poi phpr per gamba, workspace
   ri-untarrato dai tarball CONGELATI s126, watchdog, `/usr/bin/time -l`,
   pavimenti med3 per-binario PER WORKSPACE; **cifra canonica = ratio_NET**.
3. Contesa: gate ictx/s (>1,5× mediana); lock di SESSIONE verificato dallo
   script; pgrep rust-analyzer prima del lancio (il gate CPU arbitra); run
   DETACHED, SEQUENZIALE, solo a CI locale DONE.
4. Parità: ORM fail-set phpr per NOME == baseline 16 (pena cifra NULLA);
   dbal stabile tra gambe (pena NULLA) + diff vs oracle ≤1% ⇒ canonica.
5. Arbitro `s147-orm-rimisura.sh` = COPIA DICHIARATA di `s139-rimisura.sh`
   (copia-gate, manifest `s147-orm-copia.diff` committato); cifre citabili
   SOLO da `s147-orm-rimisura-verdetto.out`.
6. Lettura pre-registrata (falsificabile): nessuna leva spedita dopo s138
   (HC1 0,13% suite · RD1 quota census piccola · FR1 dim-read micro) dichiara
   quota ORM ≥1% ⇒ attesa FERMO entro ~±2% su entrambi i rapporti; qualunque
   Δ resta «direzione» (REGOLE §4). Fuori attesa = REPERTO a verbale; in ogni
   caso i denominatori del census si leggono dalla COPPIA NUOVA (viva), non
   da S-139.
