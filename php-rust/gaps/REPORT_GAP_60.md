# REPORT_GAP_60 — gap perf oracle↔phpr della sessione WP-60

> SOLO le misure della sessione WP-60; trend cumulativo in `GAP_TREND.md`.
> Sessione: revert leva B (nuova baseline **phpr-wp60** `46b517fc…`) +
> riparazioni di metodo + contatori ex-ante del compile-side.

## Media group (stessa sera, 1 run per lato, /usr/bin/time -l, DB reset)

- oracle: user **20,83s**, peak footprint **394,9MB**
- phpr (phpr-wp60): user **53,73s**, peak footprint **1.612,0MB**
- **CPU 53,73/20,83 = 2,58×** · **footprint 1,612/0,395 = 4,08×**
  (entrambi flat vs WP-58 2,61×/4,07× nel rumore inter-giornata)

## Full-suite (coppia stessa-sera run48, metro esatto time -l, tree user)

- **new (revert, 915ea27)**: user **781,39s**, peak **3,900GB**,
  fail-set **88 BYTE-ID a run33** ✓
- **old (phpr-wp58)**: user **788,72s**, peak **3,907GB**, fail-set
  **88 BYTE-ID a run33** ✓
- Δ: CPU **−0,93%** (informativo — meccanismo pool-off −0,56% + spread
  ±0,6%), footprint **−0,19%** ✓ (verdetto ±0,5%)
- Riferimento storico master-CPU: resta run46 **2,11×** (telemetria .rss
  del new agganciata al wrapper time: profilo assente, metro non toccato)

## Contatori ex-ante (census phpr-memgc60, counting-allocator clone-delta)

| contatore | media | --list-tests | FULL (master) |
|---|---|---|---|
| seed classes (n) | 599 | 1894 | **2280** |
| seed full | 50,9MB | 181,1MB | **221,2MB** |
| seed corpi (body+slots) | 42,7MB (84%) | 162,3MB (89,6%) | **195,0MB (88,2%)** |
| seed doc/attributes | 1,5MB | 3,1MB | 3,7MB |
| seed firma (resto) | 6,7MB | 15,7MB | 22,4MB |
| unit ritenute / path | 697/688 | 1950/1925 | **4726/2316** |
| dup unit (leak) | 9 = 0,18MB | 25 = 6,3MB | **2410 = 240,2MB** (counted-v1) |

- **Ripartizione WP-59 "HIR seeds ≈2/3 del compile-side (~530MB)"
  FALSIFICATA dal contatore diretto**: i seed HIR misurano 181-221MB.
  Banda seed signature-only onesta: **~130-195MB** (gate fisico ≥80%).
  Il resto del compile-side (~600MB) è nei Module compilati/payload op
  ⇒ census v2 deep (WP-61) PRIMA di firmare quella banda.
- **Leak template sul FULL = 240,2MB counted (che sotto-conta)**; tre
  file = 203,7MB: `version.php` ×899 (63,4MB),
  `script-modules-packages.php` ×704 (75,0MB),
  `script-loader-packages.php` ×400 (65,3MB). Banda compile-cache
  (Fase 0.5) ≈ **−240MB+ sul full**, quasi nulla sul media (0,18MB).
- **Target oracle (denominatore)**: C PHP `--list-tests` = **237MB phys
  / 1,52s user** vs phpr ≈1,35-1,44GB ⇒ **~5,7-6×** sul solo bootstrap.
- Colonna abandoned del visitor mimalloc: **MORTA** (positive-control:
  1,25MiB leakati da thread morto invisibili a aband E main, anche con
  MIMALLOC_VISIT_ABANDONED=1) — i 104,3MB "non-visitato" di Ob.1 vanno
  etichettati così.

## Verdetti P1/P4 (parità)

- Revert leva B **PROMOSSO** sui criteri pre-registrati (parità per nome
  su tutti i gate + footprint ±0,5%); **phpr-wp60 = nuova baseline**.
- Sentinelle Stogov i-v pinnate con oracolo; 3 divergenze PRE-ESISTENTI
  catalogate (anon-class bind-once; class docComment/startLine con
  #[Attr] prima del doc; default prop const-expr valutati nella classe
  istanziata) — dettaglio in `sessions/WP_SESSION_60.md`.
