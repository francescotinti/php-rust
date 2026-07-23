# WP_SESSION_36 — archivio storico della sessione WP-36

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-36 (2026-07-22, gated `1b2db38`, 1 commit)** — **leve A+D del
> handoff (direttive Gemini ∩ riprofilo WP-35)**. **(A) `Op::ThisMethodCall
> { method, ic }`**: fusione del bigram This→MethodCall 25,8M, emessa SOLO
> per `$this->m()` **zero-arg** non-nullsafe — ⭐ il bigram ADIACENTE è per
> definizione argc==0, e con argomenti l'errore unbound-`$this` (che oggi
> Op::This solleva PRIMA delle SEND, come Zend INIT_METHOD_CALL) si
> sposterebbe DOPO gli effetti degli argomenti ⇒ la fusione con args è
> bloccata by design. IC-hit INLINE nell'arm (pattern ThisPropGet spinto
> fino in fondo): receiver letto in place dal frame, un solo Rc-bump verso
> frame.this del callee, entra riga-per-riga come il hit di
> dispatch_instance_call; ⭐ saltare gli shunt sul hit è sound (un Object
> non è mai Generator/Closure; ArgPlace-scan vacua a argc 0; una
> sottoclasse **Fiber non può MAI stare nella cella**: method_call devia i
> Fiber prima del fill-site e l'op è l'unico scrittore della sua cella).
> Miss → deref_clone + funnel method_call condiviso (semantica identica per
> costruzione). **(D) memo `is_instance_of`**: `iof_cache:
> RefCell<FxHashMap<(ClassId, ClassId), bool>>` su Vm, ⭐ senza epoch (Vm
> per-run, tabella classi append-only, ancestry immutabile — anche
> l'auto-impl Stringable via __toString è fissa per classe);
> `Vm::instance_of` cabla ~30 siti (shunt Fiber di OGNI instance call,
> InstanceOf*, catch-matching, is_a, iterator checks, serialize/json,
> reflection, session). ⚠️ regex a prefisso sul file aveva riscritto anche
> la chiamata INTERNA del wrapper → ricorsione infinita: dopo un replace
> multiplo, rileggere il wrapper.
> **Esito (onesto): FLAT — micro fused-call dedicato −1,5%; bench36
> completo −0,2% = rumore; instanceof micro ≈ pari (gerarchie corte: il
> walk era già 2-3 deref; il memo rende sul Fiber-shunt e sulle catene
> interfacce profonde); media group 61,4 vs old STESSO GIORNO 61,1 ≈ −0,5%
> (oracle 21,06 ⇒ 2,92×; il 2,84× di WP-35 era giornata favorevole — anche
> l'old oggi misura 61,1); full ~12:05 = run26; footprint 12,1×.**
> ⭐⭐ LEZIONE: il tetto era leggibile PRIMA dal riprofilo —
> dispatch_instance_call 100 + is_instance_of 79 campioni su >3000 totali
> ≈ 3-4% massimo teorico: dimensionare l'aspettativa sul peso del canale
> prima di aprire la sessione. **run27 = run26 = run25 PER NOME** (30.472,
> 0E/2F/86W/73S = minimo teorico); gate22 TUTTO verde; cargo **1626** (+3:
> fused dispatch, unbound-$this + &ret copy/alias, memo instanceof);
> probe_wp36.php byte-id vs oracle E vs old (cdc4c4c).
> ⭐ `vm_stdout` nei test cargo = `Registry::default()`: NIENTE builtins né
> prelude — solo costrutti di linguaggio + Exception/Error engine-level
> (count()/eval()/Stringable/RuntimeException lì falliscono o APPENDONO —
> un eval nel test è rimasto appeso >60s).
> **Riprofilo (`wp36-harness/new-wp36.sample`, finestra GC-heavy)**:
> ⭐ dispatch_instance_call e is_instance_of SPARITI dalla top-of-stack;
> dominano mi_free 681 + mi_theap_collect 475 + drop Zval/Repr/Rc ~430
> (value churn + alloc), poi run_loop 89, gc_note 15 + sweep 12,
> enter_callee 11 + recycle_frame 10, resolve_method_runtime 25 (siti
> polimorfi non-IC-abili). **→ prossime: C call-site spec (sottoinsieme
> sicuro `simple_call` + arity esatta — canale enter_callee/bind_params),
> poi B SSO PhpStr (sessione dedicata, attribuzione WP-26-style prima —
> il canale allocatore la supporta), poi E gc batching.**
