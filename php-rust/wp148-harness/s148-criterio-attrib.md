# Criterio S-148 p.1 — ATTRIBUZIONE `other` 57,9% (tranche-3, Leijen): census a TAG di contesto su SUITE ORM — commit PRIMA del run

1. Oggetto: partizione COMPLETA di `galloc_n` per TAG di contesto (thread-local,
   RAII, il tag più INTERNO vince), lista CHIUSA: `frame` (Frame::with_buffers) ·
   `hostcall` (handler run.rs Op::CallBuiltin + Op::CallHostBuiltin) · `gc`
   (collect_cycles_inner) · `arrgrow` (array.rs to_hashed + set_returning_displaced
   + append + append_default) · `none` (residuo). FUORI per NOME (→ none):
   compile/eval, nativi via host.rs/host_reflect, iteratori, Stringify, props-map.
2. Crosswalk eventi: ogni nota-evento già attribuita in s144 (cum_n str/arr/obj via
   memcensus::alloc con ch≠UNIT, arrbuf, propsbuf, rczval, vecargs) bumpa `attr`
   del tag corrente; `other_tag = n − attr` (convenzione 1 evento = 1 alloc
   EREDITATA da s144, banda dichiarata lì; realloc-eventi FUORI dal denominatore,
   A-LE-104-1, stampati accanto).
3. Numeri emessi per tag: n · attr · other · bytes · istogramma 11 bucket
   (HIST_BUCKETS); ranking di `other_tag`; hist di `none` a corredo (shape del
   residuo non nominato).
4. Identità di validità: Σ_tag n == galloc_n della STESSA run (ESATTA, stesso
   hook); r1==r2 ≤1% per chiave aggregata; parità per NOME vs baseline16 (pena
   cifra NULLA); contatori s144 attesi == s144 (workload identico) — scarti >1%
   dichiarati.
5. Binario: census `--features mem-census` (probe, MAI pinnabile, hash a
   verbale); monobinario, ×2 repliche, sentinelle stampate non-gate (S-143 p.1);
   smoke a esito ESATTO: s148tag frame≥1 E hostcall≥1 E arrgrow≥1 E identità p.4
   sullo smoke; gc può restare 0 su script minimo (FUORI smoke, dichiarato).
6. Parser `s148-parse.py` committato + golden PRIMA del run; cifre citabili SOLO
   da `s148-attrib-verdetto.out`.
7. Lettura INDIZIO pre-registrata (veto S-147: prezzi pair zcell/arr0 8,7–11,8
   ns/coppia restano INDIZIO, MAI budget): soglia-conteggio = 0,293 s / 11,8 ns
   ≈ 24,8M — una classe con other < 24,8M NON può essere bersaglio-solo nemmeno
   al prezzo pair alto (esclusione CONSERVATIVA in conteggi); nessuna conversione
   in secondi come cifra di record.
8. Decisione: la TESTA del ranking other è il bersaglio della sonda-PREZZO
   propria (il «poi prezzare» di NEXT_SESSION); NESSUNA leva si scrive in S-148
   sul verdetto del census; cifre citate sempre con la qualifica «tetto su
   binario census».
9. Esiti pre-registrati: probe MUTO allo smoke ⇒ STOP rc=8, niente run; r1≠r2
   >1% su chiave aggregata ⇒ dichiara e replica; identità p.4 violata ⇒ verdetto
   NON valido (solo osservativo), si riconvoca il disegno.
