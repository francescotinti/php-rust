# WP_SESSION_53 — Fase 2 CPU aperta e chiusa alla quota vera: ret_shape+RET_DEREF (−40,2M DerefTop) + Sweep elision

> ⚡ **WP-53 (2026-07-25, `db4ae14`+`357915d`)** — **Ob.1 Fase 2.1 (UNA
> modifica): `ret_shape: u8` precomputato su Func (bit RS_HINT/RS_WRAP; il
> Ret shape-0 salta il prologo hint/by-ref su UN load+branch, eliminata la
> `ret_hint.clone()` per-return; i 4 flag Ret-shaping + CLONE_INIT del
> frame letti con UN load del byte packed) + DerefTop NON più emesso dopo
> le method-call a valore: copy REF-4b via RET_DEREF sul frame callee,
> bit `deref` nel payload di MethodCall/Named/Args (assign_ref_call=false).
> Mechanism-check op-census old/new stesso-giorno (media): DerefTop
> 40.233.697 → <1,4M (fuori top-40) = canale rimosso ≥96,5%; Ret 61,74M e
> ThisMethodCall 26.065.994 INVARIATI alla cifra; dispatch totali 713,6M →
> 673,2M = −5,66%. Ob.2 Fase 2.2 Sweep emit-time elision (kind divergenti
> + whitelist container-inerte): dinamica −146k su 53,45M (−0,27%).
> Giudici: ab53 media CPU −0,88% new 6/6 → 2,83× (pari-minimo); peak
> fisico −0,69% (guardia ≤+2%: MIGLIORATO); full run41 738,6s vs
> run41-old (phpr-wp52) 740,2s = −0,2% raw stesso-giorno; fail-set 88
> nomi BYTE-ID a run33 e new==old; 0E/2F/86W/73S. VERBALE: leva piccola
> ma positiva su TUTTI i giudici — il dispatch-overhead di DerefTop/Sweep
> non era un collo in SECONDI; il residuo ~30s vs WP-40 resta nel classify
> walk / altrove.** Parità: corpus 1421 IDENTICO ×2, cargo 1639/0 ×2,
> smoke by-ref 12 scenari BYTE-ID oracle, probe dtor 10 scenari NEW==OLD.

## Ob.1 — design (per riproporre il pattern)

- `Func::ret_shape` calcolato ai SITI DI COSTRUZIONE (ret_hint/by_ref/
  is_generator mai mutati post-costruzione — verificato con search);
  RS_HINT esclude by_ref e generator (condizione effettiva del check),
  RS_WRAP = by_ref && !is_generator. Helper `Func::ret_shape_of`.
