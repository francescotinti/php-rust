# S-160 p.1 — Istruttoria gamba phpr1 ictx (az.rev. S-159 #4: causa, non rimisura)

## Fatti agli atti (rusage completi nei .time delle coppie, riletti 2026-08-28)

| finestra | orm phpr1 | orm phpr2 | dbal phpr1 | dbal phpr2 | segnalate dal gate |
|---|---|---|---|---|---|
| s157 (@s156) | ictx 957 | 2246 | 1561 | 4951 | dbal **phpr2** (orm contesa ok) |
| s158 (@s157) | ictx 3956 | 1296 | 362 | 205 | orm **phpr1** + oracle2; dbal oracle1+2 |
| s159 (@s158) | ictx 5971 | 3176 | 3289 | 188 | orm **phpr1** · dbal **phpr1** |

1. **RETTIFICA di rotazione**: «phpr1 segnalata 3ª finestra consecutiva» è
   sovradichiarato. Su orm sono DUE consecutive (s158, s159); in s157 orm era
   contesa-ok e la segnalata era dbal phpr2. Il gate (>1,5× mediana di 4 valori)
   ha colpito nelle 3 finestre: phpr2, oracle1, oracle2, phpr1 — gambe diverse.
2. **Istruzioni ritirate phpr1 == phpr2** (s159: orm 261,81G vs 261,70G, +0,04%;
   dbal 69,21G vs 69,09G): il lavoro del motore è IDENTICO. L'elevazione è
   preemption esterna: cicli +1,1%/+2,5%, **page reclaims +6k (orm) / +12k
   (dbal) sulla gamba 1**, user quasi immune (Δ 0,11–0,13 s ≈ 0,3–1,5%).
3. In s159 la firma è «gamba-1 di ENTRAMBI i workload» ⇒ candidato: transitorio
   d'APERTURA della finestra ORM (floors + primi untar di ~decine di migliaia
   di file ⇒ mds/fseventsd/dirty-page flush), NON proprietà del workspace.
4. Lo script coppia ORM **non ha il gate di quiescenza** che il pair ha per
   gamba (`wp129-harness/s129-quiescenza.sh`): le gambe partono su untar fresco
   senza prova d'assestamento. Il pair, che il gate ce l'ha, esce 6/6 pulito.

## Esperimento (questa istruttoria, grade=ISTRUTTORIA, non di record)

`s160-istr-phpr1.sh`: replica l'apertura (untar orm-work fresco prima di OGNI
gamba, come lo script reale) e corre **3 gambe phpr ORM consecutive** con:
- sonda daemon prima/dopo ogni gamba (cputime cumulato di mds, mds_stores,
  mdworker_shared, fseventsd, deleted, backupd, mediaanalysisd,
  corespotlightd): il Δ per gamba dice CHI ha mangiato CPU durante la gamba;
- probe quiescenza non bloccante (top -l 2 CPU idle) a inizio gamba;
- rusage completo per gamba (ictx, reclaims, instructions).

**Predizioni falsificabili**:
- (a) transitorio d'apertura: ictx leg1 ≫ leg2≈leg3 E Δ-daemon leg1 ≫ leg2/3;
- (b) primo-run del binario/pagine: ictx leg1 alto MA Δ-daemon ~0 ⇒ causa
  interna (page-in), il rodaggio resta rimedio valido;
- (c) nessuna elevazione: il transitorio dipende dal contesto pieno di sessione
  (pair prima, CI, ecc.) ⇒ solo quiescenza-gate nel criterio t10, rodaggio
  facoltativo.

## Esito (esperimento eseguito 2026-08-28 01:26–01:29, verdetto s160-istr-verdetto.out)

**Predizione (b) VINCE**: leg1 ictx=23.204 (598,2/s) vs leg2=4.735 (127,2/s) e
leg3=4.375 (117,2/s) — elevazione prima-gamba RIPRODOTTA in isolamento e
amplificata (stanotte partenza interamente fredda dopo ~2 giorni); istruzioni
IDENTICHE (261,65G vs 261,51G, +0,05%); **Δ-daemon ≈ ZERO durante le gambe**
(mds +1 s, fseventsd +3 s cumulati sull'intero esperimento, mdworker_shared
0→1; idle probe 80–90%): Spotlight/fseventsd/backupd SCAGIONATI. Reclaims
QUASI PARI (63k/64k/59k): l'asimmetria reclaims delle finestre di sessione non
è il canale. Causa: transitorio INTERNO di primo-run (page-in binario phpr +
tarball freddo + writeback del primo untar), si esaurisce da solo — leg2/3
stabili ~120/s. user leg1 +0,7 s (+2%) da freddo pieno; in-finestra (s159)
l'effetto su user era +0,3%: il danno vero è il GATE ictx che tiene aperte le
attese, non la cifra.

## Esito → criterio t10 ORM (vincolante per p.2)

1. **RODAGGIO non giudicante**: dopo i floors, UNA gamba phpr ORM + UNA dbal
   NON giudicate (né tempi né parità: solo scarico del transitorio), PRIMA
   delle gambe giudicate. Costo ~50 s.
2. **Quiescenza per gamba** (come il pair): gate `s129-quiescenza.sh` prima di
   ogni gamba giudicata (l'ORM non l'ha mai avuto; il pair, che ce l'ha, esce
   6/6 pulito).
3. Gate ictx INVARIATO. Attesa: gambe pulite ⇒ la replica-AL1 diventa
   finalmente giudicabile (criterio p.2b).
4. RETTIFICA di rotazione: «phpr1 segnalata 3ª finestra consecutiva» era
   sovradichiarato — su orm sono DUE consecutive (s158, s159); s157 orm era
   contesa-ok (segnalata dbal phpr2). Il gate a mediana-di-4 ha colpito gambe
   diverse nelle 3 finestre.
