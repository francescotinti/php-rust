# predictions70 — caccia al residuo ~2 KiB/req + spike outlined (LOCK PRIMA DEI LETTI, K-70.1/KK70-1)

Pre-registro WP-70. Tassonomia CHIUSA degli esiti per OGNI predizione
(G-70.4): {DENTRO, FUORI, NON-CALCOLABILE, NON-ESEGUITA} — l'esito va
nel verdict-file macchina del probe, mai solo in prosa. Ogni pendenza
citata DICHIARA lo stimatore (endpoint | least-squares); soglie con
margine ≥ 0,3 KiB/req = divario massimo osservato tra stimatori
(E-70.4). Pendenze footprint citabili SOLO da metriche allocator-level
per-bin (used_b + used_n insieme); phys/rss MAI come slope (L-70.2 —
phys post-confine dà −19 KiB/req per MADV: corroborazione, non giudice).

## Definizione NUMERICA di "attribuito" (sblocco axum, K-70.1b)

Il residuo è ATTRIBUITO quando ≥ **80%** della mediana della tripla
(P70-T) è riconciliato PER-CAUSA da un contatore dedicato del canale
(non da differenze tra run). KG70-1: KL69-1 non si de-scatta
ri-campionando — riaprono solo attribuzione a meccanismo + verdetto
macchina della tripla (KK70-2: slope in colonna nel verdict-file).

## P70-0 — passo-0: esistenza sul RELEASE strumento-free (E-70.1 ≡ P-70.2)

Protocollo: binario RELEASE di parità WP-70 (ZERO feature census),
wpdev con DISABLE_WP_CRON, metro = statistiche PER-BIN dell'allocatore
stesso (MIMALLOC_SHOW_STATS=1 a shutdown pulito del server: colonna
"current" per size-class) su DUE boot separati a N=1000 e N=5000;
firma per-request = Δcurrent/(ΔN=4000) per bin. vmmap Physical
footprint a checkpoint = solo corroborazione (mai slope). Assert
macchina di finestra: il leg è valido solo se il log server non mostra
richieste extra (curl count == N) e wp-config è canonico (P-70.1).

Bande per-bin (n/req su ΔN, dalla firma Leijen/Hejlsberg 20,0 obj/req):
- bin 112: [7,0, 11,0] (attesa 9,0)
- bin 128: [4,8, 7,2] (attesa 6,0)
- bin 64:  [2,3, 3,7] (attesa 3,0)
- bin 160: [0,6, 1,2] (attesa 0,9)
- bin 32:  [0,7, 1,3] (attesa 1,0)
- totale byte (Σ n×size): [1,5, 2,8] KiB/req (attesa ~2,1)

Esiti: **DENTRO (= ENGINE CONFERMATO)** se totale ≥ 1,5 KiB/req E ≥3
bin su 5 in banda (KS70-1: caccia nel path DECL; la firma release ≡
census conferma il census come mappa). **FUORI-BASSO (= CENSUS-OWN)**
se totale < 0,5 KiB/req ⇒ KL69-1 riclassificato census-own; axum si
sblocca SOLO a riconciliazione ≥95% del metro (KS-P70.2).
**FUORI-FIRMA** se totale ≥1,5 ma firma per-bin ≠ census oltre ±15%
per bin ⇒ KS70-2/KL70-2: il census decade da mappa (o DUE canali);
giudice = solo metro esterno, attribuzione riaperta.
**NON-CALCOLABILE** se lo shutdown non è pulito o mi-stats non
separa i bin ⇒ si passa DIRETTO a P70-T senza letto.

## P70-T — tripla census (G-70.2, fonda self-spread/MDE)

