# Team «misura-attribuzione» — Concilio WP-101
Relatore: team Leijen (7) + Gregg (9). Fonti: verbale-7-leijen.md, verbale-9-gregg.md.

## Convergenze

1. **(a) Correzione dei rapporti peak — le DUE gambe, sempre.** Entrambi hanno verificato i raw (pair94/pair99-ratios): la frase «i rapporti peak si muovono per la gamba oracle» è vera sul FULL (phpr −0,45%, oracle +12,1%) ma INCOMPLETA sul MEDIA: phpr 1.170,8→1.202,7 MB (**+2,7% = +31,9 MB**, non piatta), oracle +28,7%. Correzione da scrivere in GAP_TREND e REPORT_GAP: sanare la riga media-peak con entrambe le gambe (A-LE-101-4) e, in forma, pubblicare la **tabella 2×2×2 delle gambe raw** (motore × workload × sera, A-GR-101-1) ogni volta che un movimento di rapporto viene attribuito a una gamba. «Piccolo» ≠ «piatto»: +31,9 MB senza banda è NON MISURATO.

2. **(b) L'anomalia oracle (+28,7% media, +12,1% full) va NOMINATA prima di riusare il metro.** Convergenza piena Leijen-R2 / Gregg-R2+A-GR-101-2: doppio run oracle media adiacente (R≥2, stessa sera) per misurare lo spread run-to-run; cause candidate da discriminare: varianza intrinseca del peak oracle, brew bump/ambiente, cache/ASLR/allocator. Fino ad allora i rapporti peak si pubblicano **come banda**, e KS-GR-101-2 vale: gamba oracle che si muove >10% senza causa nominata ⇒ rapporti non confrontabili cross-sessione.

3. **(c) Gate peak del flip = phpr-off vs phpr-on stessa-sera, MAI vs oracle** (A-LE-101-1), **con bande di accettazione pre-registrate PRIMA del run** (A-GR-101-3: full CPU e peak entro spread WP-94/99 lato phpr). I due kill-switch si compongono: senza banda pre-registrata il verdetto è VOID (KS-GR-101-1); sopra lo spread bandato la promozione si FERMA finché la crescita non è attribuita per owner (KS-LE-101-1).

4. **(e) Shape nuove (punto 4, Sub/cmp):** ogni variante Op porta nel commit N_OPS aggiornato (gate ≤255, const-assert) + `size_of::<Op>()` invariato dichiarato (A-LE-101-3); se lo stride cresce, la variante è RESPINTA qualunque sia il suo D (KS-LE-101-2). Nessuna obiezione di Gregg.

## Conflitti

Nessun conflitto sostanziale; due differenze di enfasi. (d) **Strumento H-C**: Gregg chiede di nominare nel programma lo strumento della gamba oracle (census opcode o profiler) co-equale alla tavola arith-decomposition lato phpr (A-GR-101-4); Leijen non copre H-C ma aggiunge l'asse mancante del flip: **sonda taglia-unità off vs on** (A-LE-101-2), perché la coppia peak da sola può mascherare un calo unit-cache con crescita runtime. Le due sonde sono complementari, non alternative. Su R3-Gregg (cucitura 1,1% tra blocchi «stessa finestra»): solo obbligo di dichiarazione, nessun gate.

## Priorità per l'ordine S-100

1. Sanare i file di rotazione: riga media-peak a due gambe + tabella gambe raw (A-LE-101-4, A-GR-101-1).
2. Spread oracle media R≥2 e nome all'anomalia; rapporti peak come banda fino ad allora (A-GR-101-2).
3. Bande di accettazione del flip PRE-REGISTRATE, gate phpr-off/on stessa-sera (A-GR-101-3, A-LE-101-1, KS-GR-101-1, KS-LE-101-1).
4. Sonda taglia-unità off/on al collaudo flag-on (A-LE-101-2).
5. Punto 4: vincoli N_OPS + size_of::<Op>() nel commit (A-LE-101-3, KS-LE-101-2).
6. H-C: nominare lo strumento lato oracle nel programma (A-GR-101-4).