- Bit `deref` SOLO sui 3 op emessi da `emit_method_call` (unico emettitore
  condiviso value/ref-context: assign_ref_call lo riusa e mette
  BindRefToChecked dopo — vuole il Ref raw); ThisMethodCall e
  MethodCallDynamic* sono value-context PER COSTRUZIONE (la fusione
  avviene solo nell'arm value; `$x =& $o->$m()` è compile error).
- RET_DEREF settato ai push del frame utente (IC-hit ThisMethodCall,
  IC-hit + resolved di dispatch_instance_call, build_named_frame, __call
  dopo push_magic_call), SEMPRE `deref && by_ref && !is_generator`: un
  generator by-ref parcheggia il frame e il suo Ret interno alimenta
  getReturn — il flag lo corromperebbe.
- Ritorni NATIVI (generator/fiber/closure methods): deref-guard al push.
  CallValue/callable-array: deref=false — nessun DerefTop li seguiva
  (parità con se stessi; la divergenza da Zend lì è preesistente).
- Op resta ≤48B (bit nel padding; assert permanente in `frame_is_slim`).

## Ob.2 — design e verbale onesto

- Elisione a EMIT-TIME in `block_of` (mai rimozione: nessun indirizzo
  trasla, WP-33). Prova 1: kind divergenti (Return/ReturnRef/Break/
  Continue/Goto — LINEARI; i compound NON sono kind-gated: l'exit-jump
  del `while` atterra ESATTAMENTE sullo sweep di coda, che copre i
  release dell'ULTIMA condizione fallita). Prova 2: whitelist
  container-inerte — Jump/JumpIfFalse/JumpIfTrue/IncDecSlot + pusher di
  CLONI (LoadSlot/LoadVar/LoadGlobal/This/Dup/PushConst); Pop (che NOTA)
  solo senza pusher container; **Binary/CmpJmp ESCLUSI: un ==/concat
  loose contro un oggetto guida __toString utente**.
- Census: −146k dispatch dinamici su 53,45M (media). Gli sweep dopo i
  `return` erano già mai-dispatchati (risparmio solo bytecode ritenuto).
  La stima "~47M noop elidibili" della recon presupponeva di elidere
  anche dopo gli ASSEGNAMENTI — ma StoreSlot NOTA il displaced (timing
  dei distruttori): il criterio corretto-per-parità recupera solo la
  coda inerte. Beneficio residuo: bytecode più corto (contribuisce al
  peak −0,69% dell'ab).

## Giudici (stesso-giorno, catena sequenziale orchestrate53.sh)

- **Census op-census** (binari separati phpr-op53 / phpr-op52 da worktree
  4c1cc0d, target phpr-op-target{,-old}): DerefTop 40,23M→<1,4M; Ret e
  MethodCall-famiglia INVARIATI alla cifra; totale −5,66% dispatch.
- **Full**: run41 (new) last-sample 738,6s = 12:19 vs run41-old
  (phpr-wp52) 740,2s = 12:20 → **−0,2% raw stesso-giorno**. L'old di oggi
  (740,2) vs lo STESSO binario ieri su run40 (725,1) = ambiente +2,1% ⇒
  il rapporto di giornata (2,18×) NON sostituisce run40 = 2,14× come
  riferimento; il residuo vs WP-40 2,06× resta ~30s. Fail-set 88 nomi
  BYTE-ID a run33 su entrambe; 30.472 test 0E/2F/86W/73S. RSS telemetrico
  new 2765 vs old 2916 (non-metrica, accounting MADV).
- **Media (ab53, 6 round interleaved + 2 oracle)**: CPU old 59,572 vs new
  59,050 = **−0,88%, new 6/6 stretti** → 59,05/20,90 = **2,83×**
  (pari-minimo storico). Peak fisico old 1639,2 vs new 1627,9MB =
  **−0,69%** → 1627,9/376,1 = 4,33× — guardia ≤+2% rispettata CON MARGINE
  (footprint migliorato, non peggiorato).

## Parità e gate (classe emit+GC, protocollo WP-50)

- corpus **1421 IDENTICO per nome col conteggio ×2** (albero Ob.1
  `db4ae14`, albero Ob.1+Ob.2 `357915d`)
- cargo **1639/0 ×2**; smoke by-ref/magic 12 scenari BYTE-ID con oracle
  8.5.7; probe dtor-timing 10 scenari **NEW==OLD byte-id** (3 divergenze
  dall'oracle preesistenti, già prezzate nei baseline)
- fail-set full BYTE-ID a run33 (88 nomi) su run41 E run41-old
- Binario stashed sul path canonico esterno: `phpr-wp53` (sha256
  `e75c5abb…`); old dell'A/B era `phpr-wp52` (sha256 `57607da3…`,
  verificato = release pre-modifiche al pre-flight)

## ⭐ Lezioni

- ⭐⭐ **Un canale di dispatch quotato in MILIONI non è un canale quotato in
  SECONDI**: −40,4M dispatch (−5,66% del totale!) valgono −0,9% CPU media
  e −0,2% full — l'overhead per-dispatch dei micro-op (DerefTop peek,
  Sweep noop) è ~5-15ns. Le prossime leve CPU vanno quotate in ns/evento
  PRIMA, non solo in conteggi (il mechanism-check resta obbligatorio ma
  non basta a predire i secondi).
- ⭐⭐ **Il criterio di elisione degli sweep è "chi può NOTARE/rilasciare",
  non "chi non alloca"**: gli assegnamenti (il grosso dei 47M noop) non
  sono elidibili — StoreSlot nota il displaced e il timing dei distruttori
  è contratto di parità. Whitelist con Binary/CmpJmp ESCLUSI (loose ==/
  concat vs oggetto → __toString utente).
- ⭐ Un flag di contesto value/ref per-call-site va nel PAYLOAD dell'op
  solo dove l'emettitore è condiviso (emit_method_call); gli op emessi da
  un solo contesto lo hardcodano nel handler — zero byte, zero read.
- ⭐ Worktree git per build old: aprire alla ROOT della repo
  (php-rust-experiment — il Cargo.toml sta in php-rust/) e copiare a mano
  php-server/ + Cargo.lock (crate non tracciato; recidiva WP-20).
- ⭐ Epoch/parity: il flag RET_DEREF su un frame che diventa GENERATOR
  sopravvive al parcheggio e corrompe il Ret interno (getReturn) —
  qualunque flag di frame settato al call-site va escluso per i callee
  is_generator.

## Prossimo (WP-54)

1. **Residuo ~30s vs WP-40 2,06×**: la Fase 2.1/2.2 l'ha intaccato poco —
   le lenti restanti: profilo del walk classify (borrow/iter/strong_count,
   ripiego dichiarato in WP-52) e Fase 2.3 args-Vec pool bounded (quotare
   PRIMA in ns/alloc col census alloc-rate, lezione di questa sessione).
2. **reflect-cache owner**: il descrittore di ho_reflect_method_info è
   funzione PURA di (declaring class, metodo) — memoizzare su (decl,
   mname) DOPO la resolve collasserebbe i duplicati ereditati; quotare il
   split mock-declared vs inherited col census reflect. Ritorno cap
   16384→8192 = decisione utente a dati pronti (~42MB standing sul media).
3. Footprint: hashed-array (Fase 3) e interning stringhe (Fase 1.5) — con
   la lezione ns/evento anche qui.
4. Laravel resta posticipata a valle della roadmap.
