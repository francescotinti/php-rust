# REPORT_GAP_54 — gap perf oracle↔phpr della sessione WP-54

> SOLO le misure della sessione WP-54; trend cumulativo in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 54,51/20,90 = **2,61× minimo assoluto**
  (A/B ab54 old=phpr-wp53 vs new=re-key reflect: **−7,42%, new 6/6 a
  separazione netta** — old 58,60-59,19 vs new 54,38-54,69).
- **media footprint**: 1564,1/375,8MB = **4,16×** peak fisico (old
  stesso-giorno 1660,2 = 4,42×, A/B **−5,79%**) — il re-key collassa i
  descrittori standing (inserts inherited 452.621→932).
- **full-suite master-CPU**: run43 (new) **717,3s ≈ 11:57** vs run43-old
  (phpr-wp53) 737,3s ≈ 12:17 = **−2,71% stesso-giorno**; old di serata
  replica run41 (738,6) ⇒ ambiente stabile, il numero è pulito →
  **riferimento pubblicato = run43 717,3/339 = 2,12×** (era run40 2,14×;
  residuo vs WP-40 2,06× ≈ 18s). Fail-set **BYTE-ID a run33** (88 nomi)
  su run43 E run43-old; 30.472 test 0E/2F/86W/73S ×2.
- **full-suite wall**: ~17/11,5 min = **1,4×**.
- **mechanism-check (gc-census media, stesso-giorno)**: reflect hits
  62.206→514.409, misses 469.990→17.787 (hit-rate 11,7%→**96,7%**),
  evictions 28→1, inserts inherited 452.621→932, destructors 1001==1001.
- **Attribuzione CPU del full (run42 campionata, Ob.1)**: vedi
  `sessions/WP_SESSION_54.md` — top canali: corpi+dispatch 41,9%, crypt
  11,7% (ONESTO: probe bcrypt phpr −14%), gc-walk 10,0%, str-copy 6,5%
  (`.=` O(n²) 244× vs oracle → mandato WP-55), madvise(purge=0) 5,4%,
  malloc TOTALE 4,4% (stringhe 0,3%: single-alloc falsificata come leva).
