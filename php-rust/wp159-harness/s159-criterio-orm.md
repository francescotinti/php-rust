# Criterio S-159 p.1b — rimisura ORM (+dbal companion) @ pin s158 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Metodo: `s159-orm-coppia.sh` = COPIA DICHIARATA di s158-orm-coppia.sh
   (copia-gate rc=0, manifest s159-orm-copia.diff): N=2/lato, oracle prima
   (memory_limit=-1 §3.14), workspace ri-untarrati, watchdog, /usr/bin/time -l,
   pavimenti med3 per-binario PER WORKSPACE misurati, gate ictx/s, lock della
   SESSIONE verificato. Pin atteso s158 92b0aea36573955a (pena rc=9). Fix
   summ()/names() LC_ALL=C EREDITATO. Giudizio Δ CANONICO su net
   ORACLE-NORMALIZZATO (emenda S-157) EREDITATO: net_norm(gamba) = phpr_net ×
   (ORA_REF / oracle_net(gamba)); assunzione moltiplicativa 1:1 NON validata a
   N=2 (rilievo S-158 #5), si dichiara.
2. **DOPPIO GIUDIZIO PRE-REGISTRATO** (az.rev. S-158 #3):
   (a) **attesa-RF2** vs RIF s158: REF=[34,90; 35,25] **CON NOTA CONTESA**
   (entrambi gli estremi da gambe ictx-segnalate: phpr1=3956, oracle2=1897),
   ORA_REF_RF2=4,995 = media DICHIARATA delle gambe oracle della finestra s158
   (4,97/5,02, stessa del RIF). Companion: rapporto net vs [6,952; 7,093].
   (b) **replica attesa-AL1** vs RIF s157: REF=[34,82; 34,91], ORA_REF_AL1=4,94
   (catena EREDITATA dal criterio s158). La replica CHIUDE l'attesa AL1 SOLO se:
   le 4 letture ictx della finestra ORM nuova sono TUTTE pulite al gate (nessuna
   SEGNALATA) E l'assunzione «contributo L-RF2 sotto-risoluzione» regge (cioè il
   giudizio (a) non esce fuori rumore in giù oltre-attesa). Gambe segnalate o
   assunzione caduta ⇒ attesa AL1 RESTA APERTA, si dichiara.
3. Attesa L-RF2 PRE-REGISTRATA: prezzo MECCANISMO ~7 ns/chiamata (az.rev. S-158
   #2, NON i 14,5 del giudice) × ~11,66M alloc-siti famiglia (census S-158) ⇒
   ~0,08 s, intervallo dichiarato [0,03; 0,12] — SOTTO la risoluzione 0,293
   (KS-146-1). Si dichiara QUALUNQUE esito; se Δ_norm dentro il rumore, NESSUNA
   cifra attribuita a L-RF2. ATTENZIONE attesa-oltre (precedenti BT2, CE1, RF2
   stesso al giudice): un ↓ fuori rumore NON si allarma né si attribuisce senza
   sonda.
4. Giudizio meccanico su Δ_norm = ref_net − net_norm (intervalli), soglia
   RES=0,293, per ENTRAMBI i giudizi p.2: Δ_min ≥ RES ⇒ ↓ FUORI RUMORE
   (dichiarare) · Δ_max < −RES ⇒ REGRESSIONE SEGNALATA (indagine) · |Δ| < RES ⇒
   COMPATIBILE · a cavallo ⇒ NON RISOLTA. L'assoluto si stampa come companion.
5. EMENDE S-158 EREDITATE: header coi pin dall'HASH MISURATO; sentinella
   language-server nel `.out` a inizio E fine finestra.
6. Parità: ORM fail-set per NOME == baseline 16 (pena CIFRA NULLA); dbal
   fail-set stabile tra gambe (LC_ALL=C, confronto a parità di metodo). rc SOLO
   da orm-out/rimisura.done.
