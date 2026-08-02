# VERBALE — Lars Bak, sedia 5, Concilio WP-91

VERDETTO: **CONCORDO CON EMENDAMENTI** (cifre riprodotte al byte; tre refutazioni documentali, nessuna capitale).

## Q1 — Ricomputo dai raw: COMBACIA AL BYTE, con due nei

Riletti i 40 raw `m89.slope*/slope0*` (ultima riga `tag=mi_proc win=0 … commit=`; pin coppia pre/post ==2 verificato su TUTTI i 40 file). Modi dominanti: BASE {156.762.112, 228.065.280, 316.276.736, 388.366.336}, RET0 {154.992.640, 252.575.744, 323.485.696, 389.087.232} — identici al g3. LSQ: b_base=19.575.603,2, se=584.722,9, 2σ=[18.406.157, 20.745.049], a=76.611.584 ✓; b_ret0=19.329.843,2, se=1.319.393,3, 2σ=[16.691.057, 21.968.630] ✓. Formula dell'awk (verdict89.sh:349-357) = OLS testuale, dof=n−2=2 ✓. Nei: (1) a W4 BASE il census ha un **TIE x2/x2** (157.745.152 vs 156.762.112) risolto col min in silenzio (riga :340, deterministico ma non dichiarato; sensibilità Δb=−73.728, immateriale); (2) **i marginali RET0 OUT sono DUE**, non uno: 24.395.776 (W4→8) E 16.400.384 (W12→16, sotto fascia di 29.983 B). MEASURE89_RESULTS.md:50-52 dichiara «un marginale fuori fascia» — **REFUTATO dal g3 stesso** (righe 112/114). Grade ADVISORY comunque corretto.

## Q2 — A-BB57 giusto; la soglia 0,8 NON è nel header

se(b) e giudizio fascia: corretti; δ=0,15 nominata ex-ante (COUNCIL_WP90:134, mia sedia). RET0→ADVISORY corretto (bastava già il primo OUT). Ma il header campagna (measure89-campaign.sh:26-28,137) nomina **SOLO 0,5**: «se b_ret0 ~ b_base la ritenzione è esclusa» non è una soglia. La **0,8 vive solo nel giudice** (verdict89.sh:396, post-run per costruzione g1→g3). La riga «judged against the ex-ante P-RET0 **thresholds**» (plurale) è un **overclaim: REFUTATO**. L'esito però è insensibile alla scelta: ratio=0,987 — qualunque confine in (0,5; 0,987] dà NOT-attributed.

## Q3 — Clamped dt 1-5: lettura sottodeterminata

Con purge_delay=0 il decommit è sincrono al free: «teardown del primo dentro la finestra del secondo» e «purge aggressivo in finestra» sono lo STESSO meccanismo a dt 1-2 (dt1.afirst: primo net=34.024.441 gonfiato da swallow, secondo clamped). Ma **dt5.bfirst refuta la lettura pura**: ENTRAMBI i lati deflazionati di ~4,54 M senza clamp (dA=−4.536.912, dB=−4.555.509) — il PRIMO non può essere deflazionato dal proprio teardown; è il decommit di QUALUNQUE lato dentro la finestra aperta dell'altro, con esito sub-ms (coerente con discriminatore UNSTABLE). Esperimento minimo: rerun dt∈{1,5}, entrambi gli ordini, **MIMALLOC_PURGE_DELAY ≥ finestra** con read-back in-band; predizione ex-ante: clamp sparisce e i net tornano a regime overlap ⇒ deflazione=decommit da purge-al-free; persiste ⇒ colpa del bracket/counter. Complementare e risolutivo: A-BB50 net per-thread.

## Q4 — Semantica del NOT-attributed

«verdict-grade … grade inherits the slope grades above» è autocontraddittorio: se eredita, eredita **ADVISORY** (min dei bracci). KL-90-4 dà condizione NECESSARIA (census+braccio), non sufficiente. Regola giusta: il negativo è claim di soglia; è citabile verdict-grade **solo se robusto in-band**. Qui la robustezza C'È ma l'ho calcolata IO, fuori banda: 2σ-floor ret0 16.691.057 ≥ 0,8·b_base=15.660.482; anti-moda W8 (cluster 228M al posto del plateau 252.575.744 x2, patologia del dominante: 2 outlier ripetuti battono 3 vicini non identici) dà ratio 1,019; tie-alt immateriale. Quindi: P-RET0 REFUTATA regge, ma la CITAZIONE va scritta «NOT-attributed, verdict-grade previa robustezza (ora mostrata); label g3 da leggere ADVISORY-inherited».

## Emendamenti

- **A-BB61**: tie nel mode-census dichiarato IN-BAND (`tie=K tiebreak=min`); mai più tie silenzioso.
- **A-BB62**: blocco VATTR emette la robustezza a macchina: ratio, 2σ-floor vs soglie, stimatore alternativo (min-of-R e anti-moda). Solo allora «verdict-grade».
- **A-BB63**: esperimento discriminante clamped come da Q3 (purge_delay≥finestra, read-back, predizione ex-ante nel header).
- **A-BB64** (sanatoria): correggere MEASURE89_RESULTS.md:50-52 — DUE marginali OUT, per nome e byte.

## Kill-switch

- **KB-91-1**: etichetta «verdict-grade» che eredita un braccio ADVISORY senza riga di robustezza in-band ⇒ declassata ADVISORY.
- **KB-91-2**: soglia numerica usata dal giudice ma assente dal header pre-run committato ⇒ esito citabile solo con prova d'insensibilità in-band; senza, VOID.
- **KB-91-3**: doc di risultati con conteggio difforme dal verdict.out di generazione massima (marginali OUT, modi, clamp) ⇒ doc VOID finché corretto.

— Lars Bak, sedia 5, Concilio WP-91
