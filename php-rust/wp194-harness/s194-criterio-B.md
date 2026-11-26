# S-194 — pacchetto B (costo per op del ciclo dell'interprete): criterio

Generato con assistenza AI (Claude Fable 5.2). Scritto il 26/11 02:20, PRIMA della misura.

## 0. Da dove viene
- Registro per op dell'ORM sul commit 9b9b5f11 (`s194-registro-op-C4d.out`; l'aggancio del registro aggiunge ~78 istruzioni per op, tarato sulla sonda `assign_const`: 722,8 attribuite contro 331 vere per 5 op). Minimi netti per op: `Jump` 38 (= il solo dispatch), `CheckArity` 42, `JumpIfFalse` 52, `Pop` 59, `PushConst`/`This` 67, `LoadSlot` 69, `Dup` 70, `StoreSlot` 82, **`LoadVar` 92** (63,0 M esecuzioni: l'op più eseguita dopo `Sweep`).
- `LoadVar` costa 23 istruzioni più di `LoadSlot`, che fa lo stesso lavoro: guarda lo slot due volte (prima per `Undef`, poi per leggere) e passa da `read_slot` + `Zval::clone` fuori linea anche per un intero.
- `Sweep`: 64,0 M esecuzioni, minimo netto 35, **media netta ~200**: il costo sta nei giri NON a vuoto (`gc_sweep_body`), non nel dispatch. La pulizia di fine istruzione a vuoto è già stata lavorata in S-174/176; il corpo del raccoglitore tocca l'ordine dei distruttori (osservabile) e NON si cura in questo pacchetto: resta apertura per nome, con questa cifra.

## 1. B1 — cura
`Op::LoadVar` guarda lo slot una volta sola: un intero si copia in linea, `Undef` chiama l'avviso fuori linea (`warn_undefined_var`, `#[cold]`), ogni altro valore passa da `read_slot` come prima. Il braccio perde 2 righe.

## 2. Attese
- Riga dentro `run_loop`: si giudica sul CARICO INTERO (regola di C p.24). ORM ≤ −0,3 % contro il braccio N1b, WordPress mai sopra.
- Sonda: nessuna fixture oltre +1,0 % rispetto a N1b; `assign_const`/`add_assign` e i cicli con variabili ≤ −2 %.
- Comportamento: nessun cambio (stesso avviso, stesso ordine); copre la batteria, più i denti di C4/N1.
- Se il carico intero non scende: il passo si RITIRA (nessun commit), e si registra.

## 3. Letture rapide (`out/prova.sh`: braccio + una gamba ORM, senza accettazione; direzione)
- **B1** contro N1b: ORM **−0,49 %** (219,956 → 218,871 G). Attesa raggiunta.
- **B2 (scritto prima della lettura)**: `clone_hot` (copia in linea di intero / oggetto / stringa / array al posto di `Zval::clone` fuori linea) in `LoadVar`, `This`, `Dup`; `drop_scalar` (uno scalare a fine vita non passa né dalla nota GC né dal drop fuori linea) in `Pop` e `StoreSlot`; file nuovo `vm/hot`. Lettura contro B1: ORM **−0,11 %** (218,705 → 218,473 G), al limite del rumore fra due invocazioni (~0,08 %); cicli −0,7 %. Si tiene solo se il carico intero della catena non sale.
- B1 e B2 vanno in catena insieme a N2 (`s194-criterio-N.md` p.4), tag `N2`.

