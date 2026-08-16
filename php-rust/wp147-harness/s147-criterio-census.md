# Criterio S-147 p.1b — CENSUS UNICO ORM monobinario (concilio S-146 §Ordine p.1; KS-146-1..7) — commit PRIMA del run

1. Oggetto: nella STESSA run (suite ORM, ×2 repliche) emettere (i) movimenti
   Zval per SITO VM (op del dispatch) × categoria + DIGRAMMI prev→cur
   (righe `s147mv`/`s147dg`, feature mem-census+op-census); (ii) contatore-
   PONTE: movimenti con origine slot-load (Σ righe `s147mv` dei siti della
   lista DICHIARATA {LoadSlot, LoadVar} + eventuali fused-load visti nel
   dump, nominati a verbale) accanto a `slot_reads`/`slot_reads_rc` della
   convenzione census (riga `zvalcensus`) — MAI mescolate fuori dal ponte
   (KS-146-2); (iii) take-per-tipo: `would_take_safe_{str,ref,arr,obj}`
   (riga `zvalcensus_s147`). SOLO conteggi: MAI cifre di tempo da qui.
2. Binario: census `--features "mem-census zval-census op-census"` (probe,
   MAI pinnabile, hash a verbale); PHPR_OP_CENSUS=path (arma l'attribuzione).
3. Igiene: lock sessione verificato; sentinelle stampate non-gate (S-143
   p.1); smoke a esito ESATTO ≥1 su chiavi {s147mv LoadSlot, s147dg,
   s145.clone_*}; `zvalcensus_s147` presente (arr/obj possono restare 0 su
   script minimo: FUORI smoke, dichiarato).
4. Validità: r1==r2 ≤1% per chiave aggregata (tot movimenti, tot per classe,
   ponte, would_take_safe_*), pena KS-146-2 (riconvoca); parità per NOME vs
   baseline16 (pena cifra NULLA); denominatori dal sorgente della stessa run.
5. Banda in SECONDI = conteggi × prezzi sonda s145 (per-coppia, per classe,
   da `wp145-harness/s145-sonda-prezzi-verdetto-t3.out`, citati nel verdetto)
   — mai lo SCREEN (pensionato). Parser `s147-parse.py` committato + golden
   PRIMA del run; cifre citabili SOLO da `s147-census-verdetto.out`.
6. KILL ARITMETICO (KS-146-1, pre-registrato PRIMA dei dati): soglia =
   risoluzione del giudice della scommessa = ±0,7% della coppia ORM
   RIMISURATA @ s145 (p.1a, `s147-orm-rimisura-verdetto.out`: 0,007 ×
   phpr_user_net medio, in secondi — derivazione MECCANICA nel parser).
   Banda alta borrow-first (Σ siti slot-load × prezzo classe) < 1× soglia ⇒
   ZERO codice sul bersaglio; 1×–2× ⇒ solo fette micro-judged (REGOLE §3);
   ≥2× ⇒ scommessa suite ammessa con KS-B1 ri-registrato (sintesi §soglia).
7. TakeSlot resta CHIUSO salvo le tre condizioni della sintesi; il residuo
   take-eligible si legge da (iii) × prezzo, contro la STESSA soglia (KS-M3:
   chiuso comunque se inc-dec ≤20% del churn ripartito).
8. Lettura pre-registrata (falsificabile): atteso ponte ∈ (0,1] del totale e
   ranking siti con testa Load/Push/Prop-family (coerente col 44% clone
   inline da run_loop, S-140); un ponte ≈0 o un totale s147 ≠ Σ s145.clone_*
   (stessa run) = incoerenza interna ⇒ KS-146-2, si riconvoca, non si cita.
9. Fuori perimetro: nessuna leva si scrive in S-147 sul verdetto del census;
   il census ORDINA i bersagli FR1-ext (p.2) e prezza il residuo TakeSlot.
