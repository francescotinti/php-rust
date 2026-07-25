# WP_SESSION_54 — attribuzione CPU-secondi del full (metodo WP-45 sulla CPU) + reflect re-key (decl,mname): −7,4% media / −2,7% full

> ⚡ **WP-54 (2026-07-25, `5915b7b`+`f927bf7`+`14141ab`)** — Sessione di
> MISURA con una leva sola, quella che la tabella apre. **Ob.1**: tabella
> CPU-secondi del full-master (run42 campionata: 6 finestre `sample` 30s
> stratificate col profilo cpu-rate del `.rss`, Σ=776,7s riconciliata per
> costruzione), tabella decisionale PRE-REGISTRATA prima dei numeri.
> **Ob.2**: quote ns/evento — malloc stringhe ~10-20ns/pair = 2,2s sul
> full (0,3%): fusione single-alloc FALSIFICATA come leva top; args-Vec
> 1,1s: Fase 2.3 pool MORTA. **Probe**: bcrypt phpr −14% vs oracle
> (canale crypt 91s = ONESTO); `.=` phpr 488ms vs oracle 2ms = **244×,
> O(n²) CONFERMATO** (concat2 copia LHS+RHS a ogni append; Box<[u8]>
> exact-size senza capacity) → nuovo mandato WP-55. **Census walk**:
> oggetti-foglia 37,5% dei nodi ma 6,6% degli slot → leaf-bit
> FALSIFICATA sotto soglia pre-registrata. **Census reflect**: inserts
> inherited 96,3% (452.621/469.990, hit-rate 11,7%) → **Ob.3 APRE: re-key
> di reflect_method_info_cache su (declaring class, mname)** dopo la
> resolve (descrittore = funzione pura della coppia, WP-53). Mechanism-
> check: hits 62.206→514.409, misses 469.990→17.787 (hit-rate 96,7%),
> inherited-inserts →932, evictions 28→1, destructors 1001==1001.
> **Giudici**: ab54 media CPU old 58,875 → new 54,505 = **−7,42% new 6/6
> a separazione netta** → 54,51/20,90 = **2,61× minimo assoluto** (da
> 2,83×); peak fisico 1660,2→1564,1MB = **−5,79%** → 4,16×; full
> stesso-giorno run43 **717,3s** vs run43-old (phpr-wp53) 737,3s =
> **−2,71%** → 717,3/339 = **2,12× nuovo minimo** (old di serata replica
> run41 738,6: ambiente stabile). Parità: corpus 1421 IDENTICO, refl 290
> IDENTICO, cargo 1639/0, ORM 3E/13F per nome, hk 0E/0F, fail-set full
> BYTE-ID a run33 (88 nomi) su run43 E run43-old, probe54 NEW==OLD.

## Ob.1 — tabella CPU-secondi del full (run42, binario phpr-wp53)

