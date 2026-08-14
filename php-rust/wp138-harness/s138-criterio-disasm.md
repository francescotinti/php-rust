# s138-criterio-disasm — az.rev. S-137 #1 (PRE-REGISTRATO, prima del disasm del probe)

1. Oggetto: promuovere o refutare l'IPOTESI S-137 «artefatto-inlining»: i timer
   al call-site (`s137_t5` attorno alla chiamata) rompono l'inlining di
   `field_assign_fast` nel probe 8dc582d9, e l'arm probe (56,7) paga un costo
   call/prologo/spill che il pin non paga (arm implicato dall'A/B ≈34,9).
2. Evidenza pin (GIÀ acquisita, 1e14793e): simbolo `field_assign_fast` ASSENTE
   da `nm` → INLINED in `run_loop`; bl-count run_loop = 5983; bl per NOME:
   `field_write_walk` ×2 · `field_write_prop_step` ×1 · `field_assign_fill` ×1.
3. PROMOSSA se nel probe (ricostruito con ricetta S-137: worktree HEAD ec56e93,
   sorgenti crate == pin verificato, patch s137-tp-s136.patch): simbolo
   `field_assign_fast` PRESENTE standalone E ≥1 bl dal run_loop verso di esso.
4. REFUTATA se nel probe il simbolo resta ASSENTE (inline mantenuto coi timer).
5. Caso anomalo (dichiarare): simbolo presente ma 0 bl dal run_loop = copia
   morta → REFUTATA con nota.
6. In ogni esito a verbale: bl-count run_loop del probe + delta per NOME sui
   quattro bersagli del p.2 (contesto per la banda di non-omogeneità ≥2 ns/seg,
   az.rev. S-137 #2) · hash probe dichiarato (se ≠ 8dc582d9: «ricetta identica,
   hash diverso» — build emendata, MAI pinnabile).
7. Nessuna cifra-leva da questo disasm: solo struttura (inline sì/no, bl-count).
