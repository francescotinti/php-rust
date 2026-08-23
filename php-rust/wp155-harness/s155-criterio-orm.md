# Criterio S-155 p.1b — rimisura ORM (+dbal companion) @ pin s154 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Metodo: `s155-orm-coppia.sh` = COPIA DICHIARATA di s154-orm-coppia.sh
   (copia-gate rc=0, manifest s155-orm-copia.diff): N=2/lato, oracle prima
   (memory_limit=-1 §3.14), workspace ri-untarrati, watchdog, /usr/bin/time -l,
   pavimenti med3 per-binario PER WORKSPACE misurati, gate ictx/s, lock della
   SESSIONE verificato. Pin atteso s154 bddc050320a6af4c (pena rc=9).
2. Riferimento @ s153 (s154-orm-coppia-verdetto.out): phpr ORM net
   [34,76; 34,80] s · rapporto 7,051–7,073 · oracle net ~4,9 · dbal net
   8,11–8,12 / rapporto 7,440–7,450 (companion, NON arbitra).
3. Attesa CE1 PRE-REGISTRATA (NEXT_SESSION S-155 p.1): piccola ↓ 0,03–0,07 s
   (≈4,87M chiamate class_exists/run × 2 alloc × ~7 ns), SOTTO la risoluzione
   0,293 (KS-146-1) — si dichiara QUALUNQUE esito; se Δ dentro il rumore,
   NESSUNA cifra attribuita a CE1. ATTENZIONE attesa-oltre (precedente BT2):
   il canale copy/free può eccedere — un ↓ fuori rumore NON si allarma né si
   attribuisce senza sonda (la sonda ce-count è p.2 di sessione).
4. Giudizio meccanico su Δ = ref_net − new_net (intervalli): Δ_min ≥ 0,293 ⇒
   ↓ FUORI RUMORE, oltre-attesa (dichiarare) · Δ_max < −0,293 ⇒ REGRESSIONE
   SEGNALATA (indagine) · |Δ| < 0,293 ⇒ COMPATIBILE · a cavallo ⇒ NON RISOLTA.
5. Parità: ORM fail-set per NOME == baseline 16 (pena CIFRA NULLA); dbal
   fail-set stabile tra gambe. Gate a rischio morso PRE-DICHIARATI (az.rev.
   S-154 #3): contesa ictx (SEGNALATA, non gate-kill) · nessuna build in
   finestra; rc SOLO da orm-out/rimisura.done.
