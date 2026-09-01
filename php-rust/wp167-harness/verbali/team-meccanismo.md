# TEAM-MECCANISMO — verbale riconciliato (S-167, fase 2)
Sedie: Hejlsberg · Matsakis · Stogov — 3× GO-CONDIZIONATO.

## Convergenze (unanimi)
1. Il divario arith vive DENTRO il handler (fetch operandi, match BinOp, bounds, guardie), non nel salto: phpr dispatcha ~3 op/iter contro ~6-7 di Zend ed è comunque 5,4×.
2. Canali morti: borrow ≤1 ns (L-TD1), alloc ≈0 (L-AL3), taglia Zval già 16 B ⇒ veto NaN-boxing CONFERMATO; R3-arena refutata (Drop-order, RetainSet).
3. Protocollo: decomposizione prima della chirurgia; disasm bl-count prima/dopo; slow-path `#[inline(never)]`; pin Zval 16 B kill-switch.

## Furti alle leve promosse
Slot-diretti da H-D/HD2 · pattern IC monomorfo da E1-KO/IC-NP/AP1 · cura outline da MC1 (S-165) · finestre W10 esistenti come sede del predecode.

## Sequenza fette (ognuna promovibile da sola sul pin)
- **F0 — decomposizione arith E−E2 + 3 mock firmati** (consts-predecode, BinOp cotto, hoist frames[top]) + verifica d'archivio che il 36,3 sia residuo flag-on. Giudice: chiusura ≥85%, nominare ≥70% dei 38,2 ns. Prerequisito di TUTTO.
- **F1 — predecode consts** (`Vec<Zval>` pre-materializzato per Func). Indipendente, economica. Giudice: micro arith R=5, D≥5 ns fuori banda.
- **F2 — BinOp 3-indirizzi slot-diretti Long-guarded** (+,−,*,>>), fallback al funnel: unifica Matsakis R1-3A, Hejlsberg (b)(c), Stogov (a). Giudice: D≥8 ns (pavimento 4), str/calls-fn/mc2 in banda-layout, gate pieno. Inutile senza F0 (costo non nominato); compone con F1.
- **F3 — guardie/slow-path outlined** (Undef/Ref, gc_note, SAVE-differito). Dipende da F2 (serve un fast path da proteggere). Giudice: micro arith+prop, bl-count non cresciuto.
- **F4 — cmp+jmp senza zval booleano** (SMART_BRANCH): SOLO se F0 firma costo nel controllo-loop e nomina una voce (a)-(d).
- **F5 — dispatch threaded**: in coda, solo a decomposizione rifirmata (resa attesa 2-3 ns su handler ormai magri).

## Kill-switch di campagna
Mock F0 <10 ns nominati ⇒ R1 esaurito, riallocare a R2/R3 (Hejlsberg). ΔF2 <4 ns ⇒ canale refutato, delibera R4 (Matsakis).

## Dissensi registrati
- **Hejlsberg**: VIETATE nuove finestre di fusione su arith (F4 ammessa solo sotto la sua clausola nomina-il-costo).
- **Matsakis**: niente fusione cucita sul driver (benchmark-gaming) — F2 resta superop GENERICA; contrario a specializzare per-driver.
- **Stogov**: soglia campagna più dura (46,8→≤35, −25% cumulativo dopo F1-F3); tiene F5 in rotta, non esclusa.
