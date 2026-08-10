# s127-emenda-corpus.md — emenda DICHIARATA del fail-set congelato: 1415 → 1414 (×2 modi)

Il corpus-gate della promozione L-OL1-F1 ha morso per UN nome MANCANTE dal
fail-set: `Zend/tests/bug69534.phpt` (Cycle leaks through declared properties
on internal classes) ora PASSA. Contenuto == golden (1411, carve-out 3) e
off↔on zero differenze: il flip è l'UNICA variazione.

**Meccanismo verificato bilateralmente** (run diretto, questo commit):
pin s125 → `int(8)` (contava gli array-default per-oggetto del thunk come
membri del ciclo morto) · leva s127 → `int(2)` **== oracle**. La cura è lo
STESSO meccanismo della leva: i default non-costanti condivisi COW restano
vivi nel template (come gli array condivisi immutabili di Zend) invece di
morire col ciclo, e il conteggio di `gc_collect_cycles()` torna quello di Zend.

Vizio di forma a verbale (REGOLE §9: «una cura cita i fail che deve flippare»):
la leva NON aveva dichiarato il flip in anticipo — dichiarato QUI a valle, col
meccanismo provato. Fail-set emendato per NOME: riga `bug69534.phpt` rimossa
da `wp109-harness/corpus-gate/corpus-s109-{off,on}.fails` (1415→1414);
REGOLE §5 aggiornata alla cifra 1414. Corpus-gate RIESEGUITO dopo l'emenda
(esito nel verdetto di promozione).
