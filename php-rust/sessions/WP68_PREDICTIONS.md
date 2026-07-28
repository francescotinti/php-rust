# predictions68 — pre-registro INTEGRALE (PRIMA di ogni letto nuovo)
# Regole: M-66.5 (bande bilaterali SOLO su meccanismo dichiarato),
# G-68.1 (cifra citabile solo |Δ|>spread, ABBA o ≥2 self-pair),
# K-68.3 (ogni predizione riceve disposizione in design68 §10).
# Fonti citate = dati GIÀ registrati WP-66/67 (leak-table,
# impure-cost-table, nk-table, memcensus66-l661 pid=90909).

## P68-PRE — precondizioni leva (KS-P68.1; pass/fail, non bande)

- **P68-PRE-a (KS-S67.2)**: static function-var su unit CACHEATA
  (`static $n=0; ++$n`), R1..R3 stessa URL: OGNI richiesta stampa
  `1` (reset per-richiesta). Meccanismo: statics vivono nello stato
  per-Vm, non nel Module condiviso. Se una R stampa ≥2 ⇒ persistenza
  cross-request ⇒ STOP fronte (KS-S67.2).
- **P68-PRE-b (P-67.2/KS-P67.2)**: unit cacheata con assegnazione
  condizionata `if(isset($_GET['x'])) $g=1;`; R senza `x`: il nome
  NON compare in $GLOBALS (nemmeno come NULL); R con `x`: compare
  con 1. Violazione ⇒ STOP fronte.
- **P68-PRE-c (S-67.4/KS-S67.3)**: g2′ include-vuoto poi
  var_export($GLOBALS): nessun nome della R1 affiora come NULL in
  R2 (bleed Undef→NULL da make_cell). Violazione ⇒ KS-S66.1
  riattivato, reset seed per-request.

## P68-Q — quota defer-always (S-68.4/M-68.2/K-68.5; banco KS68-2)

Meccanismo: al hit di ex-impura i DeclareDeferred ri-lowerano il SOLO
snippet (decl singola, ~10²-10³ B) contro il seed corrente; il
whole-file lower+compile (fonte dei 21 ms — impure-cost-table WP-67)
NON si ripete. Numero di siti defer attesi su wpdev/`/`: dell'ordine
dei 15 file impuri, con multipli per pomo/Requests ⇒ banda
**N_defer ∈ [10, 60] DECLARE/richiesta**.

- **P68-Q-a**: costo mediano per-DECLARE (mediana R4→R10, ×3 run,
  build census, etichetta "ns su build census"):
  **[10, 400] µs/DECLARE** (snippet ≪ file; il lower seeded paga
  O(seed-touch), non O(unit)).
- **P68-Q-b**: costo defer TOTALE per richiesta steady-state:
  **< 10 ms/req** (kill-switch KS68-2: sopra ⇒ fallback
  publish+dep-replay, mai tuning). Predizione puntuale di meccanismo:
  **[0,2, 6] ms/req**.
- **P68-Q-c (B-68.4)**: Δ(lower+compile) whole-file steady:
  **21 → <2 ms/req** (leak-table WP-67: 20,1-22,3; post-leva restano
  gli eval e il rumore di finestra).

## P68-B — comportamento cache post-leva (B-68.4/KB67-2)

- **P68-B-a**: probe ksp1 R≥3 su wpdev `/`: **hit_cross=512, cold=0,
  dc=0**, fp-seq stabile; le 15 ex-impure = hit_cross (criterio
  KB67-2 — mai "publish avvenuta"). KS-S68.2: hit_cross<15 dei 15 o
  miss_dc>0 ⇒ leva NON consegnata.
- **P68-B-b**: Δparked_modules 22 → **7/richiesta** (solo eval —
  leak-table: 22 = 15 impure + 7 eval); Δparked_bytes 1,62 MiB →
  **< 0,25 MiB/req** (solo la coda eval; le impure hittano).
- **P68-B-c**: batteria ordering-s68, 5 casi ×2 lati (cold E hit):
  **BYTE-ID all'oracle** (KS-S68.1: uno storto ⇒ leva non spedita;
  KS-S68.3: c-cond autoload su ramo morto ⇒ FAIL immediato).
