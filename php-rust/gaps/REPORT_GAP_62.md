# REPORT_GAP_62 — misure della SOLA sessione WP-62 (2026-07-26 sera)

Binario: tree WP-62 `c20a42d` (phpr-wp62 = 2f5220c7…). Cambi engine
della sessione: contatori unit-cache M1 (path include, freddo) +
refactor reloc marker-espliciti — attesi neutri; gate62 verde completo.

## Media group (gap62, orchestrate62 — 1 coppia oracle/phpr)

| | user CPU | peak phys | vs oracle |
|---|---|---|---|
| oracle 8.5.7 | 21,00s | 367,8MB | — |
| phpr wp62 | 55,05s | 1663,5MB | **CPU 2,62× · footprint 4,52×** |

⚠️ Caveat di lettura: nello STESSO A/B serale (4 run media identici nel
carico) il peak phpr ha oscillato 1611,6-1663,2MB (±1,6%) e lo user
54,17-56,03s (±1,7%); l'oracle stanotte è sceso a 367,8MB da 394,9MB
di WP-60 (−6,9%, singola run). Il 4,52× è quindi rumore di coppia
singola sul numeratore E sul denominatore: il riferimento strutturale
resta ~4,1× finché una media multi-run non dice altro. Nessuna leva
footprint/CPU è stata spedita in WP-62 (leva sospesa al decision
point): non c'è meccanismo che possa aver mosso il footprint.

## Media A/B interno (K-M1, stessa sera, interleaved a1/b1/b2/a2)

A (tree WP-62) 54,74/56,03s user · B (A+relbase-probe) 54,17/54,37s
⇒ B −2,0% = l'indirezione negli arm caldi NON emerge sul media
(micro advisory: +1,0-1,3% sui canali op-densi).

## Full-suite (coppia stessa-sera run49: new c20a42d vs old phpr-wp61)

| | user CPU | peak phys | fail-set |
|---|---|---|---|
| run49-old (phpr-wp61) | 798,11s | 3,885GB | **88 BYTE-ID a run33** |
| run49-new (phpr-wp62) | 791,49s | 3,906GB | **88 BYTE-ID a run33** |

Delta new−old: CPU **−0,83%** (informativo, dentro lo spread serale —
i contatori M1 e il refactor reloc non muovono la CPU), footprint
**+0,54%** (informativo, dentro il rumore; nessuna leva footprint
spedita). Master-CPU riferimento resta ~2,06-2,11× (oracle full non
rimisurato stanotte). PARITÀ CONFERMATA: KS-H5 del concilio (fail-set
88 byte-id come condizione di ratifica) è SODDISFATTO.

## Census62-full (mappa v2 con colonne prefix, binario phpr-memgc62)

Master (pid units=4726): net_tot **1.973.265.968** = riproduzione alla
cifra di WP-61 (…960, +8B); dup_net 1.130,2MB identico;
`nested_windows=0` sul full (K6 pulito). Colonne prefix (per-colonne =
LOWER BOUND per costruzione): globale **prefix 781,0MB / split_tot
2.024,0MB = 38,6% su TUTTE le unit**; 3 bersagli LB = 441,8MB/951,6 =
46,4% del dup. Il minuendo dell'A/B è confermato dal master (version
393,5MB/899 = 437,7KB/inc); la sotto-attribuzione delle colonne è
dimostrata quantitativamente (version "proper"-col = 211KB/inc contro
4,1KB freschi misurati: ricompile dei condizionali del seed + compile
contro ctx grande). unitcache sul full: **miss_fp=2323** (il canale
del leak contato), ways_evictions=2226, hit 0+0.
⇒ **quota stub-elision per design63: banda [781MB misurati
per-colonne … ~1,6GB tetto A/B-estrapolato]**; la ri-derivazione G1
completa (base fresca n≥5, mediana) resta in E0 di WP-63.
