# WP_SESSION_60 — Revert leva B (nuova baseline phpr-wp60) + riparazioni di metodo + contatori ex-ante: il seed HIR misura 181MB (non ~530) — la ripartizione 2/3-1/3 di WP-59 è FALSIFICATA dal contatore diretto

> ⚡ **WP-60 (2026-07-26, `915ea27`+`f0f75d4`+seg. )** — Programma del concilio
> 2° giro eseguito nell'ordine vincolante P1→P4→P2→P3 (P5 NON aperto:
> census v2 deep mancante ⇒ il perimetro non si comprime, slitta a WP-61).

## P1 — ⚖️ Revert leva B, DA SOLO (`915ea27`) → PROMOSSO

Inverso esatto di `14b8c4c` + rimozione feature `pool-off` (scan4 resta):
mod pool (shelves TLS), grow_entries/grow_packed, take/put nel KeyIndex,
Drop di PhpArray tornato al derived drop (free mimalloc diretto, impl
Drop solo sotto mem-census). Diff vs pre-leva-B `b0f7396` = SOLO i 2
hunk legittimi dei commit successivi (scan4 const + cfg census size-pin).

Verdetto sui criteri PRE-REGISTRATI (design60, Leijen §1b-iv / Gregg V3):
- **Parità per nome ✓**: corpus **1421 IDENTICO** · refl **290 IDENTICO**
  · cargo **1645/0** · ORM 3484 **3E/13F IDENTICO** · hk 1665 **0E/0F** ·
  sentinelle 5 assi (order/tomb/numkey/dtor/yieldfrom) **BYTE-ID** ·
  coppia full stessa-sera run48: fail-set **88 nomi BYTE-ID a run33 su
  ENTRAMBE** (30472 test / 4.557.508 assertion identiche).
- **Footprint ✓**: new 3,900GB vs old 3,907GB = **−0,19%** (banda ±0,5%).
- CPU (solo informativo): user 781,39s vs 788,72s = **−0,93%** — coerente
  con pool-off (−0,56%) dentro lo spread serale ±0,6%.
- **Baseline NUOVA: phpr-wp60** (sha256 `46b517fc…`), stash canonico in
  `phpr-old-target/release/` sul drive esterno.

## P4 — Sentinelle Stogov i-v pinnate (verdetto ORACOLO prima; probe
## `wp60-harness/probe60-p4*.php`, output `-{oracle,new,wp58}.txt`)

Tutte le divergenze sono PRE-ESISTENTI (wp58 == revert byte-id, 5/5):
- **(i) static-closure re-include: phpr == oracolo BYTE-ID** — ogni
  re-include non-`_once` conia una cella static NUOVA. VINCOLO Fase 0.5.
- **(ii) classe anonima re-include: DIVERGE** — oracolo 3 CE distinte;
  phpr bind-once (stessa classe, static condiviso). La cache non
  peggiora; catalogo divergenze.
- **(iii) fatal Cannot redeclare: PRESERVATO** (rc=255 entrambi); phpr
  omette "Stack trace: #0 {main}" (famiglia formato fatal, nota).
- **(iv) reflection: metodo doc/line/attrs/params ==**; class-level
  getDocComment=false + getStartLine 2-vs-4 quando `#[Attr]` precede il
  doc (pre-esistente; sentinella per il seed-thin).
- **(v) eval extends multi-livello + trait: catena TUTTA ==** tranne i
  default di proprietà const-expr ereditati: `self::K`/`parent::K`
  valutati nella classe ISTANZIATA (slot 21/mslot 40) invece della
  DICHIARANTE (oracolo 11/30). Engine-bug da backlog.

## P2 — riparazioni di metodo (census-only, `f0f75d4`)

- **(a) finestra→test IN-PROCESS**: `tag=ctx` su ogni finestra (top 3
  frame PHP via parking del VM in run_loop + `mono_ms` su mi_proc/ctx).
  VALIDATO sul media: win1 = include admin.php, win2-9 = discovery
  (`addTestFile@TestSuite.php:399`, `getMethods@phpunit`), win10 =
  `wp_get_image_editor…` ⇒ la salita È costruzione-suite, ora con
  l'indirizzo per finestra. Mai più offset stdout (Gregg R1/V4 chiusi).
- **(b) positive-control abandoned: colonna MORTA** — thread che leaka
  1,25MiB e muore: aband_b=0 E main_delta≈0 anche con
  MIMALLOC_VISIT_ABANDONED=1 (verdetto a tre esiti SEEN/ADOPTED_MAIN/
  DEAD → DEAD). I 104,3MB "mimalloc non-visitato" di Ob.1 possono
  contenere heap abbandonati vivi: etichetta obbligatoria nei report.
