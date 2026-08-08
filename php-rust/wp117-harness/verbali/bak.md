# Verbale sedia Bak — S-116/117, lente: VM di produzione (dispatch, IC, layout, path caldi)

## VERDETTO: CONCORDO CON EMENDAMENTI

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)
Ordine: **A (ridimensionata a PGO+LTO) → B (regime) → C promossa da riserva a cantiere istruito → D come alimentatore di B e C**.

Il fatto che governa tutto è il costo/op ~9-10 ns INVARIANTE tra categorie: è la firma di una tassa fissa per opcode (dispatch + ciclo di vita del valore), non di operazioni lente. Una VM di produzione non risponde a questa firma con peephole da 3-30 ns: risponde cambiando la rappresentazione dei valori. Zend stessa NON refcounta gli scalari (IS_LONG/IS_DOUBLE viaggiano senza rc); se phpr paga Rc/clone su path scalari, lì sta il 4-7× delle micro contro l'1,87× del full. L-A (fusione trigramma) è esattamente una superinstruction: giusta, ma è la classe di leva che rende 20-30 ns, non 60.

**Aritmetica che smonta "C solo se serve"**: prop lato pin ~107 ns/iter, rapporto 7,6× ⇒ per ≤3× servono ~−65 ns/iter. A al meglio 10-15% (~10-16 ns) + L-A (~27) + 2-3 vagoni (~10-20) ≈ 50-60: al limite teorico, sotto nel realistico. A+B non chiudono prop né arith. C non è riserva: è la destinazione. Va istruita ORA (censimento, non codice).

**Mossa concreta S-117**: build PGO (cargo -Cprofile-generate/use, workload pre-registrato = sei micro + held-out + slice WP; cgu=1 + fat LTO se non già attivi) → gate parità PIENO sul binario PGO (batteria, corpus per NOME ×2) → micro R=5 → **rimisura della banda leva-nulla (patch s115-zavorra2 riapplicata) sul binario PGO**. Il claim "A ripara il metro" si giudica lì, non si racconta.

## EMENDAMENTI
- **R1 — BOLT non esiste sulla nostra piattaforma**: llvm-bolt è ELF/x86-centrico; su Mach-O ARM64 il supporto è assente/sperimentale. La rotta A è PGO+LTO+cgu, non "PGO+BOLT". Misura: verifica tooling in pre-flight; atteso dichiarato 5-10%, non 5-15%.
- **R2 — "ripara il metro" è ipotesi, con rischio inverso**: PGO rende il layout funzione del PROFILO ⇒ ogni leva futura cambia profilo ⇒ possibile instabilità NUOVA. Giudice: banda nulla su binario PGO (N≥2); il claim passa solo se banda_globale scende materialmente (es. ≤5 vs 10). Se non scende, A resta guadagno una-tantum e B diventa obbligatorio.
- **R3 — profilo = artefatto pinnato**: workload PGO pre-registrato, profilo versionato, rigenerazione SOLO via scripts/pin-phpr.sh emendato; altrimenti ogni build è "emendata" con metro diverso.
- **R4 — treno B**: somma-bersaglio pre-registrata ≥ 2× banda globale vigente (oggi ≥20 ns; meno se PGO riduce la banda); ogni vagone conserva ammissione+parità+direzione individuali; le guardie non-bersaglio si giudicano SUL TRENO INTERO (la tassa layout per-vagone non è additiva — i 5 campioni calls lo mostrano).
- **R5 — C fuori dalla riserva**: istruttoria entro S-119, timebox ½ sessione: census del traffico refcount per iterazione (inc/dec per op, per categoria) phpr vs Zend. Decide la variante (scalari non contati / deferred RC / arena) coi numeri, prima di aprire il cantiere.
- **R6 — D dichiara le dipendenze**: le tecniche Zend (run-time cache/IC, interned strings) presuppongono la SUA rappresentazione valori; ogni porting D dichiara se dipende dalla repr — se sì, è un vagone di C, non di B.

## KILL-SWITCH pre-registrati
- **KS-A1**: PGO < +3% mediano sulle sei micro E banda nulla invariata ⇒ A chiusa in 1 sessione, treno B su banda attuale.
- **KS-A2**: gate parità fallisce su binario PGO e il fix costa > ½ sessione ⇒ PGO rinviata, non inseguita.
- **KS-B1**: treno da 3 vagoni sotto la somma-bersaglio ⇒ treno sciolto, vagoni conservati, C rotta primaria.
- **KS-C1**: se l'istruttoria mostra RC-traffic/op ≈ Zend (collo NON è refcount) ⇒ la variante C cambia bersaglio (dispatch/IC), niente cantiere cieco.

## APPARATO minimo
Solo l'emendamento allo script pin per la ricetta PGO (R3) e il contatore RC-traffic per R5 (strumentazione a compile-flag, MAI nel binario di misura).
