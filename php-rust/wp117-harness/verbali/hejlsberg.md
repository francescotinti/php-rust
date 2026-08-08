# Verbale Hejlsberg — Concilio S-116/117 (lente: compilatori, interning, codegen)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione (A subito / B regime / D metodo / C riserva) è giusta nell'ordine ma sbagliata in DUE punti tecnici che, non emendati, la invalidano.

**Refutazione 1 — BOLT non esiste su questa piattaforma.** Siamo su Darwin/Mach-O (Apple Silicon): BOLT lavora su ELF con profili Linux-perf; non c'è un BOLT maturo per Mach-O. «PGO + BOLT» come scritto è ineseguibile. Il sostituto nativo c'è ed è migliore per il nostro problema: **ld64 `-order_file`** — ordine di funzioni DETERMINISTICO e pinnato. È questo, non il PGO, che «ripara il metro»: il PGO da solo ritira i dadi del layout a ogni build; l'order_file li toglie dal tavolo.

**Refutazione 2 — la riserva su C è aritmeticamente una finzione.** Per ≤3× servono −45% su arith (5,5) e −60% su prop (7,6). A vale 5-15% una-tantum; il treno B decine di ns su ~107-160 ns/iter (L-A ≈ −25% su prop). Composto ottimistico: prop 7,6→~5,1. Il fatto S-103 (9-10 ns/op invarianti = ciclo di vita Zval) dice che il fattore residuo vive in C. Il «solo se >3× dopo A+B» scatterà con quasi-certezza: C non è riserva, è la coda già visibile — l'istruttoria (design su census WP-95/96, perimetro compatibile col verdetto TakeSlot) parte in parallelo, il cantiere si delibera a concilio con le cifre A+B in mano.

## ROTTA DALLA MIA LENTE (3 sessioni)
1. **S-117 = A, in due stadi.** Fatto verificato oggi: il workspace **non ha `[profile.release]`** — build a default (16 CGU, niente LTO fat). Stadio A0: `lto="fat"` + `codegen-units=1` (+ order_file estratto e versionato) — gratis, deterministico, e i 16 CGU sono essi stessi una sorgente della lotteria di layout. Stadio A1: PGO (`-Cprofile-generate` → workload → `llvm-profdata merge` → `-Cprofile-use`), profilo hashato nella ricetta di `pin-phpr.sh`. Poi: **rimisurare la banda con leva nulla sul nuovo assetto** (attesa: ben sotto 10) e ri-giudicare **L-A da sola** su binari RICOSTRUITI (mai A/B cross-pipeline coi candidati vecchi).
2. **S-118: verdetto L-A sul nuovo assetto.** Se la tassa calls persiste a layout deterministico, non era layout: cura = outlining del probe miss (`#[cold]`/`#[inline(never)]`) — coerente con S-104 (run_loop icache-bound). B (treno, max 3 vagoni firmati) solo se le leve singole affogano ANCORA.
3. **S-119: D census-gated** + istruttoria C.

## EMENDAMENTI
- **R1**: sostituire BOLT con order_file ld64; A giudicata pipeline-vs-pipeline stessa sera (micro+held-out+WP) + banda nulla rimisurata.
- **R2**: profilo PGO addestrato su WP request-loop + corpus misto, **mai sui soli sei micro** (overfit del giudice); profilo e order_file versionati; determinismo provato: due build stessa ricetta → hash .text identico.
- **R3**: tutte le bande/soglie pre-A decadono sul nuovo assetto; primo atto post-A = leva nulla.
- **R4**: per D, **veto sul threaded dispatch**: Rust stabile non garantisce tail-call (già famiglia-refutato S-111). Ordine d'istruttoria: interned strings SOLO se il census conta hash/memcmp residui sul path caldo oltre le IC già esistenti; HashTable packed per arr; specializzazione handler solo census-giustificata (tetto icache S-104).
- **R5**: C promossa da riserva a istruttoria parallela (≤20% finestra), decisione di cantiere al prossimo concilio.

## KILL-SWITCH
- A0/A1: banda nulla non ridotta E guadagno mediano sei-micro <3% ⇒ tenere solo ciò che dimezza la banda, abbandonare il resto.
- Riproducibilità rotta (due build stessa ricetta ≠ identiche) ⇒ solo order_file, niente PGO.
- B: somma del treno sotto la somma delle soglie ⇒ smontare.
- D: census interning sotto soglia pre-registrata ⇒ non portare.

## APPARATO minimo
`scripts/build-pgo.sh` (ricetta unica: profilo→merge→use→order_file→hash), integrato in `pin-phpr.sh` — senza, il pin non è «collaudo-nell'atto».
