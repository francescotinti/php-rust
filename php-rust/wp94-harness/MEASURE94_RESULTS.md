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

## Le cifre

| metrica | oracle | phpr | rapporto |
|---|---|---|---|
| media group, user CPU | 21.03 s | 55.50 s | **2,64×** [derivata] |
| media group, peak footprint | 346325904 B | 1170785648 B | **3,38×** [derivata] |
| full suite, master CPU (user+sys) | 447.84 s [derivata] | 838.59 s [derivata] | **1,87×** [derivata] |
| full suite, peak footprint | 745637040 B | 1993459800 B | **2,67×** [derivata] |

Riferimento in vigore fino a oggi (WP-85, otto sessioni fa): media CPU
2,58× · footprint media ~3,0-3,1 · full CPU 2,06-2,11× · peak FULL
~1,98-2,03 GB.

**Letture, per NOME:**

1. **Il full CPU è MIGLIORATO in modo netto**: 1,87× contro la banda
   2,06-2,11× del riferimento. È il rapporto più basso mai registrato sul
   full-suite (la tabella GAP_TREND parte da 2,9× a WP-27).
2. **Il peak footprint del full è SCESO**: 1993459800 B = 1,857 GB contro
   ~1,98-2,03 GB di riferimento.
3. **Il footprint del media group è PEGGIORATO**: 3,38× contro ~3,0-3,1.
   È l'unica metrica in regresso e va nominata come tale, non annegata
   nelle tre buone. Nessuna attribuzione è offerta qui: attribuirla
   richiederebbe un canale di misura, non una congettura, e questa
   sessione non l'ha eseguito.
4. **I conteggi sono IDENTICI sui due lati** — 762/1912/52 sul media,
   30472 test e 4558029 assertions sul full: la coppia confronta due run
   che hanno fatto lo stesso lavoro, non due lavori diversi.

## Le divergenze, per NOME (mai per conteggio)

Entrambe le full escono `rc=1`: **la suite WordPress fallisce anche
sull'oracle**, quindi il rc del runner non è il giudice della coppia.

- `Tests_Admin_wpPostsListTable::test_search_hierarchical_pages_first_page`
  — fallisce su ENTRAMBI i lati: non è una divergenza phpr.
- `Tests_Functions::test_wp_is_stream` (data set #2, `ftp://example.com`)
  — **unica divergenza phpr**. Causa nominata a macchina:
  `stream_get_wrappers()` di phpr elenca `data,file,http,https,php`
  mentre l'oracle elenca anche `compress.bzip2, compress.zlib, ftp, ftps,
  glob, phar, zip`. `wp_is_stream` interroga quella LISTA, quindi la
  reticenza del correct-or-absent (non dichiarare ciò che non si
  implementa) diventa qui una divergenza osservabile. Catalogata in
  `PHPR_DIVERGENCES_FROM_PHP.md`.

## Che cosa questa misura AUTORIZZA e che cosa no

- **Autorizza** la leva delle arene per-file del preludio (S-95.0) ad
  avere un giudice: il «prima» esiste, è di stasera, ed è sullo stesso
  binario pinnato che la leva modificherà.
- **NON autorizza** alcuna attribuzione del regresso sul footprint media
  né alcun pin dello slope per-worker: nessuna delle due è stata misurata
  qui.