3 leg INDIPENDENTI a boot separati (mai due finestre della stessa
run), binario census WP-70, PHPR_CENSUS_HITS=0 + DISABLE_WP_CRON +
PHPR_UNIT_CACHE_LOG acceso (veti WP-69), N=1000/leg. Finestra di
regime EVENT-ANCHORED con ASSERT MACCHINA nel probe (G-70.1): regime
= [primo R dopo l'ultimo evento una-tantum atteso (warmup include,
nessun cron), fine run]; gradino Δvivi>40 o Δins>40 DENTRO la
finestra ⇒ FINESTRA-INVALIDA automatica, il leg FAILa da solo.
- used_b slope (LS, per leg): banda [1,7, 2,6] KiB/req; mediana
  citabile; concordanza di segno 3/3 obbligatoria.
- **used_n slope = discriminatore PRIMARIO (KL70-1)**: leak vero ⇔
  used_n > 0,5 obj/req (pavimento nk = 0,000); banda attesa
  [15, 25] obj/req (attesa 20,0). La soglia in byte DECADE.
- self-spread della tripla (max−min degli slope) = MDE fondato per le
  cacce future; da riportare nel verdict-file.
- censusown: SOLO flat-check della finestra (L-70.3), mai
  differenziato attraverso confini.

## P70-D — prima ipotesi: defer-mini (E-70.2)

Contatore census-only per-request delle allocazioni RITENUTE del path
DECL deferred (run_deferred: snippet+program+module+delta residui).
Predizione: ~16 defer/req × ~120 B ⇒ banda [1,2, 2,2] KiB/req e
[12, 20] obj/req nei bin 112/128. **DENTRO** ⇒ attribuzione ≥80%
raggiunta a contatore ⇒ fronte axum riaperto dal verdetto macchina
della tripla. **FUORI-BASSO** (< 0,3 KiB/req) ⇒ ipotesi MORTA: si
segue l'ordine Pedersen (b): 1. census-own residuo; 2. tabelle nomi
monotone (global_slot_by_name mere-mention P-69.5, interning);
3. set dedup process-lifetime; 4. cache statement DB; 5. slack
mimalloc per ULTIMO — ognuno con contatore dedicato e banda
pre-registrata PRIMA del letto (P-70.3).

## P70-S — coppia outlined (spike, G-70.5/KB70-1; predictions69 b91b4799 RESTANO il pre-registro del profilo)

Le bande P69-S-a/b e la tabella-decisione di predictions69 (lock
b91b4799) NON si rifanno (KK70-3). Qui SOLO il recinto della coppia:
- cap |ΔIR| coppia outlined-vs-release (media group, stesso
  protocollo G-69.1): ≤ **2%** ⇒ profilo citabile per QUOTE; 2–5% ⇒
  SOLO concentrazione/ranking (quote scontate del Δ misurato); > 5%
  ⇒ outlined RIGETTATO (KG70-3: nessun letto, nemmeno ordinale).
- banda A-A′ CPU user media group pre-registrata: ±1,0% (2× lo
  self-spread IR 0,26% G-69.1, arrotondato); |ΔCPU| coppia oltre la
  banda ⇒ profilo demansionato a evidenza ORDINALE (KB70-1).
- gate70 integrale PASS fails=0 sull'outlined + K-69.3 contatore
  corpi caldi INVARIANTE a 178 (B-70.5), o nessun profilo (KB70-3).
- KH70-3: grep-gate `borrow_mut(` nudi sul diff dell'outlining = 0
  nuovi.

## Vincoli di validità trasversali

- KK70-1: un letto della caccia citato senza questo lock ⇒ NULLO.
- KS-P70.1: letto con wp-config non-canonico o server orfano ⇒
  verdict NULLO, run ripetuta. Ogni probe wpdev apre con assert di
  stato canonico (nessun *.pre-cron*, pkill nel trap).
- KS70-3: pendenza wpdev citabile solo con confine cron escluso PER
  COSTRUZIONE (DISABLE_WP_CRON), mai per timing.
- KG70-2: pendenza su finestra senza assert macchina = NON-CITABILE.
