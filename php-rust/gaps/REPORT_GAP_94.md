# REPORT_GAP_94 — il gap perf oracle↔phpr misurato in S-94.0

*(solo questa sessione; il cumulativo vive in `GAP_TREND.md`)*

**Perché conta**: il contatore full/media era fermo a WP-85, otto sessioni.
Il Concilio WP-95 ha eletto questa coppia a prima voce dell'oggetto perché
la roadmap footprint non ha giudice senza un «prima» fresco (legge WP-48).
Raw: `wp94-harness/pair94.out`, rapporti macchina in `pair94-ratios.out`,
lettura in `wp94-harness/MEASURE94_RESULTS.md`.

**Condizioni**: coppia stessa-sera, ordine oracle-prima, `/usr/bin/time -l`,
`MIMALLOC_PURGE_DELAY=0`, DB `wptests` ricreato e uploads azzerati prima di
OGNI run (guardia Gregg R7, backup verificato). phpr `d5ce86e3342f3926` —
il pin baseline, mai ricompilato in questa sessione.

## Il gap, per metrica

| metrica | rapporto phpr/oracle | contro il riferimento WP-85 |
|---|---|---|
| media group, user CPU | 2,639× | 2,58× → **leggermente peggio** |
| media group, peak footprint | 3,381× | ~3,0-3,1 → **REGRESSO** |
| full suite, master CPU | 1,873× | 2,06-2,11× → **MEGLIO, nettamente** |
| full suite, peak footprint | 2,673× (1901,11 MiB) | ~1,98-2,03 GB → **MEGLIO** |

**Il full è migliorato su entrambi gli assi; il media group è peggiorato sul
footprint.** Le due cose non si compensano e non vanno mediate: sono
workload diversi. Il regresso del media footprint è la voce che questa
sessione lascia aperta, ed è nominata senza attribuzione — attribuirla
richiede un canale di misura, non una congettura, e il probe slope v2 di
S-94.0 è slittato.

## Fedeltà: i conteggi e i nomi

Conteggi **identici** sui due lati (media 762 test / 1912 assertions / 52
skipped; full 30472 test / 4558029 assertions / 86 warnings / 73 skipped):
la coppia confronta due run che hanno fatto lo stesso lavoro.

Entrambe le full escono `rc=1` — **la suite WordPress fallisce anche
sull'oracle**, quindi il rc del runner non è il giudice:

- `Tests_Admin_wpPostsListTable::test_search_hierarchical_pages_first_page`
  fallisce su ENTRAMBI: non è una divergenza phpr.
- `Tests_Functions::test_wp_is_stream` è l'**unica** divergenza phpr, ed è
  quella già catalogata: `stream_get_wrappers()` non elenca `ftp` (né ftps,
  glob, phar, zip, compress.*). Confermata, non nuova.

## Accettazione WordPress (criterio 5 del fronte)

`wp94-harness/battery61.sh`, modo nativo, stesso pin: cinque probe BYTE-ID
(front 57428 B, asset 13031 B, permalink 71093 B, login 6758 B, login POST
302) e dashboard NORM-ID (142385 B, differenza nei soli nonce). rc=0.
