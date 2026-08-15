# S-142 — DOSSIER BUDGET DI PARITÀ ORM (per il CONCILIO a 9 — rotta utente 2026-08-15)

**Mandato**: attribuzione TOP-DOWN del divario ORM per NOME (statement-count ×
tasse), perché il concilio deliberi la scommessa STRUTTURALE (oggetti
handle+arena vs layout Zval/Option). Micro-leve ORM SOSPESE su tre
falsificazioni pre-registrate: S-139 tre-leve dim-write (Δ suite ≈ 0), HC1
(0,13% suite), L-RD1 (~1–2% atteso, census §6 in coda). REGOLE §4: ogni riga
qui sotto è «direzione+meccanismo firmati, magnitudine NON ripartita» salvo
dove è scritto A/B.

## 1. Ancore (misure proprie, datate)
- **T_phpr = 42,52–42,57 s user · T_oracle = 4,94–5,00 s user** (S-139 rimisura
  @ s138, 2 gambe/lato, net-pavimenti; `wp139-harness/s139-rimisura-verdetto.out`)
  ⇒ **divario ≈ 37,6 s = 88% del tempo phpr**. Rapporto net 8,59–8,71.
- La parità NON è raggiungibile per sottrazione di canali minori: anche
  azzerando str+compile+refl+calls (≈6% cumulato) il rapporto resta >8.

## 2. Attribuzione per FAMIGLIA (profilo campionario S-140 ×2 repliche, top-of-stack; grade=INDIZIO)
Percentuali medie r1/r2 su T_phpr ≈ 42,5 s (`wp140-harness/s140-profilo-verdetto.out`):

| famiglia | % suite | ≈ s | contenuto nominato |
|---|---|---|---|
| other (coda lunga, max singola 1,1%) | 26,6 | 11,3 | nessun canale >1,1% — S-141 riquantificazione |
| vm_inline (run_loop dispatch+inline) | 16,5 | 7,0 | dispatch, drop-glue inline nel loop (34,5% dei subtree Zval-glue) |
| memops (memcpy/memmove/memset) | 12,6 | 5,4 | movimenti Zval/Props/buffer |
| churn_zval (clone/drop outlined) | 10,3 | 4,4 | Zval clone/drop, drop-glue leaf 4,5–4,7% |
| map (hash ins/lookup FxHash) | 8,8 | 3,7 | props-map, HashMap engine |
| gc (note/sweep/collect) | 8,0 | 3,4 | gc_note 238,6M ev + Sweep 59,2M siti |
| prop_dim (prop/dim macchineria) | 6,0 | 2,6 | prop_step, dim-write (post AP1/FD1/RMW/IC) |
| alloc (mi_malloc/mi_free leaf) | 5,4 | 2,3 | v. §3 conteggi |
| calls | 3,9 | 1,7 | frame/arg plumbing (arity §3) |
| str · compile · refl | 0,8+0,8+0,2 | 0,8 | compile ≤1% leaf CONFERMATO; refl 0,2% |

## 3. CONTEGGI del census (S-141 raw datati 2026-08-15, suite ORM intera, monobinario census s140 — mai tempo)
`wp141-harness/census-out/census-r1-20260815.txt` (r2 conforme <1%):

| meccanismo | conteggio/run | prezzo unitario plausibile | ≈ s (INDIZIO) |
|---|---|---|---|
| **alloc+free heap** (galloc_n/gfree_n) | **471,3M + 468,8M** | 8–15 ns/coppia (mimalloc) | **3,8–7,1** |
| realloc | 19,5M (4,87→9,63 GB mossi) | — | dentro memops |
| **gc_note** | **238,6M** (scalar 73,7M · obj 56,5M) | 2–5 ns | 0,5–1,2 |
| drop Zval su stack-siti (drop_s+drop_c) | 33,1M + 8,3M | 3–10 ns | 0,1–0,4 (leaf; la glue vera è in churn) |
| propget (clone lettura prop) | 29,9M (26,7M Rc) | ~5–15 ns | 0,15–0,45 |
| recv_clone (load+prop) | 14,8M | — | idem famiglia churn |
| push stack totali (LoadVar 58,4M + PushConst 54,3M + …) | 118,3M | — | proxy VOLUME dispatch |
| chiamate per arity (a0 14,7M · a1 8,3M · a2 2,8M · ≥3 2,0M) | ~27,8M | — | famiglia calls |
| hint_checks (HC1, già spedita) | 35,6M | ~1,65 ns | 0,05–0,06 (misurata: 0,13%) |
| **teardown array** (rd1: 21,7M array · 118,8M elem — census S-142) | 140,5M ev | 0,5–1,0/elem + 2–5/arr | 0,10–0,23 (L-RD1 spedita ci vive qui) |
| bytes: alloc 29,4 GB · free 33,8 GB /run | — | — | pressione cache: non prezzabile qui |

