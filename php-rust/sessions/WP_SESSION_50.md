# WP_SESSION_50 — banda light-sweep chiusa (con lezione hot-arm), reflect-cap falsificata, quota Fase 1.4 positiva

> ⚡ **WP-50 (2026-07-24/25, `3922e6b`+`60c7e04`+`f034c6c`)** — **Ob.1: il
> residuo nuovo WP-49 (light-sweep 1,01G ingressi) era un MISMATCH DI
> BOUND: il fast-path sweep-empty confrontava la pressione con la sola
> soglia, il trigger con `max(soglia, purge_floor)` — nella banda [soglia,
> floor) ogni sweep di statement entrava nel corpo a fare NULLA. La PRIMA
> forma della leva (max() calcolato nell'arm caldo) REGREDIVA di +17..35s
> nonostante rimuovesse 825M ingressi: legge WP-44 applicata a una GUARDIA
> (la crescita dell'arm costa più del lavoro risparmiato). Forward-fix
> `gc_sweep_bound` (campo cache, refresh ai 4 siti di mutazione): full
> **816,8s = 13:37 = 2,41×, sotto ENTRAMBI i campioni old stesso-sera
> (828,9/832,7) di ~14s ≈ il risparmio banda teorico — bilancio
> RICONCILIATO**. Census: light 1.014,7M→209,8M (band_skipped 825,6M;
> livello pre-WP-49 ritrovato a 2.215 ingressi su 210M). Reflect-cap 16384
> FALSIFICATA come leva CPU (miss −0,3%). Ob.2: quota Fase 1.4 POSITIVA —
> il canale created media è garbage ciclica collettabile AL 100%
> (114,73MB→0 col full-scan di fine run, 874ms), ma `collect_cycles` si
> radica SOLO dai buffer: i cicli coi refcount fermi sono invisibili anche
> ai collect espliciti — serve il seed full-scan (probe
> `PHPR_GC_EOR_FULL_COLLECT`, census-only).**

## Ob.1c — distribuzione dei 297 round (dai log WP-49 esistenti, zero run)

- 149 call round-1 = **139,7s dei 141,7s** di classify (mean ~938ms/call su
  ~60k root); round-2: 146 round per 1,1s TOTALI; round-3: 2. Il
  loop-until-dry non è un costo. Max round 1,9s.
- ~94k white/call nei round-1 ≈ tutto il freed: i collect post-WP-49 sono
  lavoro UTILE. Il classify residuo non ha più grasso ovvio round-level;
  la leva successiva è ridurre il COSTO del walk, non il numero di round.

## Ob.1a — light-sweep: quota, leva, mechanism-check

- **Quota PRIMA della leva** (dai due census WP-49, workload identico alla
  cifra: notes 4,109G / inserted 282,4M su entrambi): light 209,8M→1.014,7M
  = **+805M ingressi**, main 7.741→14.010. Causa letta nel codice: noop
  (run.rs) vs trigger (mod.rs) usavano bound diversi; dopo un purge con
  vivi<soglia il floor sale a vivi+50k e il buffer RAW staziona nella banda.
- **Leva (`3922e6b`)**: noop bound = `threshold.max(purge_floor)` — nella
  banda il corpo non fa nulla (niente pop: buffer note vuoto; niente
  purge/collect: trigger sotto bound) ⇒ skip = comportamento identico.
  Contatore census `band_skipped` per l'identità di conservazione.
- **Mechanism-check (census full)**: light 209.830.152 + band 825.613.238 =
  1.035,4M ≈ 1.014,7M WP-49 (residuo 2% = pattern shift da leva 2, collects
  297→303); **light_new = 209,83M ≈ 209,83M pre-WP-49 (Δ 2.215 su 210M)**;
  classify_ms 142.407 ≈ 141.828; freed 14.220.683 ≈ 14.227.019 (−0,04%).
- A/B media 6 round: **−0,23% CPU, flat** (2/6) — atteso: su media la banda
  vale solo 8,9M ingressi (~0,2s su 60s); il peso è sulla full.

## Ob.1b — reflect-cap 16384: FALSIFICATA come leva CPU

- Census full a cap 16384: **miss 887.798 vs 890.132 (−0,3%), hit-rate
  16,5% vs 16,3%** — il working set è così sopra ENTRAMBI i cap che il
  raddoppio non converte miss in hit; le clear si dimezzano (108→54) come
  previsto ma non si monetizzano. Anche su media: 470k miss / 11,7% hit /
  28 clear — la thrash non è solo da full.
- **Costo footprint reale (attribuito via census)**: su media i descriptor
  ritenuti stazionano nei buffer ctr come possible-root — `gc-ctr-roots`
  19,14MB (cap 8192, WP-49) → **60,86MB (cap 16384)**: +41,7MB standing ≈
  il **peak +2,64%** dell'A/B leva 2 (CPU flat 0,00%). Census full: RSS max
  1718≈1717 invariato; canale end-state 16,5 vs 29,2MB (dipende dalla fase
  dell'ultima clear). TENUTA per direttiva; candidata a ridiscussione.

## Giudice full (stessa sera, 6 run sequenziali): la saga hot-arm e il fix

| run | binario | CPU (last-sample) | RSS seg |
|---|---|---|---|
| run37 | leve 1+2, forma-1 | 857,9s | 1757 |
| full-old-1 | wp49 | **828,9s** | 1640 |
| run37b | leve 1+2, forma-1 | 870,5s | 1580 |
| full-w50a | leva 1 sola, forma-1 | 848,1s | 1592 |
| full-old-2 | wp49 | **832,7s** | 1618 |
| **run37c** | **leve 1+2 + bound cache (`f034c6c`)** | **816,8s** | 2055 |

- Old n=2 con spread 3,8s ⇒ la regressione forma-1 (+17..+40s) era REALE,
  non rumore; girata in mezzo ai new ⇒ non drift macchina. I census non la
  vedevano (classify 142,4≈141,8s; freed −0,04%; miss reflect −0,3%): il
  costo era I-CACHE — la forma-1 aggiungeva un load + `max()` all'arm
  `Op::Sweep` (~1,07G dispatch). **Forward-fix: `gc_sweep_bound` cache
  (refresh ai 4 siti di mutazione di threshold/floor), arm di nuovo a un
  load+cmp ⇒ 816,8s = old−14s ≈ il risparmio banda teorico (~16s) —
  riconciliato a ~2s.**
- Il presunto "+16s della leva 2" nella forma-1 (w50a↔run37/37b) non è
  separabile dal layout-shift della forma-1: run37c include il cap 16384 e
  sta SOTTO old. Attribuzione per-leva della forma-1: non significativa.
- Tutte e 6 le run: fail-set **BYTE-ID a run33** (88 nomi).
- ⭐⭐ **la telemetria .rss di wp16 è in APPEND: il max va calcolato PER
  SEGMENTO** — il "RSS max 5431 invariato" citato da run34 in poi era il
  max cross-segmento (plateau di un'era precedente); i massimi veri di
  segmento: run35 2342, run36 2113, run37 1757, run37c 2055 (rumorosi,
  nessun segnale per-leva).
- run36 (ieri, old): 834,5s — coerente coi due old odierni.
- A/B media NON ripetuto sul bound-cache (l'A/B forma-1 era già flat
  −0,23%; il fix riduce solo l'arm — atteso ≤ forma-1). Nessuna media-run
  del binario finale: il giudice è la full.

## Ob.2 — Fase 1.4: quota POSITIVA (predizione-misurata, media group)

- **Fatto architetturale** (letto in collect_cycles_inner): il collector si
  radica SOLO da gc_buf/gc_cycle_roots/gc_ctr_roots — la garbage ciclica i
  cui refcount non si muovono più NON è mai rooted: invisibile anche a
  `gc_collect_cycles()` esplicito. Nessuna disciplina di confine può
  liberare più di un collect con seed FULL-SCAN da `created`.
- **Probe (`60c7e04`, census-only)**: `PHPR_GC_EOR_FULL_COLLECT=1` seeda
  tutto `created` prima del walk census. Media master: **created 94.565→
  22.141 (−77%), freed 723.378 zval in 874ms** (classify 686ms su 150k
  root); `created-registry-only` 114.730.474 → **0 byte** (riga omessa dal
  dump: sotto-zero); Δroots_total −114.707.484 = 99,98% del canale.
  **Il canale created media è garbage ciclica collettabile al 100%.**
- Bonus: il collect drena anche gc-ctr-roots (60,9MB→0, contenuto
  ri-attribuito al vero owner reflect-cache nel walk post-collect).
- **Numeri per-confine**: costo di UN full-scan a fine media = 874ms.
  Un collect per test = insostenibile (30k×anche solo 5ms ≫ classify
  attuale 142s); la forma sensata è cadenza per CRESCITA di `created`
  (es. ogni +50k) o fine-run. Il valore sul PEAK (mid-run, WP-48: cieco
  alle ritenzioni di fine-run) va misurato sulla FULL: primo atto WP-51 =
  census full con probe (quota canale 353,7MB e costo full-scan a scala
  full-graph).

## Parità e gate

- corpus **1421 IDENTICO per nome col conteggio** ×2 (su `3922e6b` E su
  `f034c6c` post-fix; baseline WP-46)
- 6 full run stesso-sera: fail-set **BYTE-ID a run33** (88 nomi) su tutte
- gate22 completo girato su `60c7e04` (leve forma-1 + probe; include il
  gate reflection obbligatorio per il cap); il fix `f034c6c` è GC-only →
  gate di classe GC: corpus 1421 + cargo 1639/0 + full run37c BYTE-ID
- gate22 completo VERDE col conteggio (archivio `gate-out-wp50-archived/`):
  sess 28 · date 351 · refl 290 IDENTICI · ORM 3484 3E/13F per nome · hk
  1665 0E/0F · probe gd/mysqli/media byte-id · http DIFF-set atteso (2
  item WP-14) · **option 413/1061 · restapi 3508/15947 (1E = oracle)
  IDENTICI per nome COL conteggio**
- cargo **1639/0** (gate) + run pre-commit exit-0 (conteggio troncato dal
  tee: verbalizzato, non contato ×2)

## ⭐ Lezioni

- ⭐⭐ **La legge WP-44 vale anche per le GUARDIE, non solo per i corpi
  handler**: un load+`max()` in più nell'arm caldo `Op::Sweep` è costato
  +17..35s di I-cache, PIÙ del risparmio di 825M ingressi-nulli — e i
  census non potevano vederlo (contano lavoro, non layout). Cura: campo
  cache mantenuto ai siti di mutazione, arm invariato. Il full A/B
  stesso-giorno (old n=2, spread 3,8s) è ciò che ha reso la regressione
  distinguibile dal rumore E il fix verificabile (816,8 vs 828,9/832,7).
- ⭐⭐ **Quando un fast-path e il suo trigger usano bound diversi, la banda
  tra i due è un generatore di ingressi-nulli**: la quota della banda era
  leggibile GRATIS dai due census già su disco (delta su workload identico
  alla cifra) — prima di strumentare, controllare se i numeri esistono già.
- ⭐⭐ **Telemetria in append ⇒ max per segmento, mai sul file intero** (il
  "5431 invariato" tramandato da 3 sessioni era un artefatto di lettura).
- ⭐⭐ **collect_cycles vede solo ciò che è stato notato**: l'esplicito NON è
  un full-GC; per de-pinnare cicli quiescenti serve il seed da `created`.
  (Il probe da 45 righe ha risposto in 90s di run a una domanda da fase
  intera di roadmap.)
- ⭐ Un raddoppio di cache cap sotto thrash strutturale (working set ≫ cap)
  non compra hit: compra solo ritenzione — misurare hit-rate PRIMA di
  toccare il cap, e cercare l'owner della cardinalità (ClassId freschi dei
  mock PHPUnit), non la taglia.
- ⭐ Le assegnazioni env in shell si riconoscono PRIMA delle espansioni:
  `$(cond && echo VAR=1) cmd` non è un env-prefix — serve il ramo esplicito.
- ⭐ Stash dei binari A/B: il path canonico è `/Volumes/Extreme Pro/Claude/
  phpr-old-target/` (drive esterno) — uno stash in `~/Claude/phpr-old-target`
  ha prodotto un A/B con 12 run phpr da 0,00s (accorto dal report, non dal
  done-marker: i marker dicono "finito", non "valido").

## Prossimo (WP-51)

1. **Census full con probe full-scan** (quota canale created 353,7MB a
   scala full + costo del collect a full-graph) → disegno leva Fase 1.4
   (cadenza per crescita di created; PEAK-value da misurare — WP-48: il
   peak fisico è cieco alle ritenzioni di fine-run).
2. Residuo CPU full ≈2,41× vs 2,06× WP-40: il classify (142s) è l'intero
   residuo; leve candidate = costo del walk — SOLO coi numeri per-round
   già in mano (938ms/call su 60k root; il numero di round è già minimo).
3. Fase 1.3 cold-box Object (~96B×istanze) + attribuzione transiente 5,4G*
   (*da rivalutare: il 5431 era un artefatto di lettura cross-segmento —
   il transiente vero di segmento è 1,6-2,3G).
4. reflect-cache: la vera leva è l'OWNER della cardinalità (memo keyed su
   ClassId dei mock = leak WP-47); il cap è un cerotto, 16384 non rende
   (falsificata; eventuale ritorno a 8192 = decisione utente).
