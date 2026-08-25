# S-159 p.2 — PIANO sonda surplus m-refl (az.rev. S-158 #1; orologio §4: entro S-160)
# NB: questo è il PIANO; il criterio ≤10 righe si pre-registra PRIMA della misura.

Oggetto: D=+29,0 (finestra) / +21,0 (post-pin) vs UB-alloc 13,8 = 2 alloc/iter
× miheap 6,9 — surplus non-alloc ~metà del delta, MAI ripartito (registro
[21;29], §4). La sonda deve (i) CONTARE, (ii) TARARE, (iii) RIMISURARE.

1. CONTEGGI sui due bracci (build census): tree s158 = tree s157 + patch
   L-RF2 (wp158-harness/s158-refl2-edit.patch agli atti) ⇒ probe-A = census
   build del tree con patch INVERSA (o checkout pre-RF2), probe-B = census
   build del tree corrente; prep secondo l'apparato census-prep (probe
   hash-pinnato + smoke a esiti esatti, lezione forgia-silenziosa). Attesa
   pre-registrata: Δ alloc/iter su m-refl = 2 ESATTO (method_info cache-hit +
   class_real_name, 1 vec![] ciascuno); ogni altro Δ per-NOME = 0.
2. TARATURA coefficiente (lezione S-158: 6,9 sottostima il cammino Vec
   alloc+free+doppio match): coefficiente_RF2 = D_rimisurato / Δalloc_contato;
   companion: scomposizione con sonda monobinaria kill-switch (stile BT2
   S-154) SE il conteggio non torna. Il coefficiente TARATO diventa l'UB
   falsificabile della leva p.3 (stessa classe di cammino: vec![arg] plumbing).
3. RIMISURA m-refl su finestra NUOVA (rerun az.rev. #1): A/B R=5 bracci
   stash phpr-s158-gemelloA (369ee345) vs pin s158 (92b0aea3), stesso giudice
   m-refl N=10M, per risolvere il registro [21;29] (drift-tree quantificato
   dalla finestra nuova o dichiarato irrisolto).
4. Sequenza: DOPO la coppia p.1 (run pesanti sequenziali, lock di sessione);
   build dei probe = churn dichiarato fuori-finestra; niente edit coi build
   in volo.
