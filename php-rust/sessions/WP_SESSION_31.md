# WP_SESSION_31 — archivio storico della sessione WP-31

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-31 (2026-07-21 notte, gated `8adba4b`+`7ee4bcb`, 2 commit)** — il
> punto 1 del doc Gemini validato, eseguito: **(a) il run_loop matcha su
> `&'m Op` — ZERO clone per istruzione**. `Frame.func` è `&'m Func` (Copy):
> copiata la reference fuori dalla catena di accessi, l'op si slega da
> `self` e il match gira per reference; 171 fix meccanici (puri deref di
> scalari Copy, guidati dalle suggestion del compilatore via script con
> whitelist sulla forma — vedi lezione) + 4 shorthand; ZERO clone aggiunti
> (coercion `&&T→&T`, `&Rc<[u8]>→&[u8]` coprono i siti d'uso); le celle IC
> sono ora raggiunte direttamente nell'op del Func (stessa cella condivisa
> di prima: fill persistono per costruzione). **(b) has_hints precomputato
> su Func** (la scan di param_hints per-chiamata in enter_callee → bool a
> compile time, 6 costruttori). **Esito: microbench call-heavy −29,8%**
> (5,16 vs 7,35s, A/B interleaved 5 coppie vs 06a3c5b, output identici);
> **full-suite master-CPU 15:12→13:02 = −14,3%** (run22 = run21 per nome);
> **media group phpr 80,7→72,4s = −10,3%** (oracle 20,95 → **3,5×**);
> gap full-suite **2,7×→2,3×**. gate22 TUTTO verde; cargo 1592. Riprofilo
> (`wp30-harness/ab-out/new-wp31.sample`): run_loop self 3041→1761; colli
> residui = value churn (memmove 629 + drop/clone Zval ~630 → la mossa
> grossa resta la **value-representation**), gc_note 206 + gc_sweep 155,
> memcmp 263, hashbrown get 189, slot_of 157 (i field-walker, A2.5
> parziale), enter_callee 135 + bind_params 101, mi_malloc/free ~176.

## Lezioni operative della sessione

- ⭐⭐ **L'op-clone per-istruzione era il singolo costo più grosso del
  run_loop** (−30% sul carico call-heavy, −14% full-suite): `Frame.func` è
  `&'m Func` Copy ⇒ `let func = self.frames[top].func; let op =
  &func.ops[ip];` NON borrowa self e il match gira su `&'m Op`. Le lezioni
  WP-29/30 "il dispatch CLONA l'op" sono STORICHE: ora le op sono
  raggiunte per reference (le celle IC Rc restano condivise — a maggior
  ragione, si tocca la cella originale).
- ⭐ **Refactor da centinaia di type-error = script sulle suggestion JSON
  del compilatore** (`--message-format=json`, applicare solo replacement
  con forma in whitelist: `*x`, rimozione di `&`, `x.clone()`): 171/175
  fix automatici in una passata, il resto a mano. MAI regex alla cieca sul
  sorgente.
- ⭐ Le coercion `&&T→&T` e `&Rc<[u8]>→&[u8]` coprono quasi tutti i siti
  d'uso di un match passato a reference: ZERO clone aggiunti — se un
  refactor del genere richiede molti .clone(), qualcosa è storto.
- ⚠️ `git diff` via RTK è riformattato (prefisso 2 spazi, header
  "Changes:"): i grep su `^[+-]` non matchano — usare `^\s+[+-]`.
