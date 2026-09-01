# TEAM-SONDA (Gregg·Bak·Leijen) — riconciliazione S-167, fase 2

## CONVERGENZE (3/3)
GO-CONDIZIONATO; nessun codice R1 prima della sonda arith (che NON ha decomposizione: l'attribuzione a dispatch è presa in prestito da prop). R2/R3 restano chiuse (L-TD1, L-AL3, A3c; Zval 16B pinnata). Misura bilaterale (feedback-one-sided), banda-layout fondata, verdetti a valori veri mai su tick. Fallimento sonda ⇒ istruttoria R4.

## LA SONDA (1 sessione, giudice run-micro.sh m-arith R=5, criterio pre-registrato PRIMA)
Cinque bracci, canale-verdetto a tre voci (pila / front-end / interno-handler):
- **(a) A/B PHPR_REG_LOWER on/off** su arith (phpr; flag esistente) — quanto dispatch sopravvive al reg-lowering [Gregg+Bak].
- **(b) gemello data-stride** (stessa dispatch, operandi su linee distinte): |D|≤0,5 ns ⇒ canale **pila/D-cache REFUTATO** [Leijen].
- **(c) CPU counters su ENTRAMBI i motori**: branch-miss/op, L1I-miss, IPC via `xctrace` template "CPU Counters" (per-processo; `powermetrics` è system-wide: NO; `time -l` non dà PMU). branch-miss/op phpr ≥2× oracle ⇒ **front-end FIRMATO** [Bak+Leijen].
- **(d) drop-census dcn!/iter** (apparato census esistente) — prezza il canale **interno-handler** (clone/drop Zval) [Bak].
- **(e) tetto-fuso** (KS-BAK-167-1): handler straight-line del corpo del driver dietro flag, corpus verde — SOLO se (c) firma front-end; se nemmeno il fuso scende ≤3×, R1 muore.

**Firma**: attribuzione nominata ≥60% del gap 38,2 ns/iter ai tre canali; **soglia GO-leva: canale dominante ≥15 ns/iter**. Chiusura sottrattiva bilaterale ≥90% (forma S-129/131) = **secondo gate** per DIMENSIONARE la campagna, non per la sonda.

## CONFLITTI REGISTRATI
- **Gregg** (dissenso mantenuto): il modello per-passo bilaterale chiuso ≥90% deve precedere OGNI codice R1; per lui (e) è già "codice". Mediazione: (e) è misura-di-tetto dietro flag, non leva promuovibile; il gate ≥90% resta prima della prima leva vera. Kill-switch suo: dispatch+decode <40% del gap o chiusura <90% dopo 2 sessioni ⇒ R4.
- **Bak vs Leijen** su soglia: ns/iter (≥15) vs ratio counters (≥2×). Adottate ENTRAMBE: il canale si nomina col counter, si dimensiona in ns.
- **Leijen**: KS-LE-167-1 (stride muto E counters ≈ oracle ⇒ campagna non parte) — accolto, coincide col fallimento firma.

## DURATA
1 sessione: (a)+(b)+(d) mattina, (c) xctrace, (e) solo a front-end firmato.
