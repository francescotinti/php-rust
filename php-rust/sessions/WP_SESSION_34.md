# WP_SESSION_34 — archivio storico della sessione WP-34

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-34 (2026-07-22, gated `61868ce`, 1 commit)** — **T2 dell'arco:
> fusioni bigram-driven** (dal census WP-33: This→PropGet 72M,
> PushConst→CmpJmp 34,5M, Jump→Ret 29M/run; Stringify 60,7M op = 7,6%).
> **(C1) `Op::ThisPropGet { name, ic }`** — `$this->p` rvalue non-nullsafe
> fusa a EMIT-TIME (root-match su ExprKind::PropGet con base This);
> l'IC-hit legge il receiver IN PLACE (zero clone, zero stack round-trip);
> ⭐ il tail di PropGet è ESTRATTO in `prop_get_fallback` (WP-25 fast path,
> lazy, hook, __get, resolve, IC-fill) e CONDIVISO dalle due op —
> semantica identica per costruzione, IC-hit inline in entrambe le arm
> (lezione WP-23 inlining). **(C2) `Op::CmpJmpConst`** — confronto fuso
> con operando LETTERALE inline dal const pool (`lit_const_of`:
> null/bool/int/float/str), cablato in cond_jump + switch(Eq) +
> match(Identical); `binary_value` spaccata in wrapper (pop) +
> `binary_value_ab` (core a operandi espliciti) condivisa da
> Binary/CmpJmp/CmpJmpConst; ⭐ il push di un letterale non ha effetti ⇒
> elidibile anche il lhs "fuori ordine"; ⭐ cur_line stampata sulla linea
> che il PushConst eliso avrebbe portato (parità trace). **(C3)
> `Op::ConcatN(n)` + Stringify elision** — la spina di Concat annidati
> (catene `.` E interpolazione, che il lowering desugara a Concat
> left-nested con seed "") è FLATTENED a emit-time: ordine di
> valutazione/stringify IDENTICO all'emissione pairwise (i Concat
> intermedi e gli Stringify-di-Str erano puri); parti letterali Str
> saltano lo Stringify no-op (PushConst→Stringify 15,3M), parti ""
> spariscono, catena all-literal FOLDATA a una costante; join a UNA
> allocazione (via il realloc left-assoc = bucket memmove); Echo/Print
> saltano lo Stringify su risultato già-Str. ⭐ la coercion è TUTTA in
> Op::Stringify (il Binary Concat riceve sempre Str dal compilatore) ⇒
> l'elisione è esatta per costruzione. **(C4) jump threading in-place**
> (compile_body, SOLO exc_table vuota): Jump→Jump ritargettato al landing
> finale; Jump che atterra su Ret DIVENTA quel Ret — mai rimozione di op
> (WP-32: niente peephole/shift), ⭐ replace-Ret solo a LINEA identica
> (Ret può sollevare TypeError da return-hint e la linea alimenta
> getLine); ⭐ target out-of-range (Addr::MAX su jump MORTI dietro i goto)
> = terminali, mai seguiti (5 test goto rossi prima del guard).
> **Esito: micro esteso (bench34 a-g+h-k) −6,2%** (8,10 vs 8,64s, 5+3
> coppie interleaved vs 2937b7b, rust-analyzer killato); **media
> 66,9→65,1s, rapporto 3,11×** (oracle 20,92); **full-suite ~12:35 ≈
> run24 12:20 (rumore, IO/C-lib dominated), run25 = run24 = run23 PER
> NOME** (30.472, 0E/2F/86W/73S = minimo teorico); footprint 12,0×
> invariato. gate22 TUTTO verde + refold 4 suite col binario post-fold;
> cargo 1619; probe battery wp34-harness (probe_wp34.php) old==new
> byte-id, vs oracle solo 2 drift diag PREESISTENTI.
> **Riprofilo `wp34-harness/new-wp34.sample`** (10s su media): run_loop
> 1511; ⭐⭐ **il canale prop slow-path è il blocco phpr-only #1**:
> resolve_prop_access 346 + prop_get_fallback 237 + slot_of 185 +
> lazy_prop_access 102 + prop_info 93 + magic_applies 93 — le PropIc
> fillano SOLO esiti public e le classi WP piene di protected/private
> non fillano MAI ⇒ **prossima leva (T2.5): IC scope-aware** (cella
> keyed anche sullo scope corrente, fill di risoluzioni private/protected
> con guardia (class_id, scope)); poi memcmp 332, enter_callee 307 +
> bind_params 114 (call-site specialization), drop/clone Zval 305+180,
> memmove 235 (−~48% vs WP-33), gc_note 201, dispatch_instance_call 164
> (This→MethodCall 25,8M non fusa — candidata), recycle_frame 157.
> 📌 Gap preesistente visto: `isset($this)` in metodo STATICO →
> CompileError "unsupported $this property write" (dim_base su
> PlaceBase::This) — pre-esistente, mai colpito dalle suite.
