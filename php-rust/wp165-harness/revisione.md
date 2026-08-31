# Revisione S-165 — lente SEMANTICA su L-MC1d (revisore singolo adversariale)

## VERDETTO: REGGE CON RILIEVI

Il fast path è riga-per-riga equivalente al funnel sul path felice; il claim di
identità «per ogni input ammesso» NON è sostenuto sull'error-path della
materializzazione, e la fixture non lo tocca.

### Rilievi

1. **Error-path: ordine di drop recv/args INVERTITO rispetto al funnel.**
   Fast path: locali `frame` (run.rs:6618) poi `recv` (run.rs:6632); su `Err` di
   `arg_place_read` (run.rs:6643) o `flush_diags` (run.rs:6650) Rust droppa in
   ordine inverso: **recv, poi frame(args)**. Funnel: su `Err` in
   `materialize_arg_places` (oop.rs:1300) droppano i parametri di `method_call`
   in inverso: **args, poi this**. Il progetto stesso dichiara l'ordine dei
   decrementi Rc «the only PHP-observable effect (handle-id LIFO reuse, GC
   cascades)» (mod.rs:4646-4648). Innesco concreto SENZA magia: callee ammesso +
   `$o->add($a[], 1)` → `Err("Cannot use [] for reading")` (mod.rs:11808-11810)
   dentro il fast path, catchabile.
2. **Su `Err` il frame pooled muore senza `gc_note_frame` né riciclo** — stato
   nuovo, inesistente nel funnel (lì il frame nasce DOPO la materializzazione,
   oop.rs:1298→mod.rs:11569). Parità gc coi drop della Vec del funnel plausibile
   ma non provata; i buffer persi dal pool sono solo perf.
3. **Pista rientranza `__get` (1): REFUTATA** — in entrambi i path recv e args
   sono già fuori dalla pila VM durante `__get` (Vec+param vs slots di frame non
   ancora in `self.frames`); ordine 0..n e flush unico a `cur_line` identici
   (run.rs:6634-6650 ≡ mod.rs:11722 + oop.rs:1298-1304).
4. **Pista decay (3): REFUTATA** — 0..n in entrambi (run.rs:6652-6655 ≡
   calls.rs:364-370, `decay_arg` identico).
5. **Pista IC/Fiber (4): CONFERMATA SOUND** — `fiber_method` intercetta OGNI
   metodo su istanza Fiber, anche nomi utente («Call to undefined method
   Fiber::…», coroutines.rs:406-430) ⇒ nessun fill con cid Fiber-subclass; `ic`
   passato solo da MethodCall/ThisMethodCall; fill solo public senza private
   shadow (mod.rs:11605-11610) ⇒ scope-indipendente anche sotto `Closure::bind`.
6. **`deref=false` (5): codice identico** (run.rs:6620 ≡ mod.rs:11573) **ma non
   coperto**: fx-mc non contiene `$x =& $o->retref(…)`.
7. **fx-mc non copre i casi 1-2**: nessun `__get` in ArgPlace, nessun errore a
   metà materializzazione, nessun `__destruct` da last-ref in decay, nessuna
   Fiber-subclass al sito condiviso.

### Azioni S-166

1. Fixture fx-mc2 (A/oracle/pin): `try/catch` su `$o->add($a[],1)` con
   osservazione handle-id post-catch; ArgPlace via `__get` (anche uno che
   lancia); `$x =& $o->retref(1,2)`; `__destruct` su last-ref di argomento;
   Fiber-subclass allo stesso sito di classi normali.
2. Se la fixture morde: riallineare l'error-path (drop esplicito di `frame`
   prima di `recv`, o materializzare prima del pop del ricevitore).
3. Decidere per il frame pooled su `Err`: riciclo (ordine drop conforme a
   recycle_frame) oppure divergenza documentata a catalogo.
4. Eseguire la coppia WP+ORM dovuta sul pin s165 (regola focus-oggetti).
