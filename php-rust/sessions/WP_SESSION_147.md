# WP_SESSION_147 — CENSUS UNICO ORM: KILL ARITMETICO SCATTATO (borrow-first slot MORTO al tavolo, 0 righe di leva); coppia ORM rimisurata @ s145 (↓ indicativa)

**In una frase**: il censimento pre-registrato dal concilio ha dato il suo
verdetto in cifre — l'intera strada «prestare invece di copiare» non può
comprare abbastanza (tutto il canale dei movimenti vale 1,27 s su ~37 s di
divario): la strada muore al tavolo PRIMA di scrivere codice, e il prossimo
bersaglio dichiarato sono le allocazioni non ancora attribuite (other 57,9%).

**SCOREBOARD** (pin s145 a89faf32+4a9adc51 INVARIATO): **arith 5,5 → · prop
5,5 → · calls 4,8 → · str 4,3 → · arr 3,2 → · re 2,5 →** (micro n.r.: pin
invariato) · WP full n.r. (rif resta 1,765–1,788/0,036; **replica t2 ANCORA
dovuta**) · **ORM RIMISURATA @ s145: 8,370–8,427 (↓ indicativa da 8,59–8,71
@ s138; leg1+oracle1 SEGNALATE ictx, leg2 pulita) · dbal 8,20–8,37 (→)** ·
**leve perf spedite: 0 — DICHIARATO** (sessione census-ordinata dal
concilio: il kill pre-registrato È l'esito; misure eseguite: coppia + census).

## Esiti secchi
1·**p.1a coppia dbal+ORM @ s145 rc=0** (az.rev. S-146 #5 SALDATA; arbitro
  copia-gate 18 righe, criterio PRIMA): denominatori kill rifondati
  (soglia 0,7% = 0,293 s); fuori attesa ±2% sul bordo alto = REPERTO.
2·**p.1b CENSUS UNICO ORM** (strumentazione s147mv/s147dg/zvalcensus_s147
  SOLO feature census; copia-gate 44 righe; parser golden 10/10; smoke
  coerenza esatta): repliche r1==r2 **ESATTE** (delta 0,000% su ogni chiave;
  tot 367,55M == cifra sonda). **KILL KS-146-1 SCATTATO**: ponte slot-load
  0,216 s < 0,293 s (0,74×) ⇒ **ZERO codice borrow-first su slot**; famiglia
  estesa (+ThisPropGet+FieldIsset) 0,414 s = 1,41× ⇒ SOLO fette micro;
  **tetto ASSOLUTO canale movimenti 1,27 s (~3,4% del gap)**; take_str SAFE
  0,029 s (takeable 7,4M, non 104M) ⇒ **TakeSlot chiuso a fortiori** (KS-M3).
3·Reperto operativo: la CI locale ha un MUTEX con le misure (quiet_wait sul
  measure-lock + pattern harness) — niente contesa CI in finestra protetta.
4·Aperture restanti: az.rev. S-146 #1 (replica WP t2) · #3 (test deriva
  peak) · #4 (istruttoria FR1 dimrmw: mutante a parità di layout + disasm).

## ⭐ Lezioni (max 3)
- ⭐⭐ Un kill aritmetico pre-registrato è la morte più economica possibile:
  la famiglia borrow-first è morta al tavolo con ZERO righe di leva scritte.
- ⭐⭐ Una stima su conteggi TOTALI può gonfiare 14×: take_str 0,40 s stimato
  sui clone_str totali vs 0,029 s sul takeable SAFE misurato per tipo.
- ⭐ Il ponte fra convenzioni si costruisce nella STESSA run: slot_reads
  73,3M ≠ 61,8M movimenti-da-slot; i fused Load* fuori lista si NOMINANO.
