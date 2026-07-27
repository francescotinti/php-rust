# predictions66 — predizioni PRE-letto WP-66 (K-66.1: questo file non si
# modifica dopo la registrazione; copia conservata predictions66.locked.md
# + copia committata nel repo sessions/WP66_PREDICTIONS.md PRIMA dei letti.
# I verdetti vivono in design66.md §8, file DISTINTO.)

Registrate: 2026-07-27, prima di ogni letto server/census della sessione.
Binario dei letti: build tree fe33706 (= phpr-wp65 + tripwire
seed_prefix_short FATAL + colonne census; gate66 in corso alla
registrazione — un gate rosso invalida i letti di parità, non le
predizioni). Regole: G-66.3 (letto pinnato verbatim + formula),
K-66.5, M-66.5 (istruttorie senza banda non etichettate KS).

## P66-R (KS-P1 replay wpdev, php-server porta 8080, docroot
## ~/Claude/wpdev/src, DB wp, N=10 richieste GET / identiche)

Letto pinnato: `wp66-harness/p1-out/replay/uc.log` (eventi ancorati
`^unitcache <evento> `, vocabolario chiuso) + `r<i>.body` (curl body).

- **P66-R1 (byte-id, BILATERALE stretta per meccanismo)**: body R2 ==
  body R1 byte-id, e R3..R10 == R1 (al netto di nonce/timestamp WP:
  se il body contiene entropia per-request, il confronto passa alla
  versione normalizzata SOLO per campi dichiarati nel verdetto — la
  normalizzazione si dichiara, non si tace). Bordo: QUALSIASI
  divergenza non-normalizzabile ⇒ KH66-1 STOP fronte.
- **P66-R2 (hit_cross)**: R2 produce `hit cross` ≥ 1 (lettera KS-P1) e
  `hit_cross_R2 ∈ [1, U1]` dove U1 = numero di eventi `fp` di R1
  (formula: un hit cross per unit rigiocata; non può superare le unit
  caricate da R1). Il RAPPORTO hit_cross/U1 è ISTRUTTORIO (primo
  letto server, nessuna storia — niente KS sul rapporto, M-66.5).
  Bordo duro: hit_cross_R2 = 0 ⇒ KS-P1 fallita, STOP.
- **P66-R3 (fp-seq)**: la sequenza ORDINATA degli eventi `fp` di R2
  è IDENTICA a quella di R1 (path e ordine). Bordo: divergenza ⇒
  KH66-1/K-M66.3 (prefissi non byte-identici = replay invalido).
- **P66-R4 (B-66.2, contatori piatti al warm-up)**: da R3 a R10, con
  URL identica e file INTOCCATI: zero eventi `miss cold|miss fp|miss
  dc|miss nostat` e zero `elide` (elide accade solo alla compile).
  Formula: cache warm ⇒ ogni load è hit. Bordo: un miss su R≥3 ⇒
  KB66-3, fronte fermo su P-2 prima di ogni banda.
- **P66-R5 (K-M66.1)**: zero eventi `seed_prefix_short` su TUTTA la
  batteria. Bordo: UN evento ⇒ STOP fronte (corruzione di confine).
- **P66-R6 (M-65.2, ISTRUTTORIA — nessuna banda, nessun KS)**: conteggio
  `miss dc` per componente sul workload server = primo letto del
  ceiling autoload-in-lowering (CLI miss_dc=0). Si legge e si deposita.

## P66-S (S-66.4 batteria scope-hygiene per-richiesta, fixtures
## dedicate in docroot proprio, stesso server single-worker)

Letto pinnato: `wp66-harness/s4-out/` (body delle richieste probe).

- **P66-S1 (KS-S66.1, BILATERALE [0,0])**: la 2ª richiesta NON vede
  in `$GLOBALS` alcun nome user della 1ª — né come valore né come
  NULL. Formula: VM fresca per richiesta ⇒ insieme atteso = ∅.
  Bordo: UN nome ⇒ STOP fronte, si progetta il reset per-request
  (non si cataloga).
- **P66-S2**: `$GLOBALS['x']=…` in R1 invisibile in R2 (stesso [0,0]).
- **P66-S3**: get_defined_vars/extract in template incluso
  per-richiesta: output R2 == R1 byte-id (il template rigiocato dalla
  cache nomina gli slot dal seed della VM NUOVA — è il punto della
  leva v2 sotto server).
- (superglobali `$_GET/$_POST/$_SERVER/$_COOKIE` e `session_start`:
  in batteria se il server le espone già; altrimenti si dichiara
  NON ESEGUIBILE oggi e si rimanda al fronte axum vero — non si
  finge un letto.)

## P66-L (L-66.1 census fuori-bin, run census full su phpr-memgc65b
## INVARIATO — binario 02b7d9d2, per confrontabilità con census65b)

Letto pinnato: `wp66-harness/census-out/l661-vmmap-*.txt` (vmmap
region-level del master al checkpoint exit_collect_mi) vs Σcommitted
delle righe `tag=mi_bin win=0 src=main` dello stesso checkpoint.

- **P66-L1 (attribuzione, bound KL-66.1)**: il fuori-bin del master
  (phys_footprint − Σcommitted) si attribuisce per ≥60% a regioni
  nominate da vmmap (MALLOC metadata/guard, stack, __DATA/__TEXT,
  IOAccelerator, mmap mimalloc non-committed). Sotto il 60% ⇒ ogni
  claim phys standing SOSPESO (solo counted+committed) e riga
  GAP_TREND "fronte aperto".
- **P66-L2 (ISTRUTTORIA)**: il Δ+88,5MB tra census65 e 65b si
  colloca in una classe di regione identificabile (attesa qualitativa:
  segmenti mimalloc committed-but-empty dopo il purge della leva —
  RSS retention, non heap vivo). Nessuna banda numerica: primo letto
  di questa tabella (G-66.3: banda solo dove c'è storia).
