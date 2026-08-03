# Verbale sedia 5 — Lars Bak (statistica, stimatori, ricomputo dai raw)

## VERDETTO: CONCORDO CON EMENDAMENTI

Ricomputato TUTTO dai 20 raw committati (`wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus`, righe `tag=mi_proc` per NOME, `tr -d '\0'` prima di grep). **Ogni cifra del g2 riproduce al byte**: modi dominanti per lane (BOOT 21364736/30474240/39387136/48431104; WORK 132513792/198115328/263782400/340983808; PEAK 151781376/228589568/303431680/389808128), b_boot=2252800 se=6345 a=12386304, b_work=17276928 se=501348 a=61079552, b_peak=19723059,2 se≈447215 a=71172096, residuo 193331, marginali 9/9. Il doc riporta il g2 fedelmente (Q4: sì sulle cifre). Le refutazioni sono sugli STIMATORI, non sui numeri.

## Q1 — b_peak degno di citazione? NO come testata

Tutti e 4 i punti PEAK hanno modes==R=5 (tie=4): il "modo dominante" non esiste e tiebreak=min trasforma la lane in una statistica d'ESTREMO. Ricomputo con stimatori alternativi: b_peak(min)=19.723.059, b_peak(mediana)=20.294.861, b_peak(media)=20.274.872 → il min tira giù ~0,55–0,57 MB/worker (−2,8%). Peggio: a W=12 il min NON rompe un tie, SELEZIONA la run anomala r3 (303,4M contro sorelle 325–334M: −7…−10% su TUTTI i contatori della run). Il doc stesso dichiara i punti ADVISORY (A-BB58) ma titola b_peak "PRIMA attribuzione ESATTA": incoerente. La cifra di testata onesta è la somma delle lane FORTI: **b_boot+b_work = 19.529.728 B** (a 45.875 B dal b m89 19.575.603 — continuità migliore di quella vantata).

## Q2 — tiebreak=min: distorce

Sì (vedi Q1). In più i marginali "9/9 IN fascia" sono estimator-dipendenti: con la mediana, WORK W8→12 = 22.183.936 (OUT alto) e W12→16 = 13.271.040 (OUT basso). Il 9/9 regge solo perché min-W8, min-W12 e min-W16 si allineano tra loro (la run anomala W12 DIVENTA il punto W12). Dispersione W12 ~11% del livello: nessun tiebreak la può nascondere.

## Q3 — additività: né forzata né vera, e il residuo è MAL ATTRIBUITO (refutazione capitale)

Within-run l'identità boot+Δwork+Δpost = exit_collect è aritmetica pura (tautologia). Come computata (cross-lane, punti scelti da RUN DIVERSE per lane) non è tautologica — ma allora il residuo 193.331 misura il MISMATCH DI SELEZIONE, non la fisica: per-W vale −2.097.152 / 0 / +262.144 / +393.216, dominato dal W=4 dove PEAK-min pesca r1 mentre BOOT/WORK pescano r2/r4. La label del giudice "the residue is the post-work segment: self+final census allocs" è **REFUTATA dai raw**: Δcommit post-work (exit_collect−pwork) = 0 su **17/20 raw** (non-zero solo w4.r5 +5.046.272, w16.r2 +1.048.576, w16.r3 +5.177.344). Il segmento self+census a commit costa ZERO nel caso tipico.

## Q4 — min-of-R ratio 1.000: controllo VACUO

Sì: con tie=4 su tutti i punti PEAK, dominante≡min per costruzione ⇒ ratio_minR≡1,000 non può fallire. Robustezza reale misurata: min/mediana = 0,972 (passerebbe comunque la soglia 0,8 — ma va misurata, non tautologizzata).

Nota minore: con 4 punti (df=2) il "2σ" è ~82% di confidenza, non 95% (t(0,975;2)=4,30). VCOV: ordine confermato su spot-check (0,42–0,63 per-raw), composizione non ricomputata al byte.

## Emendamenti

- **A-BB64 "Bandire tiebreak=min con modes==R"**: il punto per-W della lane PEAK diventa la MEDIANA delle R run (o il punto additivo per-run); ripubblicare b_peak (~20,29M atteso).
- **A-BB65 "Residuo VLADDER within-run"**: additività giudicata per-run (dov'è esatta) + residuo cross-lane rietichettato "selection mismatch"; la label census/self va ritirata.
- **A-BB66 "Robustezza non tautologica"**: A-BB62 sostituito da ratio min/mediana/media; se lo stimatore di controllo coincide per costruzione col primario, il check è VOID.
- **A-BB67 "Flag run-outlier"**: run con TUTTI i contatori oltre 2·MAD dalle sorelle (W12 r3) marcata per NOME nel verdict; non può diventare il punto della sua W via min.
- **A-BB68 "CI onesti"**: riportare ±t(df)·se o dichiarare la banda "±2se", mai "2σ⇒95%" con df=2.

## Kill-switch

- **KS-BB-92-1**: se una lane ha modes==R su TUTTI i punti, il suo b NON può essere cifra di testata del doc (testata = lane forti o mediana).
- **KS-BB-92-2**: ogni check di robustezza deve poter fallire sul canale reale; un ratio identicamente 1 per costruzione = check VOID, verdict declassato.

Refutazioni capitali: **sì** — la label del residuo (riga 40 del g2) è contraddetta da 17/20 raw, e b_peak min-based è uno stimatore d'estremo spacciato per modo dominante.