- **(d) igiene DB**: reset DENTRO ogni script di probe che bootstrappa
  phpunit (probe60-listtests/census60-media/census60-full).
- **(e) SHOW_STATS**: i nuovi harness usano solo FFI programmatica
  (mi_process_info/visit) + $PHPR_MI_STATS; env mimalloc mai su run con
  fail-set. (c) recon str: assorbito dal census v2 deep (WP-61).

## P3 — contatori ex-ante (il kill-switch ha COLPITO)

Metro nuovo: **counting global allocator** (census build) + clone-delta
= deep-size DIRETTO senza walker AST (sotto-conta solo l'Rc-shared,
etichettato). Binario census **phpr-memgc60** (`d1c8dc81…`).

- **(b) seed split su `--list-tests`** (bootstrap+discovery reale,
  n=1894 classi): **full 181,1MB = corpi 162,3MB (89,6%) + doc/attr
  3,1MB + firma 15,7MB**; traits 0,24MB (INTATTI per veto); main_fns
  0,94MB. ⇒ **i corpi dominano (>60%, criterio Stogov R1) MA il seed
  totale è 181MB, NON i ~530MB estrapolati in WP-59** (probe C — il
  punto debole n.1 della review Gregg). **Banda seed signature-only
  ridimensionata: ~130-160MB fisici** (gate Leijen ≥80% dei droppati).
  Il giacimento grosso (~600MB) sta nei Module compilati/payload op ⇒
  census v2 deep (WP-61) prima di firmare qualunque altra banda.
- **(c) unit-per-path** (registro census ai LEAK SITE — `Vm::modules` è
  lazy e non vede le unit): listtests = 1950 unit / 1925 path /
  **dup 25 = 6,3MB** (blocks-json.php n=5 = 5,6MB). Sul bootstrap il
  leak è PICCOLO; il bordo vero è il full (mappa P3f).
- **(g) target oracle**: C PHP `--list-tests` = **237MB phys / 1,52s
  user** vs phpr ≈1,35-1,44GB ⇒ ~**5,7-6×** sul solo bootstrap — il
  tetto "dall'alto" della dieta compile-side è quantificato.
- **(f) mappa census del FULL**: run detached con finestre+ctx+seed
  split (risultati in `census-out/memcensus60-full.txt`; v. REPORT_GAP).
- Media census: seed 599 classi / 50,9MB (corpi 84%); peak census
  1,625GB (overhead census ~2-5% sul metro giudice, documentato).

## ⭐ Lezioni

- ⭐⭐ **Il contatore diretto uccide l'estrapolazione anche quando
  l'estrapolazione "matchava"**: il probe C dava 600-800MB "MATCH con
  gli 818 osservati", ma la componente seed-HIR vera è 181MB — la
  categoria era giusta, la ripartizione no. Mai dimensionare DUE leve su
  una spaccatura non misurata (Gregg aveva marcato esattamente questo).
- ⭐⭐ **Clone-delta del counting allocator = deep-size senza walker**:
  split {firma/corpi/doc} per 1894 classi in ~40 righe di codice, al
  costo di un `#[global_allocator]` census-only. Sotto-conta l'Rc-shared
  per costruzione: etichettare, mai stimare.
- ⭐⭐ **La visita abandoned è CIECA anche col suo env**: positive-control
  a tre esiti prima di fidarsi di QUALUNQUE colonna del visitor.
- ⭐ Un `impl Drop` diagnostico su Vm rompe la destrutturazione
  dell'outcome (E0509): il parking si sgancia esplicitamente prima del
  move-out, mai col Drop.
- ⭐ La unit-cache WP-20 esiste già completa in `run_include` (chiave
  path+mtime+size + fp strutturale): il leak template è un problema di
  CHIAVE (unit_chain_fp folda ogni load ⇒ miss garantito sul
  re-include), non di infrastruttura. Fase 0.5 = rilassare la chiave.
- ⭐ `vm.modules` è popolato LAZY: qualunque census per-unit deve
  agganciare i leak site, non la lista.

## Prossimo (WP-61) — proposta

1. **Census v2 DEEP dei Module** (dedup per indirizzo, payload op
   per-kind, Const profondi, literal cross-unit, slack): è lì il ~600MB
   non attribuito — nessuna leva si scrive prima.
2. **Fase 0.5 compile-cache** (chiave contenuto + verifiche strutturali
   già esistenti; vincoli P4-i/iii pinnati; predizione ex-ante dalla
   mappa full P3f).
3. **Seed signature-only** con banda onesta ~130-160MB (gate fisico
   Leijen R3 pre-registrato; seed_traits INTATTO).
