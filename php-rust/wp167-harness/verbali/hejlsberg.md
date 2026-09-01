# Verbale HEJLSBERG — S-167 (bozza indipendente, FASE 1)

**VERDETTO: GO-CONDIZIONATO.**
**ROTTA (una riga):** campagna sì, ma R1-lowering è già SPESO su arith — si parte dal costo DENTRO il super-handler (operandi/Zval/frame), col predecode come unica fetta compilatore residua, misurata a mock prima di ogni chirurgia.

## Refutazione centrale: il conto delle op
Su `$s += $i*3 − ($i>>2)` phpr flag-on emette **UNA op per l'intero statement** (`BinarySCSCDst`, finestra W10 S-108, `crates/php-runtime/src/compile/reg_lower.rs:479-511`) più ~2 di controllo loop (CmpJmpSC, IncDecSlot*) ≈ **3 dispatch/iter contro ~6-7 handler Zend** (MUL, SR, SUB, ASSIGN_OP, PRE_INC, IS_SMALLER/JMPNZ). phpr dispatcha la METÀ di Zend ed è 5,4× più lento: ~15,6 ns/handler contro ~1,3. Il divario arith NON è dispatch del run_loop né fusione mancante: vive nel corpo del handler (`vm/run.rs:1959-2005`):
(a) 2× `consts[..].to_zval()` PER ITER — rematerializzazione const che Zend non paga (literal già zval nell'oparray): è il PREDECODE mancante;
(b) 4× match `BinOp` a runtime — l'operatore è DATO, non codice: micro-dispatch secondario dentro la superistruzione (Zend lo cuoce nel handler SPEC);
(c) ~6-8 accessi `frames[top].slots[i]` bounds-checked a doppia indirezione;
(d) guardie Undef/Ref, `read_slot` clone, check typed_refs + gc_note in `reg_store_slot`.
Tutte voci runtime/rappresentazione (R2/R3), non lowering.

## Emendamenti
1. Il quesito «quanto del dispatch 36,3 sopravvive col reg-lowering ON» NON blocca il GO: i modelli S-129/131/136 sono POSTERIORI al flip S-100 (flag-on default) — verifica d'ARCHIVIO (harness S-129) che il modo fosse flag-on; se sì, 36,3 È già il residuo flag-on. Su prop le finestre quasi non mordono (solo PropGetSlot, 2-op): lì dispatch threaded/predecode ha ancora bersaglio.
2. Predecode = fetta a sé, PRIMA ed economica: (i) consts pre-materializzati `Vec<Zval>` per Func; (ii) le 2-3 combinazioni BinOp top di census cotte in varianti monomorfe. Stima onesta ≤5-8 ns/iter: si misura, non si promette.
3. VIETATO aprire nuove finestre di fusione su arith come fetta di campagna: il margine op-count è esaurito; ogni proposta R1 deve nominare quale costo (a)-(d) rimuove.
4. La PRIMA FETTA colma l'apertura storica: il dossier stesso dichiara «arith: NESSUNA decomposizione dedicata a registro».

## PRIMA FETTA (giudice e soglia)
Decomposizione arith stile E−E2 sul handler fuso + 3 mock firmati: consts-Zval predecodificati, BinOp cotto, hoist di `frames[top]` su locale. Giudice: micro arith R=5 sul pin s166; soglia: nominare ≥70% dei 38,2 ns di divario (46,8−8,6); predecode promosso solo se D≥5 ns fuori banda pre-registrata.

## KILL-SWITCH
Se i tre mock sommano <10 ns nominati ⇒ R1/predecode dichiarato ESAURITO su arith; la campagna rialloca a R2/R3 (Zval/frame) senza ulteriori sessioni lato compilatore.
