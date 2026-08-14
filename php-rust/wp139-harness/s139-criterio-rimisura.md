# Criterio S-139 p.2 — RIMISURA dbal + ORM sul pin s138 (REGISTRAZIONE, non leva) — commit PRIMA del run

1. Oggetto: rimisura BILATERALE **doctrine/dbal** (sqlite) e **doctrine/orm**
   sul pin **s138 fa17dabd** vs oracle 8.5.7 — RICHIESTA UTENTE 2026-08-14.
   Baseline: dbal 8,36–8,45 · ORM 8,43–8,56 net @ pin s134 (S-135). Il test:
   le TRE leve dim-write spedite dopo quella baseline (AP1 s135 + FD1 s136 +
   FD1-ext RMW s138) muovono la suite? (`$this->elements[$k]=$v` in Collections
   = FieldAssign [Prop,Index] = perimetro FD1). Rapporti PER workload, MAI aggregato.
2. Metodo = ricetta S-135 INVARIATA (s135-criterio-rimisura.md p.2): N=2 per
   lato, oracle PRIMA (`-d memory_limit=-1`, §3.14) poi phpr per gamba,
   workspace ri-untarrato dai tarball CONGELATI s126, watchdog,
   `/usr/bin/time -l`, pavimenti med3 per-binario PER WORKSPACE; **cifra
   canonica = ratio_NET**, raw companion.
3. Contesa: gate ictx/s (emenda S-127), soglia >1,5× mediana del workload.
   Finestra protetta: lock `/private/tmp/phpr-measure.lock` GESTITO DALLA
   SESSIONE (creato a mano a inizio finestra, rimozione esplicita a fine
   sessione): lo script lo VERIFICA soltanto, non lo crea né lo rimuove
   (veto «lock di finestra con trap EXIT altrui»). rust-analyzer spento;
   run SEQUENZIALE: parte in CHAIN solo a coppia WP conclusa
   (s139-chain.sh attende pair-out/pair139-t1.done).
4. Parità: ORM fail-set phpr per NOME == baseline 16
   (`wp125-harness/orm-baseline-failnames.txt`, pena cifra NULLA); dbal
   fail-set phpr IDENTICO tra le 2 gambe (pena NULLA) + diff vs oracle ≤1% ⇒
   canonica (baseline 10 nomi Portability/parser-unicode).
5. Arbitro `s139-rimisura.sh` = COPIA DICHIARATA di
   `wp135-harness/s135-rimisura.sh` collaudata con `scripts/copia-gate.sh`
   (manifest `s139-rimisura-copia.diff` committato); cifre citabili SOLO da
   `s139-rimisura-verdetto.out`; PERF_MAP si aggiorna SOLO da quel verdetto.
6. Lettura pre-registrata (falsificabile): le tre leve mordono il canale
   dim-write su prop-array, canale VISIBILE nel churn ORM ⇒ attesa DIREZIONE ↓
   su entrambi i rapporti; magnitudine NON predetta (nessun A/B proprio su
   suite: qualunque Δ resta «direzione», REGOLE §4). Rapporto fermo o in
   salita = REPERTO a verbale (le leve dim-write non parlano alle suite;
   la prossima leva si sceglie dal profilo SUITE), non si aggiusta a valle.
