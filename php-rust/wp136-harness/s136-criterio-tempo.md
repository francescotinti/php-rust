# Criterio S-136 p.3a — modello del TEMPO di FieldAssign (ISTRUTTORIA, metodo s135-tempo) — commit PRIMA del run

1. Oggetto: budget del dim-write su proprietà (`$e->data['k']=$i` →
   `FieldAssign{[Prop,Index]}`, ~240 ns/iter per sottrazione stesso-binario,
   2 resolve/iter dalla sonda S-135) in segmenti NOMINATI sul sorgente,
   unilaterale phpr (indizio, mai cifra comparativa — REGOLE §4).
2. Strumento: build EMENDATA nel worktree al pin s135 (commit 88195af, MAI
   pinnabile), modulo `s136tp.rs` (= s135tp, 2 hunk dichiarati): UN segmento
   per run (`PHPR_TP`), ns via Instant, dump atexit. Calibrazione seg 9 =
   span vuoto nell'arm, sottratto. Giudice: `micro-bisez/m-dimwrite.php`,
   R=3 per segmento, mediana; parità stdout vs oracle a ogni run.
3. Segmenti: 0 arm FieldAssign intero · 1 pop value+pop_field_keys ·
   2 field_prelude_skip · 3 field_set intero (ramo F4) · 4 field_write
   intero (dentro field_set_mode) · 5 field_write_prop_step (call-site
   IntoObj) · 6 resolve_prop_access E1-KO (dentro prop_step) · 7 blocco
   guardia (child-fetch slot/nome + prop_slot_state + prop_indirect_guard) ·
   8 discesa leaf `field_write_walk(child,…)` (= passo Index: ensure/
   make_mut/coerce/set) · 9 vuoto. Derivate: dispatch+push = 0−1−2−3 ·
   plumbing field_set = 3−4 · prop_step-altro = 5−6−7−8 (borrow, key0,
   branch). Chiusura = (1+2+3)/0 (≥90% o modello INCOMPLETO a verbale).
4. Lettura pre-registrata: la leva «IC su FieldAssign + riuso leaf» ha come
   canali rimossi al hit: seg6 (resolve E1-KO) + seg7 (guardia, resolve
   interna inclusa) + parte di (5−6−7−8) e del plumbing 3−4; l'UB
   falsificabile della leva = SOMMA dei prezzi misurati dei soli canali
   che il fast path bypassa + banda giudice (criterio leva separato,
   scritto DOPO questo modello coi numeri dentro).
