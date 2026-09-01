# Verbale sedia HOARE — S-167 (bozza indipendente, FASE 1)

## VERDETTO: GO-CONDIZIONATO
Campagna strutturale SÌ, ma ristretta a R1 nella sola forma **predecode/fusione dentro l'enum `Op` esistente**. NO esplicito a due sotto-rotte di R1/R2 che dal mio angolo sono trappole.

## ROTTA (una riga)
R1-fusione: superistruzioni safe come NUOVE varianti dello stesso `Op` (cmp+jmp, load+op+store), decodificate in un passo di pre-lowering — VmGate e sigilli di tipo intatti, zero unsafe, zero nuova rappresentazione.

## REFUTAZIONI ED EMENDAMENTI
- **E1 — dispatch a tabella di fn: RESPINTO.** In Rust safe è esprimibile (`fn(&mut Vm,…)` è safe), ma il prezzo è strutturale: la chiamata indiretta è opaca a LLVM (niente inline degli arm, niente propagazione di costanti tra decode ed esecuzione), e per passare gli operandi serve o ri-fare il match (doppio dispatch) o splittare l'Op in (opcode, payload) — una nuova rappresentazione del bytecode. Il match gigante compila già a jump table; le cadute H-C2 (icache) e MC1 (+45 bl ⇒ ±5-8 ns su siti NON toccati) dicono che il collo è layout/icache, che la tabella PEGGIORA. Prezzo atteso: wash o regressione.
- **E2 — NaN-boxing (R2): TRAPPOLA di unsafe strisciante.** Bit-punning di puntatori = transmute; incompatibile con safe-only. Veto da CONFERMARE.
- **E3 — R3 arena: premessa CADUTA.** L-AL3 (alloc mimalloc ≈0 ns) toglie il movente; la variante a lifetime infetta `Vm<'a>` (refactor virale), quella a indici aggiunge un'indirezione, quella a raw ptr è unsafe strisciante. Deprioritizzare.
- **E4 — «dispatch 36,3» non è una cifra sola** (7,0 in FieldAssign): PRIMA fetta deve rimisurare E−E2 con PHPR_REG_LOWER attivo (criterio-prima), o la campagna insegue un numero stantio.

## KILL-SWITCH PRE-REGISTRABILI
1. Se la decomposizione fresca dà dispatch residuo <10 ns su arith ⇒ campagna R1 CHIUSA (canale refutato).
2. Disasm bl-count prima/dopo (protocollo S-104): se la fusione flippa l'inliner su siti non toccati oltre la banda-layout ⇒ revert al byte.
3. Corpus 1412×2 per NOME + batteria: un solo fail nominato ⇒ revert.
4. Due fette fuse consecutive sotto soglia ⇒ si passa a R4 (ridefinizione onesta).

## PRIMA FETTA
Fusione `load+op+store` sul pattern del driver arith (`$s += $i*3 − ($i>>2)`), pre-lowering nel decoder. Giudice: micro arith R=5, baseline 46,8 ns/iter. Soglia pre-registrata: D ≤ −4,5 ns (~−10%) e |Δ| > banda-layout misurata con null-lever sulla build candidata; gemello obbligatorio.
