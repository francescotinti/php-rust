# WP_SESSION_51 — classify fuso (−4,1% full), Fase 1.4 landed (full-scan seed growth-gated), full 2,31×

> ⚡ **WP-51 (2026-07-25, `ffb8e3b`+`84a22f6`)** — **Ob.1a: il probe full-scan
> a scala FULL conferma la quota media al 100%: `created-registry-only`
> 353,74MB→0 (riga omessa dal dump), roots_total 578,55M→224,38M (−354,2M),
> created 217.811→118.102, freed 1.481.070 zval, live drop ≈−545MB
> (arr −478,9M) — costo di UN full-scan a full-graph = 2.489ms. Kill-switch
> NON scattato.** Ob.2 leva A `ffb8e3b`: classify a MAPPA FUSA
> (`GcInfo{handle,in_edges,live}` in una sola FxHashMap: 3→2 hash op/arco,
> external-check gratis in iterazione, pre-reserve della taglia dell'ultimo
> walk sui call ≥1024 root) → **run38 793,7s vs old stesso-notte 827,9s =
> −34,2s (−4,1%), ~24% del classify rimosso**. Ob.1b leva B `84a22f6`
> (Fase 1.4): **full-scan seed growth-gated** (`GC_FULLSCAN_GROWTH=50k` su
> `created.len()`, check O(1) nel wrapper cold `collect_cycles`) → **run39
> 782,7s: B è GRATIS sulla full (−11s vs run38), totale notte
> old 827,9→782,7 = −45,2s (−5,5%), 2,44×→2,31×**. Tutte e 3 le full
> BYTE-ID a run33 (88 nomi); corpus 1421 IDENTICO ×2 (tree A e tree A+B);
> cargo 1639/0.

## Ob.1a — probe full-scan a scala FULL (census, binario phpr-memgc50)

- Harness: `wp51-harness/run-full-census51.sh` = run-full-census50 +
  `PHPR_GC_EOR_FULL_COLLECT=1`, output separato in `wp51-harness/fullcensus-out`.
- **Master (pid 42008)**: `eor_full_collect created_before=217811
  created_after=118102 freed=1481070 ms=2489`. Post-collect:
  `created-registry-only` OMESSA (=0; WP-50 senza probe: 353.743.905 byte);
  roots_total 224.380.610 vs 578.548.961 (Δ −354,17M ≈ 100% del canale);
  exit live: arr 606,1→127,2M, obj 230,1→187,5M, str 163,5→139,3M.
- Worker: 71 probe, created ~1-2k, freed ~0 — i processi corti non hanno
  canale (il gating a crescita li esclude gratis).
- Il residuo created 118k è raggiunto da ALTRI canali del walk (vivo altrove:
  static-props 195,0M, reflect-cache 16,6M) — non è pinning del registry.

## Ob.2 — leva A: classify a mappa fusa (`ffb8e3b`)

- WP-47 shape: 3 lookup hash per arco (handles+in_edges in pass 1, is_live in
  pass 2) e tabelle che ripartivano VUOTE a ogni call (rehash a cascata su
  walk da milioni di nodi). Fusione in un solo `GcInfo` per nodo + pre-reserve
  (`gc_classify_last_nodes: Cell<usize>`, solo call ≥1024 root, i round-2 non
  clobberano la stima). Zero footprint standing (mappa call-local).
- Stessa sequenza di scoperta ⇒ stessi whites/roots/freed (conservazione);
  cold-path ⇒ nessun vincolo hot-arm WP-44.
- **Giudice (stessa notte)**: run38 793,7s (13:13.68) vs run38-old (phpr-wp50)
  827,9s (13:47.92) = **−4,1%**; old di notte ≈ old di ieri (828,9/832,7) ⇒
  macchina stabile. Fail-set BYTE-ID a run33.

## Ob.1b — leva B: full-scan seed growth-gated (`84a22f6`, Fase 1.4)

- Fatto architetturale (WP-50): `collect_cycles` si radica SOLO dai buffer ⇒
  la garbage ciclica quiescente è invisibile per sempre. Zend non ha il buco
  (`gc_possible_root` a ogni RC-dec). Leva: quando `created.len()` è cresciuto
  di ≥50k dall'ultimo seed, il collect successivo seeda TUTTO il registry
  (stessa forma del probe); mark aggiornato POST-collect. Check O(1) nel
  wrapper cold; ~4-5 seed per full run, 0 per i worker, MAI per-test (il male
  WP-49).
- Predizione registrata PRIMA dei giudici: full freed 14,22M→~15,6-15,7M;
  created-registry-only fine-run ~0-pochi MB; media +0,7-0,9s di scan
  (guardia Fase 1 ≤+0,5% a rischio → tenere e verbalizzare).