- **P68-B-d**: gate-ordering (fixtures/ordering/): post-leva
  phpr-cold == oracle byte-id — ri-pin DELIBERATO a verbale
  (S-68.3/P-68.5), atteso NUOVO dichiarato qui PRIMA.
- **P68-B-e**: declaration-order H-68.1 (class_exists mid-file,
  get_declared_classes, extends condizionale): phpr == oracle su
  tutti i lati post-leva.
- **P68-B-f**: corpus 1421 / refl 290 / ORM 3E/13F / hk 1665 / full
  88 fails PER NOME INVARIATI (KB68-1: un nome nuovo ⇒ forma
  RITIRATA). Nota di meccanismo: i phpt CLI girano senza unit-cache
  cross-process ⇒ il canale toccato è il defer path del lowering;
  fixture con autoload-in-lowering possono cambiare ORDINE output
  solo dove oggi divergevamo da Zend (correzioni, mai regressioni
  per nome).

## P68-L — debiti P-2 (L-68.1/E-68.2/H-68.2/B-68.5)

Meccanismo del residuo (+2,549 KiB/req, nk-table): 2 record/req
modrecon mai potati × ~896 B (bin 896: +1,751 KiB/req — Leijen) +
righe census minori; i Weak tengono vivo l'RcBox (Hejlsberg d).

- **P68-L-a (E-68.2)**: dopo prune delle Weak morte al dump (o
  self-accounting L-68.1): pendenza residua su nk-doc N=1000 =
  **[0, 0,5] KiB/req** (pre-registro Hejlsberg 0,3-0,5). KS68-1:
  >1 KiB/req ⇒ "residuo=bookkeeping" decade, leak-hunt PRIMA della
  leva.
- **P68-L-b (H-68.2)**: riconciliazione alla cifra: bookkeeping
  quantificato ∈ [2,0, 2,55] KiB/req dei 2,549 osservati (≥80%
  spiegato da righe×size + RcBox shells).
- **P68-L-c (B-68.3)**: pendenza Σcommitted su WPDEV vero (memgc67,
  census-tabelle ON, slope-only) N={1,100,1000}, PRE-leva: dominata
  dal bookkeeping = Δparked 22/req × ~0,9 KiB ⇒ banda
  **[12, 40] KiB/req**; POST-leva (parked→7/req): **[3, 15] KiB/req**;
  post-prune E-68.2: **< 2 KiB/req**. KB68-3/KH68-3: sopra banda
  pre-registrata ⇒ niente banda axum.

## P68-M — metro Σcommitted, terza run (KL67-1 via KL68-2)

Fonte: memcensus66-l661.txt pid=90909 (master, PRE-P2): units=4726,
mod_owned=115,1 MB, counted_v1_tot=231,8 MB; figli tipici units≈699.
Meccanismo: post-P-2 i moduli ritenuti del master muoiono a fine run
⇒ al dump restano ~i vivi (nk: 5/2003).

- **P68-M-a**: master post-P-2 (memgc67, stessa catena l661 ma
  protocollo no-swap L-68.4): **units ≤ 1000** (da 4726),
  **mod_owned ≤ 20 MB** (da 115,1 = −≥83%).
- **P68-M-b**: Δcounted master (counted_v1_tot): **−[80, 200] MB**
  vs 231,8 (mod_owned −~100 MB + quota cls_priv dei morti; ceiling
  dichiarato: la ripartizione cls_priv vivi/morti non è nota PRIMA).
- **P68-M-c**: la run è valida SOLO sotto L-68.4 (swap ≤64 MiB,
  Δpageouts≈0, (b)≥0); altrimenti NON-ESEGUITA a verbale, metro
  resta non-promosso (KL68-2).

## P68-G — costo CPU leva (G-68.1)

- **P68-G-a**: coppia full build-adiacente (leva vs igiene-only o
  parità) + self-pair: |Δ| ≤ spread della serata ⇒ "indistinguibile";
  se |Δ|>spread con segno +: il costo VA spiegato per-file (KS-S68.4)
  prima di spedire. Nessuna cifra citabile sotto spread (KG68-2:
  quozienti a ≥4 cifre nel verdict-file).
