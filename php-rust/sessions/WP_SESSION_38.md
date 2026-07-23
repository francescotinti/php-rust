# WP_SESSION_38 — archivio storico della sessione WP-38

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-38 (2026-07-22, gated `b90f12c`+`e52a8ad`, 5 commit)** — **sessione
> B: SSO PhpStr PROVATO E BOCCIATO; salvati i costruttori zero-round-trip.**
> **Verità di layout** (misurata con probe rustc): il "PhpStr resta 24B" di
> WP-37 è IMPOSSIBILE in safe Rust — il discriminante non può sovrapporsi al
> fat pointer del `Box<[u8]>` (niche-filling: l'altra variante deve evitare i
> byte del niche). cap15 ⇒ Repr 24B / PhpStr **32B**; cap22 gratis (Repr resta
> 24B tagged); cap7 ⇒ **16B/24B via niche** (= size pre-SSO); union unsafe
> fuori roadmap (WP-33).
> **Cronologia**: round 1 (SSO puro, funnel `new`): probe new≡old BYTE-ID ma
> micro string +5,2% — diagnosi con micro SOLA-LETTURA: read-tax ≈ 0 ⇒ tassa
> nei costruttori Vec-fed (il Vec del chiamante si alloca comunque, l'SSO
> aggiunge copia+free). Round 2 (costruttori zero-Vec): micro ribaltato a
> −0,5%… **ma media reale +2,5/2,8%** (5 run new 61,1-61,6 vs 5 old
> 59,6-60,5, interleaved stesso giorno). Bisezione **cap7 (24B): ancora
> +1,5%** ⇒ ~1% dal 32B (pressione cache clone/drop/GC), il resto copie e
> branch diffusi. Sample whole-run: mi_free −20% e memmove −15% (l'SSO
> "lavora"), maxrss −3% — ma il CPU non torna. **REVERT: 59,75 = old 59,87.**
> ⭐⭐ LEZIONE 1: i micro string-heavy mentono DUE volte (branch predictor su
> repr omogenea + residenza cache): verdetto SOLO su workload reale
> interleaved con old dello stesso giorno. ⭐⭐ LEZIONE 2: mimalloc small-bin
> costa meno di copia-inline+branch ⇒ **ridurre il COUNT di alloc non è una
> leva su questo VM: il collo resta il churn Zval (drop/clone/gc)**. ⭐ Il
> −25% alloc previsto dal census WP-37 si è AVVERATO nei sample ma ha reso
> 0 CPU: attribuire in nanosecondi, non in conteggi.
> **SALVATO nel tree** (neutro sul reale, bench38-str −2,2%/bench36 −0,5%):
> `PhpStr::new` bound `AsRef<[u8]>+Into<Box<[u8]>>` (siti slice-fed senza
> to_vec: explode/substr/trim/from_str), `concat2` exact-size (ops::concat +
> fast path Concat(Str,Str)), `from_i64` (niente String/fmt; to_zstr Long +
> param-parsing weak), `concat_n_join` **#[inline(never)] FUORI da run_loop**
> (lezione WP-33 applicata — il join stava nel dispatch loop da WP-34),
> static assert 24B. Cargo **1631** (+4).
> **Gate/run**: gate22 TUTTO verde (corpus 1455/sess 28/date 351/refl 290
> identici; ORM 3E/13F; hk 0E/0F; probe gd/mysqli/media byte-id; http 16
> DIFF attesi; option/restapi identici); **run29 = run28 PER NOME** (30.472,
> 0E/2F/86W/73S) — ⚠️ al primo tentativo +1F sideload
> (`test_sideload_scaled_unique_filename` = flake WP-21): **azzerare
> `wpdev/src/wp-content/uploads` SUBITO PRIMA di ogni full run** (i run
> media di profiling li sporcano); ⭐ `progress.txt` di gate22 è APPEND-ONLY:
> leggere dal timestamp della propria run. Media pair: **59,75/20,955 =
> 2,85×**; footprint 12,0× invariato (repr revertata).
> **📌 Bonus probe** (`wp38-harness/probe_wp38.php`, new≡old ⇒ preesistenti,
> catalogati in `PHPR_DIVERGENCES_FROM_PHP.md` §3.7): (1) `sort($a,
> SORT_STRING)` ignora `$flags` — array.rs ~421 usa sempre ops::compare;
> `key_flag_cmp` esiste già per ksort, manca il value_flag_cmp (guardare
> anche rsort/asort/arsort); (2) warning "Uninitialized string offset"
> mancante su lettura a offset == strlen; (3) deprecation "Increment on
> non-numeric string" mancante su `$s++` alfabetico.
> **→ PROSSIMA SESSIONE = E (gc batching)** dal riprofilo WP-36 (mi_free 681
> + mi_theap_collect 475 + drop Zval/Repr/Rc ~430 dominano; gc_note 15 +
> sweep 12): batching delle note + sweep, ⭐⭐ sentinelle drop-order pinnate
> PRIMA di ogni cambiamento di layout (metodo WP-32). Le tre parità §3.7
> sono fix contenuti buoni come warm-up di sessione.

