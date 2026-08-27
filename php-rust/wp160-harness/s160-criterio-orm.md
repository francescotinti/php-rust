# Criterio S-160 p.2b — rimisura ORM (+dbal companion) @ pin s159 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Metodo: `s160-orm-coppia.sh` = COPIA DICHIARATA di s159-orm-coppia.sh
   (copia-gate rc=0, manifest s160-orm-copia.diff): N=2/lato, oracle prima
   (memory_limit=-1 §3.14), workspace ri-untarrati, watchdog, /usr/bin/time -l,
   pavimenti med3 per-binario PER WORKSPACE misurati, gate ictx/s, lock della
   SESSIONE verificato. Pin atteso s159 f2d17f18c00a4049 (pena rc=9). Fix
   summ()/names() LC_ALL=C EREDITATO. Giudizio Δ CANONICO su net
   ORACLE-NORMALIZZATO EREDITATO (assunzione moltiplicativa 1:1 non validata a
   N=2, si dichiara). TRE ADATTAMENTI DICHIARATI dall'istruttoria p.1
   (s160-istruttoria-phpr1.md) e dall'az.rev. S-159 #1:
   (i) **RODAGGIO non giudicante** dopo i floors: UNA gamba orm + UNA dbal per
   MOTORE, mai giudicate (né tempi né parità; ictx/s a verbale come companion
   istruttoria) — scarica il transitorio primo-run (leg1 598/s → leg2-3
   ~120/s, daemon scagionati);
   (ii) **QUIESCENZA per gamba giudicata**: `s129-quiescenza.sh` con retry ×3
   (l'ORM non l'ha mai avuta; il pair che ce l'ha esce 6/6 pulito);
   (iii) **SCALETTA A DUE ESTREMI** (emenda az.rev. S-159 #1, vale per ENTRAMBI
   i giudizi p.2): Δ_min ≥ RES ⇒ ↓ FUORI RUMORE · Δ_max < −RES ⇒ REGRESSIONE ·
   intervallo INTERO dentro (−RES; RES) ⇒ COMPATIBILE · a cavallo su QUALUNQUE
   lato ⇒ NON RISOLTA. RES=0,293 (KS-146-1).
2. **DOPPIO GIUDIZIO PRE-REGISTRATO**:
   (a) **attesa-AM1** vs RIF s159-finestra (misurata @ pin s158):
   REF=[34,81; 34,92] **CON NOTA** (estremo alto da gamba phpr1
   ictx-segnalata), ORA_REF_AM1=4,90 = media DICHIARATA gambe oracle finestra
   s159 (4,89/4,91). Companion: rapporto net vs [7,090; 7,141].
   (b) **replica attesa-AL1** vs RIF s157: REF=[34,82; 34,91], ORA_REF_AL1=4,94
   (catena EREDITATA). CHIUDE SOLO se: 4 letture ictx della finestra nuova
   TUTTE pulite E giudizio (a) non fuori rumore in giù oltre-attesa. Altrimenti
   RESTA APERTA, si dichiara. NOTA: la NON-RISOLTA attesa-RF2 (rettifica S-159)
   resta a REGISTRO e NON si riapre qui (il pin nuovo la confonde con AM1;
   ri-adiudicazione solo su stash fermi in finestra dedicata).
3. Attesa L-AM1 PRE-REGISTRATA: 1 passaggio-dispatcher/elemento × 12,0 ns
   (±2,5) × elementi array_map ammessi al fast. Census RICONFERMATO dagli atti
   (s149, r1==r2 ESATTO): **7.726.741 CHIAMATE** array_map su ORM; il census
   conta CHIAMATE, non elementi, e la frazione ammessa (closure anonima
   semplice arità-1, 1 array) non è censita ⇒ attesa DICHIARATA [0; ~0,28] s
   (lower bound 0 a frazione nulla; ~0,09 s a 1 elemento/chiamata e ammissione
   piena; RES sfiorata solo a ≥3 elementi/chiamata e ammissione piena) —
   SOTTO/ATTORNO risoluzione: si dichiara QUALUNQUE esito, NESSUNA cifra
   attribuita a L-AM1 senza sonda. ATTENZIONE attese-oltre (precedenti BT2,
   CE1): un ↓ fuori rumore NON si allarma né si attribuisce senza sonda.
4. (assorbito in 1.iii — scaletta a due estremi.)
5. EMENDE EREDITATE: header coi pin dall'HASH MISURATO; sentinella
   language-server nel `.out` a inizio E fine finestra.
6. Parità: ORM fail-set per NOME == baseline 16 (pena CIFRA NULLA); dbal
   fail-set stabile tra gambe (LC_ALL=C). Le gambe di RODAGGIO sono FUORI da
   ogni statistica e da ogni gate. rc SOLO da orm-out/rimisura.done.
