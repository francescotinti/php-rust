# Verbale Sedia 4 — Hejlsberg (pipeline di compilazione) — Concilio WP-100

**Oggetto**: report S-98.0 (H-B2 spedita) + programma S-99.0. **Mandato**: refutare.

## VERDETTO
La spedizione H-B2 è coerente sul suo asse (nessun punto in cui un'unità compilata in un modo serve l'altro: `UnitKey.reg_mode` da `enabled()` a vm/mod.rs:16058, `main_chain_fp(reg_mode)` belt-and-braces, `enabled()` OnceLock per-processo condiviso da emissione e pass). Ma il funnel dichiarato «unico» ha DUE gambe già divergenti e il programma S-99.0 porta un criterio mal derivato e un pass con un tombino aperto. **CON EMENDAMENTI VINCOLANTI; refutazioni capitali: sì (3).**

## Refutazioni capitali
**RC-1 — Il contratto del dump è VIOLATO oggi, flag-on.** `compile_hook` (func.rs:325) passa da `compile_body` ⇒ i corpi hook **vengono riscritti** dal pass; `dump_module_ops` li esclude con la clausola «a stage that rewrites those needs to widen this first» — la condizione è violata dallo stage corrente. Corpi riscritti in produzione, invisibili al dump E al battery in-process.

**RC-2 — Il helper `lowered()` NON rispecchia il funnel che dichiara.** Abbassa `prop_init`, ma `compile_prop_init` (func.rs:361) costruisce `Func` a mano SENZA il gate `enabled()` ⇒ in produzione prop_init non è mai lowered. Il battery testa una pipeline che la produzione non esegue (prop_init lowered) e non testa una che esegue (hook lowered). È la «doppia fonte di verità» a mano: ha già derivato in entrambe le direzioni.

**RC-3 — Il criterio del rollout S-99 punto 3 è derivato dal controfattuale SBAGLIATO.** D=6,07 ns/occ è il risparmio sul percorso PILA (call + marshalling 2×Zval + pop/push + Dup elisi). BinarySS/SC/Dst leggono slot e scrivono dst: il pop/push è GIÀ eliso; il rimovibile lì è solo call+match, strettamente minore. Un criterio «derivato da D=6,07» sovrastima e mis-calibra la caduta: va ri-registrato dal controfattuale per-forma.

## Punti del perimetro
**(b) `bin_op_of` (reg_lower.rs:189)**: non è lettera morta — il battery in-process compila flag-off (emette `BinaryAdd`) e applica il pass a mano: quel braccio È il suo unico esercizio. Ma legittima una pipeline mista che la produzione vieta, e **neutralizza un tripwire**: un leakage cross-mode che facesse comparire `BinaryAdd` sotto flag-on verrebbe riscritto in silenzio invece di fallire. Il controllo di sessione (census flag-on ZERO BinaryAdd) era one-shot, non un test permanente. L'equivalenza «BinaryAdd IS Binary(Add) by construction» vive in un commento, non in un test differenziale.

**(c) Siti d'emissione**: tutti e 5 i siti che producono `Op::Binary` passano da `emit_binary` (expr.rs:102,159; assign.rs:960,976,988). Unica emissione diretta: `Binary(NotEq)` (expr.rs:217) — sana, ma la porta non è recintata. Per NOME, portatori di Add FUORI da Op::Binary (funnel generico, dichiarati non mancanti): `FieldAssignOp{op}` (assign.rs:968), `AssignOpPath{op}` (assign.rs:1001); IncDec e coalesce non trasportano BinOp.

**(d) Shape nuove**: Add int-int nelle forme registro non porta `Addr` ⇒ remap e `visit_addrs` reggono senza riscrittura; il pin census «BinaryAdd chiude la tabella» morde consapevolmente. MA `visit_addrs` ha `_ => {}` (reg_lower.rs:74): l'«unica autorità» si difende con un commento, contro la dottrina WP-96 (variante nuova = NON COMPILA). La coda AssignOp in backlog può portare shape con Addr: il tombino va chiuso PRIMA.

## Emendamenti
- **A-HE-100-1**: assert permanente in `reg_lower_funnel.rs`: dump flag-on del `{main}` con ZERO `BinaryAdd` (ripristina il tripwire tolto da bin_op_of).
- **A-HE-100-2**: `visit_addrs` esaustivo (niente `_ => {}`), o dente census-driven che ogni variante Addr-bearing è visitata.
- **A-HE-100-3**: test differenziale permanente `BinaryAdd ≡ Binary(Add)` (output+diags: overflow, coercizioni, fallback MISS) — oggi è un commento.
- **A-HE-100-4**: sanare RC-1/RC-2: allineare dump + `lowered()` al funnel VERO (hook dentro, prop_init o gated o dichiarato fuori), con un test che enumeri i corpi dal Module, non a mano.

## Kill-switch
- **KS-HE-100-1**: promozione flag-on a default VIETATA finché `enabled()` resta lazy senza dente anti-putenv/eager-init — è gate di promozione, non residuo del punto 4.
- **KS-HE-100-2**: nessuna shape registro nuova finché `visit_addrs` non è esaustivo (A-HE-100-2).
- **KS-HE-100-3**: rollout punto 3 VOID se il criterio resta derivato da D=6,07 anziché dal controfattuale ricalcolato sulle forme registro (RC-3).
