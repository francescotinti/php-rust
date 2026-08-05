# Verbale Sedia 1 — Tony Hoare (design linguaggio/runtime, safe-only) — Concilio WP-102

**VERDETTO: CONCORDO CON EMENDAMENTI** (una refutazione capitale sul contratto di modo; il flip in sé regge sui suoi gate).

## Refutazioni

**R1 (CAPITALE) — «il modo è un INPUT del funnel» è FALSO in un punto del compilatore.**
`compile/mod.rs:780`, `emit_binary`: `if op == BinOp::Add && !reg_lower::enabled()` consulta il globale di processo, non `ctx.reg_lower`. Conseguenza post-flip: il braccio OFF della batteria (`compile_program_with_mode(false)` in un processo sigillato ON, il default nuovo) emette `Binary(Add)` dove la produzione OFF (`PHPR_REG_LOWER=0`) emette `BinaryAdd`. Il braccio OFF collaudato in-process NON è l'emissione OFF di produzione: due fonti di verità, esattamente la classe che S-100 dichiara eliminata («`lowered()` eliminato, niente premesse ambientali»). La copertura semantica del differenziale non sana la falsità della dichiarazione. Nota aggravante: il doc-comment di `emit_binary` («its windows match `Op::Binary(Add)` and must keep fusing it») è STALE — `bin_op_of` vede attraverso `BinaryAdd` dichiaratamente («the windows fuse both spellings»), quindi la giustificazione del sito non esiste più.

**R2 — A-HO-101-1 (sigillo di tipo) declassato a backlog DOPO un flip che colpisce tutti.** `seal_reg_lower_mode()` è una convenzione: nessun tipo obbliga un main (o un terzo binario futuro) a chiamarla. Prima del flip il processo smemorato cadeva su OFF-collaudato; oggi cade su ON con modo deciso dalla prima compile — lo stato di richiesta (putenv) promosso a configurazione di motore è di nuovo raggiungibile per omissione. Un dente di batteria non compila al posto del chiamante mancante.

**R3 — H-C1 «prestito» sottospecificato sul lato aliasing.** Il profilo stesso dice che l'oracle fa «borrow inline NEGLI HANDLER»: il prestito Zend non sopravvive mai all'handler. Un «prestito» che risiede sullo stack VM attraverso più op è un design DIVERSO e in safe-Rust esige refcount+COW, non un borrow: `$o->x` letto, poi la proprietà riassegnata/unset/`__set`-tata PRIMA del consumo nello stesso statement invalida lo storage. La lista fixture della bozza (hook, `__get`, ref, readonly, visibilità) NON contiene la classe aliasing (scrittura tra lettura e consumo; mutazione via alias visibile attraverso la condivisione).

**R4 — «equivalenza già provata» per BinaryAdd è per TEST, non per costruzione.** La riscrittura post-finestre `Binary(Add)→BinaryAdd` in `lower_func` copre l'intero stream (anche regioni exc); la sua correttezza pende dal differenziale A-HE-100-3, che è una batteria enumerata. Se i due handler VM non condividono lo stesso corpo, ogni edit futuro a UNO dei due riapre l'equivalenza in silenzio.

## Emendamenti

- **A-HO-102-1**: `emit_binary` prende il modo da `ctx.reg_lower` (o, meglio: emette `BinaryAdd` INCONDIZIONATAMENTE — le finestre fondono entrambe le grafie per dichiarazione propria — e l'`enabled()` residuo nel compile-side scompare del tutto, tripwire invariato). Doc-comment sanato nello stesso commit. Dente: il braccio OFF del funnel dump-hash-uguale alla produzione OFF (`env =0`, processo separato).
- **A-HO-102-2**: A-HO-101-1 promosso da backlog a S-101: testimone ZST (classe VmGate) reso da `seal_reg_lower_mode()` e preteso dal confine di compilazione di produzione — l'omissione del sigillo NON COMPILA.
- **A-HO-102-3**: l'iscrizione di H-C1 dichiari PRIMA quale dei due design è (a) refcount+COW con censimento dei siti di scrittura come prerequisito, oppure (b) fusione PropGet+consumatore in un handler dove il prestito non esce dall'handler (la forma dell'oracle). Fixture aliasing obbligatorie (scrittura/unset/riassegnazione tra lettura e consumo) accanto a quelle già in bozza.
- **A-HO-102-4**: esibire (o creare) il corpo condiviso fra i handler `Binary(Add)` e `BinaryAdd`; il differenziale degrada a cintura di regressione, non a prova.

## Kill-switch

- **KS-HO-102-1**: nessun nuovo lavoro d'emissione (H-C1 compresa) si iscrive finché l'`enabled()` di `emit_binary` non è rimosso o derivato da `ctx`: un funnel col braccio OFF ibrido non può fare da giudice a un cambio d'emissione.
- **KS-HO-102-2**: se H-C1 sceglie il design (a), nessuna riga senza il censimento COW dei siti di scrittura proprietà pubblicato per NOME; se sceglie (b), nessun prestito può attraversare un confine di op sullo stack VM — pena rigetto.