Metodo: run42 full detached + 6 finestre `sample <pid> 30 1` (off 90/300/
480/660/840/~928s) sul PID master dal pid-file; attribuzione ad ALBERO
(self-sample per nodo, allocatore attribuito all'antenato semantico);
quote per finestra sui sample NON-wait; stratificazione: ogni segmento
temporale (bordi = punti medi tra finestre) distribuisce la sua CPU dal
`.rss` col profilo della finestra. run42 è run di MISURA (sampling ⇒
+5%): il suo 776,7s non è riferimento. Strumenti: `wp54-harness/{attr,
aggregate,subtree}.py`, tabella integrale in
`wp54-harness/sample-out/attribution-table.txt`.

| canale | s | % | verdetto |
|---|---|---|---|
| corpi handler sotto run_loop | 210,6 | 27,1% | in banda prereg (35-50% con dispatch); arco chiuso WP-44 |
| dispatch run_loop self | 114,5 | 14,7% | idem |
| crypt (blowfish) | 91,2 | 11,7% | ONESTO (probe: phpr −14% vs oracle) — non è gap |
| gc-walk (collect+classify+note) | 77,4 | 10,0% | unico canale prereg SOPRA soglia (8%) |
| str-copy memmove (concat) | 50,2 | 6,5% | `.=` O(n²) — mandato WP-55 |
| madvise (purge=0) | 41,8 | 5,4% | in gran parte figlio dei buffer giganti del concat |
| drop/rc | 36,9 | 4,8% | churn free-side (mi_free+drop Zval > malloc) |
| memops altro | 30,9 | 4,0% | — |
| clone Zval | 23,8 | 3,1% | — |
| malloc TOTALE (tutti i canali) | ~33,8 | 4,4% | predizione 12-20% FALSIFICATA |
| → malloc stringhe | 2,2 | 0,3% | ~10-20ns/pair: single-alloc NON è leva CPU |
| → malloc args-Vec | 1,1 | 0,14% | Fase 2.3 pool: MORTA (a verbale) |
| calls/builtin/resolve-oop/pcre/hash/resto | ~58 | 7,5% | diffuso |

Caveat rappresentatività: 6 finestre × 30s coprono 180/1060s di wall; la
fase early (win1) è dominata da un singolo pattern concat-gigante — la
banda vera di str-copy è 8-50s (7,7s osservati nella sola finestra).

## Ob.2 — quote ns/evento (media group, mwin1-2 + census esistenti)

- stringhe: 51,8M×2 malloc, `PhpStr::new` self 1,06% + malloc:str 0,76%
  ⇒ ~0,6-1,2s su 59s = **~10-20ns/malloc-pair** (small-bin mimalloc):
  fusione single-alloc ≈ −0,5% media MAX. Il costo churn sta nel FREE
  (mi_free 4,7% + drop Zval 5,4%), non nel malloc.
- args-Vec: ~54M call-sites (op-census), malloc:frame 0,03-0,09% ⇒ pool
  bounded quota <0,1s: NON aprire.
- `.=`: probe 5MB in 5k append = phpr 488ms / oracle 2ms. Zend estende
  in place a refcount 1 (zend_string_extend); phpr `concat2` = alloc
  esatta + doppia copia SEMPRE ⇒ O(n²) sugli append-loop.

## Ob.3 — reflect re-key (la leva della sessione)

- Design SEMPLICE vinto sul two-level: `find_method_reflect` (lookup
  puro `&self`, zero side-effect) gira SEMPRE (prima: 88% delle
  chiamate; +~200ns sui soli ex-hit ≈ 12ms/run) e la chiave diventa
  `(decl, mname_lower)` — host_reflect.rs, `14141ab`. Cap 16384
  invariato (ora quasi mai raggiunto: 1 eviction vs 28).
- I mock-DECLARED restano per-mock per costruzione (inserts declared
  16.855 ≈ pre: 17.369) — è la coda irriducibile e corretta.
- Ritorno cap 16384→8192: ora quasi irrilevante (evictions 1);
  decisione utente se si vuole comunque (risparmio standing marginale).

## ⭐ Lezioni

- ⭐⭐ **L'attribuzione owner-level (WP-45/47) applicata alla CPU paga come
  pagò sul footprint**: la tabella ha falsificato in un colpo tre leve
  "ovvie" del backlog (single-alloc stringhe, args-pool, leaf-bit) e
  aperto quella giusta — il singolo A/B migliore da molte sessioni
  (−7,4% media) è uscito da un canale che NESSUNO aveva quotato in
  secondi (i 470k descriptor-build/run del reflect).
- ⭐⭐ **La lente ns/evento (WP-53) è falsificante in entrambe le
  direzioni**: 51,8M malloc = 0,6-1,2s (leva morta), 470k descriptor
  build = 2-4s (leva vera). Prima i secondi, poi il design.
- ⭐⭐ **`sample` legge l'albero ma NON dentro l'inlining**: collect_
  cycles_inner "self 47%" = drains+scans+classify inlined; per separare
  serve il census con timer dedicati (classify_ms). I sample-tree
  categorizzano; i census quantificano.
- ⭐ **I probe da 10 righe contro l'oracle chiudono canali in un minuto**:
  bcrypt (91s "sospetti" → onesto, phpr più veloce) e `.=` (244× = gap
  algoritmico vero). Prima di progettare leve su un canale, chiedersi se
  l'oracle lo paga uguale.
- ⭐ Il criterio leaf per il walk va quotato in SLOT, non in nodi: 37,5%
  dei nodi foglia = solo 6,6% del lavoro d'iterazione.
- ⭐ `sample` conta anche i thread parcheggiati (workq 50%) e le attese
  (read su pipe = stream_get_contents di processi figli): le categorie
  wait vanno escluse PRIMA di normalizzare le quote CPU.

## Prossimo (WP-55) — mandato riformulato dai dati

1. **PhpStr growable + append-in-place a refcount unico** (`hash +
   Vec<u8>` = +8B/stringa da quotare in size-class WP-52; `$s .= x` via
   `Rc::get_mut` quando unico → extend amortizzato). Chiude il canale
   O(n²): quota 15-90s sul full (str-copy 8-50s + gran parte di madvise
   41,8s + malloc:other della fase concat). ⚠️ il vecchio mandato
   "single-alloc via Rc<[u8]>" è INCOMPATIBILE (immutabile, niente
   capacity): i due redesign si decidono INSIEME, e la fusione da sola
   non è leva (0,3% del full).
2. Residuo full vs WP-40: 717,3 vs ~699s ≈ 18s — canali candidati
   rimasti: gc-walk fasi mock (classify del full, NON del media),
   drop/rc churn, corpi handler diffusi.
3. Checkpoint Fase 3 (ri-attribuzione BYTE post Fase 1-2) resta da fare
   per scegliere il tipo pilota — naturale farlo in WP-55 insieme al
   design PhpStr.
4. NON riproporre: single-alloc come leva CPU; args-Vec pool; leaf-bit
   walk (per SLOT è 6,6%); leve sul canale crypt (onesto).