## 📨 Direttive Gemini post-WP-38 (`2026-07-22_gemini_wp38.md`) — verdetti e integrazione

Verificate sul codice e sui dati WP-38 il 2026-07-22. Analisi retrospettiva
concordante; sui prossimi passi due correzioni tecniche e un veto di policy.

- **✅ §1 Analisi della bocciatura SSO**: CONCORDANTE coi dati (mimalloc
  thread-local free-list ≈ manciata di istruzioni; branch a ogni accesso).
  Una precisazione dai numeri: il read-tax PURO misurato è ≈0 (micro
  sola-lettura, loop omogenei dove il predictor non sbaglia mai) — il costo
  reale è distribuito tra costruttori Vec-fed, pressione cache del 32B e
  misprediction sul mix inline/heap; quest'ultima è co-fattore plausibile,
  non dimostrato dominante. Il verdetto operativo non cambia.
- **✅ §2 Warm-up gap di parità — con DUE correzioni di mira**:
  - *Increment deprecation*: il sito indicato da Gemini (`IncDecSlot`) è
    SBAGLIATO — IncDecSlot è il fast path WP-33 **solo Long-checked**, le
    stringhe non ci entrano mai. Il funnel giusto è
    `compute_incdec`/`ops::increment`, che **già ritorna `diags`** e il cui
    raise avviene PRIMA del write-back (semantica set_error_handler già
    rispettata): il plumbing c'è, va solo emessa la deprecation. ⭐ Oracle
    da consultare anche per il **decrement** su stringa non-numerica (RFC
    8.3 "saner incdec" copre entrambi i versi).
  - *sort flags*: giusto il gemello `value_flag_cmp` di `key_flag_cmp`
    (coprire anche rsort/asort/arsort). ⚠️ Subtlety oggetti: sort è un
    VALUE-sort ⇒ SORT_STRING su Stringable vorrebbe `__toString`, ma il
    gate precompute (§1.1 divergenze) è STATICO per-builtin mentre i
    `$flags` sono runtime — inserire `sort` nel gate chiamerebbe
    `__toString` spurio sotto SORT_REGULAR (side effect osservabile).
    Usare `ctx.to_zstr` col fallback-warning per gli oggetti e dichiarare
    il residuo, come oggi negli altri percorsi non-gated.
  - *Uninitialized string offset*: ✅ come da catalogo §3.7.
- **✅ §3 Leva E (gc batching)**: CONCORDANTE col handoff. Cross-check utile:
  soglia Zend `GC_THRESHOLD` = 10k root nel buffer — phpr ha già il GC
  adattivo WP-21 (soglia che cresce sui collect inefficaci): PRIMA di
  toccare, misurare sul media quante collection avvengono e con che resa
  (attribuzione data-driven, metodo WP-26/33 — la lezione WP-38 sul
  "count ≠ nanosecondi" vale anche qui). Drop batching dei Zval SOLO con
  ordine di distruzione bit-identico (LIFO + cascata FIFO WP-28):
  **sentinelle drop-order pinnate prima**, come già in piano.
- **§4 Paradigmi radicali — verdetti puntuali**:
  - *Bytecode a registri*: ✅ **unica "leva lunga" compatibile** con safe
    Rust + byte-parity. Nota di realismo: le fusioni bigram WP-33/34
    (ThisPropGet, CmpJmpConst, ThisMethodCall, ConcatN) già catturano una
    parte del beneficio (meno dispatch sugli stessi pattern); il salto vero
    è un arco multi-sessione (compiler + run_loop + unit-cache format).
    Candidarla come arco dedicato DOPO la leva E, con census WP-33 alla
    mano per stimare il tetto prima di iniziare (lezione WP-36).
  - *JIT Cranelift*: ⏸ fuori orizzonte ora — parità di diagnostica/linee ed
    exception-table nel codice nativo è un moltiplicatore di complessità; e
    un eventuale JIT presuppone comunque un IR migliore (= il punto sopra).
  - *Zval untagged union unsafe*: ❌ **VETO DI POLICY** — "unsafe fuori
    roadmap" è decisione utente (WP-33), già applicata due volte
    (NaN-boxing bocciato WP-32, union SSO rifiutata WP-38). Da non
    riproporre salvo cambio di rotta esplicito dell'utente.
  - *Arena per-request*: ⚠️ collide con la byte-parity dei distruttori
    (ordine LIFO + free-order Zend WP-28: "buttare l'arena" non è
    equivalente osservabile) e col workload di riferimento attuale (la
    test-suite è UN processo senza confini di request). Variante stretta
    eventualmente per `phpr -S` o per temporanei provabilmente senza
    distruttori — non prioritaria.
