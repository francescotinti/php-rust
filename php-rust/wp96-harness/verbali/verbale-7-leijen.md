# Verbale 7 — Daan Leijen (allocatore, footprint fisico, semantica dei contatori)
Concilio WP-96 · mandato: REFUTARE

## VERDETTO

**CONTRARIO al grado dichiarato per il footprint; FAVOREVOLE CON EMENDAMENTI
al resto.** Ho letto l'albero COSTRUITO, non la documentazione: il binario di
parità linka **mimalloc v3.0.2**, non v2 (`libmimalloc-sys 0.1.49`,
`build.rs:8-12` — `v3` è il DEFAULT, la feature `v2` non è accesa da
`mimalloc = "0.1"`). Nessun banner di questo progetto lo dice.

Due fatti sopravvivono al mio morso: (a) `MIMALLOC_PURGE_DELAY=0` **funziona
davvero** — `arena.c:2060-2067`: `delay==0` ⇒ `mi_arena_purge()` diretta;
`os.c:650-655` con `purge_decommits=1` ⇒ decommit; `prim/unix/prim.c:495-498`
⇒ su macOS `MADV_FREE_REUSABLE`, che fa contabilità immediata. Il controllo è
in grado. (b) Il picco è un **high-water**: nessun purge può abbassare un
picco già raggiunto. Da questi due fatti discende tutto il resto.

## Emendamenti

- **A-DL-67 — identità dell'allocatore IN BANDA.** Ogni identity block di
  misura registra `mi_version()` e il valore **letto dal processo**
  (`mi_option_get(mi_option_purge_delay)`), mai la riga env dello script.
  `pair94.identity` pinna phpr/oracle/rustc/head e **non** l'allocatore né
  l'opzione che dichiara di governare. *Un controllo dichiarato non è un
  controllo in vigore* — è la lezione del `#if` di WP-95 applicata a una env.
- **A-DL-68 — `max_rss` accanto a `peak_footprint`, sempre, col gap.** Dal
  raw già committato: media rss 2,555× contro pf 3,381×; full rss 1,986×
  contro pf 2,673×. E il segno: phpr ha pf **sopra** il proprio rss di
  142.820.720 B (media) e 369.592.408 B (full), l'oracle ha pf **sotto** il
  proprio rss di 55.966.832 / 72.153.936. Il «regresso» vive nel **gap**
  (compresso/charged-non-residente), non nel residente.
- **A-DL-69 — grado del picco = SCREEN a R=1.** Il picco è una statistica di
  MASSIMO; repair90 impose la MEDIANA per `b_peak` dopo due outlier.
  `pair94.out` firma VERDICT a R=1 sui due rapporti di footprint: incoerente
  con la disciplina di stima di questo stesso progetto.
- **A-DL-70 — ordine INCROCIATO per il footprint.** «oracle-prima» è un
  controllo per la CPU e un **confondente** per il footprint: la seconda gamba
  gira sempre su una macchina il cui stato di compressore è stato plasmato
  dalla prima. Alternare l'ordine fra le ripetizioni.
- **A-DL-71 — la leva si riceve SOLO sul picco CLI.** 21 MB su 44.630.520 =
  47%; sul picco media = 1,8%; sul full = 1,06%. Nessuna cifra media/full può
  ricevere né falsificare la leva.
- **A-DL-72 — α va RI-DERIVATO.** `team-leva.md:168-171` giustifica α≈1 con
  «mimalloc … NON decommitta, le pagine restano committed e riusabili». Sotto
  `PURGE_DELAY=0` è **falso** (v. sopra): decommitta subito. La banda può
  sopravvivere, l'argomento no. Corollario da firmare: 15 arene per-file =
  decommit→recommit→re-fault ripetuti; firmare anche una predizione di
  `page reclaims` dallo stesso `time -l`, o la leva paga in CPU ciò che
  incassa in footprint.
- **A-DL-73 — dente sul punto di DROP.** `N = T_tot − T_max` presuppone che
  l'arena del file *i* muoia prima del parse di *i+1*. Se l'AST del preludio è
  ritenuto attraverso i file, **N = 0** e resta solo la coda 13.738.592 B, che
  vale **zero** footprint: pagine mai faultate non sono mai addebitate. Il
  punto di drop dev'essere un dente, non un presupposto.

## Kill-switch

- **KS-DL-96-1**: ricevuta della leva che cita media/full come evidenza ⇒
  **NULLA** (l'effetto atteso è sotto ogni spread mai misurato su quei picchi).
- **KS-DL-96-2**: qualunque cifra di footprint da un albero il cui `Cargo.lock`
  o `build.rs` del `-sys` differisce dal lock di parità (MI_STAT compreso) ⇒
  **NULLA**. Estensione di KB-78-5 ai knob di build dell'allocatore.
- **KS-DL-96-3**: verdetto di picco a R=1 senza `max_rss` e senza il gap
  pf−rss nello stesso raw ⇒ declassato a **SCREEN** d'ufficio.
- **KS-DL-96-4**: probe slope v2 i cui arm non siano **entrambi** MI_STAT=1, o
  privo di **controllo positivo** (un'allocazione nota di X byte deve comparire
  nel contatore) ⇒ **VOID**; e le sue cifre non si sottraggono mai a pair94.

## Refutazioni

**Sì, una capitale.** La giustificazione di α (`team-leva.md` §3.3) poggia su
una proprietà dell'allocatore che l'albero costruito contraddice **nella
configurazione stessa in cui la predizione sarà giudicata**. Seconda, di
grado: il «regresso media 3,381×» non è stabilito come fatto di programma —
R=1 su una statistica di massimo, e su `max_rss` lo stesso raw dice 2,555×.
Terza, di coerenza: MI_STAT non è esposto fra le feature di
`libmimalloc-sys 0.1.49` (arena, debug, debug_in_debug, extended,
local_dynamic_tls, no_thp, override, secure, v2, win_direct_tls); accenderlo
richiede di patchare `build.rs` **più** la feature `extended` per leggere le
API ⇒ **per costruzione** non è il binario di parità. Il probe è coerente solo
come build di SOLI CONTATORI, mai come sorgente di livelli.

Nota non refutativa ma dovuta: `PURGE_DELAY=0` **non è la configurazione
spedita** (default `purge_delay = 1000` ms, `options.c:140`). Le cifre
descrivono una configurazione che nessun utente esegue, salvo che il binario
imposti l'opzione da codice.
