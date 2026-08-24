# Criterio S-158 p.1b — rimisura ORM (+dbal companion) @ pin s157 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Metodo: `s158-orm-coppia.sh` = COPIA DICHIARATA di s157-orm-coppia.sh
   (copia-gate rc=0, manifest s158-orm-copia.diff): N=2/lato, oracle prima
   (memory_limit=-1 §3.14), workspace ri-untarrati, watchdog, /usr/bin/time -l,
   pavimenti med3 per-binario PER WORKSPACE misurati, gate ictx/s, lock della
   SESSIONE verificato. Pin atteso s157 76787303716acd4e (pena rc=9). Fix
   summ()/names() LC_ALL=C EREDITATO (az.rev. S-155 #4).
2. **EMENDA DICHIARATA (indagine S-157, az.rev. #4 — recepita QUI, PRIMA della
   coppia)**: giudizio Δ CANONICO su net **ORACLE-NORMALIZZATO** —
   net_norm(gamba) = phpr_net × (ORA_REF / oracle_net(gamba)), ORA_REF=4,94
   (oracle net della finestra s157, stessa del riferimento assoluto); l'assoluto
   resta stampato come COMPANION con nota drift. Companion aggiuntivo: rapporto
   net vs [7,028; 7,067] (voce registrata S-157).
3. Riferimento @ s157 (s157-orm-coppia-verdetto.out + indagine HD2): phpr ORM
   net [34,82; 34,91] s (NUOVO, con nota drift oracle di giornata +0,9/1,3%) ·
   rapporto 7,028–7,067 · oracle net 4,94 · dbal net/rapporto companion
   (7,701–7,853; leg2 ictx contesa SEGNALATA in s157 — confronto dbal solo
   descrittivo). Reperto conteggi dbal ~3921/626 vs oracle 3929/594 A VERBALE.
4. Attesa L-AL1 PRE-REGISTRATA (NEXT_SESSION S-158 p.1): ≈20 ns × N_miss ⇒
   ~0,02–0,05 s ORM, SOTTO la risoluzione 0,293 (KS-146-1) — si dichiara
   QUALUNQUE esito; se Δ_norm dentro il rumore, NESSUNA cifra attribuita a
   L-AL1. ATTENZIONE attesa-oltre (precedenti BT2, CE1): un ↓ fuori rumore NON
   si allarma né si attribuisce senza sonda.
5. Giudizio meccanico su Δ_norm = ref_net − net_norm (intervalli): Δ_min ≥
   0,293 ⇒ ↓ FUORI RUMORE, oltre-attesa (dichiarare) · Δ_max < −0,293 ⇒
   REGRESSIONE SEGNALATA (indagine) · |Δ| < 0,293 ⇒ COMPATIBILE · a cavallo ⇒
   NON RISOLTA. Lo stesso conto sull'assoluto si stampa come companion.
6. EMENDE S-158 (az.rev. S-157 #3-#4): header del verdetto coi pin dall'HASH
   MISURATO; sentinella language-server nel `.out` a inizio E fine finestra.
7. Parità: ORM fail-set per NOME == baseline 16 (pena CIFRA NULLA); dbal
   fail-set stabile tra gambe (LC_ALL=C, metodo s157 — confronto a parità di
   metodo). rc SOLO da orm-out/rimisura.done.
