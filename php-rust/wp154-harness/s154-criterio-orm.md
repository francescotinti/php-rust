# Criterio S-154 p.1b — rimisura ORM (+dbal companion) @ pin s153 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Metodo: `s154-orm-coppia.sh` = COPIA DICHIARATA di s150-orm-coppia.sh
   (copia-gate rc=0, manifest s154-orm-copia.diff): N=2/lato, oracle prima
   (memory_limit=-1 §3.14), workspace ri-untarrati, watchdog, /usr/bin/time -l,
   pavimenti med3 per-binario PER WORKSPACE misurati, gate ictx/s, lock della
   SESSIONE verificato. Pin atteso s153 8370c257ae70cc8e (pena rc=9).
2. Riferimento @ s150 (s150-orm-coppia-verdetto.out): phpr ORM net
   [35,52; 35,53] s · rapporto 7,104–7,149 · oracle net ~5,0 · dbal net
   8,23–8,24 / rapporto 7,283–7,491 (companion, NON arbitra).
3. Attesa BT2 PRE-REGISTRATA (NEXT_SESSION S-154 p.1): piccola ↓ 0,05–0,15 s
   (≈473k chiamate × ~20 alloc × ~10 ns), SOTTO/al bordo della risoluzione
   0,293 (KS-146-1) — si dichiara QUALUNQUE esito; se Δ dentro il rumore,
   NESSUNA cifra attribuita a BT2.
4. Giudizio meccanico su Δ = ref_net − new_net (intervalli): Δ_min ≥ 0,293 ⇒
   ↓ FUORI RUMORE, oltre-attesa (dichiarare) · Δ_max < −0,293 ⇒ REGRESSIONE
   SEGNALATA (indagine) · |Δ| < 0,293 ⇒ COMPATIBILE · a cavallo ⇒ NON RISOLTA.
5. Parità: ORM fail-set per NOME == baseline 16 (pena CIFRA NULLA); dbal
   fail-set stabile tra gambe.