- **Giudice**: run39 (A+B) 782,7s (13:02.67) — **B gratis sulla full, anzi
  −11s vs run38** (created più piccola + località); BYTE-ID a run33.
- Mechanism-check census (conteggi): <DA COMPILARE: census51b freed totale,
  n. seed, created-registry-only fine-run>.

## Guardia A/B media (ab51: old=phpr-wp50, new=A+B, 6 round + 2 oracle)

- **CPU: FLAT** — old 59,162s vs new 59,170s = +0,01% (2/6 round new<old;
  oracle 20,79-20,85). **Guardia Fase 1 ≤+0,5% RISPETTATA**: la predizione
  "+0,7-0,9s di scan sul media" NON si è materializzata (il seed è raro col
  gating a crescita, e il classify fuso lo rende più economico del probe
  874ms census). Rapporto: 59,17/20,82 = **2,84×** — miglior media dai tempi
  di WP-35.
- **Footprint peak fisico: −0,32%** — old 1,7265G vs new 1,7209G; rapporto
  1,7209/0,3964 = **4,34×** (nuovo minimo, sotto il 4,35× WP-49). Risposta
  alla domanda PEAK-value di WP-50: il de-pin del canale created monetizza
  POCO sul picco (mid-run) — il grosso dei 353,7MB full / 114,7MB media è
  ritenzione di fine-run, che il peak fisico non vede (coerente con WP-48).

## Parità e gate (classe GC, protocollo WP-50)

- corpus **1421 IDENTICO per nome col conteggio** ×2 (tree A `ffb8e3b`,
  tree A+B `ec552cf`)
- cargo **1639/0** (tree A+B)
- 3 full run stessa notte (run38, run38-old, run39): fail-set **BYTE-ID a
  run33** (88 nomi) su tutte; 30.472 test 0E/2F/86W/73S
- Binari stashed sul path canonico esterno: `phpr-wp51a` (A), `phpr-wp51b`
  (A+B); old A/B = `phpr-wp50`

## ⭐ Lezioni

- ⭐⭐ **Il daemonizer perl con `chdir "/"` produce FINTI fail phpt**: bug60771
  scrive `./test.php` nella CWD → con CWD=/ (APFS system, read-only) il
  primo `file_put_contents` muore ("Read-only file system") — 2/2 nel corpus,
  5/5 pass in solo (dove il `cd` c'era). Cura: la CWD è PARTE DEL CONTRATTO
  della suite → `cd` root php-8.5.7 DENTRO lo script di gate. Corollario:
  `run_isolated` è SEQUENZIALE — "fallisce nel corpus ma passa solo" ⇒
  guardare l'AMBIENTE del launcher, non le collisioni tra test.
- ⭐⭐ **Le tabelle di un walk grande si fondono e si pre-riservano**: 3
  mappe→1 (`GcInfo`) + reserve dalla taglia dell'ultimo walk = −4,1% di full
  senza toccare l'algoritmo (stessi conteggi alla cifra). Il rehash a cascata
  da tabella vuota su milioni di nodi era ~metà del guadagno.
- ⭐⭐ **La disciplina di confine giusta è growth-gated, non per-test**: seed
  full-scan a +50k di crescita del registry = ~4-5 collect extra su una full
  intera, costo NEGATIVO sulla full (de-pin ⇒ created più piccola), zero per
  i worker. La predizione-misurata del probe (2.489ms una tantum) aveva già
  dimensionato il costo.
- ⭐ **Estrazione fail-set full**: `run3x-fails.txt` = righe `^\d+\) ` dal
  full-*.txt ORDINATE (88 = F+W, no skip); il junit non-P dà 161 (88+73 S).
- ⭐ **Filtro del `.rss` in append**: mai `awk '$1>=ora'` da solo (le ore di
  giorni precedenti passano il confronto stringa) — prima delimitare il
  blocco della notte (ultimo time-reset), poi la finestra oraria del run.
- ⭐ Convenzione REPORT_GAP corretta dall'utente: per-sessione + GAP_TREND
  cumulativo (la vecchia forma "copia+riga" era una trascrizione sbagliata
  auto-propagata via handoff — quando una convenzione single-source pare
  strana, RI-VERIFICARLA con l'utente).

## Prossimo (WP-52)

1. <DA COMPILARE dopo ab51/census: residuo classify post-leve (era 142s,
   −34s dalla fusione) — prossima leva candidata = in-node marks (elimina
   l'hashing residuo; costa +8B/nodo sui container: quotare col census)
   oppure Fase 1.3 cold-box Object.>
2. reflect-cache: owner della cardinalità (memo keyed su ClassId dei mock).
3. Laravel resta posticipata a valle della roadmap.
