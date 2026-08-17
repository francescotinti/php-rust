# S-152 p.3 — touch-map A3 → partizione tranche A2 (chirurgia-first, concilio S-151 §RATIFICA-A2)

Fondazione EMPIRICA: il perimetro dei siti che maneggiano l'handle Object è
GIÀ misurato — è la migrazione ObjRc della copia census S-151
(`wp151-harness/s151-census-copia.diff`, 73 hunk su 10 file; la migrazione ha
chiuso il funnel a livello di TIPO, quindi i suoi hunk SONO i siti che A3
toccherà). memcensus.rs (4 hunk) è strumentazione, fuori A2.

## Touch-map (hunk ObjRc per file + carico census per zona)
| file | hunk | zona calda (census s151, eventi/replica) |
|---|---|---|
| vm/mod.rs (25.704 r, cap dente) | **35** | teardown/sweep/gc: frame_teardown.borrow 61,0M · Sweep.borrow 50,0M · Sweep.drop 43,8M · frame_teardown(gc_note) 35,2M · gc_collect 8,5M — la CHIRURGIA A3 vive qui |
| vm/run.rs (6.786 r) | 9 | siti nel/attorno al run_loop: Ret.drop 36,6M · This.clone 13,8M · LoadVar.clone 12,5M · Dup.clone 6,7M — ULTIMO o MAI, run_loop NON spezzato |
| vm/oop.rs | 8 | PropSetPop.borrow 57,4M · ThisPropGet.borrow 37,2M · FieldIsset 16,7M (canali C2/C5) |
| vm/arrays.rs | 8 | walker Write/Unset/Cell (C5 displaced: FieldIsset.obj 52,8M) |
| php-types/object.rs | 3 | definizione Object/GcMark — sede del tipo handle |
| php-types/zval.rs | 2 | variante `Zval::Object` — il tipo del funnel |
| php-builtins/var.rs | 2 | siti WeakHandle/upgrade |
| php-types/lib.rs | 1 | re-export del tipo handle |
| vm/host.rs (7.626 r) | 1 | UN solo sito handle ⇒ host.rs RESTA backlog non bloccante (conferma concilio: fuori dal perimetro chirurgico) |

## Partizione proposta (~3 sessioni; freddi→caldi; pin per SESSIONE, coppia WP a ogni pin)
- **T1 (fredda — fondazione tipo + foglie)**: php-types object.rs/zval.rs/
  lib.rs (wrapper handle compiler-enforced, modello = migrazione ObjRc S-151
  già collaudata sulla copia) + var.rs + host.rs (1 sito) + arrays.rs
  (walker, fuori loop caldo). ~17 hunk equivalenti, tutti freddi.
- **T2 (media — il cuore di mod.rs)**: oop.rs (8) + le ZONE di mod.rs della
  touch-map (teardown/sweep/gc_collect/destructor/prop-init: la gran parte
  dei 35 hunk). mod.rs NON si spezza per intero (chirurgia-first): si
  estraggono/preparano SOLO le zone che A3 riscriverà; il resto del monolite
  è backlog. Eventuale split in T2a/T2b se il fascio-gate lo chiede.
- **T3 (calda — ultima)**: run.rs, 9 siti attorno al run_loop; run_loop
  NON spezzato (veto exec/ops_*); disasm size/bl con delta DICHIARATO.
- Gate per tranche = FASCIO (sintesi §RATIFICA-A2): batteria rc=0 · corpus
  1412×2 ZERO-FLIP per NOME · fixture bilaterali · micro R=5
  solo-regressione max(4; rumore; banda-layout) · disasm run_loop ·
  **census-eco: conteggi per canale IDENTICI pre/post** · vmmap peak.
- QUESITO UTENTE (aperto, non bloccante per T1): ratifica perimetro ~3
  sessioni vs 4–6; A3.0 sweep-preserving già adottato (dissenso agli atti).

## Nota di sequenza (dal GO/NO-GO S-152)
Se la sonda-prezzo dà NO-GO su A3c, la touch-map resta valida per A3a/A3b
(fette micro-judged sui siti teardown/sweep di mod.rs — T2 diventa la
tranche portante e T3 può DECADERE se il criterio delle fette non tocca
run.rs); la decisione si scrive nel verdetto GO/NO-GO.
