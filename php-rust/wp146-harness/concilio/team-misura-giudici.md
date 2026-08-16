# Team MISURA-GIUDICI (Gregg, Klabnik, Leijen) — nota di sintesi S-146
I verbali individuali restano la fonte VINCOLANTE.

## 1) CONVERGENZE
- **3/3 — Lo SCREEN 4,5–6,5% è morto**: ogni banda B3 si esprime in SECONDI = conteggi × prezzi per-movimento della sonda (Gregg R2; Klabnik R2 con non-citabilità di design95 §P1 e righe guadagno_*; Leijen c).
- **3/3 — Arena-conteggi: ARCHIVIARE** senza definizione ≤1 pagina con giudice nominato (Gregg d; Klabnik d; Leijen d: se è drop-a-blocco è leva di prezzo, rientra solo come A-pool con oneri S-143).
- **3/3 — Borrow-first/FR1-ext si istruisce comunque** (zero liveness, non attende censimenti): Klabnik e Leijen la ordinano PRIMARIA (unica che elimina il movimento, cioè il pavimento 69,5%); Gregg pari grado in parallelo.
- **3/3 — e)**: tetto modellato 1,52 s = tappa, mai parità; nessun claim sui ~4,4 s glue.
- **3/3 — kill aritmetico pre-registrato PRIMA del census** (Gregg R3; Klabnik R1/K1; Leijen R1 via soglia REGOLE §3).
- **2/3 (Klabnik, Leijen) — TakeSlot non compra il memcpy**: elide solo inc-dec (0,21 s) + eventualmente nota (0,46 s, non provato) ⇒ per Klabnik pre-ucciso a tavolino (KS-B3-K2); Gregg rinvia il verdetto a dopo il census.

## 2) CONFLITTI non levigati
- **Quale censimento**: Gregg = F1-su-ORM con contatore-ponte slot_reads↔movimenti (R1: VIETATO mescolare le due convenzioni) + provenienza per sito (R5); Klabnik = MI OPPONGO a F1-ORM ora, solo census SITI-CONSUMATORI; Leijen = F1-ORM sì ma dopo FR1-ext. R5-Gregg e c-Klabnik convergono di fatto sui siti; F1 resta 2/3 contro 1.
- **Soglie**: Gregg 1× banda giudice (~0,26–0,30 s); Klabnik 2× (0,6 s suite-judged, oppure 4 ns/iter micro-judged); Leijen rinvia a REGOLE §3. Non levigato.
- **Solo Leijen**: canale alloc — B3 compra ZERO dei 471,3M pair; guardia «galloc invariante per costruzione» (R2, KS-L1); prezzi pair zcell/arr0 = INDIZIO, peso nullo (gate 5% mai ricollaudato, R3); tranche-3 growth-alloc concorrente per la leva dopo (residuo 57,9%).

## 3) PRIORITÀ S-147
Un SOLO census ORM monobinario (r1==r2, criterio ≤10 righe firmato prima) che emetta nella stessa run: movimenti per SITO × prezzo per classe (soddisfa Klabnik-c e Gregg-R5) + contatore-ponte e F1 (Gregg R1); kill aritmetico registrato PRIMA — soglia da arbitrare in sintesi (1× vs 2× banda). FR1-ext procede in parallelo senza attendere.

## 4) Kill-switch del tema
KS-G1/K1 banda<soglia ⇒ zero codice; KS-G2 ponte indefinibile o r1≠r2>1% ⇒ riconvoca; KS-G3 e KS-L2 giudice sbagliato (quota memops / prezzi pair come budget) ⇒ criterio invalido; KS-L1 delta galloc ⇒ STOP fetta; KS-B3-K3 modello falsificato ⇒ stop famiglia; KS-B3-K4 4 sessioni ORM fermo ⇒ riconvoca.
