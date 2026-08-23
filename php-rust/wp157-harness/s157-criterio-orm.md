# Criterio S-157 p.1b — rimisura ORM (+dbal companion) @ pin s156 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Metodo: `s157-orm-coppia.sh` = COPIA DICHIARATA di s155-orm-coppia.sh
   (copia-gate rc=0, manifest s157-orm-copia.diff): N=2/lato, oracle prima
   (memory_limit=-1 §3.14), workspace ri-untarrati, watchdog, /usr/bin/time -l,
   pavimenti med3 per-binario PER WORKSPACE misurati, gate ictx/s, lock della
   SESSIONE verificato. Pin atteso s156 42efea3e34feb390 (pena rc=9).
2. ADATTAMENTO DICHIARATO OLTRE AI NOMI (az.rev. S-155 #4, diagnosi in
   wp156-harness/s156-dbal-summ-fix.md): `summ()` e `names()` con
   `LC_ALL=C tr | grep -a` / `LC_ALL=C sed` — l'output dbal phpr contiene
   byte non-UTF-8 (divergenza latin1 a catalogo) che uccidevano il tr BSD.
   Reperto NUOVO reso visibile dal fix, A VERBALE con questa coppia:
   dbal phpr atteso ~Tests: 3921 / Skipped: 626 vs oracle 3929/594 (il
   companion dbal NON arbitra; la differenza di conteggio si dichiara).
3. Riferimento @ s154 (s155-orm-coppia-verdetto.out): phpr ORM net
   [34,30; 34,35] s · rapporto 6,972–7,053 · oracle net ~4,9 · dbal net
   8,05–8,09 / rapporto 7,385–7,422 (companion, NON arbitra).
4. Attesa HD2-hostcall PRE-REGISTRATA (NEXT_SESSION S-157 p.1 + istruttoria
   census s156): ≈7 ns × N_chiamate dei 6 nomi ⇒ 0,01–0,05 s, SOTTO la
   risoluzione 0,293 (KS-146-1) — si dichiara QUALUNQUE esito; se Δ dentro il
   rumore, NESSUNA cifra attribuita a HD2. ATTENZIONE attesa-oltre (precedenti
   BT2, CE1): il canale alloc può eccedere — un ↓ fuori rumore NON si allarma
   né si attribuisce senza sonda.
5. Giudizio meccanico su Δ = ref_net − new_net (intervalli): Δ_min ≥ 0,293 ⇒
   ↓ FUORI RUMORE, oltre-attesa (dichiarare) · Δ_max < −0,293 ⇒ REGRESSIONE
   SEGNALATA (indagine) · |Δ| < 0,293 ⇒ COMPATIBILE · a cavallo ⇒ NON RISOLTA.
6. Parità: ORM fail-set per NOME == baseline 16 (pena CIFRA NULLA); dbal
   fail-set stabile tra gambe (ora estratto con LC_ALL=C: se il fail-set
   cambia SOLO per effetto del fix d'estrazione, si dichiara e si confronta
   a parità di metodo, non contro estrazioni s155 troncate). rc SOLO da
   orm-out/rimisura.done.
