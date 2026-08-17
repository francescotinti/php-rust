# S-151 — lettura del census tranche-5 (a corredo del verdetto; il giudizio §5-bis NON viene toccato)

Verdetto: `s151-census-verdetto.out` rc=0 (identità §3 tutte OK ×2 repliche;
s151tot r1==r2 identiche; conservazione b+c==d+l esatta su 2.393 classi;
fail-set r1==r2==baseline16).

## Reperto 1 — la testa hostcall collassa per effetto BT1 (decomposizione, non giudizio)
- s149 (pre-BT1): hostcall_n=335,8M, di cui debug_backtrace 275,0M (81,9%).
- oggi (post-BT1): hostcall_n=82,2M, di cui debug_backtrace 21,3M.
- **Residuo NON-backtrace: 60,8M (s149) vs 60,9M (oggi) = +0,16%** — il
  collasso è interamente nel conteggio frame-granulare di debug_backtrace
  (275,0/21,3 ≈ 12,9×, coerente con BT1 che onora limit/IGNORE_ARGS).
- Conseguenza sul §5-bis (giudizio pre-registrato RESTA: «fuori da entrambe
  le bande, scarto APERTO»): il confronto con 325,4M/335,8M non è più
  informativo per lo scarto +3,2% s148↔s149 — l'istruttoria passa al diff
  SORGENTE s148tag-vs-s149name (S-152), come già previsto dal ramo ≈335,8M.
- Conferma della staleness (Gregg R5): OGNI cifra census pre-BT1 era stale;
  il tetto movimenti 1,27 s va rifondato dai canali C1/C5 nuovi.

## Reperto 2 — i canali Zval (conteggi per replica, ORM ~35,5 s user sul pin)
C2 borrow/borrow_mut Object **340,9M** · C1 clone/drop handle Object
**254,0M** (di cui Sweep.drop 43,8M + Ret.drop 36,6M + frame_teardown.clone
27,0M in testa) · C5 clone/drop VALORI in PropGet/PropSet **191,2M** ·
C4 gc_note(Object) **43,2M** (frame_teardown 35,2M = 81%) · C3 alloc-oggetto
**6,4M** (alloc_instance 6,27M). Somma canali ~836M eventi.
- **Il sito dominante è la macchineria di TEARDOWN/SWEEP** (frame_teardown.
  borrow 61,0M · PropSetPop.borrow 57,4M · Sweep.borrow 50,0M), non
  l'accesso a proprietà in sé: la chirurgia A3 va prezzata su questi siti.
- Handle-clones da conservazione: 188,9M cloni contro 3,27M nascite (58:1) —
  il traffico è movimento di handle, non creazione.
- GO/NO-GO A3c: DECIDIBILE solo con le sonde-prezzo (S-152); ordine di
  grandezza: a 1–3 ns/evento la banda lorda è ~0,8–2,5 s su D_gap 30,5 s
  (3–8%) — la soglia S3 (≥4,58 s) NON è scontata: le sonde decidono.

## Reperto 3 — N2/N6 (Leijen)
props/istanza: p50=1 · p90=6 · p99=18 · ≤4=86,5% · ≤8=92,0% · dyn 245
oggetti/485 entries (trascurabile) ⇒ inline-8 ben fondato, inline-4 copre
86,5%. N6: live_end=180.126 su 3,27M nascite (5,5%): la stragrande
maggioranza degli oggetti muore dentro la richiesta.

## Teste di pesca per NOME (S-152, mandato BT1-pesca del concilio)
debug_backtrace ANCORA #1 21,3M (perché 21M eventi con limit=2? istruttoria)
· class_exists 9,7M · array_map 7,7M · get_declared_classes 4,6M ·
__reflect_* ≈14M (method_info 3,4M + class_real_name 3,3M + prop_attr_new
2,2M + prop_details 2,1M + class_loc 1,4M + method_names 1,2M) ·
array_diff 3,2M.

## Restano dovuti (A1, dichiarato — S-152)
Sonde-prezzo per canale + mock sostitutivo store-indicizzato + braccio
mi_heap (Leijen R2) + gamba SERVER (Pedersen R4, slittamento dichiarato
§8 del criterio) + istruttoria +3,2% a diff sorgente.