**Riconciliazione** (due strumenti indipendenti): conteggi×prezzi alloc/free
3,8–7,1 s ↔ famiglie memops+alloc 7,6 s: COMPATIBILE (il campionario include i
memcpy dei valori mossi). gc famiglia 3,4 s ≫ nota 0,5–1,2 s: il grosso è
sweep/collect (Sweep 59,2M siti), non la nota — coerente con WP-21 (leva
note-time refutata).

## 4. Tasse per-statement (modelli A/B chiusi S-129/S-131, giudici micro)
- Statement `Field*` phpr ≈ 300–340 ns vs oracle 23–37 ns (**tassa ~10×**,
  quasi invariante per forma — S-129 modello chiuso 96%).
- Decomposizione phpr (S-131, chiusura 93–94%): prop_step interno 130,7
  (guardie 49,4 · defer 37,0 · key+op 34,3) + dispatch 36,3 + preludio 73.
- Micro-ORM: objchurn 6,7× · objalloc 6,4× · objdatains 5,9× · objdropdef 7,5× ·
  objmap 11,7× (round-trip GC attribuito S-137) · evalcls 316,9× (cliff
  compile-per-classe, ≤1% della suite reale) · refl 42,4×.
- Costo/op del loop VM ~9–10 ns quasi invariante (S-103): il collo NON è il
  dispatch dell'op, è il CICLO DI VITA del valore che l'op muove.

## 5. La lettura che il dossier sottopone al concilio
I quattro canali maggiori (vm_inline-drop-glue, memops, churn_zval, alloc,
gc, map ≈ **26–28 s cumulati**) sono TUTTI costi di CICLO DI VITA per-valore:
alloc/free di box Rc, clone/drop ai movimenti, nota GC per scrittura, lookup
per risoluzione. Zend paga gli stessi MOVIMENTI logici con zval 16-byte
by-value, arena/pool per-request, niente Rc, niente nota per-move. Le
micro-leve comprano il singolo sito (3 falsificazioni: il sito non è mai >2%);
la struttura compra il PREZZO UNITARIO di tutti i siti insieme. Da qui la
scommessa strutturale, due opzioni NOMINATE (riesame veti SOLO in concilio):
- **A. Oggetti handle+arena**: payload oggetti in arena per-request, Zval porta
  un handle; azzera per gli oggetti la coppia alloc/free (quota di 471M da
  censire per classe), il dec/inc Rc nel churn, parte della nota GC (obj 56,5M).
  Rischi nominati: identità/refcount visibile (`===`, weakref, __destruct
  timing §3.22), sweep per-request (RetainSet, binding output-capture).
- **B. Layout Zval/Option (by-value + niche)**: riduce clone/drop e memcpy per
  movimento (memops 5,4 s + churn 4,4 s i bersagli); non tocca alloc-count né gc.
- (con dati census per-classe si può quantificare A vs B canale per canale —
  v. §7.)

## 6. Quota L-RD1 (criterio p.7 — MISURATA, verdetto s142-census-verdetto.out)
Census ×2 (r1==r2 al singolo evento; parità per NOME rc=0): **rd1_arrays
21,7M · rd1_elems 118,8M · rd1_tombs 73k per run** ⇒ **quota 0,24–0,53%
della suite ORM (INDIZIO, prezzi 0,5–1,0 ns/elem + 2–5 ns/array)** ≈
0,10–0,23 s su 42,5 s. Lettura pre-registrata p.6: citata come canale residuo;
SOTTO la risoluzione della coppia suite (banda ORM ~±0,7%): L-RD1 è vera al
suo giudice ma NON misurabile sulla suite — QUARTA conferma numerica in
quattro sessioni che il singolo sito non muove Doctrine. Il «teardown array
≈2%» S-141 era il CANALE intero: la leva ne rimuove il quarto–metà (il resto
è dec Rc, free dei buffer, costi che restano per necessità).

## 7. Residui NOMINATI prima del deliberato (il concilio li vede come limiti)
1. **Census alloc/free PER CLASSE** (CH_* histogram) su ORM: quota oggetti vs
   array vs stringhe vs Vec-args dei 471M — decide quanto compra l'opzione A.
2. Profilo equivalente lato ORACLE per famiglia (one-sided finora: REGOLE
   feedback-one-sided-profile — serve la stessa lente su Zend per non
   sopravvalutare i canali che anche Zend paga).
3. La quota di «other» 26,6% resta coda lunga non nominata (max 1,1% a voce):
   il concilio non deve trattarla come riserva di caccia.
4. Prezzi unitari alloc/gc_note: oggi plausibili cross-giudice; una sonda
   monobinaria kill-switch (classe S-138) li firmerebbe.
