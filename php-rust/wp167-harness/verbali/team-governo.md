# TEAM-GOVERNO (Hoare · Klabnik · Pedersen) — riconciliazione FORMA campagna, S-167 fase 2

## CONVERGENZE (3/3)
GO-CONDIZIONATO, dispatch-first (R1). Prezzatura PRIMA del codice (Hoare E4 = Klabnik fetta-misura = Pedersen «reg-lowering attivo»). Veti NaN-boxing/BOLT/PGO CONFERMATI. R3-arena senza movente (L-AL3) e pericolosa sul confine. Disasm bl-count prima/dopo obbligatorio; soglie mai sotto banda-layout fondata; fn-table respinta (Hoare E1: indiretta opaca a LLVM, icache peggiora — H-C2/MC1).

## CONFLITTI RICONCILIATI
- Fetta 1 codice subito (Hoare/Pedersen) vs sola misura (Klabnik) → **fetta 0 = prezzatura, fetta 1 = fusione**.
- Kill: dispatch residuo <10 ns (Hoare) e aggredibile <21 (Klabnik) sono compatibili (quantità diverse): adottati ENTRAMBI.
- Cadute consecutive: 2 (Hoare/Pedersen) vs 3+concilio (Klabnik) → 2, **dissenso Klabnik registrato**.

## LA FORMA — 8 REGOLE
1. **Safe-only**: dentro = nuove varianti dell'enum `Op` (predecode/fusione/superistruzioni), pre-lowering nel decoder, VmGate intatto. Fuori = tabella di fn, NaN-boxing, arena a lifetime/raw-ptr, qualunque transmute.
2. **Fette promovibili**: ogni fetta termina verde sul pin canonico (batteria+corpus per NOME ×2+coppia+micro R=5) o revert al byte. MAI branch lunghi-viventi; rotta non affettabile in stati verdi = NO-GO di rotta.
3. **Fetta 0 (S-168, sola misura)**: decomposizione arith a registro (chiusura ≥85%), A/B PHPR_REG_LOWER, strumento collaudato con mutante che DEVE mordere.
4. **Budget/kill di campagna** (pre-registrato in `.out` prima della fetta 1): aggredibile dispatch+decode su arith <21 ns/iter, O dispatch residuo <10 ns ⇒ R1-arith morto, si delibera R4. 2 fette di trasformazione consecutive cadute ⇒ vaglio R4.
5. **Gate Pedersen**: parità two-request byte-identica stesso processo (capture prima del reset) su OGNI fetta di campagna; per R2/R3 anche gate handle-id + ordine distruttori.
6. **Disasm bl-count** di run_loop prima/dopo in ogni criterio; flip inliner oltre banda-layout ⇒ revert.
7. **Timebox apparato: mezza sessione, PERMANENTE**; l'altra metà misura. I modelli allocano fette, non promuovono mai.
8. **R3 residua**: solo dati senza Drop osservabile, MAI candidati RetainSet, sigillo di TIPO che nulla d'arena sopravvive a request_end() (**Hoare dissente: deprioritizzare in toto**).

## DISSENSI
Klabnik: kill a 3 cadute + concilio. Hoare: R3 fuori anche nella forma sigillata. Pedersen: nessuno.
