# WP_SESSION_59 — Attribuzione fuori-canale (programma del concilio esteso): la mappa è FATTA — frag mimalloc 2% (ipotesi concilio falsificata), il ~1,1GB è compile-side VIVO (HIR seeds + payload op); leak template-include confermato; obj de-fantasmato (56,1→48,7MB)

> ⚡ **WP-59 (2026-07-26, `c52c394`+`1e06464`+`e22f373`+docs)** — Sessione di MISURA (binari di
> parità INTATTI = phpr-wp58; codice solo census-only + 2 feature
> diagnostiche). Programma del concilio eseguito nell'ordine vincolante.

## Ob.0 — MIMALLOC_SHOW_STATS + vmmap (zero codice)

- mimalloc **v3.3.2**; media: **committed peak 1,4GiB (1,50GB) = 93% del
  phys peak 1,61GB**; full: committed 3,7GiB (3,97GB) ≈ phys 3,89GB ⇒ su
  ENTRAMBI i workload il fuori-canale vive DENTRO l'allocatore. Fuori:
  ~100-125MB (binario+FFI: zone MALLOC_* di sistema ~90-120MB dirty,
  DefaultMallocZone 58% frag su 37,5M allocati).
- maxrss < phys su entrambi (media 1,55 vs 1,61G; full 1,51 vs 3,89G!) ⇒
  il metro giudice ("peak memory footprint") è onesto; sul full 2,3G era
  nel compressor al picco.
- committed CURRENT a fine processo = 1,4GiB con 9,1GiB purgati ⇒ pagine
  non-vuote (poi spiegato da Ob.1: standing vivo + frag exit 171MB).
- ⚠️ metodo: env mimalloc propaga ai phpr FIGLI → stats su stderr dei
  figli ⇒ i test che catturano stderr ERRANO (15 media / 71 full):
  le run Ob.0 NON sono confrontabili per fail-set. Il heap mimalloc
  appare come **IOAccelerator** nel vmmap di macOS 26 (os_tag 100), NON
  VM_ALLOCATE; `footprint(1)` funziona senza privilegi; v3 release non
  stampa la tabella per-bin (occupancy solo via `mi_heap_visit_blocks`).

## Ob.1 — Fase 0-bis: finestre phys nel census (`c52c394`)

Strumentazione (census-only): watermark RI-CHIAVATO su `task_info`
phys_footprint (+128MB, throttle 1/16384 eventi); a ogni finestra:
`mi_process_info` (tag=mi_proc) + occupancy per size-class via
`mi_heap_visit_blocks`/`visit_abandoned_blocks` (tag=mi_bin, main|aband)
+ snapshot canali + stats su $PHPR_MI_STATS + flag-file per il
supervisore esterno vmmap/footprint (`ob1-supervisor.sh`); exit = win 0.
Binario **phpr-memgc59** (00ca4d3d…). Reporter: `ob1-report.pl`.

**Gate pre-registrato SUPERATO alla cifra** (win10/picco, master):
phys 1436,2MB = Σused 1298,0 + frag 29,8 + (commit−Σcomm) 104,3 + 4,1.

| | valore | % fisico |
|---|---|---|
| canali valore (census live) | 361,5MB | 25% (proxy) |
| **non-censito VIVO** | **936,6MB** | **65%** |
| frammentazione mimalloc | **29,8MB** | **2%** |
| metadata/non-visitato + non-mimalloc | ~108MB | 8% |

⇒ **l'ipotesi dominante del concilio (ritenzione/frammentazione
mimalloc ≥400MB) è FALSIFICATA sul media**: la scommessa Stogov cade sul
ramo "standing non censito ⇒ la leva è la dieta di quelle strutture".
(la frag sale a 171MB solo all'EXIT, dopo i free di massa.)

**Attribuzione del 937MB (probe a catena, stessi env giudici)**:
- `--list-tests` (bootstrap+discovery, **zero test eseguiti**): phys
  1,35GB, Σused 1.076MB, census 258MB ⇒ **non-censito 818MB prima di
  ogni test** — le finestre della salita sono la COSTRUZIONE DELLA SUITE
  (load classi test + data provider), non l'esecuzione (che aggiunge
  ~100MB permanenti).
- probe C (40 file di sole classi vs 40 file di sole funzioni, stesso
  volume, netto baseline VM 7,48MB): funcs = 1,5× il counted del canale
  unit; classi = **3,8×** (il Δ = ritenzione HIR di classe) ⇒ sui 211MB
  counted ≈ 600-800MB non censiti — MATCH con gli 818 osservati.
- probe B: **LEAK template-include CONFERMATO** — 200 include dello
  STESSO file ⇒ `unit.cum_n=200`: ogni re-include non-`_once` ricompila
  e RITIENE una unit nuova (domanda WP-45 chiusa: Fase 0.5 SI APRE).

**Colpevoli con nome**: `seed_classes`/`seed_traits`/`main_hir` — l'HIR
ritenuto per il seeding di eval/include, con `MethodDecl.body: Vec<Stmt>`
= AST completo di ogni metodo di ogni classe caricata, PER SEMPRE
(≈2/3 del compile-side); payload `Rc` degli op + parti rc>1 saltate da
`module_census_bytes` (≈1/3). Il "canale unit" VERO ≈ **1,0GB** — il
channel ne conta 222MB = 1/5. Leva WP-60 candidata: **seed
signature-only** (il lowering di una sottoclasse usa forme/firme, non i
corpi) + **compile-cache keyed sul path** (Fase 0.5) — bande da quotare
con contatore dedicato PRIMA di scrivere le leve.

