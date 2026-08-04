# MEASURE94_RESULTS — la coppia full stessa-sera di S-94.0

**Che cos'è**: la PRIMA misura full/media dopo otto sessioni. Il Concilio
WP-95 l'ha eletta a prima voce dell'oggetto perché la roadmap footprint è
governata dalla regola WP-48 (predizione-misurata) e mancava il «prima»
fresco: senza una baseline della stessa sera, nessuna cifra della leva
avrebbe un giudice. Raw macchina: `wp94-harness/pair94.out` (+ i quattro
`pair-out/*.time` grezzi).

**Grado**: VERDICT per i rapporti (coppia stessa-sera, stesso metodo di
`gaps/GAP_TREND.md` §Metodo, ordine oracle-prima, DB e uploads resettati
prima di ogni run). R=1 per gamba, che è il metodo ricorrente e non un
probe: le due gambe si confrontano fra loro, non con una citazione.

**Identità**: phpr `d5ce86e3342f3926` — il PIN BASELINE, **invariato**
(mai ricompilato in S-94.0); oracle PHP 8.5.7; head `93721a5870ea`.

## Le cifre (i rapporti storici NON sono citati qui: vivono in `gaps/GAP_TREND.md`)

**media group** — 762 test, 1912 assertions, 52 skipped, identici sui due lati:

- user CPU: oracle 21,03 s, phpr 55,50 s — rapporto **2,639** (pair94-ratios.out)
- peak footprint: oracle 346325904 B, phpr 1170785648 B — rapporto **3,381** (pair94-ratios.out)

**full suite** — 30472 test, 4558029 assertions, 86 warnings, 73 skipped,
identici sui due lati:

- master CPU oracle = **447,84** s (pair94-ratios.out, user+sys)
- master CPU phpr = **838,59** s (pair94-ratios.out, user+sys)
- rapporto master CPU = **1,873** (pair94-ratios.out)
- peak footprint: oracle 745637040 B, phpr 1993459800 B — rapporto **2,673** (pair94-ratios.out)
- peak footprint phpr = 1993459800 B = **1901,11** MiB (pair94-ratios.out)

## ⚠️ SANATORIA (Concilio WP-96, Bak e Hoare in convergenza indipendente)

**Le letture comparative della prima stesura sono RITIRATE.** Dicevano «il
full CPU è MIGLIORATO nettamente», «il peak è SCESO», «il media footprint è
PEGGIORATO». Erano **artefatti del denominatore**, non fatti su phpr:

- la ricetta storica di `GAP_TREND` §Metodo divide il master-CPU per un
  oracle **congelato a 5:39 = 339 s**. Io ho diviso per l'oracle misurato
  stasera (447,84 s). Stesso numeratore, due denominatori: **838,59/339 =
  2,474** con la ricetta della tabella, **838,59/447,84 = 1,873** con il
  denominatore vivo. Il «rapporto più basso mai registrato» confrontava
  regimi diversi;
- sul media group, il rapporto peggiora perché **l'oracle è sceso**, non
  perché phpr sia cresciuto (Bak: phpr in banda, oracle −11,9%). Chiamarlo
  «regresso di phpr» era una lettura sbagliata di una frazione.

**Che cosa resta valido**: le otto cifre misurate (sono misure, e i raw non
si toccano) e i due rapporti **same-evening**, che confrontano le due gambe
FRA LORO — l'unica comparazione omogenea che questa coppia autorizza.
**Che cosa NON è più affermato**: qualunque giudizio di miglioramento o
regresso rispetto alle bande storiche. Per averlo serve un denominatore
omogeneo, che è lavoro di S-95.0 (A-TH-76/A-BB-67).

**Letture superstiti, per NOME:**

1. **I rapporti same-evening sono quelli in tabella** e valgono come
   fotografia di stasera: nessun confronto con altre sere è autorizzato
   finché il denominatore non è reso omogeneo.
2. **I conteggi sono IDENTICI sui due lati**: la coppia confronta due run
   che hanno fatto lo stesso lavoro, non due lavori diversi. Questo regge
   indipendentemente dalla sanatoria.

## Le divergenze, per NOME (mai per conteggio)

Entrambe le full escono `rc=1`: **la suite WordPress fallisce anche
sull'oracle**, quindi il rc del runner non è il giudice della coppia.

- `Tests_Admin_wpPostsListTable::test_search_hierarchical_pages_first_page`
  — fallisce su ENTRAMBI i lati: non è una divergenza phpr.
- `Tests_Functions::test_wp_is_stream` (data set #2, `ftp://example.com`)
  — **unica divergenza phpr**. Causa nominata a macchina:
  `stream_get_wrappers()` di phpr elenca `data,file,http,https,php` mentre
  l'oracle elenca anche `compress.bzip2, compress.zlib, ftp, ftps, glob,
  phar, zip`. `wp_is_stream` interroga quella LISTA, quindi la reticenza
  del correct-or-absent (non dichiarare ciò che non si implementa) diventa
  qui una divergenza osservabile.

## Che cosa questa misura AUTORIZZA e che cosa no

- **Autorizza** la leva delle arene per-file del preludio (S-95.0) ad avere
  un giudice: il «prima» esiste, è di stasera, ed è sullo stesso binario
  pinnato che la leva modificherà.
- **NON autorizza** alcuna attribuzione del regresso sul footprint media né
  alcun pin dello slope per-worker: nessuna delle due è stata misurata qui.
