# s138-criterio-sonda-v2 — arm-only (PRE-REGISTRATO: commit PRIMA del run; segue il verdetto disasm s138-disasm-verdetto.out)

1. Oggetto: chiudere l'identità dell'eccedenza FD1 SENZA ripartizione (az.rev.
   S-137 #3): arm FieldAssign su m-dimwrite col probe ARM-ONLY deve valere
   ≈ 118,2 − 83,3 = **34,9 ± 13,3** (banda spread-batch v1) → identità CHIUSA,
   blocco dim-write RIMOSSO; fuori banda → NON CHIUSA, blocco PERSISTE (⇒ p.4
   NEXT_SESSION: leva non-dim-write). La v2 mette alla prova l'IPOTESI
   SOSTITUTIVA del disasm (artefatto-inlining REFUTATO): l'arm v1 56,7 era
   gonfiato dalla DENSITÀ di call-site timer inattivi nello span (~14 siti);
   nella v2 lo span seg0 contiene SOLO la coppia t9 inattiva (2 check).
2. Strumento: patch v2 SOLO run.rs + mod.rs — seg0 ai bordi dell'arm, seg9
   calibrazione (span vuoto); call-site di `field_assign_fast` INTATTO; nessun
   sotto-segmento; hit/miss NON compilati nel call-site (regime hit provato in
   v1: 2.999.999/1 sullo stesso sorgente s136 — dichiarato, non rimisurato).
   Build emendata nel worktree, hash dichiarato, MAI pinnabile. La patch si
   GENERA con `git diff` dal worktree che compila (niente sed di copia).
3. Gate di validità PRIMA del tempo (tutti, o sonda NON interpretabile):
   (a) disasm v2: `field_assign_fast` ASSENTE da nm (inline come pin) e delta
   bl-count run_loop vs pin a verbale; (b) parità fixtures-fd1 probe==pin;
   (c) parità stdout vs oracle a ogni run; (d) gate inerzia (az.rev. S-137 #5):
   probe v2 con PHPR_TP unset vs pin su m-dimwrite, R=3 user, delta entro il
   rumore (max 4 ns/iter-equivalente o drop-1); fallito ⇒ inerzia NON regge,
   dichiarare.
4. Misura: R=3 per seg ∈ {0, 9}, mediana ns/span, overhead seg9 SOTTRATTO;
   nessuna ripartizione (solo arm). Verdetto in `s138-sonda-v2-verdetto.out`.
5. Ogni confronto per-segmento tra sonde diverse (v2 vs v1 vs s136-tempo)
   porta la banda di non-omogeneità **≥2,0 ns/segmento** (az.rev. S-137 #2,
   fondata su pop+keys 4,5→2,5); l'identità del p.1 NON contrasta segmenti:
   usa solo arm v2 vs 34,9 ± 13,3.
6. Vincoli: lock misura `/private/tmp/phpr-measure.lock` CREATO prima della
   finestra; quiescenza gate SEPARATO; `pgrep rust-analyzer` prima di ogni
   finestra; misure SEQUENZIALI (mai con build in volo); rc autoritativo da
   file, MAI da pipe.