## Ob.2 — anomalia 46k obj unreached: ARTEFATTO, fixato (`c52c394`)

Audit dei call-site: `next_id` è condiviso con closure/generator/fcc/
rebind (6 siti non-Object) i cui id NON passano mai da `Object::drop` ⇒
l'alloc CH_OBJ al choke coniava un fantasma live per ogni closure/
generator (ipotesi 1 di Gregg, firma created==reached). Fix: wrapper
`next_obj_id` chiamato SOLO dai 6 siti di costruzione `Object`.
**Validazione**: walk_recon obj `reached_n=22.141 == live_n=22.141`
ESATTO (era 67.779 vs 22.141). **obj peak 56,1 → 48,7MB** (−7,4MB
fantasma), live EOR 6,5MB. Canali valore al picco restano ≈12%.

## Ob.3 — regressione full-only: attribuzione a due assi (stessa-sera)

Metro NUOVO: CPU esatta da `/usr/bin/time -l` (elimina il troncamento
del campionatore ps — lezione WP-58). Coppia + terzo asse back-to-back:
- **pool-off** (feature `pool-off`: `class()`→None, TLS eliso;
  phpr-pooloff59 93aea5a3…): user tree **786,44s** vs **new (phpr-wp58)
  790,83s** ⇒ **−0,56%** — il pool costa sul full (direzione Leijen),
  ma spiega solo la parte BASSA della banda +1..+2,5% ⇒ parziale.
- **scan4** (feature `scan4`: SCAN_MAX 8→4; phpr-scan4-59 b955afb1…):
  user tree **796,05s = +0,66% vs new** ⇒ **scan-mode ASSOLTO**:
  ridurlo PEGGIORA (lo scan 5-8 slot batte l'indice, coerente col
  design WP-58).
- Footprint: pool-off 3,90 ≈ new 3,89 ≈ scan4 3,89GB (ritenzione ≈0 ✓).
- Parità per nome: **new = 88 nomi BYTE-ID a run33** (ennesima
  validazione della release); **scan4 = 88 IDENTICO**; pool-off = 88
  IDENTICI + 1 flake DB ambientale (`Duplicate entry` in wp_install su
  un test filesystem — nessun legame col pool; probabile stato wptests
  sporcato dal probe `--list-tests` senza reset DB: regola nuova,
  DB reset anche nei probe).
- **Lettura d'insieme**: lo spread della serata a metro ESATTO è ±0,6%
  (786,4 / 790,8 / 796,0) ⇒ la banda WP-58 "+1..+2,5%" (metro troncato)
  era in buona parte rumore di misura; l'unico costo identificato è il
  pool ≈0,6%, residuo DENTRO il rumore.
- ⚖️ **Verbale per l'utente (revert leva B)**: il pool costa ~0,6% sul
  full, footprint ±0, e il −17MB di WP-58 veniva da A+C — il beneficio
  proprio di B è nullo. La roadmap Fase 3 consente il revert della SOLA
  versione regressiva su decisione utente al verbale. Raccomandazione:
  **revertare B** (A e C restano); in alternativa B resta dietro
  feature-off, dato che il codice è già gated da `pool-off`.

## ⭐ Lezioni

- ⭐⭐ **L'ipotesi condivisa 3/3 del concilio è morta in UNA finestra di
  misura**: la riconciliazione per-bin (used vs committed) uccide in 20
  minuti ciò che tre esperti stimavano plausibile — mai spendere una
  leva sull'ipotesi dominante prima della tabella.
- ⭐⭐ **Il footprint del run PHPUnit è per ~90% costruzione della suite**
  (bootstrap+discovery+provider), non esecuzione: `--list-tests` è
  l'oracle più economico del compile-side standing (1 comando).
- ⭐⭐ Il moltiplicatore compile-side si misura col probe differenziale
  classi-vs-funzioni a parità di volume (3,8× vs 1,5×): isola l'HIR
  senza scrivere un deep-size dell'AST.
- ⭐ vmmap macOS 26: heap mimalloc = regioni IOAccelerator (os_tag 100);
  malloc_history/leaks restano ciechi; `footprint(1)` è unprivileged.
- ⭐ Env mimalloc propaga ai processi FIGLI: mai SHOW_STATS/VERBOSE su
  run di cui serve il fail-set.
- ⭐ `/usr/bin/time -l` sul full = CPU esatta a fine processo: il metro
  che chiude per sempre il ±1,3% del campionatore (adottarlo nei
  prossimi harness full).

## Prossimo (WP-60) — proposta

1. **Quota ex-ante delle due leve compile-side** (contatore dedicato nel
   census: bytes HIR per classe firma-vs-corpo; conteggio unit duplicate
   per path): poi **Fase 0.5 compile-cache** (leak) e **seed
   signature-only** (banda potenziale: centinaia di MB).
2. Esito ⚖️ revert leva B (decisione utente sul verbale Ob.3 —
   raccomandazione: revert; A e C restano).
3. Quote Stogov rimaste (duplicati str, literal-array, peak oracle
   per-test) — declassate dalla mappa: il giacimento è compile-side.
   (Il "residuo full-only" è CHIUSO: pool ~0,6% + rumore campionatore.)
